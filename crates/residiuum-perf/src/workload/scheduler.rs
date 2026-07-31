//! Batch / concurrency / outstanding-operation scheduler dimensions.

use super::op::LogicalOp;
use serde::{Deserialize, Serialize};

/// Control-plane schedule for a cell (SPEC §6.3 values).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub producer_concurrency: u32,
    pub outstanding_ops: u32,
    pub batch_size: u32,
    pub writer_shards: u32,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            producer_concurrency: 1,
            outstanding_ops: 1,
            batch_size: 1,
            writer_shards: 1,
        }
    }
}

impl ScheduleConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.producer_concurrency == 0 {
            return Err("producer_concurrency must be >= 1".into());
        }
        if self.outstanding_ops == 0 {
            return Err("outstanding_ops must be >= 1".into());
        }
        if self.batch_size == 0 {
            return Err("batch_size must be >= 1".into());
        }
        if self.writer_shards == 0 {
            return Err("writer_shards must be >= 1".into());
        }
        Ok(())
    }

    /// Canonical discrete ladders from SPEC §6.3 (for matrix construction).
    pub fn canonical_concurrency() -> &'static [u32] {
        &[1, 2, 4, 8, 16]
    }

    pub fn canonical_outstanding() -> &'static [u32] {
        &[1, 2, 4, 8, 16, 32, 64]
    }

    pub fn canonical_batch() -> &'static [u32] {
        &[1, 8, 64, 512, 4096]
    }

    pub fn canonical_shards() -> &'static [u32] {
        &[1, 2, 4, 8]
    }
}

/// Ops assigned to one producer after partitioning the global stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerPartition {
    pub producer_id: u32,
    pub ops: Vec<LogicalOp>,
}

/// Partition a logical stream across producers by `seq % concurrency`.
///
/// Partitioning is order-preserving within each producer and does **not**
/// change the multiset of logical ops — re-merge by `seq` recovers the
/// original stream.
pub fn partition_ops(ops: &[LogicalOp], concurrency: u32) -> Vec<ProducerPartition> {
    let n = concurrency.max(1);
    let mut parts: Vec<ProducerPartition> = (0..n)
        .map(|i| ProducerPartition {
            producer_id: i,
            ops: Vec::new(),
        })
        .collect();
    for op in ops {
        let pid = (op.seq % u64::from(n)) as u32;
        let mut owned = op.clone();
        owned.producer_id = pid;
        parts[pid as usize].ops.push(owned);
    }
    parts
}

/// Merge partitions back into global seq order (proves partition invariance).
pub fn merge_partitions(parts: &[ProducerPartition]) -> Vec<LogicalOp> {
    let mut all: Vec<LogicalOp> = parts.iter().flat_map(|p| p.ops.iter().cloned()).collect();
    all.sort_by_key(|o| o.seq);
    all
}

/// Batch boundaries: consecutive ops grouped by `batch_size`.
pub fn batch_groups(ops: &[LogicalOp], batch_size: u32) -> Vec<&[LogicalOp]> {
    let bs = batch_size.max(1) as usize;
    ops.chunks(bs).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workload::op::OpKind;

    fn sample_ops(n: u64) -> Vec<LogicalOp> {
        (0..n)
            .map(|seq| LogicalOp {
                seq,
                kind: OpKind::Insert,
                key_index: seq,
                generation: 0,
                payload_len: 100,
                producer_id: 0,
            })
            .collect()
    }

    #[test]
    fn partition_preserves_multiset() {
        let ops = sample_ops(100);
        let parts = partition_ops(&ops, 4);
        assert_eq!(parts.len(), 4);
        let merged = merge_partitions(&parts);
        assert_eq!(merged.len(), ops.len());
        for (a, b) in ops.iter().zip(merged.iter()) {
            assert_eq!(a.seq, b.seq);
            assert_eq!(a.key_index, b.key_index);
            assert_eq!(a.payload_len, b.payload_len);
            assert_eq!(a.kind, b.kind);
        }
    }

    #[test]
    fn batches_cover_all() {
        let ops = sample_ops(25);
        let groups = batch_groups(&ops, 8);
        assert_eq!(groups.len(), 4); // 8+8+8+1
        let total: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total, 25);
    }
}
