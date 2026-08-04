# ETQ-0 — Enrichment stage breakdown (2026-08-04)

Status: **package accept (principal)** — problem found conclusively.  
Chimera writes ~**63 MiB derived / 64 MiB auth** (~2× write amp); persist
dominates enrichment. Recipe: 2 GiB · 64 MiB seals · enrichment **on** ·
8 KiB · c=8 · seed=42.

## Floors

| Floor | Segments/sec | Budget ns/segment |
|---|---:|---:|
| Keep pace with ~47.4K ack | **5.8** | ~172 ms |
| Match ~57.6K auth engine | **7.0** | ~143 ms |

## Per-segment means (n=32 EnrichDone samples)

| Stage | Mean wall | Capacity (seg/s) | ≥5.8 | ≥7.0 |
|---|---:|---:|:---:|:---:|
| **Chimera construct+persist** | **390.5 ms** | **2.56** | FAIL | FAIL |
| ↳ Chimera persist | **366.4 ms** | **2.73** | FAIL | FAIL |
| ↳ Chimera construct | 24.1 ms | 41.6 | PASS | PASS |
| Read + decode | 110.9 ms | 9.01 | PASS | PASS |
| ↳ decode (Hydra+Chimera scans) | 83.6 ms | 12.0 | PASS | PASS |
| ↳ read | 27.4 ms | 36.6 | PASS | PASS |
| Isolation gap (50 ms min) | 51.6 ms | 19.4 | PASS | PASS |
| BLAKE3 | 28.7 ms | 34.9 | PASS | PASS |
| Hydra construct+persist | 17.1 ms | 58.3 | PASS | PASS |
| Catalog apply (writer) | 0.03 ms | ≫5.8 | PASS | PASS |
| **Wall total (incl. gap)** | **606.2 ms** | **1.65** | FAIL | FAIL |
| Service excluding gap | 554.6 ms | 1.80 | FAIL | FAIL |

Bytes/segment (mean): read **~64.0 MiB**, written **~63.1 MiB** (Hydra+Chimera).  
CPU / wall (enrich worker thread): **~0.72**.

## Dominant stage

**Chimera persistence** is the bottleneck. Alone it caps enrichment at
~2.6 seg/s — below both floors. All other measured stages clear 7 seg/s.

## Implication for ETQ-1 (frozen)

**Compact Chimera Persistence** — not more workers.
See `doc/todo/performance-qualification/ETQ1_COMPACT_CHIMERA.md`.

Default Chimera must stop embedding full payloads (today ~63 MiB `.cmr` per
64 MiB segment). Persist locators + metadata; payloads stay in authoritative
segments. Parallel workers would only amplify disk contention.

Raw: `sustained-2g-64m-enrichment-on.json`, `summary.json`.
