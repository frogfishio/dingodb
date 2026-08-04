# Security / data-integrity advisory — segment-ID collision (P0)

**Advisory id:** `RESIDIUUM-2026-08-SEGID-01`  
**Severity:** High (data integrity)  
**Affected:** `residiuum-store` **0.2.0** (and any workspace crates at packaging **0.2.0** that embed that store)  
**Fixed (engineering):** tip / upcoming **0.2.2**. Packaging **0.2.1** was published then **yanked** (red full-store suite).  
**Date:** 2026-08-04  

## Summary

`residiuum-store` **0.2.0** can **reuse an active segment identity across reopen/rotation** when the process-local high-water mark under-counts (active filenames omit the sequence). That alias can:

1. Return **another record’s payload** for a subject (cross-record read).
2. On the next seal, **replace sealed authoritative media** via destination-exists → delete → rename publication.

## Impact

- **Integrity:** Silent cross-record reads and sealed-byte replacement on affected stores that continue to write across reopen/rotation.
- **Availability:** Corrected builds **refuse open** on detected authoritative collisions; they do **not** auto-repair media.
- **Confidentiality:** Not the primary axis; wrong-body returns are an integrity failure that may surface foreign payload bytes to callers.

## Affected stores are not automatically repaired

Trees that already reminted identities or contain colliding authoritative owners are **not** healed by upgrade. Operators must:

1. **Stop writes** on 0.2.0 media that may have reminted across reopen/rotation.
2. Upgrade to **≥ 0.2.2** when published. Do **not** adopt yanked **0.2.1**.
3. If open fails with `SegmentIdCollision`, treat the tree as damaged: back up what remains, salvage/inspect offline if needed, and **create a fresh store** for continued service (Gremlin / product DBs are out of scope for this advisory’s automated path).

## Fixed behavior (0.2.1)

- Durable segment-id allocator (`segment_seq.v1`) as sole reservation authority.
- Pre-mutation authoritative media inventory; **fail closed** on collisions without modifying media.
- Crash-atomic exclusive publication (platform no-replace rename; cross-device temp → sync → publish → unlink).
- Disk locator `pread` binds event_id, item_id, subject, and segment before returning a body.

## Marking 0.2.0 unsafe

**Do not use 0.2.0 for continued writes across reopen/rotation.**

- crates.io: yank `residiuum-store` **0.2.0** when registry credentials are available (`cargo yank residiuum-store --vers 0.2.0`, and other published workspace crates at 0.2.0 that expose store open/write). Until yank completes, treat 0.2.0 as **unsupported / unsafe** per this note.
- In-tree: this advisory + `SUPPORTED_VERSIONS.md` + store README status language.

## Evidence (release gate)

Archived under `doc/todo/performance-qualification/evidence/p0-release-0.2.1/`:

- `P0_SEGID_CYCLES=1000` cell
- `media_inventory` crash-atomicity unit tests
- `p0_segment_id_collision` suite
- `cse3_stage2_segment_id_never_reuse` suite
- full `residiuum-store` suite log (`--features legacy-raw-store`)

Tag the exact tested commit as `v0.2.1`.

## Withdrawal of 0.2.1

`residiuum-*` **0.2.1** was yanked on crates.io (2026-08-04). Tag `v0.2.1` is preserved as published-but-withdrawn source. Stabilization packaging is **0.2.2** after the full store suite is green.
