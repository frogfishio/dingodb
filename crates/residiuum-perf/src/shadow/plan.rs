//! Closed PhysicalWritePlan — sizes/order/destination/sync only (no payload/identity).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PLAN_SCHEMA: &str = "residiuum-physical-write-plan-v1";

/// Where a planned write lands (class only — no path/identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DestinationClass {
    /// Primary segment / data file append stream.
    SegmentData,
    /// Segment metadata / trailer.
    SegmentMeta,
    /// Chunk payload stream.
    ChunkData,
    /// Directory / fsync of parent (no bytes).
    Directory,
}

/// Sync obligation attached after an op (or stand-alone).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncBoundary {
    None,
    DataOnly,
    FullFile,
    Directory,
}

/// One planned physical I/O step — **no payload bytes, no keys, no heap IDs**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalOp {
    /// Monotonic order in the plan (0-based).
    pub seq: u32,
    pub dest: DestinationClass,
    /// Byte length of opaque write; 0 for pure sync/dir ops.
    pub size: u64,
    /// Alignment hint (e.g. 4096); 0 = unspecified.
    pub alignment: u32,
    pub sync_after: SyncBoundary,
    /// Logical segment generation after this op (rotation tracking).
    pub segment_gen: u32,
    /// True when this op starts a new segment file (rotation).
    pub segment_rotate: bool,
    /// True when size crossed the chunk threshold boundary.
    pub chunk_boundary: bool,
    /// Ops in the current batch when this was emitted.
    pub batch_index: u32,
}

/// Closed plan: ordered physical ops + accounting digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicalWritePlan {
    pub schema: String,
    /// Stable id for this plan shape (not a run id).
    pub plan_id: String,
    pub shape_hash: String,
    pub ops: Vec<PhysicalOp>,
    pub planned_bytes: u64,
    pub planned_syncs: u32,
    pub planned_rotations: u32,
    pub segment_threshold: u64,
    pub chunk_threshold: u64,
    pub batch_size: u32,
    pub sync_every_ops: u32,
}

impl PhysicalWritePlan {
    /// Redacted view for traces: same fields (already free of payload/identity).
    pub fn redacted_trace(&self) -> PhysicalWritePlan {
        // Plan is already redacted by construction; clone for API symmetry.
        self.clone()
    }

    /// Assert no forbidden fields appear in serialized form.
    pub fn assert_redacted_json(&self) -> Result<(), String> {
        let s = serde_json::to_string(self).map_err(|e| e.to_string())?;
        for bad in [
            "\"payload\"",
            "\"key\"",
            "\"document\"",
            "\"heap_id\"",
            "\"credential\"",
            "\"body\"",
        ] {
            if s.contains(bad) {
                return Err(format!("plan JSON must not contain {bad}"));
            }
        }
        Ok(())
    }

    pub fn content_digest(&self) -> String {
        let mut h = Sha256::new();
        h.update(PLAN_SCHEMA.as_bytes());
        h.update(b"\0");
        for op in &self.ops {
            h.update(op.seq.to_le_bytes());
            h.update((op.dest as u8).to_le_bytes());
            h.update(op.size.to_le_bytes());
            h.update(op.alignment.to_le_bytes());
            h.update((op.sync_after as u8).to_le_bytes());
            h.update(op.segment_gen.to_le_bytes());
            h.update([op.segment_rotate as u8, op.chunk_boundary as u8]);
            h.update(op.batch_index.to_le_bytes());
        }
        hex::encode(h.finalize())
    }
}

/// Residiuum physical shape knobs used to **plan** I/O (store should emit same).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeConfig {
    pub plan_id: String,
    /// Logical payload sizes stream (opaque lengths only).
    pub write_sizes: Vec<u64>,
    /// Soft segment rotate threshold (bytes written to current segment).
    pub segment_threshold: u64,
    /// Chunk threshold (values >= this mark chunk_boundary).
    pub chunk_threshold: u64,
    pub batch_size: u32,
    /// Insert FullFile sync every N data ops (0 = only at end if configured).
    pub sync_every_ops: u32,
    pub alignment: u32,
    /// Append a final FullFile sync.
    pub final_sync: bool,
}

impl Default for ShapeConfig {
    fn default() -> Self {
        Self {
            plan_id: "shape-default".into(),
            write_sizes: vec![4096; 16],
            segment_threshold: 64 * 1024,
            chunk_threshold: 1024 * 1024,
            batch_size: 8,
            sync_every_ops: 4,
            alignment: 4096,
            final_sync: true,
        }
    }
}

pub struct PlanBuilder;

impl PlanBuilder {
    /// Build a closed plan from shape config + opaque size stream.
    pub fn build(cfg: &ShapeConfig) -> PhysicalWritePlan {
        let mut ops = Vec::new();
        let mut seq = 0u32;
        let mut planned_bytes = 0u64;
        let mut planned_syncs = 0u32;
        let mut planned_rotations = 0u32;
        let mut segment_gen = 0u32;
        let mut segment_bytes = 0u64;
        let mut data_ops = 0u32;
        let batch = cfg.batch_size.max(1);

        for &size in &cfg.write_sizes {
            // Rotate if adding would cross threshold (and segment non-empty).
            let mut rotate = false;
            if segment_bytes > 0 && segment_bytes.saturating_add(size) > cfg.segment_threshold {
                rotate = true;
                segment_gen = segment_gen.saturating_add(1);
                segment_bytes = 0;
                planned_rotations = planned_rotations.saturating_add(1);
            }

            let chunk_boundary = size >= cfg.chunk_threshold;
            let batch_index = data_ops % batch;
            let mut sync_after = SyncBoundary::None;
            data_ops = data_ops.saturating_add(1);
            if cfg.sync_every_ops > 0 && data_ops % cfg.sync_every_ops == 0 {
                sync_after = SyncBoundary::FullFile;
                planned_syncs = planned_syncs.saturating_add(1);
            }

            ops.push(PhysicalOp {
                seq,
                dest: DestinationClass::SegmentData,
                size,
                alignment: cfg.alignment,
                sync_after,
                segment_gen,
                segment_rotate: rotate,
                chunk_boundary,
                batch_index,
            });
            seq = seq.saturating_add(1);
            planned_bytes = planned_bytes.saturating_add(size);
            segment_bytes = segment_bytes.saturating_add(size);

            // Optional metadata cadence every batch end.
            if batch_index + 1 == batch {
                ops.push(PhysicalOp {
                    seq,
                    dest: DestinationClass::SegmentMeta,
                    size: 64, // opaque meta trailer size
                    alignment: 0,
                    sync_after: SyncBoundary::None,
                    segment_gen,
                    segment_rotate: false,
                    chunk_boundary: false,
                    batch_index,
                });
                seq = seq.saturating_add(1);
                planned_bytes = planned_bytes.saturating_add(64);
            }
        }

        if cfg.final_sync {
            ops.push(PhysicalOp {
                seq,
                dest: DestinationClass::SegmentData,
                size: 0,
                alignment: 0,
                sync_after: SyncBoundary::FullFile,
                segment_gen,
                segment_rotate: false,
                chunk_boundary: false,
                batch_index: 0,
            });
            planned_syncs = planned_syncs.saturating_add(1);
        }

        let mut plan = PhysicalWritePlan {
            schema: PLAN_SCHEMA.into(),
            plan_id: cfg.plan_id.clone(),
            shape_hash: String::new(),
            ops,
            planned_bytes,
            planned_syncs,
            planned_rotations,
            segment_threshold: cfg.segment_threshold,
            chunk_threshold: cfg.chunk_threshold,
            batch_size: batch,
            sync_every_ops: cfg.sync_every_ops,
        };
        plan.shape_hash = plan.content_digest();
        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_golden_stable() {
        let cfg = ShapeConfig {
            plan_id: "g1".into(),
            write_sizes: vec![100, 200, 300],
            segment_threshold: 250,
            chunk_threshold: 250,
            batch_size: 2,
            sync_every_ops: 2,
            alignment: 8,
            final_sync: true,
        };
        let a = PlanBuilder::build(&cfg);
        let b = PlanBuilder::build(&cfg);
        assert_eq!(a.shape_hash, b.shape_hash);
        assert_eq!(a.ops.len(), b.ops.len());
        assert!(a.planned_rotations >= 1); // 100+200 crosses 250
        assert!(a.ops.iter().any(|o| o.chunk_boundary));
        a.assert_redacted_json().unwrap();
    }

    #[test]
    fn redaction_rejects_payload_injection() {
        // Ensure our schema field names don't include payload.
        let p = PlanBuilder::build(&ShapeConfig::default());
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("payload"));
        assert!(json.contains("SegmentData") || json.contains("segment_data"));
    }

    #[test]
    fn segment_threshold_marks_rotate() {
        let cfg = ShapeConfig {
            write_sizes: vec![1000, 1000, 1000],
            segment_threshold: 1500,
            batch_size: 100,
            sync_every_ops: 0,
            final_sync: false,
            ..ShapeConfig::default()
        };
        let p = PlanBuilder::build(&cfg);
        let rotates: Vec<_> = p.ops.iter().filter(|o| o.segment_rotate).collect();
        assert!(!rotates.is_empty());
        assert_eq!(p.planned_rotations, rotates.len() as u32);
    }
}
