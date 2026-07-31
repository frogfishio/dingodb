//! Forward and reverse salvage scanning with hole reports (FORMAT_SPEC §7).
//!
//! Pure byte-buffer scans. Do not require segment descriptors or summaries.
//! After any failed forward candidate, search resumes at `q + 1` so a corrupt
//! length cannot hide later frames. Reverse discovery uses suffix magic and
//! `frame_len` (FORMAT_SPEC §7.4).

use crate::frame::{
    verify_frame_at, DecodedFrame, FrameVerifyError, END_MAGIC, FRAME_PREFIX_LEN, FRAME_SUFFIX_LEN,
    START_MAGIC,
};
use crate::limits::SafetyLimits;

/// Half-open byte range `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteRange {
    /// Inclusive start offset.
    pub start: u64,
    /// Exclusive end offset.
    pub end: u64,
}

impl ByteRange {
    /// Construct a range. Panics in debug if `end < start`.
    pub fn new(start: u64, end: u64) -> Self {
        debug_assert!(end >= start);
        Self { start, end }
    }

    /// Length in bytes.
    pub fn len(self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range is empty.
    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

/// Why a non-verified region exists (FORMAT_SPEC §7.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoleReason {
    /// Bytes that did not form a verified frame and were not a recorded candidate failure.
    UnclassifiedGarbage,
    /// Magic candidate that failed structural or body checks.
    CorruptCandidate {
        /// Offset of the candidate start magic.
        candidate_offset: u64,
        /// Failure reason.
        error: FrameVerifyError,
    },
    /// Claimed frame range where body or boundary checks failed after lengths looked plausible.
    DamagedCandidate {
        /// Offset of the candidate start magic.
        candidate_offset: u64,
        /// Failure reason.
        error: FrameVerifyError,
    },
}

/// One region in a forward scan report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanRegion {
    /// Structurally verified frame (FORMAT_SPEC §5).
    VerifiedFrame {
        /// Physical byte range occupied by the frame.
        range: ByteRange,
        /// Decoded frame contents.
        frame: DecodedFrame,
    },
    /// Explicit non-data / non-verified region.
    Hole {
        /// Physical range. May be a single-byte step for failed candidates when
        /// extent is not established.
        range: ByteRange,
        /// Classification and evidence.
        reason: HoleReason,
    },
}

impl ScanRegion {
    /// Byte range of this region.
    pub fn range(&self) -> ByteRange {
        match self {
            ScanRegion::VerifiedFrame { range, .. } | ScanRegion::Hole { range, .. } => *range,
        }
    }

    /// Whether this region is a verified frame.
    pub fn is_verified(&self) -> bool {
        matches!(self, ScanRegion::VerifiedFrame { .. })
    }
}

/// Result of a forward salvage scan (FORMAT_SPEC §7.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    /// Source buffer length.
    pub source_len: u64,
    /// Limits applied during the scan (included in the report per §7.1).
    pub limits: SafetyLimits,
    /// Ordered regions covering discovered frames and holes.
    pub regions: Vec<ScanRegion>,
}

impl ScanReport {
    /// All verified frames with their start offsets.
    pub fn verified_frames(&self) -> impl Iterator<Item = (u64, &DecodedFrame)> {
        self.regions.iter().filter_map(|r| match r {
            ScanRegion::VerifiedFrame { range, frame } => Some((range.start, frame)),
            _ => None,
        })
    }

    /// Number of verified frames.
    pub fn verified_count(&self) -> usize {
        self.verified_frames().count()
    }

    /// Hole regions only.
    pub fn holes(&self) -> impl Iterator<Item = (&ByteRange, &HoleReason)> {
        self.regions.iter().filter_map(|r| match r {
            ScanRegion::Hole { range, reason } => Some((range, reason)),
            _ => None,
        })
    }
}

/// Forward salvage scan of a byte source (FORMAT_SPEC §7.2).
///
/// Algorithm:
/// 1. From position `p`, find the next `RESIDFRM` at `q`.
/// 2. Bytes `[p, q)` are unclassified garbage (if non-empty).
/// 3. Attempt full structural verification at `q`.
/// 4. On success, emit a verified frame and set `p = q + frame_len`.
/// 5. On failure, record a hole/candidate failure and set `p = q + 1`.
///
/// Later intact frames remain discoverable after earlier corruption.
pub fn scan_forward(bytes: &[u8], limits: SafetyLimits) -> ScanReport {
    let n = bytes.len();
    let mut regions = Vec::new();
    let mut p = 0usize;

    while p + FRAME_PREFIX_LEN <= n {
        let Some(q) = find_start_magic(bytes, p) else {
            // Remaining bytes after last search position are garbage.
            if p < n {
                push_garbage(&mut regions, p as u64, n as u64);
            }
            return ScanReport {
                source_len: n as u64,
                limits,
                regions,
            };
        };

        if q > p {
            push_garbage(&mut regions, p as u64, q as u64);
        }

        match try_candidate(bytes, q, limits) {
            Ok((frame, frame_len)) => {
                let end = q as u64 + frame_len;
                regions.push(ScanRegion::VerifiedFrame {
                    range: ByteRange::new(q as u64, end),
                    frame,
                });
                p = q + frame_len as usize;
            }
            Err(error) => {
                let reason = classify_failure(q as u64, error);
                // Do not claim a full frame extent from a failed header.
                regions.push(ScanRegion::Hole {
                    range: ByteRange::new(q as u64, (q + 1) as u64),
                    reason,
                });
                p = q + 1;
            }
        }
    }

    if p < n {
        push_garbage(&mut regions, p as u64, n as u64);
    }

    ScanReport {
        source_len: n as u64,
        limits,
        regions,
    }
}

fn try_candidate(
    bytes: &[u8],
    offset: usize,
    limits: SafetyLimits,
) -> Result<(DecodedFrame, u64), FrameVerifyError> {
    let window = &bytes[offset..];
    let (header, envelope, body, hash, frame_len) = verify_frame_at(window, limits)?;
    Ok((
        DecodedFrame {
            header,
            envelope: envelope.to_vec(),
            body: body.to_vec(),
            body_hash: hash,
            frame_len,
        },
        frame_len,
    ))
}

fn classify_failure(candidate_offset: u64, error: FrameVerifyError) -> HoleReason {
    match error {
        FrameVerifyError::BadBodyHash
        | FrameVerifyError::BadEndMagic
        | FrameVerifyError::BadSuffixCrc
        | FrameVerifyError::FrameLenMismatch { .. }
        | FrameVerifyError::Truncated { .. } => HoleReason::DamagedCandidate {
            candidate_offset,
            error,
        },
        other => HoleReason::CorruptCandidate {
            candidate_offset,
            error: other,
        },
    }
}

fn push_garbage(regions: &mut Vec<ScanRegion>, start: u64, end: u64) {
    if start >= end {
        return;
    }
    // Coalesce with previous unclassified garbage when adjacent.
    if let Some(ScanRegion::Hole {
        range,
        reason: HoleReason::UnclassifiedGarbage,
    }) = regions.last_mut()
    {
        if range.end == start {
            range.end = end;
            return;
        }
    }
    regions.push(ScanRegion::Hole {
        range: ByteRange::new(start, end),
        reason: HoleReason::UnclassifiedGarbage,
    });
}

/// Find the next `RESIDFRM` at or after `from`.
pub fn find_start_magic(haystack: &[u8], from: usize) -> Option<usize> {
    if from >= haystack.len() || haystack.len() - from < START_MAGIC.len() {
        return None;
    }
    haystack[from..]
        .windows(START_MAGIC.len())
        .position(|w| w == START_MAGIC.as_slice())
        .map(|i| from + i)
}

/// Find the rightmost `RESIDEND` whose start is strictly less than `exclusive_end`.
pub fn find_end_magic_rightmost(haystack: &[u8], exclusive_end: usize) -> Option<usize> {
    if exclusive_end < END_MAGIC.len() || haystack.is_empty() {
        return None;
    }
    let end = exclusive_end.min(haystack.len());
    if end < END_MAGIC.len() {
        return None;
    }
    haystack[..end]
        .windows(END_MAGIC.len())
        .rposition(|w| w == END_MAGIC.as_slice())
}

/// Reverse salvage scan assisted by suffix magic and `frame_len` (FORMAT_SPEC §7.4).
///
/// Algorithm (right-to-left):
/// 1. From exclusive end `e`, find the rightmost `RESIDEND` at `s` with `s < e`.
/// 2. Bytes `(s + claimed_frame_len, e)` that cannot be claimed become garbage.
/// 3. Attempt full structural verification of the candidate ending at that suffix.
/// 4. On success, emit a verified frame and set `e = frame_start`.
/// 5. On failure, record a hole and set `e = s` (resume left of this magic).
///
/// Regions are returned in **forward** (ascending offset) order so they compare
/// directly with [`scan_forward`] results.
pub fn scan_reverse(bytes: &[u8], limits: SafetyLimits) -> ScanReport {
    let n = bytes.len();
    let mut regions_rev = Vec::new();
    let mut e = n;

    while e >= FRAME_SUFFIX_LEN {
        let Some(s) = find_end_magic_rightmost(bytes, e) else {
            if e > 0 {
                regions_rev.push(ScanRegion::Hole {
                    range: ByteRange::new(0, e as u64),
                    reason: HoleReason::UnclassifiedGarbage,
                });
            }
            break;
        };

        // Bytes after a successful previous frame and before this candidate's
        // claimed end are unclassified when the candidate fails to establish them.
        match try_reverse_candidate(bytes, s, e, limits) {
            Ok((frame, start, frame_len)) => {
                let frame_end = start + frame_len;
                if frame_end < e as u64 {
                    regions_rev.push(ScanRegion::Hole {
                        range: ByteRange::new(frame_end, e as u64),
                        reason: HoleReason::UnclassifiedGarbage,
                    });
                }
                regions_rev.push(ScanRegion::VerifiedFrame {
                    range: ByteRange::new(start, frame_end),
                    frame,
                });
                e = start as usize;
            }
            Err(error) => {
                if s + 1 < e {
                    // Gap after the failed magic position within the search window.
                    // Only the single magic-start byte is attributed to the candidate;
                    // trailing bytes until e stay unclassified (may coalesce later).
                }
                if s + END_MAGIC.len() < e {
                    regions_rev.push(ScanRegion::Hole {
                        range: ByteRange::new((s + 1) as u64, e as u64),
                        reason: HoleReason::UnclassifiedGarbage,
                    });
                }
                let reason = classify_failure(s as u64, error);
                regions_rev.push(ScanRegion::Hole {
                    range: ByteRange::new(s as u64, (s + 1) as u64),
                    reason,
                });
                e = s;
            }
        }
    }

    if e > 0 && e < FRAME_SUFFIX_LEN {
        // Residual prefix of the buffer that could not hold a suffix.
        regions_rev.push(ScanRegion::Hole {
            range: ByteRange::new(0, e as u64),
            reason: HoleReason::UnclassifiedGarbage,
        });
    }

    // regions_rev is newest-first (right to left). Reverse for forward order.
    regions_rev.reverse();
    coalesce_adjacent_garbage(&mut regions_rev);

    ScanReport {
        source_len: n as u64,
        limits,
        regions: regions_rev,
    }
}

fn try_reverse_candidate(
    bytes: &[u8],
    end_magic_off: usize,
    exclusive_end: usize,
    limits: SafetyLimits,
) -> Result<(DecodedFrame, u64, u64), FrameVerifyError> {
    if end_magic_off + FRAME_SUFFIX_LEN > exclusive_end
        || end_magic_off + FRAME_SUFFIX_LEN > bytes.len()
    {
        return Err(FrameVerifyError::Truncated {
            need: end_magic_off + FRAME_SUFFIX_LEN,
            have: exclusive_end.min(bytes.len()),
        });
    }

    let suffix: &[u8; FRAME_SUFFIX_LEN] = bytes[end_magic_off..end_magic_off + FRAME_SUFFIX_LEN]
        .try_into()
        .expect("suffix length checked");

    if &suffix[0..8] != END_MAGIC.as_slice() {
        return Err(FrameVerifyError::BadEndMagic);
    }

    let claimed_frame_len = u64::from_le_bytes(suffix[8..16].try_into().unwrap());
    if claimed_frame_len < (FRAME_PREFIX_LEN + FRAME_SUFFIX_LEN) as u64 {
        return Err(FrameVerifyError::LengthsOutOfLimits);
    }
    if claimed_frame_len > exclusive_end as u64 {
        return Err(FrameVerifyError::Truncated {
            need: claimed_frame_len as usize,
            have: exclusive_end,
        });
    }

    let frame_end = end_magic_off as u64 + FRAME_SUFFIX_LEN as u64;
    if frame_end < claimed_frame_len {
        return Err(FrameVerifyError::Truncated {
            need: claimed_frame_len as usize,
            have: frame_end as usize,
        });
    }
    let start = (frame_end - claimed_frame_len) as usize;
    if start + claimed_frame_len as usize > exclusive_end {
        return Err(FrameVerifyError::Truncated {
            need: start + claimed_frame_len as usize,
            have: exclusive_end,
        });
    }

    let window = &bytes[start..start + claimed_frame_len as usize];
    let (header, envelope, body, hash, frame_len) = verify_frame_at(window, limits)?;
    if frame_len != claimed_frame_len {
        return Err(FrameVerifyError::FrameLenMismatch {
            suffix: claimed_frame_len,
            computed: frame_len,
        });
    }
    Ok((
        DecodedFrame {
            header,
            envelope: envelope.to_vec(),
            body: body.to_vec(),
            body_hash: hash,
            frame_len,
        },
        start as u64,
        frame_len,
    ))
}

fn coalesce_adjacent_garbage(regions: &mut Vec<ScanRegion>) {
    if regions.len() < 2 {
        return;
    }
    let mut out = Vec::with_capacity(regions.len());
    for region in regions.drain(..) {
        match (&mut out.last_mut(), &region) {
            (
                Some(ScanRegion::Hole {
                    range,
                    reason: HoleReason::UnclassifiedGarbage,
                }),
                ScanRegion::Hole {
                    range: r2,
                    reason: HoleReason::UnclassifiedGarbage,
                },
            ) if range.end == r2.start => {
                range.end = r2.end;
            }
            _ => out.push(region),
        }
    }
    *regions = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor_envelope::EMPTY_ENVELOPE;
    use crate::frame::{encode_frame, FrameHeader, FrameParts};
    use crate::kinds::FrameKind;

    fn item(body: &[u8]) -> Vec<u8> {
        encode_frame(&FrameParts {
            header: FrameHeader::new_draft(
                FrameKind::ItemEvent,
                EMPTY_ENVELOPE.len() as u32,
                body.len() as u64,
                [1u8; 16],
            ),
            envelope: EMPTY_ENVELOPE.to_vec(),
            body: body.to_vec(),
        })
        .unwrap()
    }

    #[test]
    fn clean_concatenation_two_frames() {
        let mut buf = item(b"a");
        buf.extend_from_slice(&item(b"b"));
        let report = scan_forward(&buf, SafetyLimits::default());
        assert_eq!(report.verified_count(), 2);
        assert_eq!(report.holes().count(), 0);
        let bodies: Vec<_> = report
            .verified_frames()
            .map(|(_, f)| f.body.clone())
            .collect();
        assert_eq!(bodies, vec![b"a".to_vec(), b"b".to_vec()]);
    }

    #[test]
    fn garbage_then_frame() {
        let mut buf = b"GARBAGE!!!".to_vec();
        buf.extend_from_slice(&item(b"ok"));
        let report = scan_forward(&buf, SafetyLimits::default());
        assert_eq!(report.verified_count(), 1);
        let holes: Vec<_> = report.holes().collect();
        assert_eq!(holes.len(), 1);
        assert_eq!(holes[0].0.start, 0);
        assert!(matches!(holes[0].1, HoleReason::UnclassifiedGarbage));
    }

    #[test]
    fn corrupt_length_does_not_hide_later_frame() {
        let good = item(b"survivor");
        let mut bad = item(b"x");
        // Corrupt envelope_len to a huge value without fixing CRC → reject, step +1.
        bad[12] = 0xff;
        bad[13] = 0xff;
        bad[14] = 0xff;
        bad[15] = 0x7f;
        let mut buf = bad;
        buf.extend_from_slice(&good);
        let report = scan_forward(&buf, SafetyLimits::default());
        let survivors: Vec<_> = report
            .verified_frames()
            .filter(|(_, f)| f.body == b"survivor")
            .collect();
        assert_eq!(survivors.len(), 1, "later frame must remain discoverable");
    }

    #[test]
    fn false_magic_in_body_does_not_split_parent() {
        let mut body = Vec::new();
        body.extend_from_slice(START_MAGIC);
        body.extend_from_slice(&[0u8; 40]);
        let frame = item(&body);
        let report = scan_forward(&frame, SafetyLimits::default());
        // Parent verifies; any false-magic attempts inside body are only searched
        // after the parent advances by frame_len — so one verified frame, no hole.
        assert_eq!(report.verified_count(), 1);
        assert_eq!(report.holes().count(), 0);
    }

    #[test]
    fn reverse_scan_matches_forward_on_clean_concat() {
        let mut buf = item(b"a");
        buf.extend_from_slice(&item(b"b"));
        let fwd = scan_forward(&buf, SafetyLimits::default());
        let rev = scan_reverse(&buf, SafetyLimits::default());
        let fwd_bodies: Vec<_> = fwd
            .verified_frames()
            .map(|(o, f)| (o, f.body.clone()))
            .collect();
        let rev_bodies: Vec<_> = rev
            .verified_frames()
            .map(|(o, f)| (o, f.body.clone()))
            .collect();
        assert_eq!(fwd_bodies, rev_bodies);
    }

    #[test]
    fn reverse_scan_finds_frame_after_garbage() {
        let mut buf = b"xxxGARBAGEyyy".to_vec();
        buf.extend_from_slice(&item(b"island"));
        let rev = scan_reverse(&buf, SafetyLimits::default());
        let survivors: Vec<_> = rev
            .verified_frames()
            .filter(|(_, f)| f.body == b"island")
            .collect();
        assert_eq!(survivors.len(), 1);
    }
}
