//! Opt-in active-segment growth policy (product path).
//!
//! Diagnostic pre-touch / bulk-zero spikes showed that paying first-touch
//! *before* hot-path appends can lift Mode A thr vs grow-on-append (see
//! `doc/todo/performance-qualification/FIFTY_TO_TEN.md`). Default remains
//! [`SegmentGrowthPolicy::GrowOnAppend`]. Enabling watermark changes space
//! amplification and setup cost; it does **not** change CSQ durability labels.
//! Do not cite withdrawn diag ~32k figures as product thr.
//!
//! Product watermark zeros via a **background runway preparer** (not on the put
//! path). Puts only consume already-ready runway and fail closed if empty.
//! Capacity is a **host knob**, not a fixed ½ GiB religion: small stores may
//! never need more than tens of MiB reserved; large ones may want multi‑GiB
//! or 10 GiB extend steps. See
//! `doc/todo/performance-qualification/PRINCIPAL_STEER_WM_CAPACITY_CONFIGURABLE.md`.
//!
//! [`ensure_zero_watermark`] remains for create/bootstrap and diagnostic sinks.

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

use crate::error::StoreError;

/// Default reserved capacity for [`SegmentGrowthPolicy::watermark_default`] (64 MiB).
///
/// Deliberately modest: many DBs never fill ½ GiB. Hosts that need larger
/// runway pass [`SegmentGrowthPolicy::watermark`] with multi‑GiB values.
pub const WATERMARK_DEFAULT_CAPACITY_BYTES: u64 = 64 * 1024 * 1024;

/// Default zero-runway step for [`SegmentGrowthPolicy::watermark_default`] (64 MiB).
pub const WATERMARK_DEFAULT_CHUNK_BYTES: u64 = 64 * 1024 * 1024;

/// How the store grows active segment files on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SegmentGrowthPolicy {
    /// Append grows the file on demand (historic default).
    #[default]
    GrowOnAppend,
    /// OS block reserve + background ahead-of-write zero (opt-in).
    ///
    /// Not default-on: reserves [`Self::Watermark::capacity_bytes`] per active
    /// segment. A background preparer zeros [`Self::Watermark::chunk_bytes`] at
    /// a time ahead of the write head; the put path never bulk-zeros and fails
    /// closed if runway is empty. Both fields are host-configurable
    /// (64 MiB … multi‑GiB). Durability receipts stay Buffered/Durable as before.
    Watermark {
        /// Logical file capacity to reserve (`set_len` + OS preallocate).
        capacity_bytes: u64,
        /// Zero runway chunk size ahead of the write head.
        chunk_bytes: u64,
    },
}

impl SegmentGrowthPolicy {
    /// Build a watermark policy with explicit capacity and zero-chunk sizes.
    ///
    /// `capacity_bytes` may be 64 MiB for small DBs or 10 GiB+ when the host
    /// wants large extend steps. `chunk_bytes` is clamped to `capacity_bytes`
    /// at use sites via [`Self::initial_zeroed_thru`] / ensure helpers.
    pub fn watermark(capacity_bytes: u64, chunk_bytes: u64) -> Self {
        Self::Watermark {
            capacity_bytes,
            chunk_bytes,
        }
    }

    /// Product default watermark knobs: 64 MiB capacity, 64 MiB zero chunks.
    ///
    /// Not locked to the historical 512 MiB spike capacity — that size remains
    /// available via [`Self::watermark`].
    pub fn watermark_default() -> Self {
        Self::watermark(
            WATERMARK_DEFAULT_CAPACITY_BYTES,
            WATERMARK_DEFAULT_CHUNK_BYTES,
        )
    }

    /// Bytes known bulk-zeroed after create-time setup (0 for grow-on-append).
    pub fn initial_zeroed_thru(self) -> u64 {
        match self {
            Self::GrowOnAppend => 0,
            Self::Watermark {
                capacity_bytes,
                chunk_bytes,
            } => chunk_bytes.min(capacity_bytes),
        }
    }

    /// True when watermark growth is selected.
    pub fn is_watermark(self) -> bool {
        matches!(self, Self::Watermark { .. })
    }
}

/// Apply create-time watermark setup to a newly opened active segment file.
pub(crate) fn prepare_active_file(
    file: &mut File,
    policy: SegmentGrowthPolicy,
) -> Result<(), StoreError> {
    let SegmentGrowthPolicy::Watermark {
        capacity_bytes,
        chunk_bytes,
    } = policy
    else {
        return Ok(());
    };
    if capacity_bytes == 0 || chunk_bytes == 0 {
        return Err(StoreError::CorruptMeta(
            "segment growth watermark requires capacity_bytes>0 and chunk_bytes>0",
        ));
    }
    os_preallocate(file, capacity_bytes)?;
    file.set_len(capacity_bytes)?;
    let first = chunk_bytes.min(capacity_bytes);
    bulk_zero_range(file, 0, first)?;
    file.seek(SeekFrom::Start(0))?;
    Ok(())
}

/// Extend bulk-zero through `need_thru` in `chunk_bytes` steps (put-path amortize).
pub(crate) fn ensure_zero_watermark(
    file: &mut File,
    zeroed_thru: &mut u64,
    need_thru: u64,
    capacity_bytes: u64,
    chunk_bytes: u64,
) -> Result<(), StoreError> {
    if chunk_bytes == 0 {
        return Err(StoreError::CorruptMeta(
            "segment growth watermark chunk_bytes must be > 0",
        ));
    }
    while *zeroed_thru < need_thru && *zeroed_thru < capacity_bytes {
        let end = zeroed_thru.saturating_add(chunk_bytes).min(capacity_bytes);
        bulk_zero_range(file, *zeroed_thru, end)?;
        *zeroed_thru = end;
    }
    Ok(())
}

/// Write zeros across `[start, end)` in 1 MiB chunks.
pub(crate) fn bulk_zero_range(file: &mut File, start: u64, end: u64) -> Result<(), StoreError> {
    if end <= start {
        return Ok(());
    }
    let chunk = vec![0u8; 1024 * 1024];
    let mut off = start;
    while off < end {
        let n = ((end - off) as usize).min(chunk.len());
        file.seek(SeekFrom::Start(off))?;
        file.write_all(&chunk[..n])?;
        off = off.saturating_add(n as u64);
    }
    Ok(())
}

/// Platform physical block reserve. macOS: `F_PREALLOCATE`; Linux: `posix_fallocate`.
pub(crate) fn os_preallocate(file: &File, bytes: u64) -> Result<(), StoreError> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::io::AsRawFd;
        #[repr(C)]
        struct FStore {
            fst_flags: u32,
            fst_posmode: i32,
            fst_offset: i64,
            fst_length: i64,
            fst_bytesalloc: i64,
        }
        const F_PREALLOCATE: i32 = 42;
        const F_ALLOCATECONTIG: u32 = 0x0000_0002;
        const F_ALLOCATEALL: u32 = 0x0000_0004;
        const F_PEOFPOSMODE: i32 = 3;
        extern "C" {
            fn fcntl(fd: i32, cmd: i32, ...) -> i32;
        }
        let fd = file.as_raw_fd();
        let mut store = FStore {
            fst_flags: F_ALLOCATECONTIG,
            fst_posmode: F_PEOFPOSMODE,
            fst_offset: 0,
            fst_length: bytes as i64,
            fst_bytesalloc: 0,
        };
        let rc = unsafe { fcntl(fd, F_PREALLOCATE, &mut store as *mut FStore) };
        if rc != 0 {
            store.fst_flags = F_ALLOCATEALL;
            store.fst_bytesalloc = 0;
            let rc2 = unsafe { fcntl(fd, F_PREALLOCATE, &mut store as *mut FStore) };
            if rc2 != 0 {
                return Err(StoreError::Io(std::io::Error::last_os_error()));
            }
        }
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        extern "C" {
            fn posix_fallocate(fd: i32, offset: i64, len: i64) -> i32;
        }
        let rc = unsafe { posix_fallocate(file.as_raw_fd(), 0, bytes as i64) };
        if rc != 0 {
            return Err(StoreError::Io(std::io::Error::from_raw_os_error(rc)));
        }
        return Ok(());
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (file, bytes);
        Err(StoreError::CorruptMeta(
            "segment growth watermark preallocate unsupported on this OS",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watermark_default_is_modest_and_configurable() {
        match SegmentGrowthPolicy::watermark_default() {
            SegmentGrowthPolicy::Watermark {
                capacity_bytes,
                chunk_bytes,
            } => {
                assert_eq!(capacity_bytes, WATERMARK_DEFAULT_CAPACITY_BYTES);
                assert_eq!(chunk_bytes, WATERMARK_DEFAULT_CHUNK_BYTES);
                assert_eq!(capacity_bytes, 64 * 1024 * 1024);
                assert_eq!(
                    SegmentGrowthPolicy::watermark_default().initial_zeroed_thru(),
                    64 * 1024 * 1024
                );
            }
            SegmentGrowthPolicy::GrowOnAppend => panic!("expected watermark"),
        }
        // Large-DB host knobs stay first-class (e.g. 10 GiB capacity).
        let big = SegmentGrowthPolicy::watermark(10 * 1024 * 1024 * 1024, 1024 * 1024 * 1024);
        match big {
            SegmentGrowthPolicy::Watermark {
                capacity_bytes,
                chunk_bytes,
            } => {
                assert_eq!(capacity_bytes, 10 * 1024 * 1024 * 1024);
                assert_eq!(chunk_bytes, 1024 * 1024 * 1024);
            }
            SegmentGrowthPolicy::GrowOnAppend => panic!("expected watermark"),
        }
    }

    #[test]
    fn grow_on_append_is_default() {
        assert_eq!(
            SegmentGrowthPolicy::default(),
            SegmentGrowthPolicy::GrowOnAppend
        );
        assert!(!SegmentGrowthPolicy::default().is_watermark());
    }
}
