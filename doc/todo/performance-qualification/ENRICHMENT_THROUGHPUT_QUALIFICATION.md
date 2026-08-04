# Enrichment Throughput Qualification (ETQ) — frozen next package

Status: **next package** (principal freeze 2026-08-04).  
AWO: **paused**. Three-cell attribution residual: **deprioritized**.

Depends on evidence:
`doc/archive/performance-qualification/2026-08-04-enrichment-on-2g/`.

## Honest product numbers (locked)

> Residiuum’s **complete sustainable throughput** is currently approximately
> **12.4K 8 KiB writes/sec**.

| Label | Value | Meaning |
|---|---:|---|
| Acknowledgement TPS | ~47.4K | Burst — financed by deferred enrichment debt |
| Enrichment service | ~1.61 segments/s | Measured completed-enrichment rate |
| Ops per 64 MiB segment | ~8 192 | 64 MiB / 8 KiB |
| Enrichment capacity | ~13.2K ops/s | \(1.61 \times 8192\) |
| Complete-lifecycle TPS | ~12.4K | Matches enrichment capacity |
| Segment create rate @ ack | ~5.8 seg/s | 47428 / 8192 |
| Backlog slope | ~+4.1 jobs/s | Create − service; unbounded under continuous load |

### Verdict

| Gate | Result |
|---|---|
| Correctness campaign (reopen exact, digests Known, index/query) | **PASS** |
| Full-product performance qualification | **FAIL** |
| Actual sustainable full-product throughput | **~12–13K TPS** |

The 2 GiB enrichment-on campaign succeeded as a **measurement and correctness**
test. It decisively failed as a **performance** qualification. The bottleneck
is derived enrichment throughput, not authoritative seal finalize.

## Required service floor

| Floor | Segments/sec | Implied ops/sec (8 KiB @ 64 MiB) | Rationale |
|---|---:|---:|---|
| Minimum | **≥ 5.8** | ≥ ~47.4K | Match current acknowledgement create rate |
| Prefer | **≥ 7.0** | ≥ ~57.3K | Match enrichment-off authoritative engine (~57.6K) |

## Work items (this package)

1. **Stage breakdown** — measure BLAKE3, Hydra, Chimera, and catalog time
   independently on the enrichment path (not only aggregate drain).
2. **Service floor** — prove enrichment can sustain ≥5.8 seg/s (prefer ≥7).
3. **Bounded parallel enrichment workers** — test N>1 with contention honesty.
4. **Cooked-index handoff** — investigate eliminating sealed-segment rereads by
   passing cooked index material asynchronously from the auth lane.
5. **Lazy Chimera** — only if product semantics explicitly permit deferred
   Chimera construction; otherwise leave on the critical enrich path.
6. **Accept gates (all required):**
   - backlog slope ≤ 0 after warm-up;
   - bounded final backlog under sustained load;
   - complete-lifecycle TPS close to acknowledgement TPS;
   - exact reopen + query/index verification;
   - no major acknowledgement collapse from worker contention.

## Non-goals (until ETQ exits)

- AWO / append-path optimisation (remains paused).
- Three-cell attribution medians (auth64 / ceiling) — not the priority.
- Speculative seal-architecture changes unrelated to enrichment service rate.

## Evidence home

When labor runs: archive under
`doc/archive/performance-qualification/YYYY-MM-DD-enrichment-throughput-qualification/`.
