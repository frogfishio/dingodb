# Full-product 2 GiB campaign — enrichment on (2026-08-04)

Status: **measured** — correctness **PASS**; full-product performance **FAIL**.  
Honest sustainable throughput ≈ **12.4K** 8 KiB writes/sec.  
The **47.4K acknowledgement TPS is burst financed by enrichment debt**.  
Not a published SLO.

## Recipe

| Knob | Value |
|---|---|
| Cell | Real Full |
| Logical | **2 GiB** |
| Seal threshold | 64 MiB |
| Payload / concurrency / seed | 8 KiB / 8 / 42 |
| Enrichment | **on** |
| AWO | disabled |
| Binary | see `binary.sha256` |

## Headline numbers

| Metric | Value |
|---|---:|
| Acknowledgement TPS | **47 428** |
| Complete-lifecycle TPS (ack + auth seal + enrich drain) | **12 429** |
| Campaign TPS (incl. reopen/verify) | **7 698** |
| Completed-enrichment throughput (jobs / ack+enrich-drain wall) | **1.61** jobs/s |
| Backlog slope (OLS, jobs/s during ack) | **+4.14** |
| Peak backlog | **24** |
| Final backlog @ last ack | **24** |
| Backlog after enrich drain | **0** |
| Auth `drain_lifecycle` | **29.3 ms** |
| Enrichment drain | **14.94 s** |
| Reopen exact (`coverage_scan`) | **yes** |
| Index/query sample verify | **yes** |
| Segment digests Known / Pending | **33 / 0** |
| Sealed @ last ack | **32** |

Raw: `sustained-2g-64m-enrichment-on.json`, `summary.json`.

## Interpretation (decisive)

1. One 64 MiB segment ≈ **8 192** × 8 KiB ops. Enrichment at **1.61 seg/s**
   implies capacity \(1.61 \times 8192 \approx 13\,189\) ops/s — matches
   measured complete-lifecycle TPS (**12 429**).
2. Writes at **47.4K** ack create ≈ **5.8 seg/s**, so enrichment falls behind
   by ~**4.1 jobs/s**. Under continuous load the backlog grows without bound.
3. Therefore **47.4K ack is burst**, not sustainable full-product throughput.
   Sustainable ≈ **12–13K TPS**.
4. Correctness after drain: digests Known, reopen exact, index/query OK.

## Harness note

`ack-finalize` records enrichment telemetry (peak/final backlog, slope,
enrich drain, complete-lifecycle TPS, digest counts, index/query sample).
Derived enrichment is submitted from **`SealDone` apply** (sealed file ready).

## Next

**Enrichment Throughput Qualification** —
`doc/todo/performance-qualification/ENRICHMENT_THROUGHPUT_QUALIFICATION.md`.
Three-cell attribution residual deprioritized. AWO remains paused.
