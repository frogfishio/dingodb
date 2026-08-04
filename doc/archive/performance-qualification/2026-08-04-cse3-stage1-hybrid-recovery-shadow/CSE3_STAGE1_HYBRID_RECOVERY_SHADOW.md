# CSE-3 Stage 1 — Hybrid: Compact Chimera + Recovery Shadow

Status: **principal-accepted** (2026-08-04) — specification complete with
deletion/lifecycle addendum. **No** product flip; Stage 2 implements.  
Date: 2026-08-04.  
Principal fork: **C — Hybrid** (after Stage 0 P★ bound).  
Depends: Stage 0
[`CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md`](./CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md).

## Decision

Separate **query acceleration** from **full-copy salvage**:

```text
Compact Chimera  → query acceleration; tiny, derived, disposable
Recovery Shadow  → full-copy salvage; authoritative recovery artifact
```

This **preserves P★** (Stage 0): ≈100% independent redundancy for incompressible
live data. It does **not** defeat information theory. The win is making the
required copy **cheap to produce** (streaming sequential image) instead of the
old Materialized Chimera query-oriented persistence path (~366 ms/segment class).

| Structure | Role | Amp class | Disposable? |
|---|---|---|---|
| Compact Chimera (`.cmr` v2 SegmentFrame) | Hot / explicit query locators | ~0.74% | **Yes** — derived |
| Recovery Shadow (new) | P★ salvage of \(V_S\) | ~100% of \(V_S\) | **No** — recovery artifact |
| Materialized Chimera (current product) | Interim P★ until Shadow passes CSE | ~98% | Remains until equivalence |

## Two structures (normative)

### Compact Chimera (query)

- Locators only (`SegmentFrame`: segment id, frame offset, body_len, generation).
- May be wiped, corrupt, or absent without affecting Recovery Shadow decode.
- Must **not** be the sole carrier of P★ salvage.
- Rebuildable from segments + PrimaryIndex (true derived acceleration).

### Recovery Shadow (salvage)

1. **Not “derived” / not “safely disposable.”** Loss of Shadow for segment \(S\)
   **withdraws** P★ for \(V_S\) until Shadow is rebuilt. Label in docs/code:
   `recovery artifact` (not “derived index”).
2. **Contents:** every subject in the sealed live projection for \(S\), as either
   a **put** (live value) or a **tombstone** (delete that must suppress older
   puts under recovery). Each record carries:
   - subject identity (key bytes),
   - generation binding (seal / layout generation; exactness for current state),
   - tag: Put | Tombstone,
   - for Put: value length, integrity binding, payload bytes (sequential),
   - for Tombstone: integrity binding only (no payload).
   See **Addendum A** for lifecycle obligations (compaction, retention, frontier).
3. **Placement:** separate file from the authoritative segment, same class of
   file-level independence as Materialized `.cmr` vs `segments/{S}` (CSE-0 F4).
   Proposed path: `recovery/shadow/{hex16(S)}.rsh` (not under `indexes/chimera/`).
4. **Decoder:** reconstructs exact \(V_S\) after complete authoritative-payload
   loss (P★ / CSE F3–F4 `layout_direct` analogue = `shadow_direct`).
5. **Independence from Compact Chimera:** deleting `.cmr` must leave Shadow
   decode intact.
6. **P★ validity moment (explicit):**
   - Durable **acknowledgement** of puts continues to mean: bytes are on the
     authoritative segment path under current durability mode (unchanged).
   - **P★ is not implied by ack.** P★ for segment \(S\) becomes valid only when
     Recovery Shadow \(R_S\) has been **atomically persisted and fsynced**
     (or equivalent durability) for that seal generation.
   - Between seal/ack and Shadow durable: product may claim segment authority
     only; operators must **not** claim format salvage under total segment loss.
   - Lifecycle / enrichment backlog must expose Shadow lag (same honesty as
     enrichment-behind-ack today).
7. **Construction:** streaming, sequential append of records; **no** Chimera
   containers, value-log packing for query, or sorted query tables on the
   hot Shadow write path. Optional post-pass indexes for Shadow are out of
   Stage 1 scope and must not gate P★.

## Wire sketch (non-normative until implement package)

```text
RSHD0001 | store_id | segment_id | generation | n_records | …
for each live-or-tombstone record (deterministic order; freeze in implement):
  tag: Put | Tombstone
  key_len | key | gen | [value_len | value | record_hash]  # Put only
  # Tombstone: key | gen | tombstone_hash (no payload)
trailer: shadow_content_hash
```

Atomic replace via existing `atomic_file` pattern. Decode fail-closed on hash /
length / store-segment mismatch (never invent bodies). **Incomplete / partial /
crash-truncated `.rsh` files never enter P★** and never update
`protected_frontier`.

## Addendum A — Deletion and lifecycle semantics

A Recovery Shadow is a **real second copy**. It inherits obligations disposable
Chimera never had.

### A.1 Tombstones

- Deletes that affect the sealed live projection **must** appear in Shadow as
  **tombstone** records (subject + generation + integrity binding; **no** payload).
- `shadow_direct` / multi-shadow merge **must not** resurrect a value whose
  latest generation across recovery coverage is a tombstone.
- Put-after-delete at a newer generation supersedes the tombstone normally.

### A.2 Latest generation across shadows

- When multiple Shadows (or generations of Shadow for related segments) cover a
  subject, recovery selects the **exact latest generation** (put or tombstone).
- Ties / ambiguous generations are **fail-closed** (no guess).
- Generation semantics match the store’s live-projection generation for that
  subject at seal time — not an independent clock.

### A.3 Compaction and Shadow retirement

- Compaction / live-projection that would destroy or replace segment \(S\)
  **must not** delete old Shadow \(R_S\) until a **replacement** Shadow
  \(R_{S'}\) (or equivalent coverage set) is **atomically durable** and advances
  `protected_frontier` to cover the same live set.
- Crash between new Shadow durable and old Shadow delete: both may exist;
  recovery still picks latest generation; P★ remains on the durable frontier.

### A.4 Retention expiry and secure deletion

- Retention expiry and secure-delete paths **must** remove or cryptographically
  erase Shadow payloads for expired subjects (tombstone-only or wipe), not only
  authoritative segment bytes.
- Leaving plaintext values only in `.rsh` after primary wipe is a **defect**.

### A.5 Backup, scrub, encryption, key rotation

- `.rsh` is **recovery-authoritative** media for P★.
- Backup / restore inventories must include `recovery/shadow/`.
- Scrub must verify Shadow integrity (and report P★ holes).
- Encryption / key rotation must treat Shadow payloads with the same key
  lifecycle as other durable user data (no orphan cleartext after rotate).

### A.6 Partial or interrupted Shadows

- Temp / incomplete / non-trailer-hash-valid files **never** qualify for P★.
- Readers ignore them for salvage and for `protected_frontier`.
- Corrupt complete Shadows: fail-closed for those records; **must not**
  contaminate healthy primary (`auth`) reads.

### A.7 Observable protection frontier

- Store exposes `protected_frontier`: the highest sealed generation (or ordered
  seal identity) for which Shadow coverage is durable and complete.
- `protected_frontier` **never** claims incomplete Shadows.
- Lag: sealed frontier may lead `protected_frontier`; operators see the gap
  (same honesty class as enrichment-behind-ack).
- Deleting a Shadow **explicitly removes** P★ for its coverage and **retreats**
  or holes `protected_frontier` accordingly; authoritative put/get remain intact.

## Channel map (CSE equivalence)

| CSE channel | Materialized today | Hybrid target |
|---|---|---|
| `auth` | Segment / index | Unchanged |
| `chimera` | Materialized `.cmr` (index-gated) | Compact `.cmr` optional; **not** P★ |
| `layout_direct` | Materialized `.cmr` get | **`shadow_direct`** = decode Recovery Shadow |

CSE-1 Compact gaps (F3 chimera `t`, F0/F3/F4 layout_direct) are closed by
**Shadow**, not by Compact.

## Acceptance — Stage 1 (this document)

| Gate | Requirement |
|---|---|
| Structures | Compact vs Shadow roles frozen; Shadow not labeled derived/disposable |
| P★ moment | Ack ≠ P★; P★ only after Shadow atomic durable |
| Lifecycle addendum | §§A.1–A.7 present and normative for Stage 2 |

**Stage 1 status: principal-accepted** (2026-08-04) with this addendum.

## Acceptance — Stage 2 (implement)

| Gate | Requirement |
|---|---|
| P★ CSE suite | Identical F0–F5 with Shadow as format salvage; generation-exact; fail-closed |
| No resurrection | Total authoritative loss **never** resurrects a deleted value |
| Compaction/crash | Compaction + crash combinations preserve exact current live state |
| Expiry | Expired data gone from **both** authoritative and recovery media |
| Isolation | Corrupt Shadow never contaminates healthy primary reads |
| Shadow delete | Authoritative ops intact; P★ coverage explicitly removed |
| `protected_frontier` | Reaches sealed frontier under sustained load; never claims incomplete Shadows |
| Perf | Shadow ≥ **7** seg/s; backlog slope ≤ **0**; lifecycle TPS → ack TPS |
| Interim | **Materialized Chimera remains product** until Shadow passes equivalence |
| Compact | Remains derived / disposable; amp class ~0.74% |

## Explicit non-claims (Stage 1)

- Spec only — **no** Recovery Shadow implementation in this package.
- Does **not** flip product default off Materialized yet.
- Does **not** resume ETQ-2.
- Does **not** claim multi-media / multi-disk independence (file-level only, as CSE-0).
- Does **not** claim historical multi-generation archive beyond live projection + tombstones needed for exact current state.
- Does **not** claim ack ≡ P★.

## Implementation package outline (Stage 2)

1. Spec freeze + schemas under `spec/` (amend via ARCHITECTURE map).
2. `recovery_shadow` module in `residiuum-store` (encode/decode/atomic write;
   tombstones; `protected_frontier`).
3. Seal / enrichment: write Compact Chimera **and** stream Shadow; keep
   Materialized path until dual-run equivalence.
4. CSE campaign + lifecycle gates (resurrection, compaction/crash, expiry,
   corrupt isolation, frontier).
5. Perf gates: ≥7 seg/s, backlog ≤0, lifecycle ≈ ack.
6. Only then: principal may retire Materialized product default.

## Evidence

- This spec: `doc/todo/performance-qualification/CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`
- Archive: `doc/archive/performance-qualification/2026-08-04-cse3-stage1-hybrid-recovery-shadow/`
- Stage 0 bound: `…/cse3-stage0-recovery-bound/`