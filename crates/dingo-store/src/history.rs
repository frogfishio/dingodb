//! Per-subject event history (OVERVIEW §5.4, DX_SPEC §10.1).
//!
//! History is rebuilt from authoritative segment scans. Derived indexes never
//! invent events that do not exist as verified frames.

use crate::envelope::EventKind;
use crate::error::StoreError;
use crate::store::{collect_item_events_pub, cmp_disk_events_pub, DiskEventPub};
use crate::layout::StorePaths;
use dingo_format::SafetyLimits;
use std::collections::HashSet;

/// One immutable storage event for a subject key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEvent {
    /// Event kind (`put` or `delete`).
    pub kind: EventKind,
    /// Item lineage identifier.
    pub item_id: [u8; 16],
    /// Unique event identifier.
    pub event_id: [u8; 16],
    /// Segment that holds the frame.
    pub segment_id: [u8; 16],
    /// Writer-local sequence within the segment.
    pub writer_sequence: u64,
    /// Byte offset of the frame within the segment file.
    pub offset: u64,
    /// Payload body for puts (empty for deletes). May be a chunk manifest.
    pub body: Vec<u8>,
    /// Whether a hole was observed in the scan path that produced this stream
    /// (advisory; individual events remain verified).
    pub known_gap_before: bool,
}

/// Full history stream for one subject (recovery order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectHistory {
    /// Subject bytes as stored.
    pub subject: Vec<u8>,
    /// Events in recovery order (oldest first).
    pub events: Vec<HistoryEvent>,
    /// True when the salvage scan reported any hole in any scanned segment.
    /// Callers MUST NOT treat the stream as a gap-free complete history when set.
    pub has_known_holes: bool,
}

/// Collect history for `subject` by scanning all segment files.
pub fn subject_history(
    paths: &StorePaths,
    limits: SafetyLimits,
    subject: &[u8],
) -> Result<SubjectHistory, StoreError> {
    let (mut events, has_known_holes) = collect_subject_disk_events(paths, limits, subject)?;
    events.sort_by(cmp_disk_events_pub);

    let mut seen: HashSet<[u8; 16]> = HashSet::new();
    let mut out = Vec::new();
    let mut gap_before = false;
    for ev in events {
        if !seen.insert(ev.event_id) {
            // Duplicate physical copy of the same event_id: keep first.
            continue;
        }
        out.push(HistoryEvent {
            kind: ev.kind,
            item_id: ev.item_id,
            event_id: ev.event_id,
            segment_id: ev.segment_id,
            writer_sequence: ev.writer_sequence,
            offset: ev.offset,
            body: ev.body,
            known_gap_before: gap_before,
        });
        // After first event, subsequent events may sit after holes elsewhere;
        // we only set known_gap_before on the stream-level flag, not per event
        // except the stream-level has_known_holes disclosure.
        let _ = gap_before;
        gap_before = has_known_holes;
    }

    Ok(SubjectHistory {
        subject: subject.to_vec(),
        events: out,
        has_known_holes,
    })
}

fn collect_subject_disk_events(
    paths: &StorePaths,
    limits: SafetyLimits,
    subject: &[u8],
) -> Result<(Vec<DiskEventPub>, bool), StoreError> {
    let (all, has_holes) = collect_item_events_pub(paths, limits)?;
    let filtered: Vec<_> = all
        .into_iter()
        .filter(|e| e.subject.as_slice() == subject)
        .collect();
    Ok((filtered, has_holes))
}
