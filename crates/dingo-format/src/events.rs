//! Duplicate and conflicting event-identifier analysis (FORMAT_SPEC §9).
//!
//! Physical replicas may share an event identifier with identical bytes.
//! Differing envelope or body for the same identifier is a conflict: both
//! survive; the recovery result is `conflicting`. Encounter order alone MUST
//! NOT pick a winner.

use crate::frame::DecodedFrame;
use std::collections::BTreeMap;

/// How verified frames that share an event identifier relate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventIdOutcome {
    /// Exactly one verified frame for this identifier.
    Unique {
        /// Physical start offset of the frame.
        offset: u64,
        /// Frame contents.
        frame: DecodedFrame,
    },
    /// Two or more byte-identical frames (replicas) for this identifier.
    Replicas {
        /// Offsets of each replica (ascending).
        offsets: Vec<u64>,
        /// Shared frame contents.
        frame: DecodedFrame,
    },
    /// Same event identifier, differing envelope and/or body.
    Conflicting {
        /// Each physical occurrence (offset + frame), ascending by offset.
        occurrences: Vec<(u64, DecodedFrame)>,
    },
}

impl EventIdOutcome {
    /// Event identifier for this group.
    pub fn event_id(&self) -> [u8; 16] {
        match self {
            EventIdOutcome::Unique { frame, .. } | EventIdOutcome::Replicas { frame, .. } => {
                frame.header.event_id
            }
            EventIdOutcome::Conflicting { occurrences } => occurrences[0].1.header.event_id,
        }
    }

    /// Whether this outcome is a conflict (FORMAT_SPEC §9).
    pub fn is_conflicting(&self) -> bool {
        matches!(self, EventIdOutcome::Conflicting { .. })
    }
}

/// Content key used to decide replica vs conflict: envelope + body.
fn content_key(frame: &DecodedFrame) -> (Vec<u8>, Vec<u8>) {
    (frame.envelope.clone(), frame.body.clone())
}

/// Group verified frames by `event_id` (FORMAT_SPEC §9).
///
/// Input is typically the verified frames from a scan report. Frame kinds are
/// not filtered here; callers may restrict to item/chunk kinds first.
pub fn group_by_event_id(
    frames: impl IntoIterator<Item = (u64, DecodedFrame)>,
) -> Vec<EventIdOutcome> {
    let mut by_id: BTreeMap<[u8; 16], Vec<(u64, DecodedFrame)>> = BTreeMap::new();
    for (offset, frame) in frames {
        by_id
            .entry(frame.header.event_id)
            .or_default()
            .push((offset, frame));
    }

    let mut out = Vec::with_capacity(by_id.len());
    for (_id, mut group) in by_id {
        group.sort_by_key(|(o, _)| *o);
        debug_assert!(!group.is_empty());
        if group.len() == 1 {
            let (offset, frame) = group.pop().unwrap();
            out.push(EventIdOutcome::Unique { offset, frame });
            continue;
        }

        let first_key = content_key(&group[0].1);
        let all_same = group.iter().all(|(_, f)| content_key(f) == first_key);
        if all_same {
            let offsets = group.iter().map(|(o, _)| *o).collect();
            let frame = group.into_iter().next().unwrap().1;
            out.push(EventIdOutcome::Replicas { offsets, frame });
        } else {
            out.push(EventIdOutcome::Conflicting { occurrences: group });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{encode_frame, FrameHeader, FrameParts};
    use crate::kinds::FrameKind;
    use crate::limits::SafetyLimits;
    use crate::scan::scan_forward;

    fn item(event: u8, body: &[u8]) -> Vec<u8> {
        let mut event_id = [0u8; 16];
        event_id[0] = event;
        encode_frame(&FrameParts {
            header: FrameHeader::new_draft(FrameKind::ItemEvent, 0, body.len() as u64, event_id),
            envelope: vec![],
            body: body.to_vec(),
        })
        .unwrap()
    }

    #[test]
    fn unique_and_replicas_and_conflict() {
        // Two replicas of body "same" plus a conflicting body for event 1,
        // and a unique event 2.
        let mut buf = item(1, b"same");
        buf.extend_from_slice(&item(1, b"same"));
        buf.extend_from_slice(&item(1, b"other"));
        buf.extend_from_slice(&item(2, b"solo"));

        let report = scan_forward(&buf, SafetyLimits::default());
        let frames: Vec<_> = report
            .verified_frames()
            .map(|(o, f)| (o, f.clone()))
            .collect();
        let groups = group_by_event_id(frames);
        assert_eq!(groups.len(), 2);

        let g1 = groups.iter().find(|g| g.event_id()[0] == 1).unwrap();
        assert!(g1.is_conflicting());

        let g2 = groups.iter().find(|g| g.event_id()[0] == 2).unwrap();
        assert!(matches!(g2, EventIdOutcome::Unique { .. }));
    }

    #[test]
    fn pure_replicas() {
        let mut buf = item(9, b"x");
        buf.extend_from_slice(&item(9, b"x"));
        let report = scan_forward(&buf, SafetyLimits::default());
        let frames: Vec<_> = report
            .verified_frames()
            .map(|(o, f)| (o, f.clone()))
            .collect();
        let groups = group_by_event_id(frames);
        assert_eq!(groups.len(), 1);
        match &groups[0] {
            EventIdOutcome::Replicas { offsets, frame } => {
                assert_eq!(offsets.len(), 2);
                assert_eq!(frame.body, b"x");
            }
            other => panic!("expected replicas, got {other:?}"),
        }
    }
}
