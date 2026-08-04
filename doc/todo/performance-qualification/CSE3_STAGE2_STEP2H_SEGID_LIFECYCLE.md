# CSE-3 Stage 2h — segment-ID never-reuse P0 + lifecycle honesty

Status: **labor active** (2026-08-04)  
Default product remains **Materialized**.

## P0 — segment ID never-reuse

Prior counter bumps were insufficient. Durable allocator:

- File: `store-info/segment_seq.v1` (`SEGSEQ01` + store_id + reserved_thru u64 LE)
- **Reserve before media:** persist `reserved_thru` then mint; failpoints at
  `segalloc.before_reserve_persist`, `after_reserve_persist`,
  `after_reserve_before_media`, `after_active_media`
- Open reconstructs `max(durable, sealed/pending/active/shadow/chimera ids)`
- Corrupt durable + empty media → **refuse** without mutating media
- Matrix: `tests/cse3_stage2_segment_id_never_reuse.rs` (`--test-threads=1`)

## Lifecycle vs ack (Step 9 honesty)

Prior Step 9 “23.3K ack” excluded seal wall (burst). Sustainable product
throughput is puts **including** seal/P★. Target: lifecycle ≥ 80% of ack
(same wall — no seal exclusion).

### Measured dominant seal stages (pre-fix)

On 2 GiB / 64 MiB, ~10.9 s seal tax was dominated by sync Hydra/Chimera +
whole-segment BLAKE3 on `seal_active`, and by applying `EnrichDone` inside
the next seal’s `drain_lifecycle`.

### Optimizations (measured, CompactShadow only)

1. CompactShadow: enqueue EnrichDerived (Hydra/Compact Chimera) async; keep
   Shadow dual finalize sync (P★).
2. Defer whole-segment BLAKE3 to enrichment (`ContentHashState::Pending`).
3. Async enrich worker writes **Compact** Chimera under CompactShadow (no
   Materialized path; no RSHD0003 re-mirror).
4. Seal drain waits authoritative seals only — does not serialize EnrichDone
   onto the seal critical path.
5. Materialized dual-run keeps sync Chimera on explicit seal (Step 8 ceremony).

## Posture

Do **not** flip universal default until lifecycle TPS approaches the product
ack class (~28K band) under 2 GiB/64 MiB without seal-exclusion accounting.
