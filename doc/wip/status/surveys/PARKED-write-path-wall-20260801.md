# PARKED — Write-path wall ladder (Mode A → cook → disk)

**Status:** **PARKED** (2026-08-01)  
**Class:** diagnostic survey evidence — **not** a published SLO  
**Priority:** **P1-DX / adoption evidence** follow-on — **does not** move P0-GATE, CSQ-12, PQH accept, Heap app-ready, or M2  

**Disclosure:** [BENCHMARK_DISCLOSURE.md](../../reference/operations/BENCHMARK_DISCLOSURE.md)  
**Peer contract:** [README-PEER-SQL.md](./README-PEER-SQL.md)

---

## Why park

We traced Residiuum Buffered put performance from “why aren’t we SQLite?” through
data-cooking (Blake/encode) and multi-core `put_many` cook, then falsified the
headline multi‑hundred‑k ops/s as a **short page-cache micro** once multi‑GiB
and seals hit **real disk**.

**Conclusion worth keeping:**

1. **Mode A long peer ≈ SQLite autocommit class (~10k puts/s)** on the PEER-SQL
   Scratch bed — good enough as table stakes for “SQLite + loose files”
   replacement narrative.
2. **Cook CPU is real** on short/batch-rich micros; **parallel full-record cook
   works** (~1.8× cook1→cook4 on Scratch 20k-op micro).
3. **The next wall for multi‑GiB, multi-seal load is the disk** (write_all +
   page cache drain + seal/fsync shape) — not more Blake workers.
4. Product differentiator remains **survival under damage**, not beating every
   database on clean-media TPS.

**Do not** freestyle further write-path perf as P0. Resume only under criteria
at the bottom.

---

## Product framing (locked language)

| Phrase | Use? | Notes |
|--------|------|--------|
| puts/s, records ACKed/s | Yes | Prefer over “TPS” / “transactions” |
| Buffered ACK after OS transfer of batch | Yes | CSQ honesty; not SQL COMMIT |
| SQLite Mode A parity (~10k class, same bed) | Yes, diagnostic | PEER-SQL A vs A |
| cook4 ~330k on short Scratch micro | Yes, **labeled micro** | Not media sustained |
| “T3 sustains multi‑GiB/s” | **No** | Never measured |
| Ingress coalesces → `put_many` | Architecture direction | Not fully productized as claim |
| Extra cores = burstable cook when batch depth exists | Yes | Idle single-put stays ~1-core |
| Survival: hole = telemetry + islands, not DBA weekend | Thesis | Code ≠ accept; still prove on assurance chain |

**Burstable SKU story (parked product idea):** fine on 1 vCPU; when the cloud
bundles extra CPUs with the RAM tier, use them on **batched cook** — not “HA
requires multi-CPU.” HA (N instances) ≠ per-node cook cores.

---

## Wall ladder (chronology)

```text
PEER-SQL Mode A ~10k  ──►  cook CPU (Blake ~half micro)  ──►  parallel cook
                                                              (Scratch micro)
                                      │
                                      ▼
                         multi‑GiB + seals on real volume
                                      │
                                      ▼
                              DISK WALL (park here)
```

| Stage | Question | Answer (diagnostic) | Primary artifacts |
|-------|----------|---------------------|-------------------|
| 0 | Fair SQLite peer? | Mode A Residiuum ≈ SQLite within noise on Scratch | `scratch-sqlite-peer-20260801/`, `README-PEER-SQL.md` |
| 1 | Where is Mode A wall time? | Not index-primary; **data cooking** (append/Blake) | `scratch-index-bisect-*`, `scratch-append-shortcircuit-*`, `scratch-blake-shortcircuit-*` |
| 2 | Disk detached? | On short micro, detach only partial; cook still heavy | `scratch-disk-bisect-20260801/` |
| 3 | Multi-core Blake-only? | Weak alone; need full record cook | `scratch-multicore-4-20260801/` |
| 4 | Parallel cooker (Option C)? | cook1→cook4 **~1.8×** on Scratch 20k × 8 KiB | `scratch-parallel-cooker-20260801/` |
| 5 | Is ~330k “real media”? | **No** — short Buffered + cache; logical MiB/s ≠ sustained media | this doc § Numbers |
| 6 | Real `/tmp` multi‑GiB? | 1 GiB phase: ~100–160k batch; **2 GiB multi-seal: ~10k / ~80 MiB/s** | `tmp-real-disk-20260801/` |

---

## Numbers (keep these three bands)

All: payload **8 KiB**, Buffered puts unless noted. **Diagnostic only.**

### Band 1 — PEER-SQL / long multi-seal peer (adoption floor)

| Engine / mode | Bed | Order of magnitude |
|---------------|-----|--------------------|
| Residiuum Mode A (batch=1) | Scratch peer campaign | **~10k** puts/s |
| SQLite Mode A (autocommit) | same | **~10k** puts/s |
| Residiuum Mode B peer-pump, 2 GiB, seal 64 MiB, cook1 | APFS `/tmp` | **~10.2k** puts/s, **~80** logical MiB/s, **~4.1 GiB** on disk |

### Band 2 — Batch `put_many` on real `/tmp` (1 GiB phase-bench)

| Phase | ops/s | logical MiB/s |
|-------|------:|-------------:|
| Discard Buffered (disk detached) | ~140k | ~1095 |
| Real Buffered Mode A batch=1 | ~81k | ~636 |
| put_many cook1 | ~131k | ~1025 |
| put_many cook2 | ~158k | ~1231 |
| put_many cook4 | ~116k | ~908 |

Real ≈ **58%** of Discard → I/O visible. cook4 **does not** win vs cook1.

### Band 3 — Short Scratch micro (20k ops, seal ≥ payload, USB Scratch)

| Phase | ops/s | logical MiB/s |
|-------|------:|-------------:|
| put_many cook1 | ~184k | ~1.4k |
| put_many cook4 | **~327–330k** | **~2.5k** |

**Use only as cook-scaling evidence**, not capacity planning for cloud volumes.

### Same software, three answers

| Bed | cook4-ish / peak batch | Meaning |
|-----|------------------------|---------|
| Scratch 20k micro | ~330k | Cook scales when disk is slack |
| `/tmp` 1 GiB phase | ~116k cook4 | Disk already competing |
| `/tmp` 2 GiB multi-seal | ~10k (cook1) | **Disk + seal wall** |

---

## Code landed (keep; do not expand while parked)

| Item | Where | Notes |
|------|--------|------|
| `Store::set_cook_parallelism(n)` | `residiuum-store` | default 1; parallel env+Blake+frame encode, ordered install, one tail write per batch |
| `append_preencoded_frame` | `residiuum-format` / segment | install path for pre-cooked frames |
| phase-bench cook1/2/4 + disk/Blake/index short-circuits | `residiuum-testrig` | diagnostic only |
| Peer `RESIDIUUM_COOK_PARALLELISM` | `testrig` peer-pump | env for cook N on peer path |
| Cook seal auto-size in phase-bench | `phase_bench.rs` | seal ≥ phase payload so mid-batch rotate does not abort install |

Integrity: parallel cook uses **real Blake** (no product weakening).

---

## Known gaps (parked; not active work)

1. **Seal mid parallel cook:** `put_many` with cook N>1 **fails** if the active segment rotates mid-batch install (`segment rotated mid parallel cook install`). Multi-seal peer + cook4 needs seal-safe install or seal ≥ batch footprint policy.
2. **Ingress coalescing:** architecture (clients → ingress → `put_many`) is direction only; not a finished multi-tenant coaleascer claim.
3. **Default product cook parallelism:** not productized as “auto min(cores, …)” with disclosure.
4. **Durable / fsync-heavy beds:** not re-run as the parked “next wall” study; expect lower numbers.
5. **Cloud gp3/T3 media matrix:** not measured; infer only that multi‑GiB/s media claims are false for cheap volumes.

---

## Next wall (when unparked)

**Disk-bound write path** under honest multi‑GiB + seal policy:

| Lever | Intent | CSQ risk |
|-------|--------|----------|
| Seal policy / fewer mid-path Durable flushes for Buffered | Less seal wall without lying | High if silent durability change |
| Seal-safe parallel cook install | Allow cook N across rotate | Correctness first |
| Larger amortised tails / fewer seeks | I/O efficiency | Low if ack table unchanged |
| Faster media SKU narrative | Honest capacity | Disclosure only |
| Ingress batch windows | Create batch depth | Latency vs throughput trade |

**Not** the next lever: more Blake-only micro-parallelism on sealed multi‑GiB load.

---

## Resume criteria (unpark only if)

- Principal prioritises **P1 write-path** after critical path allows, **or**
- PEER-SQL / adoption needs a **named** follow-on cell (e.g. Durable Mode C, cloud volume matrix), **or**
- A **spec amendment** defines seal-safe parallel cook / Buffered seal behaviour.

Otherwise leave parked. Critical path remains:
**CSQ-12 evidence + PQH accept → Heap application-ready + APB → M2 early access.**

---

## Artifact index

| Dir | Role |
|-----|------|
| [README-PEER-SQL.md](./README-PEER-SQL.md) | Peer contract |
| [scratch-sqlite-peer-20260801/](./scratch-sqlite-peer-20260801/) | PEER-SQL A/B Scratch |
| [scratch-mode-a-breakdown-20260801/](./scratch-mode-a-breakdown-20260801/) | Mode A instrumentation |
| [scratch-disk-bisect-20260801/](./scratch-disk-bisect-20260801/) | Disk detach |
| [scratch-index-bisect-20260801/](./scratch-index-bisect-20260801/) | Index vs cook |
| [scratch-append-shortcircuit-20260801/](./scratch-append-shortcircuit-20260801/) | append_frame |
| [scratch-blake-shortcircuit-20260801/](./scratch-blake-shortcircuit-20260801/) | Blake short-circuit |
| [scratch-mem-algo-20260801/](./scratch-mem-algo-20260801/) | Memory path |
| [scratch-multicore-4-20260801/](./scratch-multicore-4-20260801/) | Naive multi-core attempts |
| [scratch-parallel-cooker-20260801/](./scratch-parallel-cooker-20260801/) | Option C cook pool |
| [tmp-real-disk-20260801/](./tmp-real-disk-20260801/) | APFS `/tmp` 1 GiB + 2 GiB multi-seal |
| `TEST_RESULTS.md` (repo root) | Campaign narratives F+ |

---

## One-line park stamp

> **Mode A ≈ SQLite; cook scales on short batch micros; multi‑GiB multi-seal load hits the disk wall — parked P1 diagnostic, not M2.**
