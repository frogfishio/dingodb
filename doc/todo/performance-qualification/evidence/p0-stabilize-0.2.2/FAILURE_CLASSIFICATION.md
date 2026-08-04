# 0.2.2 stabilization — failure classification (in progress)

## Immediate (done)
- Yanked crates.io **0.2.1** for residiuum-{sda,format,heap,store,client,sdk,examine,server,cli,cluster}.
- Kept **0.2.0** yanked.
- Did **not** push `main` / retag `v0.2.1` (tag preserved at published tip).

## Disk discipline (principal stop — 2026-08-04)
- `/tmp` was cleared (~30 GiB agent waste). Prefer `TMPDIR=$REPO/.tmp-test` (gitignored).
- No full-suite dumps / CSE-3 step9 product campaigns until residuals are green.
- Clean `.tmp-test` after each targeted run; watch `df` + `target/` size.

## Product regressions fixed
| Failure cluster | Fix |
|---|---|
| `explicit_seal_active_still_sync_and_drains` | `seal_active` waits for protected-pair finalize (`wait_seals_applied` after submit) |
| Salvage / backup reassign FailClosed | `InventoryPolicy::TolerateUnidentified` for salvage dest + reassign restore; foreign descriptors; resume accepts foreign store_id |
| Tiering / seal_cost empty catalog | `ProtectedPairDone` calls `note_sealed_segment` |
| Compact reclaim Shadow missing | publish mirror Shadow for compact output before reclaim |
| Chimera Materialized tests under CompactShadow default | create with `RecoveryMode::Materialized` + `drain_lifecycle` for sidecar asserts |
| Shadow republish collision (same bytes) | `publish_mirror_shadow*` idempotent when intended image matches |
| DEF-022 Fast Lane under CompactShadow | ProtectedPair hits `before_authoritative_rename` / `after_authoritative_publish`; `run_seal` uses Error (worker-safe) |
| step7 smoke reminting Shadow over CompactShadow P★ | skip `write_shadow` when Shadow already verified at seal |

## Intentional CompactShadow / P0 contract updates
| Failure | Classification |
|---|---|
| Hydra/Chimera after `seal_active` without drain | CompactShadow: derived enrichment async — tests call `drain_lifecycle` |
| §16 case05/case10 writable open on planted damage/dup | FailClosed writable; inspect/salvage for survivors |
| CSE-2R Materialized embed | Fixture must create Materialized mode (not CompactShadow product default) |

## Disk-safe residual evidence (this turn)
- `media_inventory::tests` 9/9 (`disksafe_media_inventory.log`)
- `cse3_stage2_segment_id_never_reuse` 8/8 (`disksafe_never_reuse.log`) — restored after corrupted `fp_lock` injection
- `awo_crash_matrix` 6/6; `rshd0004` 16/16; mirror republish unit
- `stage_def_022` `ci_subset_failpoints_respect_reopen_invariants` green
- step7 smoke + medians: CompactShadow-aware (no remint)

## Still open
- Full `residiuum-store` suite green confirmation (disk-aware, once)
- CSE-3 **step9** product campaign (large — defer)
- Version bump to **0.2.2** + publish/tag only after green + P0 1000-cycle + collision matrices

## Progress
- Was **686 / 16 failed** after first stabilization pass.
- Residual failpoint/serialization + CompactShadow seal-matrix + step7 remint addressed this turn; **do not claim full suite green** without a fresh disk-safe full run.
- **Do not publish 0.2.2** until full suite is green + P0 matrices.
