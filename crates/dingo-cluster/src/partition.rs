//! Virtual partition mapping (CLUSTER_SPEC §8).

use crate::id::PartitionId;
use serde::{Deserialize, Serialize};

/// Published hash profile name for partition-key → virtual partition mapping.
///
/// Changing the profile or virtual partition count is a store-generation
/// migration (CLUSTER_SPEC §8.2).
pub const HASH_PROFILE_BLAKE3_MOD: &str = "blake3-mod-v1";

/// Default virtual partition count for new clusters (power of two for mask).
pub const DEFAULT_VIRTUAL_PARTITIONS: u32 = 64;

/// Parameters of the deterministic partition map for one store generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionMap {
    /// Number of virtual partitions (must be ≥ 1).
    pub virtual_partitions: u32,
    /// Published hash profile identifier.
    pub hash_profile: String,
}

impl Default for PartitionMap {
    fn default() -> Self {
        Self {
            virtual_partitions: DEFAULT_VIRTUAL_PARTITIONS,
            hash_profile: HASH_PROFILE_BLAKE3_MOD.to_string(),
        }
    }
}

impl PartitionMap {
    /// Create with an explicit virtual partition count.
    pub fn new(virtual_partitions: u32) -> Self {
        assert!(
            virtual_partitions >= 1,
            "need at least one virtual partition"
        );
        Self {
            virtual_partitions,
            hash_profile: HASH_PROFILE_BLAKE3_MOD.to_string(),
        }
    }

    /// Map a partition key to a virtual partition (CLUSTER_SPEC §8.1–§8.2).
    ///
    /// Default partition key for application data is the subject / logical key
    /// bytes. Events that must share strong ordering MUST use the same key.
    pub fn partition_of(&self, partition_key: &[u8]) -> PartitionId {
        let hash = blake3::hash(partition_key);
        // Take first 8 bytes as little-endian u64, then reduce.
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&hash.as_bytes()[..8]);
        let n = u64::from_le_bytes(buf);
        let id = (n % u64::from(self.virtual_partitions)) as u32;
        PartitionId::new(id)
    }

    /// All virtual partition ids in ascending order.
    pub fn all_partitions(&self) -> impl Iterator<Item = PartitionId> {
        let n = self.virtual_partitions;
        (0..n).map(PartitionId::new)
    }
}

/// Default partition key selection (CLUSTER_SPEC §8.1): use subject bytes.
///
/// Higher layers that have `subject_id` / `item_id` / `event_id` may choose
/// among those; Stage 8a uses the full subject string as the stable key.
pub fn default_partition_key(subject: &str) -> &[u8] {
    subject.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_in_range() {
        let map = PartitionMap::new(16);
        let a = map.partition_of(b"users/alice");
        let b = map.partition_of(b"users/alice");
        assert_eq!(a, b);
        assert!(a.get() < 16);
        let c = map.partition_of(b"users/bob");
        // Different keys usually differ; allow rare collision.
        let _ = c;
    }

    #[test]
    fn spreads_keys() {
        let map = PartitionMap::new(8);
        let mut seen = [false; 8];
        for i in 0..200 {
            let key = format!("k{i}");
            seen[map.partition_of(key.as_bytes()).get() as usize] = true;
        }
        // With 200 keys into 8 partitions, expect all hit in practice.
        assert!(seen.iter().filter(|&&x| x).count() >= 6);
    }
}
