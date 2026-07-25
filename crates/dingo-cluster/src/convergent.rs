//! Convergent-append path (Stage 8c, CLUSTER_SPEC §9.2, §15.2).
//!
//! In `convergent-append` mode any authorized reachable ingest node may accept
//! a uniquely identified append. Events do not claim one real-time total order;
//! replicas reconcile by subject + content hash. Conflicts stay explicit.
//!
//! This mode MUST NOT be advertised as linearizable.

use crate::id::{NodeId, PartitionId};
use serde::{Deserialize, Serialize};

/// One live body variant observed for a subject during reconcile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectVariant {
    /// Node that held this live body.
    pub node: NodeId,
    /// Live payload bytes.
    pub body: Vec<u8>,
    /// Blake3 content hash of the body (32 bytes).
    pub content_hash: [u8; 32],
}

/// Same subject with differing live bodies across replicas (no silent winner).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectConflict {
    /// Subject key.
    pub subject: String,
    /// Partition that owns the subject.
    pub partition: PartitionId,
    /// Distinct live variants (at least two).
    pub variants: Vec<SubjectVariant>,
}

/// Outcome of a cluster-wide convergent reconcile.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReconcileReport {
    /// Number of `(subject, body)` copies written to replicas that lacked them.
    pub events_replicated: u32,
    /// Subjects where online replicas disagree on the live body.
    pub conflicts: Vec<SubjectConflict>,
    /// Nodes that participated (online at reconcile time).
    pub participants: Vec<NodeId>,
}

/// Stable content hash for convergent event identity (body only).
pub fn body_content_hash(body: &[u8]) -> [u8; 32] {
    *blake3::hash(body).as_bytes()
}

/// Identity key for a convergent event: subject + body content hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConvergentIdentity {
    /// Subject bytes as stored.
    pub subject: String,
    /// Hex of blake3(body).
    pub content_hash_hex: String,
}

impl ConvergentIdentity {
    /// Build from subject and body.
    pub fn from_subject_body(subject: &str, body: &[u8]) -> Self {
        Self {
            subject: subject.to_string(),
            content_hash_hex: hex32(&body_content_hash(body)),
        }
    }
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_stable() {
        let a = body_content_hash(b"hello");
        let b = body_content_hash(b"hello");
        let c = body_content_hash(b"world");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
