# P0 — Segment-ID reuse & immutable-media collision

Status: **accepted — packaging 0.2.1** (2026-08-04) — disposable Residiuum stores only.
No Gremlin database access, salvage, or repair in this package.

## RCA (released 0.2.0)

`residiuum-store 0.2.0` could remint an active segment id after reopen when the
process-local counter under-counted (active filenames omit the seq). That alias
could return another record’s payload and, on the next seal, overwrite sealed
authoritative media via **destination-exists → delete → rename** publication.

Stage 2h added a durable reserve-before-use allocator (`segment_seq.v1`), which
stops the happy-path remint. The P0 remained incomplete until:

1. Pre-mutation **authoritative media inventory** fails closed on collisions.
2. Every authoritative publish path uses **atomic exclusive** no-replace semantics.
3. Active/sealed (and other dual-owner) descriptor duplicates **refuse open**.
4. Index install/rebuild **does not mutate** allocator `segment_seq` (allocator /
   reservation path is sole authority; observers use `note_in_memory_high_water`).
5. Disk `pread` validates event_id, item_id, subject (and segment) before returning
   a body (closes reconstructed-locator cross-record reads).

## Atomic exclusive publish

`rename_exclusive` is crash-atomic and exclusive (P0 final gap closed 2026-08-04):

1. Dest exists + identical bytes → idempotent unlink of source.
2. Dest exists + different bytes → `SegmentIdCollision` (both preserved).
3. **Same filesystem:** platform no-replace rename (`renameat2` /
   `RENAME_NOREPLACE` on Linux; `renamex_np` / `RENAME_EXCL` on macOS) —
   atomic move with no dual-name window.
4. **Hard-link fallback** (when exclusive rename unavailable): link then unlink;
   crash between leaves same-inode aliases that
   `heal_identical_publish_aliases` collapses on open (inode identity only —
   byte-identical distinct files still refuse as collisions).
5. **Cross-device (`EXDEV`):** unique temp in dest dir → copy+verify →
   `sync_all` → no-replace publish temp→final → dir sync → unlink source.
   Never writes a partial final pathname.

Failpoints: `media.publish.after_link`, `after_create`, `partial_copy`,
`after_file_sync`, `after_dest_publish`, `before_source_unlink` (plus
`force_cross_device` / `force_hard_link` test arms).

Wired through sync/async seal finalize, summary-frame publish, pending recovery,
protected-pair pending→sealed, tier `copy_verified`, and Shadow mirror/dual-stream
publication. Sync seal fallback also stages via temp + `rename_exclusive`.

## Overwrite-on-collision paths removed

| Location | Before | After |
|---|---|---|
| `store.rs` sync/async seal publish | `remove_file(dest)` then rename / truncate-create | refuse `SegmentIdCollision` / `create_new_exclusive` |
| `seal_pipeline.rs` protected-pair finalize | delete sealed then rename pending | refuse if sealed exists |
| `seal_pipeline.rs` `publish_sealed_from_summary_frame` | delete sealed then rename | `rename_exclusive` |
| `seal_pipeline.rs` `publish_sealed_from_pending` | delete sealed then rename | `rename_exclusive` |
| `seal_pipeline.rs` `finalize_seal_authoritative` fallback | `rename(tmp, sealed)` replace | `rename_exclusive` |
| `protected_pair.rs` recover pending→sealed | delete sealed then rename | `rename_exclusive` |
| `dual_stream.rs` Shadow finalize (×2) | delete `.rsh` then rename | `rename_exclusive` |
| `wire.rs` / `qualify.rs` / `mirror.rs` Shadow publish | `write_atomic` / `fs::rename` replace | exclusive create + `rename_exclusive` |
| `tier.rs` `copy_verified` | rename replace; delete dest on hash fail | `rename_exclusive`; leave dest on mismatch |
| `store.rs` `resume_or_start_active_shard` | delete active if sealed exists; mint new | refuse open with `SegmentIdCollision` |

## Open-time collision diagnostic (example)

Planted active = sealed bytes (same descriptor id):

```text
StoreError::SegmentIdCollision {
  segment_id: <16 bytes>,
  paths: [ ".../active.residiuum", ".../segments/<hex>.residiuum", ... ],
}
```

Both files remain untouched. Index is not built from colliding media.

## Release note

**`residiuum-store 0.2.0` is unsafe for continued writes across reopen/rotation**
on stores that may have reminted segment ids or that contain colliding
authoritative media. Upgrade to a build that includes this P0 before further
writes. Existing damaged trees are **not** auto-repaired; open fails closed on
detected collisions.

Packaging **0.2.1** is the corrected emergency release. **0.2.0 remains unsafe**
(see [SECURITY_ADVISORY_SEGID_0.2.0.md](./SECURITY_ADVISORY_SEGID_0.2.0.md)).

## Evidence

```text
cargo test -p residiuum-store --features legacy-raw-store \
  --test p0_segment_id_collision -- --test-threads=1
# 22 passed (2026-08-04)

cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_segment_id_never_reuse -- --test-threads=1
# 8 passed

cargo test -p residiuum-store --features legacy-raw-store --lib media_inventory \
  -- --test-threads=1
# rename_exclusive crash-atomicity unit tests (failpoint matrix) passed
```

### 1,000-cycle release gate (archived 2026-08-04)

```text
$ P0_SEGID_CYCLES=1000 cargo test -p residiuum-store --features legacy-raw-store \
    --test p0_segment_id_collision p0_thousand_reopen_rotation_unique_ids \
    -- --test-threads=1 --nocapture
running 1 test
test p0_thousand_reopen_rotation_unique_ids ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 110.79s
```

Ordinary CI keeps the default **64** cycles inside the same test when `P0_SEGID_CYCLES` is unset.

### Publication-path + planted matrix (gap closure)

| Path / plant | Assertion |
|---|---|
| `finalize_seal_authoritative` (sync/async) | typed collision; both preserved |
| `recover_all_pending` | typed collision; both preserved |
| `publish_sealed_from_summary_frame` | typed collision; both preserved |
| protected-pair `rename_exclusive` | typed collision; both preserved |
| compaction residual + sealed | open refuses |
| tier copy + sealed | open refuses |
| `transfer_segment_to_tier` planted dest | typed collision; both preserved |
| Shadow mirror re-publish | typed collision; `.rsh` preserved |
| filename ≠ descriptor | refuse (`CorruptMeta` / collision) |
| active/pending dual claim | open refuses |
| active/sealed dual claim | open refuses |
| collision before cache-hit open | open refuses |
| collision before index rebuild | open refuses |
| wrong-record frame at locator offset | `pread_item_body_matching` refuses |

## Non-goals (honored)

- No Gremlin DB access / salvage / repair utility.
- No performance optimization.
- No weakening of collision handling to keep damaged stores writable.
- No packaging bump until principal accepts the gates above.
