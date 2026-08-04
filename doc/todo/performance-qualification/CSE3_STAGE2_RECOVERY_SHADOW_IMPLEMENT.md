# CSE-3 Stage 2 — Recovery Shadow implement

Status: **active** (2026-08-04) — Stage 2a invariants confirmed; step 5
lifecycle dual-run landed; **step 6 CSE F0–F5 + lifecycle/security matrix
principal-accepted**; **step 7 perf harness active** (no product flip).
Steps 8–9 open. **No product flip** until step 8.  
Depends: Stage 1 principal-accepted
[`CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`](./CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md).

## Stage 2a foundation invariants (accepted)

1. **Atomic publication:** tmp write → `File::sync_all` → rename → parent
   directory sync before protection is claimed.
2. **Self-verifying Shadow:** store/segment identity, magic version, record
   boundaries/count, per-record + whole-artifact integrity.
3. **Gap-aware frontier:** downward closed — seq 12 completing cannot conceal
   missing seq 11.
4. **Multi-shard:** per-shard coverage; aggregate claim is **min** prefix.

Formally: \(s \in ProtectedFrontier \Rightarrow \forall p \preceq s,\ p \in DurableShadow\).

## Dual-run vs post-flip reclaim (locked — step 6)

> During dual-run, Materialized may satisfy recovery authority. After the
> flip, reclaim must **always** require durable replacement Shadow coverage;
> “when present” is no longer sufficient.

`ShadowReclaimPolicy::DualRunMaterializedAuthority` (default) vs
`RequireReplacementShadow` (step 8+). Compaction refuses source deletion under
post-flip policy when replacement `.rsh` is missing.

## Delivery sequence (normative)

Until step **8**, **Materialized Chimera remains the safe product path**.
Compact Chimera + Recovery Shadow must not become authoritative sealing.

| Step | Work | Authoritative? |
|---|---|---|
| **1** | Versioned `.rsh` wire format + atomic publication | **Done** (2a) |
| **2** | Streaming sequential writer | **Done** (2a) |
| **3** | Generation-exact salvage including tombstones | **Done** (2a) |
| **4** | `protected_frontier` + protection-lag telemetry (gap-aware, per-shard) | **Done** (2a) |
| **5** | Integrate compaction, retention, secure deletion, encryption, backup, scrub | **Done** (lifecycle dual-run; no flip) |
| **6** | Complete CSE F0–F5 damage/crash suite (+ lifecycle/security) | **Principal-accepted** — [`CSE3_STAGE2_STEP6_CSE_MATRIX.md`](./CSE3_STAGE2_STEP6_CSE_MATRIX.md) |
| **7** | Prove ≥7 segments/sec with non-growing backlog | **Labor evidence PASS (experimental dual-stream)** — RSHD0004 write-time dual stream; 2 GiB×3 median **55.57** seg/s ([`CSE3_STAGE2_STEP7_SHADOW_PERF.md`](./CSE3_STAGE2_STEP7_SHADOW_PERF.md); archive `2026-08-04-cse3-stage2-step7-dual-stream`). Principal accept still open; no product flip. |
| **8** | Switch product sealing: Materialized → Compact + Recovery Shadow | **Yes — only here** |
| **9** | Re-run full-product throughput qualification | Post-flip |

## Boundary (no ambiguity)

- Steps 1–7 may land dual-run / experimental Shadow writers beside Materialized.
- **Ack ≠ P★.** P★ only after Shadow atomic durable for that seal generation.
- Incomplete / corrupt `.rsh` never advances `protected_frontier` and never
  contaminates healthy primary reads.
- Product seal/enrichment stays on Materialized until step 8 principal gate.

## Wire freeze

Path: `recovery/shadow/{hex16(segment_id)}.rsh`

### Experimental current (RSHD0004 — write-time dual stream)

```text
magic[8] = "RSHD0004"
store_id[16]
segment_id[16]
encoded_len[u64 LE]
image[encoded_len]     # exact sealed .residiuum bytes (dual-streamed)
commitment[32]         # blake3(store‖seg‖len‖ ordered body_hash‖off‖len‖gen)
```

Cook once → append auth + append Shadow staging (independent alloc; no reflink;
no per-record sync; bounded buffer). Seal appends the same summary to both,
`sync_all`s Shadow independently, publishes atomically, then advances
`protected_frontier`. Salvage scans `image`. Async rotate disabled while armed.

### Prior (RSHD0003 — post-seal canonical mirror; still readable)

```text
magic[8] = "RSHD0003"
store_id[16]
segment_id[16]
encoded_len[u64 LE]
image[encoded_len]     # exact sealed .residiuum bytes (physical copy)
commitment[32]         # blake3(envelope‖image)
```

Publication: buffered physical copy (no clone/reflink) → `sync_all` → rename →
dir sync. V1/V2 record Shadows remain readable for fixtures / older media.

### Legacy record form (RSHD0002 / RSHD0001 — still readable)

```text
magic[8] = "RSHD0002"   # RSHD0001 still readable
store_id[16]
segment_id[16]
generation[u64 LE]     # seal / layout generation for this Shadow file
n_records[u32 LE]
records[n] sorted by (key ascending, gen ascending):
  tag[u8]              # 1=Put, 2=Tombstone
  key_len[u32 LE] key[key_len]
  gen[u64 LE]
  Put: value_len[u32 LE] value[value_len] record_hash[32]
       # V2 record_hash = blake3(tag‖key‖gen‖blake3(value))  [body_hash form]
       # V1 record_hash = blake3(tag‖key‖gen‖value)
  Tombstone: record_hash[32]
trailer: content_hash[32] = blake3(bytes without trailer)
```

## Module

`crates/residiuum-store/src/recovery_shadow/` — encode/decode, streaming writer,
salvage projection, frontier. Seal/enrichment wiring is step 5+; product flip
is step 8 only.

## Non-claims (this Stage 2 labor slice)

- Does **not** flip product default off Materialized.
- Does **not** claim CSE F0–F5 pass until step 6 evidence.
- Does **not** resume ETQ-2.
- Does **not** claim ≥7 seg/s until step 7 evidence.
