//! Cluster identity types (CLUSTER_SPEC §5, §8.3).
//!
//! Random cluster ids use the shared OS CSPRNG path (`residiuum_store::random_id`,
//! DEF-025 / `dingo-id-v1`). Deterministic seed derivation is for tests only.

use residiuum_store::{random_id, StoreError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable cluster identifier (16 random bytes, hex in diagnostics).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClusterId(pub [u8; 16]);

impl ClusterId {
    /// Mint a new cluster id from the OS CSPRNG (DEF-025). Fails closed.
    pub fn generate() -> Result<Self, StoreError> {
        Ok(Self(random_id()?))
    }

    /// Deterministic id from an explicit seed (tests / fixtures only).
    ///
    /// Not suitable for production cluster identity: same seed → same id.
    pub fn from_seed(seed: &[u8]) -> Self {
        let hash = blake3::hash(seed);
        let mut id = [0u8; 16];
        id.copy_from_slice(&hash.as_bytes()[..16]);
        Self(id)
    }

    /// Lowercase hex encoding.
    pub fn to_hex(self) -> String {
        hex16(&self.0)
    }

    /// Parse 32-char hex.
    pub fn from_hex(s: &str) -> Option<Self> {
        let bytes = unhex16(s)?;
        Some(Self(bytes))
    }
}

impl fmt::Debug for ClusterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClusterId({})", self.to_hex())
    }
}

/// Storage / control node identifier within a cluster.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    /// Construct from a dense node index (0..N).
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    /// Dense index used for directory layout (`node-0`, …).
    pub fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node-{}", self.0)
    }
}

/// Virtual partition identifier (CLUSTER_SPEC §8.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PartitionId(pub u32);

impl PartitionId {
    /// Wrap a virtual partition index in `0..virtual_partition_count`.
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Raw index.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for PartitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PartitionId({})", self.0)
    }
}

impl fmt::Display for PartitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "p{}", self.0)
    }
}

/// Monotonic leadership generation for one partition (CLUSTER_SPEC §5, §10).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub struct Term(pub u64);

/// Monotonic generation of the partition-to-replica assignment.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub struct PlacementEpoch(pub u64);

/// Partition-local log / event position (Stage 8a: opaque monotonic counter).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub struct LogPosition(pub u64);

fn hex16(bytes: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn unhex16(s: &str) -> Option<[u8; 16]> {
    if s.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_id_roundtrip() {
        let id = ClusterId::from_seed(b"test-seed");
        let hex = id.to_hex();
        assert_eq!(hex.len(), 32);
        assert_eq!(ClusterId::from_hex(&hex), Some(id));
    }

    #[test]
    fn cluster_id_csprng_unique() {
        let a = ClusterId::generate().expect("CSPRNG");
        let b = ClusterId::generate().expect("CSPRNG");
        assert_ne!(a, b);
        assert_ne!(a.0, [0u8; 16]);
    }
}
