//! Safety limits for frame candidates (FORMAT_SPEC §7.1).

/// Bounds applied when accepting or scanning frame candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyLimits {
    /// Maximum envelope length in bytes.
    pub max_envelope_len: u32,
    /// Maximum stored body length in bytes.
    pub max_body_len: u64,
    /// Maximum complete frame length in bytes.
    pub max_frame_len: u64,
}

impl SafetyLimits {
    /// Default draft limits: large enough for ordinary frames, small enough to
    /// bound adversarial progress on a single candidate.
    pub const fn draft_defaults() -> Self {
        Self {
            max_envelope_len: 64 * 1024,
            max_body_len: 16 * 1024 * 1024,
            max_frame_len: 17 * 1024 * 1024,
        }
    }

    /// Whether the declared lengths fit within these limits and checked arithmetic.
    pub fn accepts_lengths(self, envelope_len: u32, body_len: u64) -> bool {
        if envelope_len > self.max_envelope_len || body_len > self.max_body_len {
            return false;
        }
        match checked_frame_len(envelope_len, body_len) {
            Some(frame_len) => frame_len <= self.max_frame_len,
            None => false,
        }
    }
}

impl Default for SafetyLimits {
    fn default() -> Self {
        Self::draft_defaults()
    }
}

/// `64 + envelope_len + body_len + 56` with overflow checks.
pub fn checked_frame_len(envelope_len: u32, body_len: u64) -> Option<u64> {
    let fixed = 64u64.checked_add(56)?;
    let env = u64::from(envelope_len);
    fixed.checked_add(env)?.checked_add(body_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_accept_small_frames() {
        let limits = SafetyLimits::draft_defaults();
        assert!(limits.accepts_lengths(0, 0));
        assert!(limits.accepts_lengths(100, 1000));
    }

    #[test]
    fn rejects_oversize_envelope() {
        let limits = SafetyLimits::draft_defaults();
        assert!(!limits.accepts_lengths(limits.max_envelope_len + 1, 0));
    }

    #[test]
    fn checked_frame_len_matches_spec() {
        assert_eq!(checked_frame_len(0, 0), Some(120));
        assert_eq!(checked_frame_len(10, 20), Some(150));
    }

    #[test]
    fn checked_frame_len_overflow_fails_closed() {
        assert_eq!(checked_frame_len(u32::MAX, u64::MAX), None);
        assert_eq!(checked_frame_len(0, u64::MAX), None);
        // 120 + body overflows when body > u64::MAX - 120
        assert_eq!(checked_frame_len(0, u64::MAX - 100), None);
        assert_eq!(checked_frame_len(0, u64::MAX - 119), None);
        // Largest body that still fits with empty envelope: u64::MAX - 120
        assert_eq!(checked_frame_len(0, u64::MAX - 120), Some(u64::MAX));
    }
}
