//! Rebuildable current-state projection (OVERVIEW §5.5, §4.5).
//!
//! Derived only — never the sole map of surviving data.

use crate::envelope::EventKind;
use std::collections::BTreeMap;

/// Live value for a subject after applying surviving put/delete events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveValue {
    /// Payload bytes from the latest put (empty for delete tombstones that are
    /// not exposed via get).
    pub body: Vec<u8>,
    /// Item id lineage for this subject.
    pub item_id: [u8; 16],
    /// Event id of the latest event that established this state.
    pub event_id: [u8; 16],
    /// Segment that holds the establishing event.
    pub segment_id: [u8; 16],
    /// Writer-local sequence of the establishing event (diagnostic).
    pub writer_sequence: u64,
}

/// Index entry: either a live put or a delete tombstone (get returns None).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexEntry {
    /// Subject has a current payload.
    Live(LiveValue),
    /// Subject was deleted; get returns absence.
    Deleted {
        /// Item id lineage.
        item_id: [u8; 16],
        /// Delete event id.
        event_id: [u8; 16],
        /// Segment of the delete event.
        segment_id: [u8; 16],
        /// Writer sequence of the delete.
        writer_sequence: u64,
    },
}

impl IndexEntry {
    /// Current live body, if any.
    pub fn live_body(&self) -> Option<&[u8]> {
        match self {
            Self::Live(v) => Some(&v.body),
            Self::Deleted { .. } => None,
        }
    }

    /// Item id for this subject lineage.
    pub fn item_id(&self) -> [u8; 16] {
        match self {
            Self::Live(v) => v.item_id,
            Self::Deleted { item_id, .. } => *item_id,
        }
    }
}

/// Subject → current projection. Ordered map for deterministic rebuild tests.
#[derive(Debug, Clone, Default)]
pub struct PrimaryIndex {
    map: BTreeMap<Vec<u8>, IndexEntry>,
}

impl PrimaryIndex {
    /// Empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of subjects with any recorded state (including deleted).
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Number of live (non-deleted) subjects.
    pub fn live_len(&self) -> usize {
        self.live_entries().count()
    }

    /// Lookup by subject bytes.
    pub fn get(&self, subject: &[u8]) -> Option<&IndexEntry> {
        self.map.get(subject)
    }

    /// Live body only.
    pub fn get_live(&self, subject: &[u8]) -> Option<&[u8]> {
        self.map.get(subject).and_then(|e| e.live_body())
    }

    /// Apply one verified item event in recovery order.
    pub fn apply_event(
        &mut self,
        subject: Vec<u8>,
        kind: EventKind,
        body: Vec<u8>,
        item_id: [u8; 16],
        event_id: [u8; 16],
        segment_id: [u8; 16],
        writer_sequence: u64,
    ) {
        match kind {
            EventKind::Put => {
                self.map.insert(
                    subject,
                    IndexEntry::Live(LiveValue {
                        body,
                        item_id,
                        event_id,
                        segment_id,
                        writer_sequence,
                    }),
                );
            }
            EventKind::Delete => {
                self.map.insert(
                    subject,
                    IndexEntry::Deleted {
                        item_id,
                        event_id,
                        segment_id,
                        writer_sequence,
                    },
                );
            }
        }
    }

    /// Iterate live subjects only.
    pub fn live_entries(&self) -> impl Iterator<Item = (&Vec<u8>, &LiveValue)> {
        self.map.iter().filter_map(|(k, v)| match v {
            IndexEntry::Live(lv) => Some((k, lv)),
            IndexEntry::Deleted { .. } => None,
        })
    }

    /// Iterate every subject entry (live and deleted), in subject order.
    pub fn iter_all(&self) -> impl Iterator<Item = (&Vec<u8>, &IndexEntry)> {
        self.map.iter()
    }

    /// Insert a fully formed entry (used by the optional index cache loader).
    pub fn insert_entry(&mut self, subject: Vec<u8>, entry: IndexEntry) {
        self.map.insert(subject, entry);
    }

    /// Clear all entries (before rebuild).
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.map.clear();
    }
}
