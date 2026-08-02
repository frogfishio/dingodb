//! Byte/entry credit ledger for AWO admission (plan §8, AWO-2).
//!
//! Reservation is atomic under a single mutex: either both entry and byte
//! credits are taken, or neither. Checked arithmetic rejects overflow before
//! any mutation. Failure before enqueue must call [`CreditLedger::release`].

use residiuum_format::{FRAME_PREFIX_LEN, FRAME_SUFFIX_LEN};
use std::sync::Mutex;

/// Fixed frame framing (prefix + suffix) from `residiuum-format`.
pub const FRAME_FRAMING_OVERHEAD: usize = FRAME_PREFIX_LEN + FRAME_SUFFIX_LEN;

/// Conservative CBOR envelope overhead excluding subject payload bytes
/// (map keys, four bstr-16 fields, event kind, created_ns).
pub const ENVELOPE_FIXED_OVERHEAD: usize = 128;

/// Accounting for in-flight request metadata (plan §8).
pub const REQUEST_META_OVERHEAD: usize = 256;

/// Accounting for a completion slot (plan §8).
pub const COMPLETION_SLOT_OVERHEAD: usize = 128;

/// Credit reservation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditError {
    /// Entry limit would be exceeded.
    EntriesExhausted,
    /// Byte limit would be exceeded.
    BytesExhausted,
    /// Checked arithmetic overflowed.
    Overflow,
    /// Release amount exceeded reserved (programmer error / double free).
    Underflow,
}

/// Compute conservative mutation credit for one admitted request (plan §8).
///
/// ```text
/// credit = subject_len + body_len + MAX_FRAME_OVERHEAD
///        + size_of(request metadata) + size_of(completion slot)
/// ```
///
/// `MAX_FRAME_OVERHEAD` = framing + fixed envelope overhead (subject bytes are
/// counted separately as `subject_len`).
pub fn mutation_credit(subject_len: usize, body_len: usize) -> Result<usize, CreditError> {
    subject_len
        .checked_add(body_len)
        .and_then(|v| v.checked_add(FRAME_FRAMING_OVERHEAD))
        .and_then(|v| v.checked_add(ENVELOPE_FIXED_OVERHEAD))
        .and_then(|v| v.checked_add(REQUEST_META_OVERHEAD))
        .and_then(|v| v.checked_add(COMPLETION_SLOT_OVERHEAD))
        .ok_or(CreditError::Overflow)
}

#[derive(Debug)]
struct CreditState {
    entry_limit: usize,
    byte_limit: usize,
    entries_used: usize,
    bytes_used: usize,
}

/// Thread-safe entry+byte credit ledger.
#[derive(Debug)]
pub struct CreditLedger {
    inner: Mutex<CreditState>,
}

impl CreditLedger {
    /// Create a ledger with hard entry and byte limits.
    pub fn new(entry_limit: usize, byte_limit: usize) -> Self {
        Self {
            inner: Mutex::new(CreditState {
                entry_limit,
                byte_limit,
                entries_used: 0,
                bytes_used: 0,
            }),
        }
    }

    /// Atomically reserve `entries` and `bytes` (both-or-neither).
    pub fn try_reserve(&self, entries: usize, bytes: usize) -> Result<(), CreditError> {
        let mut g = self.inner.lock().expect("credit ledger lock");
        let next_entries = g
            .entries_used
            .checked_add(entries)
            .ok_or(CreditError::Overflow)?;
        let next_bytes = g
            .bytes_used
            .checked_add(bytes)
            .ok_or(CreditError::Overflow)?;
        if next_entries > g.entry_limit {
            return Err(CreditError::EntriesExhausted);
        }
        if next_bytes > g.byte_limit {
            return Err(CreditError::BytesExhausted);
        }
        g.entries_used = next_entries;
        g.bytes_used = next_bytes;
        Ok(())
    }

    /// Return previously reserved credits (checked; underflow is an error).
    pub fn release(&self, entries: usize, bytes: usize) -> Result<(), CreditError> {
        let mut g = self.inner.lock().expect("credit ledger lock");
        if entries > g.entries_used || bytes > g.bytes_used {
            return Err(CreditError::Underflow);
        }
        g.entries_used -= entries;
        g.bytes_used -= bytes;
        Ok(())
    }

    /// Currently reserved entry count.
    pub fn entries_used(&self) -> usize {
        self.inner.lock().expect("credit ledger lock").entries_used
    }

    /// Currently reserved byte count.
    pub fn bytes_used(&self) -> usize {
        self.inner.lock().expect("credit ledger lock").bytes_used
    }

    /// Remaining entry capacity.
    pub fn entries_available(&self) -> usize {
        let g = self.inner.lock().expect("credit ledger lock");
        g.entry_limit.saturating_sub(g.entries_used)
    }

    /// Remaining byte capacity.
    pub fn bytes_available(&self) -> usize {
        let g = self.inner.lock().expect("credit ledger lock");
        g.byte_limit.saturating_sub(g.bytes_used)
    }

    /// Configured hard limits.
    pub fn limits(&self) -> (usize, usize) {
        let g = self.inner.lock().expect("credit ledger lock");
        (g.entry_limit, g.byte_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_and_release_roundtrip() {
        let ledger = CreditLedger::new(4, 1000);
        let c = mutation_credit(10, 100).unwrap();
        ledger.try_reserve(1, c).unwrap();
        assert_eq!(ledger.entries_used(), 1);
        assert_eq!(ledger.bytes_used(), c);
        ledger.release(1, c).unwrap();
        assert_eq!(ledger.entries_used(), 0);
        assert_eq!(ledger.bytes_used(), 0);
    }

    #[test]
    fn atomic_reject_leaves_unchanged() {
        let ledger = CreditLedger::new(1, 100);
        ledger.try_reserve(1, 50).unwrap();
        let err = ledger.try_reserve(1, 10).unwrap_err();
        assert_eq!(err, CreditError::EntriesExhausted);
        assert_eq!(ledger.entries_used(), 1);
        assert_eq!(ledger.bytes_used(), 50);
    }

    #[test]
    fn byte_exhaustion() {
        let ledger = CreditLedger::new(10, 100);
        assert_eq!(
            ledger.try_reserve(1, 101).unwrap_err(),
            CreditError::BytesExhausted
        );
        assert_eq!(ledger.entries_used(), 0);
    }

    #[test]
    fn mutation_credit_includes_framing() {
        let c = mutation_credit(0, 0).unwrap();
        assert!(c >= FRAME_FRAMING_OVERHEAD + ENVELOPE_FIXED_OVERHEAD);
        assert!(mutation_credit(usize::MAX, usize::MAX).is_err());
    }
}
