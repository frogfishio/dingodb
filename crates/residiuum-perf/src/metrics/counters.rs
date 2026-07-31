//! Fixed counter registry — saturating, no per-op maps.

use serde::{Deserialize, Serialize};

/// Closed counter identifiers used by the result kernel.
pub const COUNTER_IDS: &[&str] = &[
    "ops_attempted",
    "ops_admitted",
    "ops_acknowledged",
    "ops_failed",
    "logical_payload_bytes_ack",
    "logical_key_bytes",
    "batch_dispatch_count",
    "sync_count",
    "rotation_count",
    "probe_samples",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterId {
    OpsAttempted,
    OpsAdmitted,
    OpsAcknowledged,
    OpsFailed,
    LogicalPayloadBytesAck,
    LogicalKeyBytes,
    BatchDispatchCount,
    SyncCount,
    RotationCount,
    ProbeSamples,
}

impl CounterId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpsAttempted => "ops_attempted",
            Self::OpsAdmitted => "ops_admitted",
            Self::OpsAcknowledged => "ops_acknowledged",
            Self::OpsFailed => "ops_failed",
            Self::LogicalPayloadBytesAck => "logical_payload_bytes_ack",
            Self::LogicalKeyBytes => "logical_key_bytes",
            Self::BatchDispatchCount => "batch_dispatch_count",
            Self::SyncCount => "sync_count",
            Self::RotationCount => "rotation_count",
            Self::ProbeSamples => "probe_samples",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::OpsAttempted => 0,
            Self::OpsAdmitted => 1,
            Self::OpsAcknowledged => 2,
            Self::OpsFailed => 3,
            Self::LogicalPayloadBytesAck => 4,
            Self::LogicalKeyBytes => 5,
            Self::BatchDispatchCount => 6,
            Self::SyncCount => 7,
            Self::RotationCount => 8,
            Self::ProbeSamples => 9,
        }
    }

    pub const COUNT: usize = 10;
}

/// Fixed-size saturating counter set (bounded memory, mergeable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterSet {
    values: [u64; CounterId::COUNT],
    saturated: bool,
}

impl Default for CounterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterSet {
    pub fn new() -> Self {
        Self {
            values: [0; CounterId::COUNT],
            saturated: false,
        }
    }

    pub fn get(&self, id: CounterId) -> u64 {
        self.values[id.index()]
    }

    pub fn is_saturated(&self) -> bool {
        self.saturated
    }

    pub fn add(&mut self, id: CounterId, n: u64) {
        let i = id.index();
        let (sum, overflow) = self.values[i].overflowing_add(n);
        if overflow {
            self.values[i] = u64::MAX;
            self.saturated = true;
        } else {
            self.values[i] = sum;
        }
    }

    pub fn inc(&mut self, id: CounterId) {
        self.add(id, 1);
    }

    pub fn merge_from(&mut self, other: &Self) {
        for i in 0..CounterId::COUNT {
            let (sum, overflow) = self.values[i].overflowing_add(other.values[i]);
            if overflow {
                self.values[i] = u64::MAX;
                self.saturated = true;
            } else {
                self.values[i] = sum;
            }
        }
        self.saturated |= other.saturated;
    }

    /// Snapshot as name → value (for result JSON). Zero is a legal count.
    pub fn as_map(&self) -> Vec<(String, u64)> {
        (0..CounterId::COUNT)
            .map(|i| {
                let name = COUNTER_IDS[i].to_string();
                (name, self.values[i])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturates_not_wraps() {
        let mut c = CounterSet::new();
        c.add(CounterId::OpsAttempted, u64::MAX);
        c.inc(CounterId::OpsAttempted);
        assert_eq!(c.get(CounterId::OpsAttempted), u64::MAX);
        assert!(c.is_saturated());
    }

    #[test]
    fn merge_no_lost_counts() {
        let mut a = CounterSet::new();
        let mut b = CounterSet::new();
        a.add(CounterId::OpsAcknowledged, 40);
        b.add(CounterId::OpsAcknowledged, 60);
        a.merge_from(&b);
        assert_eq!(a.get(CounterId::OpsAcknowledged), 100);
    }
}
