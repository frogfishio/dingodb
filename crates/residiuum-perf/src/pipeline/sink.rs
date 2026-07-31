//! Bounded null/memory sink — never touches the filesystem.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkMode {
    /// Discard all bytes (null sink).
    Null,
    /// Keep a rolling digest + optional bounded ring of last N bytes.
    Memory,
}

/// Bounded in-memory sink used as the L3 destination.
#[derive(Debug, Clone)]
pub struct BoundedSink {
    mode: SinkMode,
    cap_bytes: usize,
    ring: Vec<u8>,
    total_written: u64,
    digest: Sha256,
    /// Guard: true if any FS path was ever requested (must stay false in L3).
    fs_touch_attempted: bool,
}

impl BoundedSink {
    pub fn null() -> Self {
        Self {
            mode: SinkMode::Null,
            cap_bytes: 0,
            ring: Vec::new(),
            total_written: 0,
            digest: Sha256::new(),
            fs_touch_attempted: false,
        }
    }

    pub fn memory(cap_bytes: usize) -> Self {
        Self {
            mode: SinkMode::Memory,
            cap_bytes,
            ring: Vec::with_capacity(cap_bytes.min(64 * 1024)),
            total_written: 0,
            digest: Sha256::new(),
            fs_touch_attempted: false,
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.total_written = self.total_written.saturating_add(bytes.len() as u64);
        self.digest.update(bytes);
        if matches!(self.mode, SinkMode::Memory) && self.cap_bytes > 0 {
            // Rolling: keep only the tail up to cap.
            let need = bytes.len().min(self.cap_bytes);
            let start = bytes.len() - need;
            let tail = &bytes[start..];
            if self.ring.len() + tail.len() <= self.cap_bytes {
                self.ring.extend_from_slice(tail);
            } else {
                let overflow = self.ring.len() + tail.len() - self.cap_bytes;
                if overflow < self.ring.len() {
                    self.ring.drain(0..overflow);
                    self.ring.extend_from_slice(tail);
                } else {
                    self.ring.clear();
                    let take = tail.len().min(self.cap_bytes);
                    self.ring.extend_from_slice(&tail[tail.len() - take..]);
                }
            }
        }
    }

    pub fn total_written(&self) -> u64 {
        self.total_written
    }

    pub fn digest_hex(&self) -> String {
        // Clone hasher state by finalizing a copy: re-hash ring+total for stability
        // of "same stream → same digest" without cloning Sha256 mid-stream.
        // We keep running digest via update; finalize consumes — so clone via
        // intermediate: sha2 Digest doesn't clone easily; keep second tracker.
        hex::encode(self.digest.clone().finalize())
    }

    /// L3 must never call this; tests assert it stays false.
    pub fn attempt_filesystem_write(&mut self, _path: &str) {
        self.fs_touch_attempted = true;
    }

    pub fn filesystem_touched(&self) -> bool {
        self.fs_touch_attempted
    }

    pub fn mode(&self) -> SinkMode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_sink_digests_without_retain() {
        let mut s = BoundedSink::null();
        s.write(b"abc");
        s.write(b"def");
        assert_eq!(s.total_written(), 6);
        assert!(s.ring.is_empty());
        assert!(!s.digest_hex().is_empty());
    }

    #[test]
    fn memory_sink_caps() {
        let mut s = BoundedSink::memory(4);
        s.write(b"abcdefgh");
        assert!(s.ring.len() <= 4);
        assert_eq!(s.total_written(), 8);
    }
}
