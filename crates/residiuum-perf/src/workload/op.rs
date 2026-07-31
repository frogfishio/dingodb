//! Logical operations in the deterministic stream.

use serde::{Deserialize, Serialize};

/// Closed operation kinds for insert / rewrite / history streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    /// Insert a new key with a payload of `payload_len` bytes.
    Insert,
    /// Rewrite an existing key (same key space index) with a new generation.
    Rewrite,
    /// Append a new generation for history-oriented workloads.
    HistoryAppend,
}

impl OpKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Rewrite => "rewrite",
            Self::HistoryAppend => "history_append",
        }
    }
}

/// One logical operation descriptor — **no payload body**.
///
/// Payloads are materialised on demand via [`crate::workload::fill_payload`]
/// so large streams never grow RAM with workload size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalOp {
    /// Monotonic logical sequence number in the global stream (0-based).
    pub seq: u64,
    pub kind: OpKind,
    /// Stable key index in the workload key space.
    pub key_index: u64,
    /// Generation for rewrite / history (0 for first insert).
    pub generation: u32,
    /// Payload length in bytes (materialised later).
    pub payload_len: u64,
    /// Producer that will execute this op after partition (filled by scheduler).
    #[serde(default)]
    pub producer_id: u32,
}

/// Cursor that yields ops from a config without buffering the full stream.
#[derive(Debug, Clone)]
pub struct OpCursor {
    next_seq: u64,
    end_seq: u64,
    plan: Vec<LogicalOp>,
}

impl OpCursor {
    /// Build a cursor over a precomputed plan (small/diagnostic) or use
    /// [`crate::workload::WorkloadManifest::stream`] for streaming.
    pub fn from_plan(plan: Vec<LogicalOp>) -> Self {
        let end_seq = plan.len() as u64;
        Self {
            next_seq: 0,
            end_seq,
            plan,
        }
    }

    pub fn remaining(&self) -> u64 {
        self.end_seq.saturating_sub(self.next_seq)
    }

    pub fn next_op(&mut self) -> Option<LogicalOp> {
        if self.next_seq >= self.end_seq {
            return None;
        }
        let idx = self.next_seq as usize;
        self.next_seq += 1;
        self.plan.get(idx).cloned()
    }
}

impl Iterator for OpCursor {
    type Item = LogicalOp;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_op()
    }
}
