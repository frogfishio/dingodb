//! Trusted time floor (`HEAP_SPEC` §8.7.1).

use crate::error::{HeapError, HeapUnavailableCause};

/// Monotonic security-time floor.
#[derive(Debug, Clone)]
pub struct SecurityTimeFloor {
    floor_unix_s: u64,
}

impl SecurityTimeFloor {
    /// Create with an initial floor.
    pub fn new(floor_unix_s: u64) -> Self {
        Self { floor_unix_s }
    }

    /// Observe a trusted instant; advances floor.
    pub fn observe(&mut self, unix_s: u64) -> Result<TrustedInstant, HeapError> {
        if unix_s < self.floor_unix_s {
            return Err(HeapError::unavailable(
                HeapUnavailableCause::NotYetValidOrExpired,
            ));
        }
        self.floor_unix_s = unix_s;
        Ok(TrustedInstant { unix_s })
    }

    /// Current floor.
    pub fn floor(&self) -> u64 {
        self.floor_unix_s
    }
}

/// Instant accepted by the security-time floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustedInstant {
    /// Unix seconds.
    pub unix_s: u64,
}

/// Time window decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDecision {
    /// Within validity.
    Accept,
    /// Before not_before.
    NotYetValid,
    /// After expiry / grace.
    Expired,
}
