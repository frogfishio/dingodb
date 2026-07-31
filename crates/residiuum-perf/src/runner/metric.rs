//! Metric observations: available value vs unavailable reason (never zero-as-absent).

use serde::{Deserialize, Serialize};

/// Closed representation: a metric is either a real number or unavailable with reason.
/// Zero is a legal measured value and MUST NOT encode unavailability
/// (`zero_is_not_unavailable` policy from PQH-0 registries).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MetricObservation {
    Available {
        value: f64,
        unit: String,
    },
    Unavailable {
        reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
}

impl MetricObservation {
    pub fn available(value: f64, unit: impl Into<String>) -> Self {
        Self::Available {
            value,
            unit: unit.into(),
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self::Unavailable {
            reason: reason.into(),
            detail: None,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    /// Fail-closed: treating zero as "missing" is forbidden.
    pub fn reject_zero_as_unavailable(value: f64, unit: impl Into<String>) -> Self {
        // Callers that would map "missing" → 0 must use Unavailable instead.
        // This helper always records an available zero when value is 0.0.
        Self::available(value, unit)
    }
}

/// Typed unavailable reason codes used by the runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnavailableMetric {
    PlatformUnsupported,
    ObserverFailed,
    NotSampled,
    PermissionDenied,
}

impl UnavailableMetric {
    pub fn as_reason(&self) -> &'static str {
        match self {
            Self::PlatformUnsupported => "platform_unsupported",
            Self::ObserverFailed => "observer_failed",
            Self::NotSampled => "not_sampled",
            Self::PermissionDenied => "permission_denied",
        }
    }

    pub fn observation(self) -> MetricObservation {
        MetricObservation::unavailable(self.as_reason())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_available_not_missing() {
        let m = MetricObservation::reject_zero_as_unavailable(0.0, "ops/s");
        assert!(m.is_available());
        match m {
            MetricObservation::Available { value, .. } => assert_eq!(value, 0.0),
            _ => panic!("expected available"),
        }
    }

    #[test]
    fn unavailable_serializes_without_fake_zero() {
        let m = UnavailableMetric::PlatformUnsupported.observation();
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("unavailable"));
        assert!(!json.contains("\"value\":0"));
    }
}
