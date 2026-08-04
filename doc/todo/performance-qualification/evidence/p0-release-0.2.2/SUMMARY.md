# P0 correctness release — packaging 0.2.2

Date: 2026-08-04  
Gate: principal final 0.2.2 correctness release (no 2 GiB Step 9 campaign).

## Pre-conditions

| Check | Result |
|-------|--------|
| crates.io **0.2.1** yanked (all residiuum-* publish set) | confirmed (`yank_verify_crates_io.txt` under stabilize evidence) |
| crates.io **0.2.0** still yanked | confirmed |
| Tag `v0.2.1` immutable | `7285c1e312e46b912c10108f4acbdb244e0aa178` — do not move |

## Full `residiuum-store` suite (disk-aware)

```text
TMPDIR=$REPO/.tmp-test
cargo test -p residiuum-store --features legacy-raw-store --no-fail-fast -- --test-threads=1
```

| Run | Result | Log |
|-----|--------|-----|
| After step9 perf-gate split | **EXIT=0**, zero failed tests | `../p0-stabilize-0.2.2/full_suite_diskaware_v2.log` + `_SUMMARY.txt` |

Step 9 default smoke keeps correctness asserts; Compact ≤5% perf gate is opt-in
(`CSE3_STEP9_PERF_GATES=1` or target ≥ 64 MiB). Principal: 2 GiB Step 9 not required.

## Final P0 cells

| Cell | Result | Log |
|------|--------|-----|
| `p0_segment_id_collision` | 22/22 | `p0_segment_id_collision.log` |
| `cse3_stage2_segment_id_never_reuse` | 8/8 | `never_reuse.log` |
| `media_inventory::tests` | 9/9 | `media_inventory.log` |
| `P0_SEGID_CYCLES=1000` | ok (~98.7s) | `p0_segid_1000.log` |

## Ordinary-product smoke (CompactShadow)

`cse3_stage2_step9_product_campaign` binary (6/6): fresh CompactShadow default,
write/seal sizes, reopen+continue, P★ recovery (`recovery=true reopen=true continue=true`).

Logs: `product_smoke.log`, `product_smoke_pstar.log`.

## Residual risk

- Compact amp % on small smoke targets can exceed 5% — not a P0 correctness fail;
  large-campaign perf gate remains available via env.
- Damaged trees are not auto-repaired (FailClosed writable inventory).
- No Gremlin product DB access in this gate.

## Release actions

- Package bump **0.2.2**
- Publish residiuum-{sda,format,heap,store,client,sdk,examine,server,cli,cluster}
- Tag **`v0.2.2`** on exact green commit (never move `v0.2.1`)
- Push commit + tag

## Tagged tip

- Commit: `84a77c0262e1d6691b7190bc17749370d2240cfb`
- Tag: `v0.2.2`
- crates.io: residiuum-{sda,format,heap,store,client,sdk,examine,server,cli,cluster} **0.2.2** published (not yanked).
- `v0.2.1` unchanged at `7285c1e312e46b912c10108f4acbdb244e0aa178`.
