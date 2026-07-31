//! Adaptive buffered vs direct/async I/O selection (FINAL DESIGN §6).
//!
//! Does **not** assume `io_uring` or O_DIRECT is always faster. Chooses a path
//! from transfer size and locality so the store can register buffers and queues
//! later without rewriting callers.

/// Selected I/O strategy for one transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum IoPath {
    /// Buffered I/O — tiny or highly cached access; page cache is a feature.
    Buffered = 1,
    /// Direct I/O — controlled large reads/writes that should skip page cache.
    Direct = 2,
    /// Async submission (e.g. io_uring) with registered buffers — batched work.
    AsyncDirect = 3,
    /// Async but buffered — concurrent small reads without pinning DMA buffers.
    AsyncBuffered = 4,
}

impl IoPath {
    /// Wire / diagnostic discriminant.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Human-readable label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Buffered => "buffered",
            Self::Direct => "direct",
            Self::AsyncDirect => "async_direct",
            Self::AsyncBuffered => "async_buffered",
        }
    }
}

/// Hints for path selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoHints {
    /// Transfer size in bytes (one read or write).
    pub transfer_bytes: u64,
    /// Caller expects the data is already hot in page cache / RAM.
    pub likely_cached: bool,
    /// Caller can batch many independent operations.
    pub batchable: bool,
    /// Platform / build has a working async submission path (io_uring etc.).
    pub async_available: bool,
    /// Platform allows O_DIRECT for this file.
    pub direct_available: bool,
}

impl Default for IoHints {
    fn default() -> Self {
        Self {
            transfer_bytes: 0,
            likely_cached: false,
            batchable: false,
            async_available: false,
            direct_available: false,
        }
    }
}

/// Thresholds for adaptive selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoSelectOptions {
    /// Transfers ≤ this use buffered paths unless forced otherwise.
    pub buffered_max: u64,
    /// Transfers ≥ this prefer direct I/O when available and not cached.
    pub direct_min: u64,
}

impl Default for IoSelectOptions {
    fn default() -> Self {
        Self {
            // ~ one typical micro-page slot or small page.
            buffered_max: 32 * 1024,
            // large container / value-log slab.
            direct_min: 256 * 1024,
        }
    }
}

/// Select buffered vs direct and sync vs async from size and locality.
pub fn select_io_path(hints: &IoHints, opts: &IoSelectOptions) -> IoPath {
    let small = hints.transfer_bytes <= opts.buffered_max;
    let large = hints.transfer_bytes >= opts.direct_min;
    let want_direct = large && !hints.likely_cached && hints.direct_available;
    let want_async = hints.batchable && hints.async_available;

    match (want_direct, want_async, small, hints.likely_cached) {
        // Hot tiny/medium: stay buffered (cache is correct).
        (_, _, _, true) if !large => {
            if want_async {
                IoPath::AsyncBuffered
            } else {
                IoPath::Buffered
            }
        }
        (true, true, _, _) => IoPath::AsyncDirect,
        (true, false, _, _) => IoPath::Direct,
        (false, true, _, _) => IoPath::AsyncBuffered,
        _ => IoPath::Buffered,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiny_cached_stays_buffered() {
        let path = select_io_path(
            &IoHints {
                transfer_bytes: 128,
                likely_cached: true,
                batchable: true,
                async_available: true,
                direct_available: true,
            },
            &IoSelectOptions::default(),
        );
        assert_eq!(path, IoPath::AsyncBuffered);
    }

    #[test]
    fn large_cold_batch_uses_async_direct() {
        let path = select_io_path(
            &IoHints {
                transfer_bytes: 1024 * 1024,
                likely_cached: false,
                batchable: true,
                async_available: true,
                direct_available: true,
            },
            &IoSelectOptions::default(),
        );
        assert_eq!(path, IoPath::AsyncDirect);
    }

    #[test]
    fn large_without_async_uses_direct() {
        let path = select_io_path(
            &IoHints {
                transfer_bytes: 1024 * 1024,
                likely_cached: false,
                batchable: false,
                async_available: false,
                direct_available: true,
            },
            &IoSelectOptions::default(),
        );
        assert_eq!(path, IoPath::Direct);
    }

    #[test]
    fn medium_default_buffered() {
        let path = select_io_path(
            &IoHints {
                transfer_bytes: 8 * 1024,
                ..Default::default()
            },
            &IoSelectOptions::default(),
        );
        assert_eq!(path, IoPath::Buffered);
    }
}
