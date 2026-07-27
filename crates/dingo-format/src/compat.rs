//! Wire format reader/writer compatibility matrix (DEF-052 / FORMAT_SPEC §12).
//!
//! Major versions may change framing semantics. Minor versions may add kinds,
//! flags, or envelope fields while preserving the ability of an older
//! same-major reader to locate, bound, verify, and retain unknown frames.
//!
//! This module is the **declared support window** for the current build. It
//! does not freeze the draft wire (`WIRE_PROFILE_LABEL`); freeze remains DEF-053.

use crate::frame::{WIRE_MAJOR, WIRE_MINOR};
use crate::WIRE_PROFILE_LABEL;

/// Support status for a wire major generation in this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireSupportStatus {
    /// This build writes this generation and fully reads it.
    Current,
    /// This build can read (locate/bound/verify/retain) but does not write it.
    ReadOnly,
    /// Still readable but scheduled for removal after a support window.
    Deprecated,
    /// Not supported: frames must be preserved as opaque evidence only.
    Unsupported,
}

/// One entry in the reader/writer matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireSupportEntry {
    /// Wire major version.
    pub major: u8,
    /// Lowest minor this entry covers (inclusive).
    pub min_minor: u8,
    /// Highest minor this entry covers (inclusive). `None` = open-ended.
    pub max_minor: Option<u8>,
    /// Whether this build can fully interpret frames of this major.
    pub can_read: bool,
    /// Whether this build emits this major on encode.
    pub can_write: bool,
    /// Lifecycle status.
    pub status: WireSupportStatus,
}

/// Wire majors this build can fully read (locate, bound, verify, retain).
///
/// Adjacent-generation dual-read is required before a new major is introduced
/// as a writer (FORMAT_SPEC §12, DEF-052).
pub const SUPPORTED_READER_MAJORS: &[u8] = &[WIRE_MAJOR];

/// Wire major this build writes on encode.
pub const WRITER_WIRE_MAJOR: u8 = WIRE_MAJOR;

/// Wire minor this build writes on encode.
pub const WRITER_WIRE_MINOR: u8 = WIRE_MINOR;

/// Declared reader/writer matrix for this build.
///
/// Today only major `1` (draft) is current. When a future major is introduced,
/// keep the prior major as [`WireSupportStatus::ReadOnly`] or
/// [`WireSupportStatus::Deprecated`] until the support window ends.
pub fn wire_compat_matrix() -> &'static [WireSupportEntry] {
    // Static table so operators and migration preflight share one source of truth.
    const MATRIX: &[WireSupportEntry] = &[WireSupportEntry {
        major: WIRE_MAJOR,
        min_minor: 0,
        max_minor: None,
        can_read: true,
        can_write: true,
        status: WireSupportStatus::Current,
    }];
    MATRIX
}

/// Whether this build can fully interpret frames of the given major.
pub fn wire_reader_supports(major: u8) -> bool {
    SUPPORTED_READER_MAJORS.contains(&major)
}

/// Whether this build encodes frames with the given major.
pub fn wire_writer_emits(major: u8) -> bool {
    major == WRITER_WIRE_MAJOR
}

/// Look up a matrix entry for `major` (first match).
pub fn wire_support_for(major: u8) -> Option<&'static WireSupportEntry> {
    wire_compat_matrix().iter().find(|e| e.major == major)
}

/// Human-readable summary of the support window (diagnostics / CLI).
pub fn wire_support_summary() -> String {
    format!(
        "writer={}.{} ({}); readers={:?}; matrix_entries={}",
        WRITER_WIRE_MAJOR,
        WRITER_WIRE_MINOR,
        WIRE_PROFILE_LABEL,
        SUPPORTED_READER_MAJORS,
        wire_compat_matrix().len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_major_is_readable_and_writable() {
        assert!(wire_reader_supports(WIRE_MAJOR));
        assert!(wire_writer_emits(WIRE_MAJOR));
        let e = wire_support_for(WIRE_MAJOR).expect("entry");
        assert!(e.can_read && e.can_write);
        assert_eq!(e.status, WireSupportStatus::Current);
    }

    #[test]
    fn future_major_is_not_supported() {
        assert!(!wire_reader_supports(WIRE_MAJOR.saturating_add(1)));
        assert!(!wire_writer_emits(99));
        assert!(wire_support_for(99).is_none());
    }

    #[test]
    fn summary_mentions_draft_profile() {
        let s = wire_support_summary();
        assert!(s.contains(WIRE_PROFILE_LABEL));
        assert!(s.contains("writer="));
    }
}
