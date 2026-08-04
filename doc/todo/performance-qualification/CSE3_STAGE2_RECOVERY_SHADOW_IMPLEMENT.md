# CSE-3 Stage 2 — Recovery Shadow implement

Status: **active** (2026-08-04) — delivery sequence frozen; **steps 1–4 landed**
in `residiuum-store::recovery_shadow` (7 unit tests). Steps 5–9 open.
**No product flip** until step 8.  
Depends: Stage 1 principal-accepted
[`CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`](./CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md).

## Delivery sequence (normative)

Until step **8**, **Materialized Chimera remains the safe product path**.
Compact Chimera + Recovery Shadow must not become authoritative sealing.

| Step | Work | Authoritative? |
|---|---|---|
| **1** | Versioned `.rsh` wire format + atomic publication | No |
| **2** | Streaming sequential writer | No |
| **3** | Generation-exact salvage including tombstones | No |
| **4** | `protected_frontier` + protection-lag telemetry | No |
| **5** | Integrate compaction, retention, secure deletion, encryption, backup, scrub | No |
| **6** | Complete CSE F0–F5 damage/crash suite | No |
| **7** | Prove ≥7 segments/sec with non-growing backlog | No |
| **8** | Switch product sealing: Materialized → Compact + Recovery Shadow | **Yes — only here** |
| **9** | Re-run full-product throughput qualification | Post-flip |

## Boundary (no ambiguity)

- Steps 1–7 may land dual-run / experimental Shadow writers beside Materialized.
- **Ack ≠ P★.** P★ only after Shadow atomic durable for that seal generation.
- Incomplete / corrupt `.rsh` never advances `protected_frontier` and never
  contaminates healthy primary reads.
- Product seal/enrichment stays on Materialized until step 8 principal gate.

## Wire freeze (step 1)

Path: `recovery/shadow/{hex16(segment_id)}.rsh`

```text
magic[8] = "RSHD0001"
store_id[16]
segment_id[16]
generation[u64 LE]     # seal / layout generation for this Shadow file
n_records[u32 LE]
records[n] sorted by (key ascending, gen ascending):
  tag[u8]              # 1=Put, 2=Tombstone
  key_len[u32 LE] key[key_len]
  gen[u64 LE]
  Put: value_len[u32 LE] value[value_len] record_hash[32]
  Tombstone: record_hash[32]
trailer: content_hash[32] = blake3(bytes without trailer)
```

Record hash: blake3 over `tag || key || gen_le || [value]` (value only for Put).
Publication: existing `atomic_file` protocol (temp → sync → rename → dir sync).

## Module

`crates/residiuum-store/src/recovery_shadow/` — encode/decode, streaming writer,
salvage projection, frontier. Seal/enrichment wiring is step 5+; product flip
is step 8 only.

## Non-claims (this Stage 2 labor slice)

- Does **not** flip product default off Materialized.
- Does **not** claim CSE F0–F5 pass until step 6 evidence.
- Does **not** resume ETQ-2.
- Does **not** claim ≥7 seg/s until step 7 evidence.
