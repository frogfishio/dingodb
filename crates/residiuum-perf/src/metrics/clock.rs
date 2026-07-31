//! Monotonic stage clock — wall time is metadata only (SPEC §7).

use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TimestampError {
    #[error("reordered timestamp: got {got}, last {last}")]
    Reordered { got: u64, last: u64 },
    #[error("timestamp not set for stage")]
    Missing,
}

/// Nanosecond offset from an arbitrary monotonic origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct StageTimestamp(pub u64);

/// Monotonic clock for stage probes. Rejects reordered timestamps.
#[derive(Debug, Clone)]
pub struct MonotonicClock {
    origin: Instant,
    last_ns: u64,
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            last_ns: 0,
        }
    }

    /// Read current monotonic ns since origin (does not advance `last`).
    pub fn now_ns(&self) -> u64 {
        self.origin.elapsed().as_nanos() as u64
    }

    /// Observe a timestamp, rejecting reorder (strictly non-decreasing).
    pub fn observe(&mut self, ts_ns: u64) -> Result<StageTimestamp, TimestampError> {
        if ts_ns < self.last_ns {
            return Err(TimestampError::Reordered {
                got: ts_ns,
                last: self.last_ns,
            });
        }
        self.last_ns = ts_ns;
        Ok(StageTimestamp(ts_ns))
    }

    /// Tick: read now and observe it.
    pub fn tick(&mut self) -> Result<StageTimestamp, TimestampError> {
        let n = self.now_ns();
        self.observe(n)
    }

    pub fn last_ns(&self) -> u64 {
        self.last_ns
    }

    /// Inject a synthetic timeline (tests / fake adapters).
    pub fn observe_synthetic(&mut self, delta_ns: u64) -> Result<StageTimestamp, TimestampError> {
        let next = self.last_ns.saturating_add(delta_ns);
        self.observe(next)
    }
}

/// Duration between two stage timestamps (ns).
pub fn stage_delta(start: StageTimestamp, end: StageTimestamp) -> Result<u64, TimestampError> {
    if end.0 < start.0 {
        return Err(TimestampError::Reordered {
            got: end.0,
            last: start.0,
        });
    }
    Ok(end.0 - start.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_reorder() {
        let mut c = MonotonicClock::new();
        c.observe(100).unwrap();
        c.observe(100).unwrap(); // equal OK
        assert!(matches!(
            c.observe(99),
            Err(TimestampError::Reordered { .. })
        ));
    }

    #[test]
    fn synthetic_monotonic() {
        let mut c = MonotonicClock::new();
        let a = c.observe_synthetic(10).unwrap();
        let b = c.observe_synthetic(5).unwrap();
        assert_eq!(stage_delta(a, b).unwrap(), 5);
    }

    #[test]
    fn live_tick_advances() {
        let mut c = MonotonicClock::new();
        let a = c.tick().unwrap();
        // busy wait tiny
        let mut x = 0u64;
        for i in 0..1000 {
            x = x.wrapping_add(i);
        }
        let _ = x;
        let b = c.tick().unwrap();
        assert!(b.0 >= a.0);
    }
}
