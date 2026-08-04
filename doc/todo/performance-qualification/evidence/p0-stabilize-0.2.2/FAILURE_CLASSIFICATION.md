# 0.2.2 stabilization — failure classification (closed)

## Immediate (done)
- Yanked crates.io **0.2.1** (+ kept **0.2.0** yanked).
- Tag `v0.2.1` preserved immutable.

## Disk discipline
- Prefer `TMPDIR=$REPO/.tmp-test`; clean after runs.
- No 2 GiB Step 9 campaign for this correctness release.

## Full suite
- `full_suite_diskaware_v2`: **EXIT=0**, zero failed tests (2026-08-04).
- Step9 smoke: correctness asserts always; Compact ≤5% only with perf env / ≥64 MiB.

## Release
- Packaging **0.2.2** — see `../p0-release-0.2.2/SUMMARY.md`.
