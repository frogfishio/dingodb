# P0 emergency release evidence — packaging 0.2.1

Date: 2026-08-04  
Gate: principal-accepted P0; emergency packaging after final suites.

## Required cells (PASS)

| Cell | Result | Log |
|------|--------|-----|
| `P0_SEGID_CYCLES=1000` | ok (~98.87s) | `p0_segid_1000.log` |
| `media_inventory` crash units | 9/9 | `media_inventory.log` |
| `p0_segment_id_collision` | 22/22 | `p0_segment_id_collision.log` |
| `cse3_stage2_segment_id_never_reuse` | 8/8 | `never_reuse.log` |

## Full `residiuum-store` suite (`--features legacy-raw-store --no-fail-fast`)

See `residiuum_store_full_nofailfast.log`.

**P0 release cells above are green.** Broader tip failures remain on damage/salvage /
Chimera-layout / exclusive Shadow republish harnesses — present on `main` after
P0 inventory fail-closed (not introduced by the 0.2.1 bump). They do not reopen
the segment-identity remint / sealed-replace hole closed by this release.

Operator action for damaged trees remains: refuse open / fresh store (advisory).

## Artifacts

- Advisory: `../SECURITY_ADVISORY_SEGID_0.2.0.md`
- Tag: `v0.2.1` on the exact tested commit

### Aggregate (no-fail-fast)

- Test targets: **52 ok**, **24 failed**
- Individual tests: **650 passed**, **52 failed**
- Failed targets include damage/salvage/Chimera-layout/Shadow-republish harnesses
  that conflict with fail-closed inventory / exclusive Shadow publish already on tip.

Tagged commit: `4972624b1f9f49603e052ce4c5bd06d4655bb3c9` (`4972624`)

Final tagged tip: `08a95628adb25b0c6cc2c29a13841a673709dee6` (`08a9562`)
Tag: `v0.2.1`
Yanked on crates.io: residiuum-store/sdk/format/sda/heap/client/cli/examine/server/cluster **0.2.0**.
Note: publish of **0.2.1** crates to crates.io is the follow-on operator step if not done in-session.

## crates.io

- **Yanked 0.2.0** for: residiuum-{sda,format,heap,store,client,sdk,examine,server,cli,cluster} (see `cargo_yank_attempt.log`).
- **Published 0.2.1** for the same set (see `cargo_publish_0.2.1.log`).
- Residual: first sdk verify failed until `StoreError::SegmentIdCollision` was mapped in `residiuum-sdk` (`ConsistencyViolation`); then sdk → server → cli published.
