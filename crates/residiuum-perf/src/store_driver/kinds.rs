//! Driver kind and measurement-surface labels (honesty gate).

use serde::{Deserialize, Serialize};

pub const DRIVER_KIND_SYNTHETIC: &str = "synthetic";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverKind {
    /// Harness proxy — never a product performance claim.
    Synthetic,
    /// Real `residiuum-store` L4/L5/L6 path (feature-gated).
    RealStore,
}

impl DriverKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Synthetic => DRIVER_KIND_SYNTHETIC,
            Self::RealStore => "real_store",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "synthetic" | "proxy" | "harness" => Some(Self::Synthetic),
            "real_store" | "store" | "residiuum-store" => Some(Self::RealStore),
            _ => None,
        }
    }
}

/// What kind of measurement surface a report describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementSurface {
    /// Synthetic/proxy — **non-product**.
    NonProductSynthetic,
    /// Real store on a non-baseline platform (e.g. developer laptop).
    RealStoreUncontrolled,
    /// Real store on a controlled runner platform class.
    RealStoreControlledEligible,
}

impl MeasurementSurface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NonProductSynthetic => "non_product_synthetic",
            Self::RealStoreUncontrolled => "real_store_uncontrolled",
            Self::RealStoreControlledEligible => "real_store_controlled_eligible",
        }
    }

    pub fn allows_product_claim(self) -> bool {
        matches!(self, Self::RealStoreControlledEligible)
    }
}
