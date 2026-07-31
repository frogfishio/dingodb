//! Store-boundary I/O instrumentation (PQH-11).
//!
//! Records **actual** write-path observations at the store seam — append,
//! durability sync, segment rotation, visibility publication, and lifecycle
//! seal — without payloads, subjects, or estimated frame sizes.
//!
//! Default off (zero cost). Enable via [`BoundaryProbe::enable`] /
//! [`crate::Store::enable_boundary_probe`] for qualification harnesses.

use crate::durability::DurabilityMode;
use serde::{Deserialize, Serialize};

/// Kind of store-boundary I/O observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryKind {
    /// Wire-encoded frame(s) appended to the active segment buffer.
    AppendEncodedFrame,
    /// Bytes flushed from the active buffer to the segment file (write_all).
    FileWrite,
    /// Full-file durability barrier (`sync_all`) on the segment file.
    FileSync,
    /// Directory sync for active-shard durability.
    DirectorySync,
    /// Segment rotated or sealed due to size / explicit seal (lifecycle).
    SegmentRotate,
    /// Visibility published into the durable projection after append (DEF-023).
    PublishVisibility,
    /// Lifecycle finalize / seal pipeline work completed for a rotated segment.
    LifecycleSeal,
}

/// One redacted boundary observation (no payload, no subject, no heap id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryEvent {
    /// Monotonic probe sequence (process-local, restarts at enable).
    pub seq: u64,
    /// Kind of store-boundary I/O observation.
    pub kind: BoundaryKind,
    /// Exact encoded frame length for appends; file bytes for FileWrite; else 0.
    pub encoded_bytes: u64,
    /// Logical payload length for appends (0 for non-append events).
    pub logical_len: u64,
    /// Segment byte offset of the frame (append/publish); 0 otherwise.
    pub offset: u64,
    /// Opaque segment generation counter (not a product identity claim).
    pub segment_gen: u32,
    /// Durability mode applied for this step when relevant.
    pub durability: String,
    /// True when this step opened/rotated a segment relative to the prior append.
    pub segment_rotate: bool,
    /// Chunked layout flag for appends.
    pub chunked: bool,
    /// Chunk count when chunked (0 otherwise).
    pub chunk_count: u32,
}

/// Bounded in-memory probe attached to a [`crate::Store`].
#[derive(Debug, Clone, Default)]
pub struct BoundaryProbe {
    enabled: bool,
    next_seq: u64,
    events: Vec<BoundaryEvent>,
    /// Soft cap to avoid unbounded growth during long qualification runs.
    max_events: usize,
    /// Running segment generation observed by the probe.
    segment_gen: u32,
}

impl BoundaryProbe {
    /// Disabled probe (default): all record methods are no-ops.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            next_seq: 0,
            events: Vec::new(),
            max_events: 256 * 1024,
            segment_gen: 0,
        }
    }

    /// Enable recording (idempotent).
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Whether recording is active.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Borrow recorded events (empty when disabled).
    pub fn events(&self) -> &[BoundaryEvent] {
        &self.events
    }

    /// Drain recorded events (leaves probe enabled).
    pub fn take_events(&mut self) -> Vec<BoundaryEvent> {
        std::mem::take(&mut self.events)
    }

    /// Clear without disabling.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    fn push(&mut self, mut ev: BoundaryEvent) {
        if !self.enabled {
            return;
        }
        if self.events.len() >= self.max_events {
            // Drop oldest half when capped (keep recent tail for plan emission).
            let drop_n = self.max_events / 2;
            self.events.drain(0..drop_n);
        }
        ev.seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        self.events.push(ev);
    }

    /// Record an append of wire-encoded frame bytes (post-append length).
    pub fn record_append(
        &mut self,
        encoded_bytes: u64,
        logical_len: u64,
        offset: u64,
        durability: DurabilityMode,
        segment_rotate: bool,
        chunked: bool,
        chunk_count: u32,
    ) {
        if segment_rotate {
            self.segment_gen = self.segment_gen.saturating_add(1);
        }
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::AppendEncodedFrame,
            encoded_bytes,
            logical_len,
            offset,
            segment_gen: self.segment_gen,
            durability: durability.as_str().into(),
            segment_rotate,
            chunked,
            chunk_count,
        });
    }

    /// Record a file write of pending segment bytes.
    pub fn record_file_write(&mut self, bytes: u64, durability: DurabilityMode) {
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::FileWrite,
            encoded_bytes: bytes,
            logical_len: 0,
            offset: 0,
            segment_gen: self.segment_gen,
            durability: durability.as_str().into(),
            segment_rotate: false,
            chunked: false,
            chunk_count: 0,
        });
    }

    /// Record a full-file sync barrier.
    pub fn record_file_sync(&mut self, durability: DurabilityMode) {
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::FileSync,
            encoded_bytes: 0,
            logical_len: 0,
            offset: 0,
            segment_gen: self.segment_gen,
            durability: durability.as_str().into(),
            segment_rotate: false,
            chunked: false,
            chunk_count: 0,
        });
    }

    /// Record a directory sync.
    pub fn record_directory_sync(&mut self) {
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::DirectorySync,
            encoded_bytes: 0,
            logical_len: 0,
            offset: 0,
            segment_gen: self.segment_gen,
            durability: DurabilityMode::Durable.as_str().into(),
            segment_rotate: false,
            chunked: false,
            chunk_count: 0,
        });
    }

    /// Record segment rotation / seal start.
    pub fn record_segment_rotate(&mut self) {
        self.segment_gen = self.segment_gen.saturating_add(1);
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::SegmentRotate,
            encoded_bytes: 0,
            logical_len: 0,
            offset: 0,
            segment_gen: self.segment_gen,
            durability: String::new(),
            segment_rotate: true,
            chunked: false,
            chunk_count: 0,
        });
    }

    /// Record durable visibility publication after append.
    pub fn record_publish(&mut self, offset: u64, durability: DurabilityMode) {
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::PublishVisibility,
            encoded_bytes: 0,
            logical_len: 0,
            offset,
            segment_gen: self.segment_gen,
            durability: durability.as_str().into(),
            segment_rotate: false,
            chunked: false,
            chunk_count: 0,
        });
    }

    /// Record lifecycle seal finalize for a rotated segment.
    pub fn record_lifecycle_seal(&mut self) {
        self.push(BoundaryEvent {
            seq: 0,
            kind: BoundaryKind::LifecycleSeal,
            encoded_bytes: 0,
            logical_len: 0,
            offset: 0,
            segment_gen: self.segment_gen,
            durability: String::new(),
            segment_rotate: false,
            chunked: false,
            chunk_count: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_is_noop() {
        let mut p = BoundaryProbe::disabled();
        p.record_append(100, 100, 0, DurabilityMode::Durable, false, false, 0);
        assert!(p.events().is_empty());
    }

    #[test]
    fn enabled_records_kinds() {
        let mut p = BoundaryProbe::disabled();
        p.enable();
        p.record_append(164, 100, 0, DurabilityMode::Buffered, false, false, 0);
        p.record_file_write(164, DurabilityMode::Buffered);
        p.record_file_sync(DurabilityMode::Durable);
        p.record_publish(0, DurabilityMode::Buffered);
        p.record_segment_rotate();
        p.record_lifecycle_seal();
        assert_eq!(p.events().len(), 6);
        assert_eq!(p.events()[0].kind, BoundaryKind::AppendEncodedFrame);
        assert_eq!(p.events()[0].encoded_bytes, 164);
        assert_eq!(p.events()[1].kind, BoundaryKind::FileWrite);
        assert_eq!(p.events()[2].kind, BoundaryKind::FileSync);
        assert_eq!(p.events()[4].kind, BoundaryKind::SegmentRotate);
        let taken = p.take_events();
        assert_eq!(taken.len(), 6);
        assert!(p.events().is_empty());
    }
}