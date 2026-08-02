# AWO-1 residual checklist

Status: **labor floor delivered for G2 path — not package accept**  
Date: 2026-08-02  
Card: `c01e33ad-69ef-46a4-8538-46df4cda6075`

## Delivered

| Item | Evidence |
|---|---|
| ActiveSegment checkpoint/restore | `residiuum-format` unit test + store batch paths |
| Single-shard put_many persist-before-publish | `awo_persist_before_publish` |
| Parallel-cook put_many staged publish | `parallel_cook_put_many_roundtrip` |
| Multi-shard all-or-nothing publish | `multi_shard_put_many_persist_fail_publishes_nothing` + stage_def_096 |
| Failpoints (install/persist/publish/complete) | wired in store; plan name set in `AWO_FAILPOINTS` |
| Writer poison on short write | `awo_writer_poisoned`, `AdaptiveWriterPoisoned` |
| put/delete refuse when poisoned | `awo_direct_writer_lease`, `awo_partial_write_recovery` |
| Publication failpoint | `awo_publication_failure` |
| Partial-write recovery posture | poison + reopen clears (new handle) |

## Plan failpoints coverage

| Name | Hit site |
|---|---|
| awo.reserve.after | Deferred (coordinator AWO-2/3) — name reserved |
| awo.cook.before/after | Deferred (persistent cooker AWO-2) — name reserved |
| awo.install.frame.before/after | Single-shard + parallel-cook install |
| awo.persist.before/after_write/after_sync | `finish_staged_batch_persist_publish` |
| awo.publish.before/after | Staged finish + multi-shard publish |
| awo.complete.before | Staged finish + multi-shard complete |

## Explicit residuals (not closed by this labor)

1. **Full AdaptiveWriteLease** — fences direct `Store::put` while AWO runtime owns the store (AWO-3 Static Arbiter). Poison bit is the AWO-1 precursor only.
2. **Coordinator BatchReservation live path** — types scaffolded in `adaptive_write/persist.rs`; not yet the sole mutation authority.
3. **awo.reserve.* / awo.cook.*** live hits — require cooker/coordinator (AWO-2).
4. **Multi-process crash matrix cells** for every failpoint — expand under AWO-6 / CSQ harness reuse.
5. **Package accept** — principal/process only.

## Exit commands re-run

```bash
cargo test -p residiuum-store --features legacy-raw-store --test awo_persist_before_publish
cargo test -p residiuum-store --features legacy-raw-store --test awo_partial_write_recovery
cargo test -p residiuum-store --features legacy-raw-store --test awo_publication_failure
cargo test -p residiuum-store --features legacy-raw-store --test awo_direct_writer_lease
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_096_sharded_writers
bash scripts/verify-awo.sh
```

## Next package

**AWO-2** — persistent cooker + credits + ordered ready (depends AWO-1 labor floor; formal package accept optional per principal award).
