# CSE-1 — Compact Chimera equivalence campaign (2026-08-04)

Status: **labor complete** — inequality **fails**; **CSE-2 required**.  
Not package `accept`. Charter:
[`CHIMERA_SALVAGE_EQUIVALENCE.md`](../../todo/performance-qualification/CHIMERA_SALVAGE_EQUIVALENCE.md).

## Claim (honest)

Under frozen failures F0–F5, Compact Chimera does **not** recover at least
Materialized’s sets on all channels. Compact remains provisional performance
architecture only — **not** durability-equivalent, **not** product default.

## Recipe

- Seed keys `t` / `m` / `l`, seal (default Compact `SegmentFrame` `.cmr`).
- Apply one failure from \(F\), reopen, measure recoverable exact bodies.
- Compare to CSE-0 Materialized RHS (`…/2026-08-04-cse0-materialized-recovery-baseline/baseline.json`).
- Test: `cargo test -p residiuum-store --features legacy-raw-store --test cse1_compact_chimera_equivalence`

## Headline gaps

1. **F3:** Materialized ChimeraGet still yields damaged `t` from embedded body;
   Compact points at the XOR’d segment frame → loses `t` on chimera channel.
2. **F0/F3/F4 layout_direct:** Materialized format resolves from `.cmr` alone;
   Compact `ChimeraLayout::get` cannot resolve `SegmentFrame` without store pread
   → empty format-only sets (including when segment is deleted under F4).

Auth channel matches Materialized on every cell (segment authority unchanged).

## Evidence

| Artifact | Path |
|---|---|
| Equivalence JSON | `equivalence.json` |
| Failure table | `FAILURE_TABLE.md` |
| Test source | `crates/residiuum-store/tests/cse1_compact_chimera_equivalence.rs` |
| Run log | `run.log` |
| Git HEAD at run | `git-head.txt` |

## Next

**CSE-2R** was a Materialized **safety rollback** (not Compact parity).
**CSE-3** — Compact + explicit recovery code
([`CSE3_COMPACT_RECOVERY_CODE.md`](../../todo/performance-qualification/CSE3_COMPACT_RECOVERY_CODE.md)).
ETQ-2 stays paused.
