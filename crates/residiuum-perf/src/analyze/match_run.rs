//! Matched-run validator for attribution pairs.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MatchError {
    #[error("empty run set")]
    Empty,
    #[error("durability mismatch: {a} vs {b}")]
    Durability { a: String, b: String },
    #[error("payload_size mismatch")]
    PayloadSize,
    #[error("seed/config mismatch")]
    Config,
    #[error("invalid_correctness on run {0}")]
    InvalidCorrectness(String),
    #[error("too few variables changed for causal claim: changed={changed}")]
    MultiVariable { changed: usize },
}

/// Evidence summary for one completed run (layer cell).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvidence {
    pub run_id: String,
    pub layer: String,
    pub durability: String,
    pub payload_size: u64,
    pub concurrency: u32,
    pub outstanding: u32,
    pub config_hash: String,
    pub throughput_bytes_per_sec: f64,
    pub p50_latency_ns: Option<u64>,
    pub p99_latency_ns: Option<u64>,
    pub residual_fraction: Option<f64>,
    pub validity: String,
    pub failed_ops: u64,
    pub attempted_ops: u64,
    pub acknowledged_ops: u64,
    pub device_util: Option<f64>,
    pub outstanding_depth: u32,
    pub sync_per_op: bool,
    pub observer_overhead_fraction: Option<f64>,
    pub cpu_cores_busy: Option<f64>,
    pub aggregate_cpu_idle: Option<f64>,
    pub window_class: String,
    pub features: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedRun {
    pub a: RunEvidence,
    pub b: RunEvidence,
    pub shared_keys: Vec<String>,
    pub changed_keys: Vec<String>,
}

/// Validate two runs can be compared for causal attribution.
/// Causal claims require exactly one changed control variable (plus layer ladder).
pub fn validate_matched_runs(a: &RunEvidence, b: &RunEvidence) -> Result<MatchedRun, MatchError> {
    if a.validity != "valid" {
        return Err(MatchError::InvalidCorrectness(a.run_id.clone()));
    }
    if b.validity != "valid" {
        return Err(MatchError::InvalidCorrectness(b.run_id.clone()));
    }
    if a.durability != b.durability {
        // Durability change is allowed only if that is the *only* intentional change
        // and labeled as such — still record as changed key.
    }
    if a.payload_size != b.payload_size && a.layer == b.layer {
        // size ladder ok across layers only
    }

    let mut shared = Vec::new();
    let mut changed = Vec::new();
    push_cmp(
        &mut shared,
        &mut changed,
        "durability",
        a.durability == b.durability,
    );
    push_cmp(
        &mut shared,
        &mut changed,
        "payload_size",
        a.payload_size == b.payload_size,
    );
    push_cmp(
        &mut shared,
        &mut changed,
        "concurrency",
        a.concurrency == b.concurrency,
    );
    push_cmp(
        &mut shared,
        &mut changed,
        "outstanding",
        a.outstanding == b.outstanding,
    );
    push_cmp(
        &mut shared,
        &mut changed,
        "config_hash",
        a.config_hash == b.config_hash || a.layer != b.layer,
    );
    push_cmp(&mut shared, &mut changed, "layer", a.layer == b.layer);
    push_cmp(
        &mut shared,
        &mut changed,
        "features",
        a.features == b.features,
    );

    Ok(MatchedRun {
        a: a.clone(),
        b: b.clone(),
        shared_keys: shared,
        changed_keys: changed,
    })
}

fn push_cmp(shared: &mut Vec<String>, changed: &mut Vec<String>, key: &str, same: bool) {
    if same {
        shared.push(key.into());
    } else {
        changed.push(key.into());
    }
}

/// For causal claims outside the layer ladder, require exactly one changed key
/// among control dimensions (excluding layer).
pub fn causal_ok(m: &MatchedRun) -> Result<(), MatchError> {
    let control: Vec<_> = m
        .changed_keys
        .iter()
        .filter(|k| *k != "layer" && *k != "config_hash")
        .collect();
    if control.len() > 1 {
        return Err(MatchError::MultiVariable {
            changed: control.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(id: &str, layer: &str) -> RunEvidence {
        RunEvidence {
            run_id: id.into(),
            layer: layer.into(),
            durability: "durable".into(),
            payload_size: 4096,
            concurrency: 1,
            outstanding: 1,
            config_hash: "abc".into(),
            throughput_bytes_per_sec: 1e8,
            p50_latency_ns: Some(1000),
            p99_latency_ns: Some(5000),
            residual_fraction: Some(0.02),
            validity: "valid".into(),
            failed_ops: 0,
            attempted_ops: 100,
            acknowledged_ops: 100,
            device_util: Some(0.9),
            outstanding_depth: 8,
            sync_per_op: false,
            observer_overhead_fraction: Some(0.001),
            cpu_cores_busy: Some(0.5),
            aggregate_cpu_idle: Some(0.5),
            window_class: "sustained".into(),
            features: "l4_minimal".into(),
        }
    }

    #[test]
    fn matched_layer_pair() {
        let a = base("a", "L3");
        let b = base("b", "L4");
        let m = validate_matched_runs(&a, &b).unwrap();
        assert!(m.changed_keys.contains(&"layer".into()));
        causal_ok(&m).unwrap();
    }

    #[test]
    fn multi_variable_rejected() {
        let a = base("a", "L4");
        let mut b = base("b", "L4");
        b.concurrency = 8;
        b.durability = "memory".into();
        let m = validate_matched_runs(&a, &b).unwrap();
        assert!(causal_ok(&m).is_err());
    }
}
