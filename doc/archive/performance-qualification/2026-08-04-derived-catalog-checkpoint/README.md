# Derived Catalog Checkpointing (2026-08-04)

Status: **package accept (principal)** — gates met; `catalog_apply` off the
writer hot path; O(n²) lifecycle persist risk removed.
Sustained ack **47 759 → 57 640 TPS** (~+20.5%).

## Defect

Persisting full derived tier/segment-catalog state on every `SealDone` was
O(sealed segments) per rotation → O(n²) over long retention. Sustained
rotation had measured **catalog_apply ≈ 13.9%** of ack wall.

## Fix

1. `SealDone` / `EnrichDone` update **in-memory** placement + segment catalog only.
2. Durable writes coalesce on the seal worker via `DerivedCatalogCheckpoint`
   (every 32 seals or 2 s; lag/loss permitted).
3. `drain_lifecycle` best-effort flushes; not an authority condition.
4. Open rebuilds via `discover_placements` + `rebuild_segment_catalog`.

## Proofs (unit)

`crates/residiuum-store/tests/derived_catalog_checkpoint.rs`:

- Reopen exact before a pending checkpoint (catalogs deleted; no drain flush).
- Reopen exact after deleting derived catalogs.
- Per-rotation `catalog_apply` flat across 32 / 256 / 1 024 tiny segments.

## Sustained campaign (2 GiB @ 64 MiB, enrichment off)

| Knob | Value |
|---|---|
| Cell | Real Full |
| Logical | **2 GiB** |
| Seal threshold | 64 MiB |
| Payload / concurrency / seed | 8 KiB / 8 / 42 |
| Enrichment | **off** |
| Binary | see `binary.sha256` |

| Metric | Before (sustained-rotation) | After |
|---|---:|---:|
| Ack TPS | 47 759 | **57 640** |
| Rotations | 32 | 32 |
| Reopen exact | yes | yes |
| `catalog_apply` % of ack wall | **13.91%** | **0.005%** |
| `catalog_apply_ns` | 763 593 795 | 247 209 |

Acceptance `catalog_apply` **&lt; 1%**: **PASS**.

Raw: `sustained-2g-64m.json`.

## Seal Fast Lane

Architectural accept language stands (derived off auth lane; meta publish).
This package removes the asymptotic catalog-persist defect uncovered by that
measurement.

**Next (frozen):** three-cell lifecycle attribution on the same binary —
`doc/todo/performance-qualification/THREE_CELL_LIFECYCLE_ATTRIBUTION.md`.
AWO remains paused until those medians exist. Do not return to AWO as a
substitute for lifecycle accounting.
