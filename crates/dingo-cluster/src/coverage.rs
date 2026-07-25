//! Coverage records for distributed results (CLUSTER_SPEC §6.7, §17).

use crate::id::{LogPosition, PartitionId, Term};
use crate::modes::ReadMode;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Per-partition frontier observed during an operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionFrontier {
    /// Partition that was contacted or requested.
    pub partition: PartitionId,
    /// Leadership term observed (0 if unknown).
    pub term: Term,
    /// Log / event position observed (0 if unknown).
    pub position: LogPosition,
    /// Node that served this partition, if any.
    pub served_by: Option<u32>,
}

/// Coverage evidence attached to every distributed query/scan/recovery result.
///
/// An unavailable partition MUST NOT be represented as an empty successful
/// partition (CLUSTER_SPEC §6.7, §17.2). A partial result is valid data with
/// incomplete coverage — never a silent complete empty success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Coverage {
    /// Partitions required by the declared scope.
    pub requested: Vec<PartitionId>,
    /// Partitions that completed successfully.
    pub completed: Vec<PartitionId>,
    /// Partitions that could not be covered (offline, no quorum, …).
    pub unavailable: Vec<PartitionId>,
    /// Frontiers for completed (and optionally partial) partitions.
    pub frontiers: Vec<PartitionFrontier>,
    /// Human-readable notes (e.g. development-profile warnings).
    pub notes: Vec<String>,
    /// Read mode used for the distributed operation, when applicable.
    pub read_mode: Option<String>,
    /// True when a declared resource budget truncated the scan/query.
    pub resource_limit_reached: bool,
}

impl Coverage {
    /// Empty coverage builder for a declared scope.
    pub fn for_partitions(requested: impl IntoIterator<Item = PartitionId>) -> Self {
        let mut requested: Vec<PartitionId> = requested.into_iter().collect();
        requested.sort();
        requested.dedup();
        Self {
            requested,
            completed: Vec::new(),
            unavailable: Vec::new(),
            frontiers: Vec::new(),
            notes: Vec::new(),
            read_mode: None,
            resource_limit_reached: false,
        }
    }

    /// Single-partition scope.
    pub fn single(partition: PartitionId) -> Self {
        Self::for_partitions([partition])
    }

    /// Mark a partition completed with frontier evidence.
    pub fn mark_completed(
        &mut self,
        partition: PartitionId,
        term: Term,
        position: LogPosition,
        served_by: Option<u32>,
    ) {
        if !self.completed.contains(&partition) {
            self.completed.push(partition);
            self.completed.sort();
        }
        self.unavailable.retain(|p| *p != partition);
        self.frontiers.retain(|f| f.partition != partition);
        self.frontiers.push(PartitionFrontier {
            partition,
            term,
            position,
            served_by,
        });
        self.frontiers.sort_by_key(|f| f.partition);
    }

    /// Mark a partition unavailable (must not look like empty success).
    pub fn mark_unavailable(&mut self, partition: PartitionId) {
        if !self.unavailable.contains(&partition) {
            self.unavailable.push(partition);
            self.unavailable.sort();
        }
        self.completed.retain(|p| *p != partition);
        self.frontiers.retain(|f| f.partition != partition);
    }

    /// True when every requested partition completed and none are unavailable.
    pub fn is_complete(&self) -> bool {
        !self.resource_limit_reached
            && self.unavailable.is_empty()
            && self.requested.iter().all(|p| self.completed.contains(p))
    }

    /// True when at least one requested partition is missing or truncated.
    pub fn is_incomplete(&self) -> bool {
        !self.is_complete()
    }

    /// Attach a free-form note (e.g. profile warning).
    pub fn note(&mut self, msg: impl Into<String>) {
        self.notes.push(msg.into());
    }

    /// Record the read mode used for this result.
    pub fn with_read_mode(&mut self, mode: ReadMode) {
        self.read_mode = Some(mode.as_str().to_string());
    }

    /// Mark that a resource budget stopped the scan before full coverage.
    pub fn mark_resource_limit(&mut self, detail: impl Into<String>) {
        self.resource_limit_reached = true;
        self.note(detail);
    }
}

/// Result of a cluster get: value plus separate coverage claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetResult {
    /// Live body when found; `None` only means absence when coverage is complete
    /// under a linearizable (or otherwise conclusive) read.
    pub value: Option<Vec<u8>>,
    /// Coverage for the partitions involved.
    pub coverage: Coverage,
    /// Whether the implementation claims absence is proven (`None` + complete).
    pub absence_proven: bool,
}

/// Result of a multi-partition scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    /// Live `(subject, body)` pairs from completed partitions only.
    pub entries: Vec<(String, Vec<u8>)>,
    /// Coverage — incomplete scans still return whatever was found.
    pub coverage: Coverage,
}

/// Result of a distributed find/query (CLUSTER_SPEC §17).
///
/// Partial results remain valid data. Callers MUST inspect [`Coverage::is_complete`]
/// before treating absence of matches as proof that no matching subjects exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindResult {
    /// Matching `(subject, body)` pairs from completed partitions only.
    pub entries: Vec<(String, Vec<u8>)>,
    /// Coverage for the declared partition scope.
    pub coverage: Coverage,
    /// Stable query identity for pagination / coordinator replacement (§17.4).
    pub query_id: String,
    /// True when a limit or budget truncated the match list (coverage may still
    /// list completed partitions; resource_limit_reached is set when a budget
    /// stopped partition examination).
    pub truncated: bool,
}

impl FindResult {
    /// Build a deterministic query id from scope and options.
    pub fn make_query_id(
        scope_partitions: &[PartitionId],
        prefix: Option<&str>,
        limit: Option<usize>,
    ) -> String {
        let mut h = DefaultHasher::new();
        "dingo-find-v1".hash(&mut h);
        for p in scope_partitions {
            p.get().hash(&mut h);
        }
        prefix.unwrap_or("").hash(&mut h);
        limit.unwrap_or(usize::MAX).hash(&mut h);
        format!("q-{:016x}", h.finish())
    }
}

/// Options for distributed scan/find (Stage 8e).
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// Only include subjects with this UTF-8 prefix (collection routing).
    pub subject_prefix: Option<String>,
    /// Cap the number of returned entries (deterministic subject order).
    pub limit: Option<usize>,
    /// Cap how many live subjects may be examined before stopping (budget).
    pub max_docs_scanned: Option<usize>,
    /// Optional subset of partitions; default is the full virtual map.
    pub partitions: Option<Vec<PartitionId>>,
    /// Read mode for partition contact (default: available-style scan).
    pub read_mode: ReadMode,
}

impl ScanOptions {
    /// Full-map scan with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restrict to subjects with this prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.subject_prefix = Some(prefix.into());
        self
    }

    /// Cap returned rows.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Cap documents examined (resource budget).
    pub fn max_docs_scanned(mut self, n: usize) -> Self {
        self.max_docs_scanned = Some(n);
        self
    }

    /// Restrict partition scope.
    pub fn partitions(mut self, parts: impl IntoIterator<Item = PartitionId>) -> Self {
        self.partitions = Some(parts.into_iter().collect());
        self
    }
}

// Default for ReadMode is needed for ScanOptions::default.
// ReadMode may not implement Default — check modes.rs
// We'll set read_mode manually if needed.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_is_not_complete() {
        let p0 = PartitionId::new(0);
        let p1 = PartitionId::new(1);
        let mut c = Coverage::for_partitions([p0, p1]);
        c.mark_completed(p0, Term(1), LogPosition(3), Some(0));
        c.mark_unavailable(p1);
        assert!(!c.is_complete());
        assert!(c.unavailable.contains(&p1));
        assert!(!c.completed.contains(&p1));
    }

    #[test]
    fn complete_when_all_done() {
        let p0 = PartitionId::new(0);
        let mut c = Coverage::single(p0);
        c.mark_completed(p0, Term(1), LogPosition(1), Some(0));
        assert!(c.is_complete());
    }

    #[test]
    fn resource_limit_makes_incomplete() {
        let p0 = PartitionId::new(0);
        let mut c = Coverage::single(p0);
        c.mark_completed(p0, Term(1), LogPosition(1), Some(0));
        c.mark_resource_limit("budget");
        assert!(c.is_incomplete());
    }

    #[test]
    fn query_id_stable() {
        let p = [PartitionId::new(1), PartitionId::new(2)];
        let a = FindResult::make_query_id(&p, Some("users/"), Some(10));
        let b = FindResult::make_query_id(&p, Some("users/"), Some(10));
        assert_eq!(a, b);
        let c = FindResult::make_query_id(&p, Some("other/"), Some(10));
        assert_ne!(a, c);
    }
}
