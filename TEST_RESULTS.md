# Seal-threshold survey — diagnosis + Scratch campaigns

Diagnostic only — not a published SLO (see `doc/reference/operations/BENCHMARK_DISCLOSURE.md`).
Use **within-volume ratios**, not absolute product MiB/s claims.

## What was wrong with the first run

The first campaign (internal disk, `/private/tmp/residiuum-survey-yRCsAz`) is a **real signal that segment threshold matters**, but it is **not clean proof**:

| Issue | Evidence |
|-------|----------|
| **Disk pressure** | Free space fell to ~7 GiB / 97% full during multi‑GiB pumps; absolute rates and later runs are contaminated |
| **Order not counterbalanced** | Configs not run as 64→4→4→64; order effects mixed with threshold effects |
| **CPU / RSS missing** | Original manifests have `peak_rss_bytes` / `peak_cpu_pct` = null — process sampling did not land (older binary or failed `ps` samples). Current `residiuum-testrig` **does** sample successfully |
| **Target size** | 2 GiB on-disk × 6 stores ≈ heavy local pressure |

So: **~2× 64 MiB vs 4 MiB was overstated under near-full disk**, not invented.

## Issues that are *not* product store bugs

1. **Footprint ~2.05×** is consistent and expected (encoding + segment layout overhead vs logical payload). That is **directory growth / footprint**, not measured physical-write amplification.
2. **Four shards not beating single-shard 64 MiB** is real under the current single-process `put_many` path (see multi-shard ladder below); not a silent regression, but also **not** multi-process capacity.
3. **CPU/RSS “deficiency”** is a harness observability gap on the first run, not a store failure — fixed for Scratch runs with current testrig.

---

## Campaign A — Scratch 1 GiB seal counterbalance (baseline retest)

**Host:** `Kazoo.local` arm64 Darwin 24.5  
**Volume:** `/Volumes/Scratch/TEST/` (~153 GiB free throughout; internal root not used)  
**Binary:** `target/release/residiuum-testrig`  
**Pump:** target **1 GiB** on-disk, payload **8 KiB**, durability **buffered**, **1** writer shard  
**Order (counterbalanced):** `64M → 4M → 4M → 64M`  
**Artifacts:**  
- Scratch: `/Volumes/Scratch/TEST/residiuum-survey-results-20260731/`  
- Workspace: `doc/wip/status/surveys/scratch-20260731/`

| Run | Seal | Logical MiB/s | Disk-growth MiB/s | Footprint | Keys | Elapsed | Peak RSS | Peak CPU% |
|-----|------|-------------:|------------------:|----------:|-----:|--------:|---------:|----------:|
| s01 | 64 MiB | 65.2 | 133.6 | 2.05× | 73728 | 8.83 s | 531 MiB | 36% |
| s02 | 4 MiB | 43.2 | 88.7 | 2.05× | 65536 | 11.85 s | 196 MiB | 47% |
| s03 | 4 MiB | 43.4 | 89.1 | 2.05× | 65536 | 11.81 s | 225 MiB | 47% |
| s04 | 64 MiB | 65.7 | 134.7 | 2.05× | 73728 | 8.76 s | 468 MiB | 39% |

**Medians (logical):** 4 MiB **43.3**; 64 MiB **65.5**; ratio **1.51×** (64/4).

---

## Campaign B — Scratch 2 GiB seal counterbalance (scale check)

Same host/volume/binary; target **2 GiB** on-disk; same counterbalance order.  
**Artifacts:**  
- Scratch: `/Volumes/Scratch/TEST/residiuum-survey-2g-20260731/`  
- Workspace: `doc/wip/status/surveys/scratch-2g-20260731/`

| Run | Seal | Logical MiB/s | Disk-growth MiB/s | Footprint | Keys | Elapsed | Peak RSS | Peak CPU% |
|-----|------|-------------:|------------------:|----------:|-----:|--------:|---------:|----------:|
| s01 | 64 MiB | 67.2 | 137.8 | 2.05× | 139264 | 16.18 s | 540 MiB | 37% |
| s02 | 4 MiB | 43.4 | 89.2 | 2.05× | 131072 | 23.58 s | 280 MiB | 47% |
| s03 | 4 MiB | 43.6 | 89.5 | 2.05× | 131072 | 23.50 s | 287 MiB | 48% |
| s04 | 64 MiB | 66.2 | 135.6 | 2.05× | 139264 | 16.44 s | 524 MiB | 44% |

**Medians (logical):** 4 MiB **43.5**; 64 MiB **66.7**; ratio **1.53×** (64/4).

### Interpretation (A + B)

- Threshold effect is **stable across 1 GiB and 2 GiB** targets on Scratch (~**1.5×**).
- Pairs match tightly — order bias is small.
- **64 MiB** holds more live segment RSS; **4 MiB** spends more CPU% on seal/rotate.
- Internal contaminated ratio (~2.16×) remains **overstated**; do not use it for product claims.

---

## Campaign C — Scratch multi-shard ladder (Axis B, single process)

**Target:** 1 GiB on-disk · seal **64 MiB** · payload 8 KiB · buffered · seed 20260731  
**Shards:** 1 / 2 / 4 via `--writer-shards` (single process, `put_many` path)  
**Artifacts:**  
- Scratch: `/Volumes/Scratch/TEST/residiuum-survey-multishard-20260731/`  
- Workspace: `doc/wip/status/surveys/scratch-multishard-20260731/`

| Run | Shards | Logical MiB/s | Disk-growth MiB/s | Keys | Elapsed | Peak RSS | Peak CPU% | Writer model |
|-----|-------:|-------------:|------------------:|-----:|--------:|---------:|----------:|--------------|
| ms01 | 1 | 65.3 | 133.7 | 73728 | 8.83 s | 480 MiB | 44% | single_active_segment |
| ms02 | 2 | 61.9 | 126.2 | 81920 | 10.34 s | 607 MiB | 42% | sharded_active_segments |
| ms04 | 4 | 57.8 | 117.8 | 98304 | 13.29 s | 721 MiB | 76% | sharded_active_segments |

### Interpretation (multi-shard)

- Under this harness, **more writer shards does not increase** single-store logical throughput; 2- and 4-shard runs are **slower** than 1 shard while RSS and (at 4) CPU rise.
- This is **not** a multi-process capacity result (Axis C). Do **not** claim “4 shards = more store capacity” from these numbers.
- Key counts differ slightly (shard packing / seal cadence); compare rates, not raw keys alone.
- Further work if product wants multi-core write scaling: multi-process stores and/or concurrent producers — separate from this ladder.

---

## Harness: free-space refuse floor (landed)

`residiuum-testrig` now refuses pumps when free space is below a floor (prevents silent near-full contamination):

| Piece | Location |
|-------|----------|
| `free_space_bytes` / `default_min_free_for_target` / `ensure_free_space` | `crates/residiuum-testrig/src/size.rs` |
| Checked at `run_pump` start | `crates/residiuum-testrig/src/pump.rs` |
| CLI `--min-free auto\|SIZE\|0` (default **auto** ≈ 2.5× target + 512 MiB) | `crates/residiuum-testrig/src/main.rs` (`pump` + `run`) |
| Multi-process children pass `--min-free 0` (parent already checked total) | `pump.rs` child spawn |

Smoke: `--min-free 999G` → exit 1 with clear refuse message. Unit tests for parse + default floor pass.

---

## Campaign D — Write-path throttle diagnosis (not disk-bound)

**Policy:** All bulk pumps go on **`/Volumes/Scratch/TEST/` only**. Do **not** use the Mac internal volume / `/tmp` for multi‑hundred‑MiB stores — earlier internal surveys contaminated rates and nearly filled the local disk.

### Device ceiling (Scratch SSD)

| Probe | Result |
|-------|-------:|
| Sequential 512 MiB write + fsync | ~**360 MiB/s** |
| Sequential 512 MiB flush only | ~**390 MiB/s** |
| Residiuum pump disk-growth (1 GiB, 8 KiB payload, buffered, 64 MiB seal) | ~**130–135 MiB/s** |
| Residiuum logical payload rate | ~**63–67 MiB/s** |
| Peak process CPU% (macOS: 100% ≈ 1 core) | ~**36–48%** of one core on M4 (10 cores) |

**Conclusion:** We are **well below** SSD sequential bandwidth and **not** saturating the machine. Disk is **not** the limiter at current rates. There is substantial headroom for store/harness optimization before device throughput becomes the story.

### What was killing the path (code evidence)

| Killer | Where | Effect |
|--------|-------|--------|
| **Per-key `put` + batch_size=1** (pre-fix) | testrig `put_batch_size` / `flush_puts` | One full `write_event` per key; no amortization |
| **Per-put `seek` + `write_all` of segment tail** | `store::write_segment_tail` on every put | Syscall + small write per ~8 KiB frame instead of large sequential chunks |
| **Dual primary index publish** | `apply_durable_event` → `index` + `durable_index` | Two BTree inserts + subject clone every key |
| **Full active segment held in RAM** | `ActiveSegment.bytes: Vec<u8>` | Every byte is appended in memory **and** written to the file (double copy / large RSS ~0.5 GiB) |
| **Envelope encode + id mint per key** | `write_event` / batched put | Per-op CPU fixed cost → ~8–9k ops/s ceiling class |
| Mid-run **recursive dir_size** (pre-fix) | testrig pump loop | Extra metadata work during write (now removed for progress) |

### Fixes landed this slice

1. **Store `put_many` single-shard batching** — N in-memory appends, **one** `write_segment_tail` per batch (`put_many_single_shard_batched`). Multi-shard parallel path also tails once per shard batch.
2. **Locator-first index**: durable put path no longer does `body.to_vec()` only to slim it away.
3. **Testrig** always uses `put_many` with **batch 128** (single shard); multi-shard batches ≥128.
4. **Testrig progress** from `WriteReceipt.encoded_frame_len` sum — **no mid-run dir walks**; one final `dir_size` for acceptance.
5. **Free-space refuse** (Campaign earlier) so near-full volumes fail closed.

### After-fix Scratch sample (still not device-bound)

Artifacts: `doc/wip/status/surveys/scratch-batchfix-20260731/`, `scratch-bottleneck-20260731/`

| Run | Notes | Logical MiB/s | Disk MiB/s | CPU% |
|-----|-------|-------------:|-----------:|-----:|
| b02 | 1 shard, 64 MiB seal, 1 GiB, batch=128 | 65.7 | 134.5 | 44 |
| b03 | 4 shards, 64 MiB seal, 1 GiB, batch=256 | 68.4 | 139.4 | 49 |
| confirm | 1 shard, 256 MiB, batch=128 | 63.3 | 129.1 | 38 |

Batching **helped multi-shard slightly** (57.8 → ~68 logical) but **did not** approach the ~360 MiB/s sequential ceiling. Remaining limit is **per-key store CPU/structure** (encode + dual index + dual-buffer segment), not the SSD.

### Next optimization targets (ranked)

1. **Write-through / ring buffer for active segment** — stop retaining full sealed-prefix bytes in RAM after durable_len advances (cuts RSS and memcpy).
2. **Single index publish path** for bulk load / or cheaper dual-map update.
3. **Multi-process Axis C** capacity (independent store roots) when the goal is machine-level throughput, not single-writer ops/s.
4. **Larger payloads / bulk APIs** for qualification cells that claim “device bound.”
5. Keep **all large pumps on Scratch**; treat internal disk as forbidden for GB-scale surveys.

---

## Campaign E — General load vs SQLite-class expectations

**Question:** Why low disk activity *and* low CPU when SQLite is said to go much faster?  
**Policy:** Scratch only (`/Volumes/Scratch/TEST/`).

### Fairness: what “SQLite throughput” usually measures

| SQLite headline number | Residiuum pump (this survey) |
|------------------------|------------------------------|
| Many inserts **inside one transaction** → one WAL/commit write | Each put is its own durability unit (`Buffered` = OS write before ack) |
| Small rows (tens of bytes) → **ops/s** looking huge | 8 KiB opaque payloads + full frame + **BLAKE3 body hash every put** |
| Often mmap / page cache, single B-tree | Dual primary index (`index` + `durable_index`) + full active segment `Vec` in RAM |
| Autocommit-per-insert SQLite is **much** slower than batched txn numbers | Our single-put path is the honest “general load” analogue |

Comparing Residiuum **per-put Buffered** to SQLite **multi-row COMMIT** is not like-for-like.

### Evidence: single-put vs batched (Scratch, 256 MiB, 8 KiB, buffered, 64 MiB seal)

Artifacts: `doc/wip/status/surveys/scratch-general-load-20260801/`  
Harness: `--put-batch-size 1` vs `128` (new flag).

| Mode | put_batch | Logical MiB/s | Disk MiB/s | ops/s | Peak CPU% |
|------|----------:|-------------:|-----------:|------:|----------:|
| **General single-put** | 1 | 63.0 | 128.4 | ~8060 | 36 |
| Batched put_many | 128 | 64.8 | 132.2 | ~8300 | 42 |

**Batching barely helps at 8 KiB.** So the story is **not** “we forgot to batch OS writes.” The limiter is **per-key fixed work** that still leaves average CPU and disk gauges low.

### Why both gauges look idle (logic)

```text
single thread:
  encode envelope → BLAKE3 body → dual BTree insert → write_all (~16KiB) → repeat
```

- **Not multi-core** — only one producer; 40% of *one* core ≈ 4% of the machine.
- **Not QD>1 disk** — one outstanding write; Activity Monitor “disk activity” stays modest.
- **Not “sleeping on a lock” as the main story** — batch=1 ≈ batch=128; syscall count is not the dominant delta.
- **Copy storm (fixed this slice):** `append_raw` used to `body.to_vec()` into `encode_frame` then `extend_from_slice` (second full payload copy). Now `encode_frame_into` writes once into the segment buffer. Good hygiene; **did not** jump us to SQLite-class MiB/s by itself.

### What we are doing “wrong” relative to SQLite-class bulk

1. **No application transaction** that amortizes durability across many puts (product API gap vs SQLite `BEGIN…COMMIT`).
2. **Integrity cost every put** — BLAKE3 of full body is format-required; SQLite does not hash 8 KiB payloads per insert.
3. **Dual in-memory indexes + growing active segment RAM** — extra CPU/cache pressure vs a page cache design.
4. **Single-threaded load generator** — will never light up 10 cores or a modern SSD queue.

None of that means the store is “idle-correct.” It means **the measured path is a serial, integrity-heavy, per-ack write pipeline** — exactly what produces *low* average CPU *and* *low* disk gauges while absolute rates stay ~⅓ of sequential SSD.

### Fixes landed this slice

| Change | Intent |
|--------|--------|
| `encode_frame_into` + segment append without double payload clone | Cut encode copy storm on **every** put (general + bulk) |
| `--put-batch-size` on testrig pump | Measure general load (1) vs batched (N) honestly |
| This section | Stop false “disk bound” / false SQLite apples-to-apples narratives |

### Next levers (general load, ranked)

1. **Explicit write batch / transaction API** (ack many puts after one Buffered/Durable boundary) — closest to SQLite txn semantics.  
2. **Cheaper dual-index update** (or single map + derived durable projection).  
3. **Write-through active segment** (drop RAM prefix after OS transfer) — RSS + memcpy.  
4. **Multi-producer / Axis C** only when machine-level capacity is the goal.  
5. Always quote **durability mode + batching + payload size** next to any rate.

### Correction: low CPU means the “Blake bound” story was wrong

Principal pushback: if pure processing stuck us, one core would sit near 90%+, not ~25–50%.

**Measured on Scratch** (`residiuum-testrig phase-bench`, 20 000 ops × 8 KiB):

| Phase | ops/s | Notes |
|-------|------:|-------|
| Pure BLAKE3 body hash | ~200–310k | Fast user CPU |
| `encode_frame_into` growing Vec | ~230–250k | Format encode including Blake |
| Raw `write_all` / seek+write | ~650k–1M | OS path alone is fine |
| **Store Memory put** | ~560–640k | Index path only — no segment file write |
| **Store Buffered put (batch 1)** | ~29–32k | ~20× slower than Memory |
| Buffered put_many batch 128 | ~32–33k | Batching barely helps in this bench |

**Boundary probe on Buffered batch-1 (same run):**

```text
wall ≈ 688 ms
append (encode into segment) sum ≈ 90 ms   (~4.5 µs/op)
file_write sum ≈ 36 ms                     (~1.8 µs/op)
file_sync sum ≈ 443 ms  (n=4)              ← ~64% of wall
append+write = only ~18% of wall
ps %cpu ≈ 20→25 (of one core)
```

**What this means**

1. **Not Blake-bound.** Pure Blake is ~10× the Buffered put rate. If Blake were the limiter, CPU would be pegged near 100% of one core.
2. **Not “disk can’t keep up” either.** Raw writes are ~1M ops/s; file_write mean ~2 µs. The volume is barely asked to work between stalls.
3. **Wall time is dominated by wait**, especially **`sync_all` on auto-seal** (probe `file_sync` n=4 for ~320 MiB encoded / 64 MiB seal threshold). Seal path **forces Durable flush + sealed-file fsync** even when puts were only `Buffered` (`store.rs` `flush_active_file(..., Durable)` on rotate/seal).
4. Low CPU + low disk gauges **do compute**: the thread is often **blocked in fsync** (not runnable), not busy hashing. That matches Activity Monitor far better than the Blake narrative.
5. A SQLite-style **txn product API is not required** to explain this gap. The next honest engineering target is **seal/fsync policy vs durability mode** (and how often we rewrite full sealed images), not “add transactions because Blake is slow.”

Harness: `residiuum-testrig phase-bench -w /Volumes/Scratch/TEST/...`  
Artifacts: `doc/wip/status/surveys/scratch-phase-bench-20260801/`

### Strategy: fix seal cost without weakening CSQ durability

**Problem:** Auto-seal was calling `flush_active_file(..., Durable)` + sealed-file `sync_all` even when **every put on that segment was only `Buffered`**. That is stronger than CSQ-ACK-004 requires and dominated wall time (probe: ~64% in `file_sync`).

**Integrity-preserving rule (normative):**

| Put acks on the active segment | Seal/rotate must |
|--------------------------------|------------------|
| Includes any **`Durable`** | Full stable path: `write` + `sync_all` + dir sync (unchanged) |
| Only **`Buffered`** (or empty) | **`write` / rename only** — same failure domain as Buffered put (process kill usually OK via page cache; **no** power-loss claim) |
| **`Memory` only** | No frames on disk; no upgrade to Durable |

Never return a `Durable` receipt without a Durable boundary. Never relabel Buffered as something weaker.

**Implemented (this slice):**

1. Per-active-segment `max_ack_durability` (upgraded on each non-Memory put).
2. `seal_flush_mode()` → seal/rotate flush + sealed publish + new-active create use that strength.
3. `finalize_seal(..., require_fsync)` — fsync only when Durable path required.
4. Open/recovery finalize still uses `require_fsync: true` (fail closed).

**After seal-durability match (Scratch phase-bench, 20k × 8 KiB Buffered):**

```text
file_sync n=0  (was n=4, ~443 ms)
Buffered put still ~31k ops/s  (was ~29–32k)
```

Fsync tax gone; rate barely moved until rename-seal (below).

**Rename-based seal (landed next):** finalize appends **only the summary suffix** to pending and `rename`s into `segments/` (prefix preserved; no full ~64 MiB rewrite). Durable still fsyncs when `require_fsync`.

```text
After rename-seal (same phase-bench):
  Buffered put  ~41k ops/s   (was ~31k)
  put_many 128  ~44k ops/s
  file_sync n=0
  256 MiB pump  ~81 logical MiB/s, peak CPU ~75%  (was ~63–65 MiB/s / ~40% CPU)
```

Remaining gap vs Memory (~600k ops/s) is still encode/index + seal scan/hash work — not “need a product txn API” first.

**Still integrity-safe next:**

1. Stream/mmap seal scan to avoid holding two full segment copies in RAM during finalize.  
2. Larger seal threshold for Buffered bulk (fewer seals).  
3. Durable path stays strict.  
4. Do not skip frame BLAKE3 / weaken wire integrity.

**What we will not do:** silent “faster Buffered” that acks before OS write, or Durable without `sync_all`.

### Robustness / guarantees (principal Q: “would this affect our guarantees?”)

**Normative SoT:** `doc/reference/operations/CRASH_AND_RECOVERY_CONTRACT.md` (durability ack table). Performance work must not silently change those modes.

| Change class | Affects robustness / guarantees? | Notes |
|--------------|----------------------------------|--------|
| **`encode_frame_into` (landed)** | **No** — same bytes on the wire | Only removes an intermediate full-frame `Vec` clone before segment append. Frame layout, BLAKE3, CRC, recovery unchanged. |
| **`put_many` batch tail write (landed)** | **No if ack after OS transfer of the whole batch** | All receipts still mean “Buffered/Durable boundary crossed” for that batch. Process kill mid-batch without returned receipts → same **unknown/old/new** rules as mid-single-put. |
| **Testrig `--put-batch-size` / free-space refuse** | **No** (harness only) | Measurement; does not change product store contract. |
| **Write-batch / transaction API (proposed)** | **Only if designed carefully** | Safe: hold frames, one OS write (+ `sync_all` for Durable), **then** return all receipts — same guarantees as today, amortized. **Unsafe / contract change:** return receipts before OS transfer while still labeling `Buffered`/`Durable`. That would require a new mode or an explicit “uncommitted batch” surface. |
| **Write-through / drop RAM prefix after durable_len** | **No if recovery still rebuilds from on-disk frames** | Must keep locator offsets valid and seal/open paths correct. Risk is implementation bugs (torn tail, wrong durable_len), not a deliberate weaker mode. Needs crash-matrix re-run. |
| **Cheaper dual-index (single map + derived projection)** | **No if visibility after durable ack is identical** | Risk is publish-order bugs (DEF-023: visibility only after authoritative append). Must not make incomplete frames visible. |
| **Skip BLAKE3 / weaken frame integrity for speed** | **Yes — guarantee change** | Format integrity and salvage would change. Not recommended as a silent perf flag. |
| **Treating SQLite-style uncommitted multi-put as `Buffered` without OS write** | **Yes — weakens CSQ-ACK-004 Buffered** | Buffered today = “handed to OS page cache.” Deferring OS write past ack is Memory-class loss domain until flush. |

**Bottom line:** The optimizations already landed (encode path, put_many tail batching) are **performance hygiene under the existing durability contract**. The big SQLite-shaped win is a **batch/txn API that keeps the same ack table**, not relaxing Durable/Buffered. Any change that returns success before the named boundary must be a **new, named mode** — never a quiet rewrite of `Buffered`/`Durable`.

---

## Comparison to contaminated internal medians

```text
Internal (2 GiB target, not counterbalanced, disk pressure):
  4 MiB median logical  ≈ 38.6 MiB/s
  64 MiB median logical ≈ 83.5 MiB/s
  ratio ≈ 2.16×

Scratch 1 GiB counterbalance (free space OK):
  4 MiB median logical  ≈ 43.3 MiB/s
  64 MiB median logical ≈ 65.5 MiB/s
  ratio ≈ 1.51×

Scratch 2 GiB counterbalance (free space OK):
  4 MiB median logical  ≈ 43.5 MiB/s
  64 MiB median logical ≈ 66.7 MiB/s
  ratio ≈ 1.53×
```

---

## Where the “issues” actually are

| Area | Status |
|------|--------|
| Store / PQH code from prior packages | Accepted; not implicated by disk-pressure noise |
| **Survey methodology** | Contaminated first run fixed by Scratch + counterbalance |
| **Testrig process sampling** | Works on current binary |
| **Free-space floor** | **Done** — pumps refuse when volume is too tight |
| **Seal default / threshold** | Product-relevant: ~**1.5×** clean 64 vs 4 MiB on Scratch |
| **Multi-shard capacity claim** | Single-process Axis B: no big win; post-batch ~same as 1-shard. Axis C multi-process still open |
| **SSD saturation** | **Not reached** — ~1/3 of sequential ceiling; CPU ~40% of one core |
| **Write-path throttle** | Per-key encode/index/hash + dual-buffer segment; OS batching is **not** the main 8 KiB limiter |
| **SQLite comparison** | Batch txn numbers ≠ our per-put Buffered path; see Campaign E |
| **PEER-SQL same-bed peer** | Contract + first Scratch campaign below (modes A/B) |

---

## Campaign F — PEER-SQL same-bed Residiuum vs SQLite (2026-08-01)

**Diagnostic only** — not a published SLO. Contract:
[doc/wip/status/surveys/README-PEER-SQL.md](./doc/wip/status/surveys/README-PEER-SQL.md).
Disclosure: [BENCHMARK_DISCLOSURE.md](./doc/reference/operations/BENCHMARK_DISCLOSURE.md).

**Host:** `Kazoo.local` arm64 Darwin · **Volume:** `/Volumes/Scratch/TEST/` (~153 GiB free before; ~151 GiB after)  
**Binary:** `target/release/residiuum-testrig` `peer-pump`  
**Fixed knobs:** logical target **256 MiB**, payload **8192**, seed **20260801**, single thread  
**SQLite:** WAL, `synchronous=NORMAL` · **Residiuum:** `buffered`  
**Artifacts:** `doc/wip/status/surveys/scratch-sqlite-peer-20260801/`  
**Scratch work:** `/Volumes/Scratch/TEST/residiuum-peer-20260801/`

| Cell | Engine | Mode | Keys | ops/s | Logical MiB/s | Disk MiB/s | Peak CPU% | Peak RSS | Wall |
|------|--------|------|-----:|------:|-------------:|-----------:|----------:|---------:|-----:|
| residiuum-A | residiuum | A_autocommit (batch=1) | 32768 | **9924** | **77.5** | 158.1 | 77 | 595 MiB | 3.30 s |
| sqlite-A | sqlite | A_autocommit | 32768 | **9458** | **73.9** | 78.9 | 17 | 6.8 MiB | 3.46 s |
| residiuum-B | residiuum | B_txn_128 (batch=128) | 32768 | **10221** | **79.8** | 162.8 | 70 | 441 MiB | 3.21 s |
| sqlite-B | sqlite | B_txn_128 | 32768 | **18675** | **145.9** | 155.7 | 15 | 6.1 MiB | 1.75 s |

### Ratios (Residiuum / SQLite) — same mode only

| Mode | ops/s ratio | Logical MiB/s ratio | Read |
|------|------------:|--------------------:|------|
| **A** (autocommit / per-put Buffered) | **1.05** | **1.05** | Residiuum ≈ SQLite (slightly ahead on this run) |
| **B** (txn-128 / put_many 128) | **0.55** | **0.55** | SQLite ~1.8× Residiuum (amortized commit) |

**Do not** compare Residiuum-A to SQLite-B (or cross modes).

### Interpretation

1. **Mode A is the fair “general load” peer.** On this Scratch run, Residiuum matches SQLite within noise (~5% ops/s). That is the first measured same-bed answer for per-ack / autocommit 8 KiB blobs — not a legend number.
2. **Mode B is not equal semantics.** SQLite `BEGIN…COMMIT` amortizes durability; Residiuum batch=128 still pays Buffered path costs without a product txn API. SQLite leads ~1.8× — expected shape, not a silent store bug.
3. Residiuum **RSS** is much higher (active segment + indexes in RAM); SQLite stays small. Disk-growth MiB/s is not the comparison metric; **logical MiB/s** is.
4. Residiuum A ≈ B again (batching barely helps at 8 KiB) — consistent with Campaign E.
5. Re-run on the same volume after seal/index changes; keep seed/payload/target fixed.

### Command recipe

See `crates/residiuum-testrig/README.md` (PEER-SQL section).

### Campaign F re-run — post write-through (same bed, 2026-08-01 later)

Same knobs: 256 MiB logical · 8 KiB · seed 20260801 · Scratch · Residiuum seal **64 MiB**.  
Artifacts: `doc/wip/status/surveys/scratch-sqlite-peer-20260801/post-wt-*.json`

| Cell | ops/s | Logical MiB/s | Peak RSS | Wall |
|------|------:|-------------:|---------:|-----:|
| residiuum-A | **9577** | **74.8** | 301 MiB | 3.42 s |
| sqlite-A | **9778** | **76.4** | 6.4 MiB | 3.35 s |
| residiuum-B | **10131** | **79.1** | 330 MiB | 3.23 s |
| sqlite-B | **18534** | **144.8** | 6.3 MiB | 1.77 s |

| Mode | Residiuum/SQLite ops/s | vs Campaign F |
|------|-----------------------:|---------------|
| **A** | **0.98×** (~parity) | was 1.05× — same story within noise |
| **B** | **0.55×** | was 0.55× — unchanged shape |

Write-through did **not** move the Mode A peer vs SQLite (still fair parity). It cut Residiuum RSS (Campaign F A was ~595 MiB → ~301 MiB here) without a Mode A throughput leap on the long peer. Mode B gap remains SQLite txn amortization.

---

## Campaign G — Mode A put-path instrumentation + first squeezes (2026-08-01)

**Diagnostic only.** Artifact:
[doc/wip/status/surveys/scratch-mode-a-breakdown-20260801/](./doc/wip/status/surveys/scratch-mode-a-breakdown-20260801/).

### Probe phases (Buffered single-put)

prep · encode_env · append_frame · publish_index · post_derived · file_write ·
**seal_rotate (timed)** · file_sync.

### Corrected elimination (20k × 8 KiB Scratch)

| Scenario | ops/s | Dominant phase |
|----------|------:|----------------|
| 64 MiB seal (2 mid-run rotates) | ~42k | **seal_rotate ~65%** of wall |
| 512 MiB seal (**0** mid-run rotates) | **~108k** | **append_frame ~53%**, file_write ~17%, prep ~3% |

First “prep 65%” was **seal cost mis-binned** into prep. After split:

- Per-put hot prep (ids + env + clock) is **~1–3%**.
- Long Mode A runs with default 64 MiB seal are **seal-bound**.
- Continuous put without seals is **append-bound** (Blake+copy), then per-put `write_all`.

### Hygiene opts landed

| Change | Effect |
|--------|--------|
| CSPRNG **refill pool** (`ids.rs`) | Fewer `getrandom` syscalls; still OS entropy only |
| **Cached `now_ns`** (Instant + ~1 ms wall refresh) | Avoid `SystemTime::now()` every put |
| Memory put after opts | **~1.0M ops/s** (was ~0.58M) |

### Peer-A 256 MiB logical

Raising seal threshold to 512 MiB did **not** beat 64 MiB on the long peer (~67 vs ~80 logical MiB/s) — large active segment **RSS/cache** pressure. Need **faster seals** and/or **write-through**, not only “seal less.”

### Next squeezes (implemented this slice)

1. Seal/finalize cost (still rename-seal; cut remaining work).  
2. append_frame micro + segment reserve.  
3. Write-through so large thresholds stay RAM-safe.

### Campaign G.2 — write-through + seal finalize lean (2026-08-01)

**Changes (integrity-preserving):**

| Change | Intent |
|--------|--------|
| `ActiveSegment::base_offset` + `discard_through` | Drop RAM for bytes already transferred to the OS |
| `write_segment_tail` write-through after Buffered/Durable write | Active segment RSS ≈ unflushed tail (not full seal threshold) |
| File-based sync seal when prefix discarded | Same sealed bytes; recovery still from on-disk frames |
| `seal_pending_bytes` in-place truncate + `into_bytes` | Fewer full-segment clones on finalize |
| Modest 256 KiB create capacity (not full threshold) | Append micro without pinning seal_threshold RAM |

**Scratch phase-bench** (20k × 8 KiB, 512 MiB seal micro — no mid seals):

| Phase | Before (G) | After write-through |
|-------|----------:|--------------------:|
| Buffered put batch=1 | ~41–108k ops/s | **~135k ops/s** |
| append_frame share | ~53% (no-seal) | ~51% |
| file_sync | n=0 | n=0 |

**Peer-A 256 MiB logical** (same knobs as Campaign F residiuum-A):

| Seal | Logical MiB/s | ops/s | Peak RSS | Notes |
|------|-------------:|------:|---------:|-------|
| 64 MiB (Campaign F) | 77.5 | 9924 | 595 MiB | pre write-through |
| **64 MiB (after)** | **80.5** | **10303** | **339 MiB** | ~same rate, **~43% less RSS** |
| 512 MiB (pre) | ~67 | — | multi-hundred MiB | RSS/cache pressure |
| **512 MiB (after)** | 63.2 | 8093 | **22 MiB** | write-through RSS proof |

Artifacts: `doc/wip/status/surveys/scratch-mode-a-breakdown-20260801/phase-bench-after-write-through.txt`, `peer-A-after-write-through.json`, `peer-A-after-write-through-512.json`.

**Read:** Write-through lands the “large seal threshold without multi-hundred-MiB RSS” goal. Long peer-A at 64 MiB seal stays ~parity with Campaign F (±noise); the microbench put path is clearly faster when seals are not mid-run. Remaining Mode A headroom is still append Blake/copy + seal worker backpressure on long runs — not a silent weaker Buffered mode.

---

## Campaign H — Cook path → disk wall — **PARKED** (2026-08-01)

**Status: PARKED.** Full writeup + resume criteria:  
[doc/wip/status/surveys/PARKED-write-path-wall-20260801.md](./doc/wip/status/surveys/PARKED-write-path-wall-20260801.md)

**Diagnostic only — not a published SLO.** Does **not** move CSQ-12 / PQH / M2.

### Ladder (one screen)

| Stage | Finding |
|-------|---------|
| PEER-SQL Mode A | Residiuum ≈ SQLite **~10k** puts/s same bed |
| Short Scratch phase-bench | **Cook CPU** (Blake/append) dominates; disk detach only partial |
| Parallel cooker Option C | `set_cook_parallelism`; cook4 ~**1.8×** cook1 on 20k × 8 KiB Scratch (~**330k** ops/s micro) |
| `/tmp` 1 GiB phase-bench | Real ≈ **58%** of Discard; cook4 **≤** cook1 (~116–158k) |
| `/tmp` 2 GiB peer multi-seal | **~10.2k** puts/s · **~80** logical MiB/s · **~4.1 GiB** disk — **disk + seal wall** |

### Three-band rule (do not mix)

1. **~10k** — multi-seal / PEER long peer (adoption floor)  
2. **~100–160k** — 1 GiB batch on real APFS `/tmp`  
3. **~330k** — short Scratch cook micro only  

### Code kept while parked

- `Store::set_cook_parallelism` + preencoded frame install  
- phase-bench cook1/2/4 + short-circuits  
- Peer env `RESIDIUUM_COOK_PARALLELISM`  

### Known gap

Parallel cook **fails** if segment seals mid-batch install.

### Next wall when unparked

**Disk / seal policy / seal-safe parallel install** — not more Blake-only workers.  
Artifacts: `scratch-parallel-cooker-20260801/`, `tmp-real-disk-20260801/`, bisect surveys under `doc/wip/status/surveys/`.

---

## Bottom line

There **was** a real problem with the **measurement environment** (full internal disk + incomplete order + missing process samples), not a silent “everything is broken in the store.”  
On Scratch with free space (early campaigns):

1. Segment threshold still matters (~**1.5×** at both 1 GiB and 2 GiB targets).  
2. Short QD=1 pumps can look **not disk-bound** (~130 MiB/s growth vs higher sequential ceilings) while still **cook-bound**.  
3. **General load** (batch=1) ≈ **batched** (batch=128) at 8 KiB on some early beds — limiter often **per-key integrity/index/cook**, not “forgot to batch writes.”  
4. Low CPU + low disk together is expected on a **single-threaded, QD=1, per-ack** pipeline; SQLite-class bulk numbers usually mean **transactions + small rows**, not this path.  
5. Prefer **`/Volumes/Scratch/TEST/`** for large Scratch surveys; **Campaign H** also used **`/tmp` deliberately** for a real multi‑GiB disk check (budget-capped).

**Campaign H park:** multi‑GiB **multi-seal** load hits the **disk wall** (~10k / ~80 MiB/s on APFS `/tmp`). Short cook micros (~330k) must not be sold as media capacity. See parked doc above.

**Do not** publish absolute product MB/s from these diagnostics without Benchmark Disclosure + controlled runner class.