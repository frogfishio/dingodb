//! Deterministic sampling (default 1 in 1024) — SPEC §7.

use serde::{Deserialize, Serialize};

/// Default sample rate: 1 in 1024 operations.
pub const DEFAULT_SAMPLE_RATE: u64 = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplerConfig {
    /// Sample every N-th op when using modular policy (`seq % rate == 0`).
    /// `rate == 1` samples all; `rate == 0` samples none.
    pub rate: u64,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            rate: DEFAULT_SAMPLE_RATE,
        }
    }
}

/// Deterministic, platform-independent sample decision for logical sequence `seq`.
///
/// Uses pure modular arithmetic (not OS RNG) so the same workload seed/config
/// samples the same ops on every host.
pub fn should_sample(seq: u64, cfg: &SamplerConfig) -> bool {
    if cfg.rate == 0 {
        return false;
    }
    if cfg.rate == 1 {
        return true;
    }
    seq % cfg.rate == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rate_one_in_1024() {
        let cfg = SamplerConfig::default();
        assert!(should_sample(0, &cfg));
        assert!(!should_sample(1, &cfg));
        assert!(should_sample(1024, &cfg));
        let mut n = 0u64;
        for s in 0..10_000 {
            if should_sample(s, &cfg) {
                n += 1;
            }
        }
        // floor(9999/1024)+1 for seq 0 = about 10
        assert_eq!(n, 10);
    }

    #[test]
    fn deterministic_across_calls() {
        let cfg = SamplerConfig { rate: 7 };
        for s in 0..100 {
            assert_eq!(should_sample(s, &cfg), should_sample(s, &cfg));
        }
    }
}
