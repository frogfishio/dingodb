# DEF-SCAN-001 — `segment not found` on heap collection scan (P0)

**Status:** labor investigation / repro **in_review** (not fixed; not package accept)  
**Date:** 2026-08-02  
**Board:** Feature `DEF-SCAN-001 — segment not found on collection scan (P0)` · task T1  
**Severity:** **P0** (storage integrity / scan honesty — exclusive-writer heaps can look empty while data remains)

## Report (external)

Gremlin dogfood (embedded exclusive writer, high Kanban churn, head ~395):

- `HeapStore::scan_collection` (via SDK / Gremlin adapter) fails with **`residiuum: segment not found`**
- Adapter stops paging → **`recovered=0`** for entity collections
- Project head + some commit point-gets still valid → product shows empty board

Source write-up (Gremlin): `docs/residiuum-bug-report-segment-not-found-2026-08-02.md` (out-of-tree).

## Residiuum-native repro

Crate test only — **`residiuum-store` + `residiuum-heap`** (no SDK, no Gremlin):

```text
cargo test -p residiuum-store --test def_scan_001_segment_not_found
```

| Test | Result (2026-08-02) | Meaning |
|------|---------------------|---------|
| `high_churn_exclusive_writer_scan_still_complete` | pass | Short multi-seal rewrite load does **not** by itself produce missing segments |
| `missing_segment_scan_hard_aborts_while_point_get_survives` | pass | **Symptom shape pinned:** after deleting one sealed segment, cohort-B point-get survives; `scan_collection` hard-aborts `SegmentNotFound` with **no rows** |
| `compact_reclaim_live_scan_remains_complete` | pass | Compact + reclaim (history-loss ack) keeps live scan complete under this harness |

## Root cause (scan path)

Physical `Store::scan_live_page` (DEF-100) treats `StoreError::SegmentNotFound` as **incomplete** and continues enumeration.

`HeapStore::scan_collection` does:

```text
for subject in index_live_after(...) {
    match self.get(&subject)? {   // ← SegmentNotFound aborts entire page
        Some(body) => push,
        None => continue,
    }
}
```

So any live index locator whose segment file is gone turns a **partial island** into a **hard empty scan** at the heap façade — exactly the dogfood UI failure mode when the first keys hit the bad locator.

## Open residual (not yet reproduced organically)

What **creates** catalog/index → missing segment under pure exclusive-writer high churn (no external `unlink`) is still open. Candidates:

1. Compaction / reclaim race or incomplete generation activate
2. Seal / pending-segment lifecycle leaving index locators without durable files
3. Primary-cache restore of stale locators after reopen
4. Crash mid-seal with index publish ahead of durable segment rename

Next labor: extend crash/failpoint matrix + long soak; then fix scan semantics (continue-with-incomplete **or** typed corruption that cannot be confused with empty) and any integrity bug that plants dead locators.

## Explicit non-claims

- Natural dogfood root-cause of *how segments disappeared* is **not** closed.
- No durable product fix in this labor slice.
- No CSQ / M2 / package accept change.
