//! Additive feature profiles (L5) and background interference.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdditiveFeature {
    PrimaryIndex,
    SecondaryIndex,
    InlineValues,
    ChunkedValues,
    SealLifecycle,
    CheckpointSidecar,
    IntegrityVerify,
    Encryption,
    BoundedTelemetry,
}

impl AdditiveFeature {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PrimaryIndex => "primary_index",
            Self::SecondaryIndex => "secondary_index",
            Self::InlineValues => "inline_values",
            Self::ChunkedValues => "chunked_values",
            Self::SealLifecycle => "seal_lifecycle",
            Self::CheckpointSidecar => "checkpoint_sidecar",
            Self::IntegrityVerify => "integrity_verify",
            Self::Encryption => "encryption",
            Self::BoundedTelemetry => "bounded_telemetry",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureProfile {
    pub name: String,
    pub features: Vec<AdditiveFeature>,
}

impl FeatureProfile {
    pub fn l4_minimal() -> Self {
        Self {
            name: "l4_minimal".into(),
            features: vec![AdditiveFeature::PrimaryIndex, AdditiveFeature::InlineValues],
        }
    }

    pub fn single(f: AdditiveFeature) -> Self {
        Self {
            name: format!("l5_{}", f.as_str()),
            features: vec![AdditiveFeature::PrimaryIndex, f],
        }
    }

    pub fn realistic() -> Self {
        Self {
            name: "l5_realistic".into(),
            features: vec![
                AdditiveFeature::PrimaryIndex,
                AdditiveFeature::SecondaryIndex,
                AdditiveFeature::ChunkedValues,
                AdditiveFeature::SealLifecycle,
                AdditiveFeature::IntegrityVerify,
            ],
        }
    }

    pub fn l6_complete() -> Self {
        Self {
            name: "l6_complete".into(),
            features: vec![
                AdditiveFeature::PrimaryIndex,
                AdditiveFeature::SecondaryIndex,
                AdditiveFeature::ChunkedValues,
                AdditiveFeature::SealLifecycle,
                AdditiveFeature::CheckpointSidecar,
                AdditiveFeature::IntegrityVerify,
                AdditiveFeature::BoundedTelemetry,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundInterference {
    Absent,
    SealCheckpoint,
    Scrub,
    Chaos,
    Telemetry,
}

impl BackgroundInterference {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::SealCheckpoint => "seal_checkpoint",
            Self::Scrub => "scrub",
            Self::Chaos => "chaos",
            Self::Telemetry => "telemetry",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterferenceProfile {
    pub kind: BackgroundInterference,
    /// Synthetic cost units added per op when active.
    pub cost_ns_per_op: u64,
}

impl InterferenceProfile {
    pub fn absent() -> Self {
        Self {
            kind: BackgroundInterference::Absent,
            cost_ns_per_op: 0,
        }
    }

    pub fn for_kind(kind: BackgroundInterference) -> Self {
        let cost = match kind {
            BackgroundInterference::Absent => 0,
            BackgroundInterference::SealCheckpoint => 200,
            BackgroundInterference::Scrub => 150,
            BackgroundInterference::Chaos => 300,
            BackgroundInterference::Telemetry => 50,
        };
        Self {
            kind,
            cost_ns_per_op: cost,
        }
    }
}
