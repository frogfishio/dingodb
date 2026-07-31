# Crash-and-recovery contract (DEF-104)

```text
contract id: dingo-crash-recovery-v1
status: labor shipped (executable journeys + this page)
audience: application developers, operators, implementers
```

This page is the **single normative product contract** for what a durability
receipt proves, what happens on crash, how incomplete/damaged evidence is
spoken, how to recover without inventing data, and which reactions are
**forbidden**. It unifies DEF-022, DEF-098–103, and the relevant CSQ invariants.

Companions (depth, not substitutes):

| Topic | Document / suite |
|-------|------------------|
| Failpoint crash matrix | [CRASH_CONSISTENCY.md](CRASH_CONSISTENCY.md), `crash_matrix.v1.json`, `stage_def_022_crash_matrix` |
| Chunk generations | DEFECTS DEF-098, `stage_def_098_chunk_generation` |
| Historical get | DEFECTS DEF-099, `stage_def_099_historical_get` |
| Coverage scans | DEFECTS DEF-100, `stage_def_100_coverage_scans` |
| Writer lock | DEFECTS DEF-101, `stage_def_101_writer_lock` |
| Primary cache lifecycle | [PRIMARY_INDEX_LIFECYCLE.md](PRIMARY_INDEX_LIFECYCLE.md), `stage_def_102_primary_cache_diag` |
| Large / rewrite-heavy | [LARGE_VALUE_AND_REWRITE_HEAVY.md](LARGE_VALUE_AND_REWRITE_HEAVY.md), `stage_def_103_large_value_policy` |
| CSQ invariants | [CORE_STORAGE_QUALIFICATION_SPEC.md](../CORE_STORAGE_QUALIFICATION_SPEC.md) |
| Executable contract suite | `crates/residuum-store/tests/stage_def_104_crash_recovery_contract.rs` |

---

## 1. Durability-mode acknowledgement table

`DurabilityMode` is the **failure boundary** named on every successful
`WriteReceipt`. Performance claims must name the mode measured.

| Mode | When ack returns | Survives process kill after ack? | Survives power loss after ack? | CSQ |
|------|------------------|----------------------------------|--------------------------------|-----|
| `Memory` | Process-memory publication | **No** | **No** | CSQ-ACK-004 |
| `Buffered` | Bytes handed to OS page cache / device queue (no required `fsync`) | Usually yes on clean kill if OS flushed | **No** guarantee | CSQ-ACK-004 |
| `Durable` | Authoritative bytes + required metadata crossed this build’s stable-storage boundary (`write` + `sync_all` on active segment / dir where applicable) | **Yes** | **Yes** under the qualified FS model | CSQ-ACK-001, CSQ-ACK-002 |

### What a returned durable receipt proves

A successful `put` / `delete` returning `WriteReceipt` with
`durability == Durable` proves:

1. The complete event crossed the durable boundary (CSQ-ACK-001 / CSQ-ACK-002).
2. After crash/reopen, the event is observable in authority (`active/` +
   `segments/`, plus `chunks/` when used) (CSQ-PUB-001).
3. Receipt fields describe that event: `store_id`, `segment_id`, `item_id`,
   `event_id`, `event_kind`, `offset`, plus DEF-103 layout facts
   (`layout`, `logical_len`, `chunk_count`, `profile_id`) (CSQ-ACK-005).

### What absence of a receipt proves

If the call does **not** return a receipt (error, kill mid-call, client drop):

| Outcome class | Meaning | Application reaction |
|---------------|---------|----------------------|
| **old** | Prior complete generation still current | Continue using prior value; retry with new op_id if needed |
| **new** | New event fully durable despite missing client receipt | Read-back / history by `event_id` if known; do not double-apply with different content |
| **unknown** | Cannot prove old vs new | Retry only with **exact mutation + same operation id** when supported; otherwise inspect history; never invent `[]` / `{}` |

Never a hybrid fabricated event (CSQ-ACK-003).

CLI: `dingo put` / `dingo get` for smoke; crash cells via
`DINGO_CRASH_*` harness (see CRASH_CONSISTENCY).

---

## 2. Inline and chunked publication (state diagrams)

### 2.1 Inline put (body ≤ chunk threshold)

```text
admit(logical_len) ──fail──► PayloadTooLarge (zero durable effect)
        │
        ok
        ▼
 mint item/event ids → append item event (inline body)
        │
        ▼
 durability barrier for selected mode
        │
        ▼
 return WriteReceipt → primary index / catalogs may update (derived)
```

### 2.2 Chunked put (body > chunk threshold under `LargeValuePolicy`)

```text
admit(logical_len + layout) ──fail──► PayloadTooLarge (zero effect)
        │
        ok
        ▼
 publish chunk frames (verified) ──kill──► unacked; may leave orphan chunks
        │                                   (not current generation unless
        │                                    manifest also durable)
        ▼
 append item event with chunk manifest (chunk_event_ids[])
        │
        ▼
 durability barrier → WriteReceipt
        │
        ▼
 ordinary get uses ONLY this generation’s chunk_event_ids (DEF-098 / CSQ-GEN-002)
```

**Kill before receipt:** any of old / new / unknown. Surviving orphan chunks
without a durable current manifest must not become ordinary success for a
different generation.

**Overwrite:** dual durable chunked puts to the same key → get returns the
**latest complete generation**, never a cross-generation mix (DEF-098).

---

## 3. Read outcome decision table

Use completeness-aware APIs when the decision matters.

| Observation | Ordinary `get` | Completeness-aware | Safe application decision |
|-------------|----------------|--------------------|---------------------------|
| No live subject | `Ok(None)` | `Ok(None)` | Key absent / never written / deleted |
| Live, inline or chunked complete | `Ok(Some(body))` | `PayloadResult::Complete` | Use body |
| Live, some chunks missing | `Err(PayloadPartial)` | `Partial` | Treat as **damage**, not absence; export prior if needed (DEF-099) |
| Live, no chunk bodies | `Err(PayloadPartial)` | `Unavailable` | Same as partial class for ordinary get |
| Live, conflicting chunk at index | `Err(PayloadConflict)` | `Conflicting` | Fail closed; do not pick a winner |
| Writer lock contention | `WriterLockHeld(obs)` | n/a | Retry / inspect; **not** empty store |
| Key coverage incomplete on legacy full drain | `CoverageIncomplete` | page flags `coverage_complete=false` | Do not claim complete key set |
| Over-limit new write | `PayloadTooLarge` | n/a | Reject client input; zero durable effect |

**Forbidden:** map any of the error rows to `[]`, `{}`, “not found”, or silent
overwrite of evidence.

---

## 4. Exact Store and Collection recovery APIs

### Store (embedded, shipped labor)

| API | Role | DEF |
|-----|------|-----|
| `put` / `delete` → `WriteReceipt` | Authoritative mutation + ack | 022, 103 |
| `get` / `get_payload` | Current generation; fail-closed partial/conflict | 098 |
| `get_payload_version(subject, event_id, ReadBudget)` → `VersionedPayloadResult` | Exact historical reconstruction; read-only | 099 |
| `find_last_complete_version(subject, BeforeEvent, RecoveryReadOptions)` → `HistoricalSearchResult` | Walk older puts for last complete body | 099 |
| `scan_live_keys_page` / `scan_live_documents_page` | Coverage-honest pages | 100 |
| `open` / `open_with_options` / `try_open` / `open_inspect` | Writer ownership vs inspect | 101 |
| `writer_lock_status` | Observe lock without taking it | 101 |
| `primary_cache_diag` / `lifecycle_diag` | Derived-cache classification | 102 |
| `large_value_policy` / `set_large_value_policy` | Admission profile | 103 |

### Collection (SDK, where wired)

| API | Notes |
|-----|-------|
| `Collection::get` | Ordinary current get; same fail-closed rules |
| `Collection::get_version` / `find_last_complete` | DEF-099; embedded first |
| `Collection::scan_keys_page` / `scan_json_partial_page` | DEF-100; embedded first |
| Legacy `scan_keys` / `scan_json` | Fail-closed or `CoverageIncomplete`; never silent partial `[]` |

Remote/cluster parity for new recovery/scan APIs may still be residual — check
[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md).

### Result shapes (non-secret)

```text
WriteReceipt {
  store_id, segment_id, item_id, event_id, event_kind,
  durability, offset, layout, logical_len, chunk_count, profile_id
}

VersionedPayloadResult {
  selected_event_id, current_event_id, selected: Option<PayloadResult>,
  is_tombstone, known_gap_before, history_coverage_complete,
  tombstone_crossed, events_examined, bytes_examined, …
}

HistoricalSearchResult {
  found: Option<VersionedPayloadResult>,
  incomplete_candidates, tombstone_stopped, budget_exhausted, …
}

KeyScanPage / DocumentScanPage {
  keys|rows, incomplete, coverage flags, continuation, has_more, examined
}
```

---

## 5. Key coverage versus body completeness (DEF-100)

| Question | API | Rule |
|----------|-----|------|
| Which keys exist? | `scan_live_keys_page` | Does **not** reassemble bodies; body damage cannot hide a verified key |
| Stream healthy docs around damage? | `scan_live_documents_page` | Healthy rows + `incomplete[]` / undecodable; flags for key coverage |
| May I claim a complete key set? | Only when page/drain reports `coverage_complete` | Else `CoverageIncomplete` on legacy drain |

Body incomplete + key event verified → **key listable**, document in
`incomplete[]`. Key-bearing authority damaged → known keys listable with
**explicit incomplete coverage**.

---

## 6. Historical-version selection and tombstones (DEF-099)

1. Ordinary `get` stays **current-generation fail-closed**. No silent fallback
   to an older complete body.
2. Exact selection is by **`event_id` only** (not timestamp, not array index).
3. Chunked historical reconstruction uses **that event’s** manifest
   `chunk_event_ids` only (DEF-098 path).
4. Results disclose current vs selected event, completeness, history gaps,
   `tombstone_crossed`.
5. APIs are **read-only**: no mutate, repair, promote, or hide of current
   authority.
6. `find_last_complete_version` stops at the first delete tombstone by default;
   `RecoveryReadOptions { cross_tombstone: true }` is forensic and **labelled**.

---

## 7. Writer-lock recovery and inspect pattern (DEF-101)

| Situation | API | Reaction |
|-----------|-----|----------|
| Need exclusive writer | `Store::open` / `open_with_options(wait)` | On `WriterLockHeld`, read `obs.class`, `retryable` |
| Non-create open | `try_open` | Never creates; `NotAStore` if missing |
| Concurrent doctor/read | `open_inspect` | No writer lock; read authority |
| Status without open | `writer_lock_status` | Diagnostic only |

**Authority:** OS exclusive lock (+ in-process registry). Diagnostic PID text in
`store-info/writer.lock` **never** grants or breaks the lock.

**Forbidden:** delete `writer.lock` to force unlock; treat `WriterLockHeld` as
empty database; kill peer processes as the product “unlock” path.

CLI: `residuum doctor` prints `writer_lock` class + guidance.

---

## 8. Authority versus derived artifacts (DEF-102)

| Path | Role |
|------|------|
| `active/*.dingo` | **Authority** (writer tail) |
| `segments/*.dingo` | **Authority** (sealed) |
| `chunks/` | **Authority** (chunk frames when used) |
| `indexes/primary.idx` | **Derived** frontier/cache — never health-by-size |
| `catalogs/*`, `indexes/seg/*`, `snapshots/` | **Derived** / rebuildable |

Healthy shape: large `active/`, sparse `segments/`, tiny `primary.idx` is
**normal**. Classify with `primary_cache_diag` /
`lifecycle_diag` / `residuum doctor` (`primary_cache`, `lifecycle` JSON).

Deleting all derived dirs and reopening must reconstruct the same logical live
state (DEF-023 / DEF-102).

---

## 9. Large and rewrite-heavy modelling (DEF-103)

- Chunk threshold is a **layout** switch, not a product max-document claim.
- Application profile admits up to **16 MiB** logical payload by default
  (`dingo-large-value-v1`); effective ceiling is min(store, client, transport).
- Admission is **before** event mint / append; reject → zero effect.
- Rewrite-heavy workloads (transcripts, agents, timelines): **independent keys
  per turn/block**, not one ever-growing document under one key.

```text
transcript/{id}/meta
transcript/{id}/turn/{monotonic-id}
transcript/{id}/timeline/{bounded-block-id}
transcript/{id}/snapshot/{generation}   # derived only
```

Helpers: `residuum_store::rewrite_heavy::*`. Detail:
[LARGE_VALUE_AND_REWRITE_HEAVY.md](LARGE_VALUE_AND_REWRITE_HEAVY.md).

---

## 10. Operator decision tree and forbidden actions

### Decision tree (incident)

```text
1. Can you open?
   ├─ WriterLockHeld → writer_lock_status / open_inspect; wait or find holder
   │                    NEVER delete writer.lock
   └─ NotAStore → wrong path; do not create over unknown data without intent
2. residuum doctor --json-out PATH
   ├─ healthy + primary_cache.validation → interpret per DEF-102 (size ≠ data)
   └─ holes/damaged → salvage / examine; preserve source
3. Application read fails?
   ├─ PayloadPartial / PayloadConflict → get_payload + find_last_complete_version
   │                                    (export prior; do not overwrite with {})
   ├─ CoverageIncomplete → page APIs with coverage flags
   └─ PayloadTooLarge on write → shrink payload / policy; zero prior effect
4. Suspected missing data after crash without receipt?
   → history by subject; classify old/new/unknown; retry only with safe op id
5. Derived looks empty (tiny primary.idx)?
   → primary_cache_diag; authority is active/+segments/; wipe derived OK
```

### Forbidden actions (lint / test fixtures reject these patterns)

| Forbidden reaction | Why |
|--------------------|-----|
| `error → []` / empty map as success | Converts damage/uncertainty to absence |
| Silent overwrite of partial key with `{}` | Destroys evidence |
| Delete `writer.lock` to unlock | Breaks exclusive writer model; OS lock is authority |
| Treat lock failure as “empty store” | Data still present under another holder |
| Treat `primary.idx` byte size as stored-data size | Derived checkpoint only |
| Claim complete key enumeration without coverage flags | False completeness |
| Ordinary `get` auto-fallback to older generation | Hides current damage (DEF-099) |
| Cross-generation chunk mix as success | DEF-098 / CSQ-GEN-002 |

---

## 11. Capability limitations and assumption ledger

| Assumption / limit | Status |
|--------------------|--------|
| Embedded store is the qualified recovery surface for 098–103 | **shipped labor** |
| Remote/cluster parity for new scan/history APIs | **residual** (matrix) |
| Studio UI for primary_cache/lifecycle | **residual** (DEF-102) |
| Client/server negotiation of `LargeValuePolicy` | **residual** (DEF-103) |
| Full multi-process abort matrix for every chunk phase | DEF-022 matrix + CI subset; full nightly |
| Promotion of historical complete → current via write API | **not** this contract (read-only recovery) |
| CSQ program may still mark higher gates open | See CORE_STORAGE_QUALIFICATION_SPEC |

**Filesystem assumption:** durable ack is meaningful only on the filesystem
model qualified for this build (ordered metadata + data durability as
documented in CRASH_CONSISTENCY / CSQ).

---

## Executable journeys (CI)

Suite: `cargo test -p residuum-store --features legacy-raw-store --test stage_def_104_crash_recovery_contract`

| Journey | Test / linked suite | CSQ / DEF |
|---------|---------------------|-----------|
| Durable put ack then reopen | `journey_durable_put_ack_survives_reopen` | CSQ-ACK-002 |
| Chunked dual overwrite + reopen | `journey_chunked_overwrite_generation_exact` + stage_def_098 | CSQ-GEN-002, DEF-098 |
| Unacked / mid-publication cells | stage_def_022_crash_matrix (CI subset + crash-child) | CSQ-ACK-003 |
| Prior complete export while current exists | `journey_prior_complete_via_history_apis` + stage_def_099 | DEF-099 |
| Key list + document page around multi keys | `journey_key_and_document_pages` + stage_def_100 | DEF-100 |
| Writer held → inspect still works | `journey_writer_held_inspect_not_empty` + stage_def_101 | DEF-101, CSQ-ID-008/009 |
| Wipe derived → same logical state + diag | `journey_derived_cache_wipe_neutral` + stage_def_102 | DEF-102 |
| Transcript turns independent | `journey_transcript_independent_turns` + stage_def_103 | DEF-103 |
| Forbidden reactions rejected | `forbidden_*` tests in stage_def_104 | this contract §10 |

Verify script (doc sections + suite): `scripts/verify-crash-recovery-contract.sh`.

---

## Versioning

- Contract id: **`dingo-crash-recovery-v1`**
- Breaking changes to normative tables or forbidden reactions require a new
  contract id and CAPABILITY_MATRIX honesty.
