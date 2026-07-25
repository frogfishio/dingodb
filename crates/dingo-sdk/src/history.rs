//! Per-key history projection (DX_SPEC §10.1, Stage 6).

use crate::error::Error;
use crate::value::{decode_bytes, decode_json, TAG_BYTES, TAG_JSON};
use dingo_store::{EventKind, HistoryEvent, SubjectHistory};
use serde_json::Value as JsonValue;

/// One application-visible history version for a collection key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Event kind name (`put` or `delete`).
    pub kind: &'static str,
    /// Hex event id.
    pub event_id: String,
    /// Hex item lineage id.
    pub item_id: String,
    /// Hex segment id.
    pub segment_id: String,
    /// JSON document for put events that stored JSON; `None` for deletes/bytes.
    pub json: Option<JsonValue>,
    /// Raw logical body when available (typed store body for puts).
    pub body: Option<Vec<u8>>,
    /// Whether a known salvage hole was observed before this event in the stream.
    pub known_gap_before: bool,
}

/// History stream for one key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHistory {
    /// Application key.
    pub key: String,
    /// Versions oldest-first.
    pub versions: Vec<Version>,
    /// True when any hole was observed while scanning segments.
    pub has_known_holes: bool,
}

impl KeyHistory {
    pub(crate) fn from_store(key: String, hist: SubjectHistory) -> Result<Self, Error> {
        let mut versions = Vec::with_capacity(hist.events.len());
        for ev in hist.events {
            versions.push(project_event(ev)?);
        }
        Ok(Self {
            key,
            versions,
            has_known_holes: hist.has_known_holes,
        })
    }
}

fn project_event(ev: HistoryEvent) -> Result<Version, Error> {
    let kind = match ev.kind {
        EventKind::Put => "put",
        EventKind::Delete => "delete",
    };
    let (json, body) = match ev.kind {
        EventKind::Delete => (None, None),
        EventKind::Put => {
            // Chunk manifests are not application JSON; surface as body only.
            if dingo_store::is_chunk_manifest(&ev.body) {
                (None, Some(ev.body))
            } else if ev.body.first() == Some(&TAG_JSON) {
                let j = decode_json(&ev.body)?;
                (Some(j), Some(ev.body))
            } else if ev.body.first() == Some(&TAG_BYTES) {
                let _ = decode_bytes(&ev.body)?;
                (None, Some(ev.body))
            } else {
                (None, Some(ev.body))
            }
        }
    };
    Ok(Version {
        kind,
        event_id: hex16(&ev.event_id),
        item_id: hex16(&ev.item_id),
        segment_id: hex16(&ev.segment_id),
        json,
        body,
        known_gap_before: ev.known_gap_before,
    })
}

fn hex16(id: &[u8; 16]) -> String {
    dingo_store::hex16(id)
}
