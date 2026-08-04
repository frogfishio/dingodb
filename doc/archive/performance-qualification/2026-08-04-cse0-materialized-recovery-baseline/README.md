# CSE-0 — Materialized Chimera recovery baseline (2026-08-04)

Status: **labor complete** (failure set \(F\) + recovery oracle frozen).  
Not package `accept`. Charter:
[`CHIMERA_SALVAGE_EQUIVALENCE.md`](../../todo/performance-qualification/CHIMERA_SALVAGE_EQUIVALENCE.md).

## Claim (honest)

Under frozen failures F0–F5, measure what **Materialized** Chimera recovers
via three channels after cold reopen:

| Channel | Meaning |
|---|---|
| `auth` | `Store::get` (PrimaryIndex → segment pread) |
| `chimera` | `Store::get_via_chimera` (needs index live entry + `.cmr`) |
| `layout_direct` | `load_chimera_layout` + `ChimeraLayout::get` (format-only) |

Materialized encode/reader stay intact. This does **not** accept Compact as
product default or durability-equivalent.

## Recipe

- Seed keys `t` / `m` / `l` (tiny / medium / large), seal, **overwrite** Compact
  seal `.cmr` with `build_materialized_layout`.
- Apply one failure from \(F\), reopen, measure recoverable exact bodies.
- Test: `cargo test -p residiuum-store --features legacy-raw-store --test cse0_materialized_chimera_recovery`

## Frozen failure set \(F\)

| Id | Damage |
|---|---|
| F0 | Control (no damage) |
| F1 | Wipe Chimera sidecars |
| F2 | Corrupt Materialized `.cmr` bytes |
| F3 | XOR establishing item body for key `t` in sealed segment |
| F4 | Delete sealed segment file (Chimera left intact) |
| F5 | F3 + wipe Chimera |

## Recovery oracle (measured)

See `baseline.json` / `FAILURE_TABLE.md`. Headline:

- **F3:** Materialized **does** expand product ChimeraGet salvage for damaged
  `t` while PrimaryIndex still has a live entry (auth must not invent wrong `t`).
- **F4:** Product reopen ChimeraGet is **empty** — `get_via_chimera` requires a
  PrimaryIndex live entry; segment delete rebuilds index with no keys.
  **Format** `layout_direct` still recovers all three keys from embedded `.cmr`.
- **F1/F2/F5:** Chimera channels empty when sidecar gone/corrupt; auth unchanged
  except damaged `t` on F5.

## Evidence

| Artifact | Path |
|---|---|
| Baseline JSON | `baseline.json` |
| Failure table | `FAILURE_TABLE.md` |
| Test source | `crates/residiuum-store/tests/cse0_materialized_chimera_recovery.rs` |
| Run log | `run.log` |
| Git HEAD at run | `git-head.txt` |

## Next

**CSE-1** — identical \(F\) on Compact; require
\(\mathrm{Recoverable}_{compact}\supseteq\mathrm{Recoverable}_{materialized}\)
per channel (product + layout_direct as defined in CSE-0).
