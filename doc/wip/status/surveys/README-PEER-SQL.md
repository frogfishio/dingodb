# PEER-SQL — same-bed Residiuum vs SQLite peer contract

**Status:** contract for Feature PEER-SQL (v1 micro-bed).  
**Numbers class:** **diagnostic only** — not a published SLO. See
[BENCHMARK_DISCLOSURE.md](../../reference/operations/BENCHMARK_DISCLOSURE.md).

This document is the shared contract for implementers (T2+), Scratch campaigns
(T3), and anyone reading ratios in `TEST_RESULTS.md`. It does **not** implement
the harness.

---

## Goal

Obtain **ground-truth** Residiuum vs SQLite throughput on **the same machine,
same volume, same payload, same concurrency**, under **named durability shapes**.

Without this, “match or exceed SQLite” is undefined (anecdotal blog numbers on
random hosts/modes are not targets).

With this, each cell is a **measured ratio** Residiuum/SQLite under one mode only.

---

## Priority and non-claims

| Claim | Allowed? |
|-------|----------|
| Diagnostic peer for adoption narrative | Yes |
| Input to “close the gap” engineering | Yes (after numbers exist) |
| P0-GATE / CSQ-12 / PQH / M2 exit | **No** — does not replace them |
| Published product SLO or marketing “faster than SQLite” | **No** without full disclosure ladder |

Priority class: **P1-DX / adoption evidence**, not P0-SAFETY.

---

## Fixed knobs (v1 — do not freestyle)

| Knob | Value |
|------|--------|
| Host volume | `/Volumes/Scratch/TEST/` only for reportable runs |
| Payload | **8192** bytes opaque blob (fixed) |
| Concurrency | **1** thread, QD=1 |
| Default target | **256 MiB** logical payload budget (on-disk target may be larger for Residiuum footprint) |
| Engines | `residiuum` \| `sqlite` only |
| Modes | **A** and **B** only (below) |
| Free space | Reuse testrig `min_free` refuse floor; no near-full internal disk |

Optional later (not v1): target `1G`, mode **C** (power-loss class / `synchronous=FULL` vs Residiuum Durable).

---

## Peer modes (named)

Always report **A vs A** and **B vs B**. Never cross modes in a ratio.

### Mode A — honest general load

| Engine | Shape |
|--------|--------|
| **SQLite** | Autocommit **per row**; insert one `(k, BLOB)` at a time |
| **Residiuum** | `durability=buffered`, `--put-batch-size 1` (one Buffered OS write ack per key) |

**Purpose:** Closest “simple put” peer for “SQLite + loose files” replacement narrative.

### Mode B — bulk / amortized durability

| Engine | Shape |
|--------|--------|
| **SQLite** | `BEGIN` … **N=128** inserts … `COMMIT` |
| **Residiuum** | `durability=buffered`, `--put-batch-size 128` (`put_many` flush) |

**Purpose:** Where bulk SQLite legends usually come from.  
**Not equal semantics:** SQLite amortizes commit; Residiuum batch still follows CSQ Buffered ack rules for the batch. Label both modes explicitly.

### SQLite schema (v1)

```sql
CREATE TABLE kv (
  k TEXT PRIMARY KEY NOT NULL,
  v BLOB NOT NULL
);
```

Opaque blobs only — no secondary indexes, no SQL join workload.

### SQLite durability knobs (v1 — document on every run)

| Knob | v1 value |
|------|----------|
| Journal | **WAL** |
| `synchronous` | **NORMAL** |

Mode **C** (later): `synchronous=FULL` vs Residiuum `durable` — not in v1 exit criteria.

---

## Metrics (JSON / tables)

Minimum fields for every peer-pump result (align with Residiuum testrig pump JSON):

- `engine` — `residiuum` \| `sqlite`
- `mode` — `A` / `A_autocommit` or `B` / `B_txn_128` (pick one scheme and stick to it)
- `payload_size`, `target_bytes`
- `keys_written`, `elapsed_ms`
- `ops_per_sec`
- `mb_per_sec` — **logical payload** MiB/s (keys × payload / wall), not Residiuum on-disk growth alone
- `peak_cpu_pct`, `peak_rss_bytes` when sampling works
- `ok`, `disclosure` string pointing at this contract + BENCHMARK_DISCLOSURE

Report ratios only as:

```text
ratio_A = residiuum_ops_A / sqlite_ops_A
ratio_B = residiuum_ops_B / sqlite_ops_B
```

(and the same for logical MiB/s if both cells present).

---

## Non-goals (v1)

- Multi-client / TPC / OLTP mix  
- Multi-shard Residiuum peer vs single SQLite  
- Multi-process Axis C capacity vs SQLite  
- Product batch/txn API design (except noting B may need a **named** Residiuum API later to close bulk gap)  
- Weakening CSQ Buffered/Durable to “win” a number  
- Matching arbitrary public SQLite leaderboards  

---

## Fairness background (already measured)

See **Campaign E** in [TEST_RESULTS.md](../../../TEST_RESULTS.md):

- Residiuum **per-put Buffered** is not like SQLite **multi-row COMMIT**.
- At 8 KiB, Residiuum batch=1 ≈ batch=128 in earlier general-load pumps — so “forgot to batch OS writes” is not the main story for Residiuum alone.
- Integrity cost (frame + BLAKE3), dual indexes, and seal policy are Residiuum-specific costs SQLite does not pay the same way.

PEER-SQL does not replace phase-bench diagnosis; it **anchors the SQLite side of the adoption claim**.

---

## Artifact layout

| Item | Path |
|------|------|
| This contract | `doc/wip/status/surveys/README-PEER-SQL.md` |
| Campaign results | `doc/wip/status/surveys/scratch-sqlite-peer-YYYYMMDD/` |
| Writeup | section in `TEST_RESULTS.md` after first campaign |
| Harness | `crates/residiuum-testrig` (`peer-pump` or equivalent — T2) |

Scratch work roots: under `/Volumes/Scratch/TEST/` (e.g. `residiuum-peer-*`), then copy JSON into the surveys tree for git.

---

## Exit criteria (Feature PEER-SQL)

1. Contract (this file) accepted.  
2. `peer-pump` supports both engines × modes A/B (T2).  
3. Scratch four-cell campaign + table/ratios (T3).  
4. testrig README re-run recipe (T4).  

Closing Residiuum’s absolute gap to SQLite is **follow-on engineering**, not PEER-SQL exit.

---

## Implementation order

1. **T1** — this contract  
2. **T2** — testrig peer-pump  
3. **T3** — Scratch campaign + TEST_RESULTS  
4. **T4** — README recipe  

Board: Feature PEER-SQL, project `16acf5e0-f30f-450f-a1e7-8ae442ed1d7a`.
