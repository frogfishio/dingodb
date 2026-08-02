# DEF-SCAN-001 — scan-amplification + locator residual (P0)

**Status:** T1 repro + T2 soft-skip + T3 forensics/error honesty **in_review**  
**Date:** 2026-08-02  
**Board:** Feature `DEF-SCAN-001`  
**Severity:** **P0** (scan honesty; media/error taxonomy residual)

## Defect split (do not conflate)

| Layer | Condition | Status |
|-------|-----------|--------|
| **Scan amplification** | One unresolved locator → hard-abort → zero rows look empty | **Fixed T2** — soft-skip + return survivors |
| **Error mislabel** | Present media + unreadable frame reported as `SegmentNotFound` | **Fixed T3** — `PayloadPartial` when candidate file exists |
| **Organic locator failure** | Why a *live* writer index might fail resolve (dogfood) | **Forensics: media currently healthy under open_inspect** |

## Emergency fix T2 (scan)

`HeapStore::scan_collection` soft-skips `SegmentNotFound` / `TierOffline` / `PayloadPartial` / `PayloadConflict` and returns complete survivors (DEF-100 parity).

## T3 — media exam + error honesty

### Dogfood store (read-only, 2026-08-02)

Path: `~/.gremlin/store/gremlin.residiuum` (daemon may hold writer.lock).

| Observation | Value |
|-------------|--------|
| Sealed segment files | 103–104 present; chimera + seg indexes 1:1 with segments |
| `primary.idx` mtime | Can lag newest sealed segment (rate-limited cache write) |
| `open_inspect` live subjects | **98 596** |
| Resolve ok | **98 596 (100%)** |
| Resolve errors | **0** |

Command:

```text
cargo run -p residiuum-store --features legacy-raw-store \
  --example inspect_unresolved_locators -- ~/.gremlin/store/gremlin.residiuum
```

**Interpretation:** Controlled file-delete is **not** the dogfood media state. Under inspect (rebuild/load index from durable segments, no writer), **all live locators resolve**. Original `segment not found` during live exclusive-writer scan was either:

1. **Amplification** from a transient unresolvable locator in the live index (since overwritten / healed), and/or  
2. **Mislabel** of present-media frame failures as `SegmentNotFound` (T3 honesty fix), and/or  
3. A **live-only** index/seal race not visible after inspect rebuild.

No claim that the live daemon process is free of all future holes — soft-skip remains the product guard.

### Code: pread honesty

`pread_body_for_locator`: if any candidate media path exists but no verified frame is recovered → `PayloadPartial` (not `SegmentNotFound`). True absence of media for the segment id remains `SegmentNotFound`.

## Evidence

```text
cargo test -p residiuum-store --features legacy-raw-store \
  --test def_scan_001_segment_not_found
```

| Test | Role |
|------|------|
| `high_churn_exclusive_writer_scan_still_complete` | healthy multi-seal baseline |
| `missing_segment_scan_returns_survivors_not_empty_abort` | amplification guard (deleted file) |
| `present_media_unreadable_frame_is_payload_partial_not_segment_not_found` | T3 error honesty |
| `compact_reclaim_live_scan_remains_complete` | reclaim does not empty scan |

## Explicit non-claims

- No package accept; Heap not production-ready.  
- Organic **live-process** seal/index race not reproduced as a unit test.  
- No typed incomplete scan API yet (`Ok([])` still ambiguous if every key is a hole).  
- Dogfood inspect health ≠ forever-healthy live exclusive writer.
