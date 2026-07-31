//! Stage residual accounting (SPEC ε / 5% attribution bound).

use serde::{Deserialize, Serialize};

/// Normative residual bound for stage attribution completeness.
pub const RESIDUAL_MAX_FRACTION: f64 = 0.05;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResidualReport {
    pub e2e_ns: u64,
    pub stages_sum_ns: u64,
    pub residual_ns: i64,
    pub residual_fraction: Option<f64>,
    pub attribution_complete: bool,
}

/// residual = e2e - sum(stages); fraction = |residual| / e2e.
pub fn residual_from_stage_ns(e2e_ns: u64, stages_sum_ns: u64) -> ResidualReport {
    let residual_ns = e2e_ns as i64 - stages_sum_ns as i64;
    let residual_fraction = if e2e_ns > 0 {
        Some((residual_ns as f64).abs() / e2e_ns as f64)
    } else {
        None
    };
    let attribution_complete = residual_fraction
        .map(|f| f <= RESIDUAL_MAX_FRACTION)
        .unwrap_or(false);
    ResidualReport {
        e2e_ns,
        stages_sum_ns,
        residual_ns,
        residual_fraction,
        attribution_complete,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_vector_zero_residual() {
        let r = residual_from_stage_ns(1000, 1000);
        assert_eq!(r.residual_ns, 0);
        assert_eq!(r.residual_fraction, Some(0.0));
        assert!(r.attribution_complete);
    }

    #[test]
    fn independent_vector_over_bound() {
        let r = residual_from_stage_ns(1000, 900); // 10% residual
        assert!(!r.attribution_complete);
        assert!((r.residual_fraction.unwrap() - 0.1).abs() < 1e-9);
    }

    #[test]
    fn under_five_percent_ok() {
        let r = residual_from_stage_ns(1000, 960); // 4%
        assert!(r.attribution_complete);
    }
}
