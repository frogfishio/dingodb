# DEF-SCAN-001 — structured locator faults + honest heap scan pages (P0)

**Status:** T7–T8 **accepted (labor)**; T9–T11 **accepted (blocker #5 closure, labor only)**; T12 **in_review** (`scan_collection_page` + fail-closed legacy `scan_collection` → `Vec`); package **not** accepted  
**Date:** 2026-08-02  
**Board:** Feature `DEF-SCAN-001`  
**Severity:** **P0**

## Rejected claims (do not reassert)

1. **`PayloadPartial` is not a bucket for locator damage.** Bad frame offset, frame verify failure, and segment-id mismatch are **distinct** kinds.
2. **Soft-skip into a plain `Vec` is unsafe.** Hiding holes makes `Ok([])` look like an empty collection. Scan results must surface incompleteness in the type.
3. **Unit locator error variants are not enough.** Category-only errors lose segment id, offset, path, file length, and underlying I/O/verify cause — field diagnosis becomes impossible.

## T5 posture (supersedes T4 unit variants)

### Structured `StoreError::LocatorFault(Box<LocatorFault>)`

| `LocatorFaultKind` | Meaning |
|--------------------|---------|
| `SegmentNotFound` | No named media for segment id (and salvage failed) |
| `OffsetInvalid` | Offset past EOF / frame spans past end |
| `FrameVerifyFailed` | Checksum / framing verify failed |
| `SegmentIdMismatch` | Envelope segment id ≠ index locator |

Each fault carries field diagnostics:

- `segment_id`, `frame_offset` (from index locator)
- `path`, `file_len` (when a media file was examined)
- `observed_segment_id` (mismatch)
- `cause` (I/O / verify detail string)

Legacy unit `StoreError::SegmentNotFound` remains for non-resolve call sites (e.g. tier registry); resolve paths prefer structured `LocatorFault`.

Named media is tried first; if present but unreadable → that distinct kind with path/len/cause.  
Only when **no** named media exists is salvage attempted; salvage failures do **not** re-label a missing segment as offset-invalid on another file.

### Honest scan page

`HeapStore::scan_collection_page` → `CollectionScanPage` (preferred):

- `entries` — complete rows  
- `incomplete` — holes with `CollectionScanHoleReason` **and** optional `locator: Option<LocatorFault>`  
- `examined` — subjects walked (complete + holes)  
- `examine_budget` — `limit * 8` clamped to `[limit, 4096]` so hole-only collections cannot unbounded-walk  
- `complete` — true only when `incomplete` is empty  
- `is_empty_live()` — true only when both empty  

Legacy `HeapStore::scan_collection` → `Result<Vec<(Vec<u8>, Vec<u8>)>, StoreError>`:
returns `page.entries` only when `page.complete`; **any hole hard-fails** as
`StoreError` (first incomplete via `CollectionScanHole::to_store_error`). Does
**not** soft-skip corruption into `Ok(partial survivors)`. New code must use
`scan_collection_page` for honest incomplete pages.

Remote scan JSON (`hole_to_json`) includes `reason` plus `segment_id`, `frame_offset`, `path`, `file_len`, `observed_segment_id`, `cause` when present.

### Wire pagination (T6 / blocker #3)

Op **115** `scan_json` returns:

- `has_more`
- **`next_after_key`** = `page.last_key` (last **examined** key: complete row **or** hole)

Clients must **not** resume from the last successful row alone — a hole may have been examined after it. SDK `RemoteHeap::scan_json` → `ScanJsonWirePage`; `CollectionClient::scan_json` remote path uses wire `next_after_key` / `has_more` / holes.

### Cursor UTF-8 (T7)

Wire product keys and `next_after_key` are **exact UTF-8**. Never `from_utf8_lossy` for a cursor (replacement would resume at the wrong key). If `has_more` and `last_key` is non-UTF-8, or a hole key is non-UTF-8, the op returns **`data_damaged`**.

### Required wire fields + invariants (T8 / blocker #4)

Op **115** result **requires**:

| Field | Rule |
|-------|------|
| `rows` | array |
| `incomplete` | array (empty allowed) |
| `coverage_complete` | bool; ⇔ `incomplete` empty |
| `has_more` | bool |
| `exhausted` | bool; **must** `== !has_more` |
| `next_after_key` | string or null; non-null **iff** `has_more` |

SDK `parse_scan_json_wire` rejects missing fields and contradictions (`ProtocolViolation`).

### Secondary-index safety

#### Query-time materialization (T9 — useful, not the original defect)

Op **116** `find` tracks incompleteness when **candidates are already in the index**.

#### Construction-time omission (T10 — true blocker #5)

```text
document B has an unresolved locator
→ index build silently omits B
→ index marked Ready + complete_coverage
→ later query returns zero candidates for B
→ no candidate exists to generate a hole
→ false authoritative absence
```

**Required:** heap `create_index` / `fill_index_from_collection` must set `saw_incomplete` when scan pages report holes, and **`mark_partial`** (not `mark_ready`) so `may_prove_absence()` is false. Resume uses `page.last_key` (examined, including holes).

Legacy flat-SDK build already used `scan_live_bodies_for_build` + Partial on incomplete; heap path was the gap.

#### Partial hit lists are not exclusive (T11 residual)

Even after Partial (not Ready):

```text
A matches x and was indexed
B matches x but omitted by damaged build
lookup(x) returns [A] only
materialize A → coverage looks complete
B silently omitted
```

**Required:** exclusive index paths (`lookup_index_keys`, SDK `try_index_lookup`) use only `may_supply_exclusive_candidates()` = Ready+`complete_coverage`. Partial is skipped even for non-empty hits; caller falls through to scan (holes + survivors).

Physical `IncompleteReason` still maps the distinct locator kinds.

## Evidence

```text
cargo test -p residiuum-store --features legacy-raw-store \
  --test def_scan_001_segment_not_found
# 11/11
```

Tests assert locator diagnostics, multipage `last_key`, shared `from_error`, index build Partial, Partial never exclusive lookup, complete-path `scan_collection` == page.entries, and fail-closed legacy scan on holes.

### API compatibility (T12)

```rust
// Preferred (explicit incomplete page)
HeapStore::scan_collection_page(...) -> Result<CollectionScanPage, StoreError>

// Legacy wrapper — Vec type, fail-closed behavior
HeapStore::scan_collection(...) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StoreError>
// let page = self.scan_collection_page(...)?;
// if let Some(hole) = page.incomplete.first() {
//     return Err(hole.to_store_error());
// }
// Ok(page.entries)
```

Dogfood inspect (T3) remains informative: durable media currently 100% resolve under `open_inspect`; that does **not** license silent soft-skip.

## Explicit non-claims

- No package accept.  
- Live exclusive-writer seal/index race not unit-reproduced.  
- Wire/SDK product language may still evolve around incomplete scan semantics.