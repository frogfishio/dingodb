//! Finite vs sustained window detector (SPEC §10 steady-state subset).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowClass {
    /// Short finite run; not claimed as sustained.
    Finite,
    /// Sustained: no monotonic trend >10% across accepted window samples.
    Sustained,
    /// Trend too large or too few samples.
    Inconclusive,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowSample {
    pub throughput_bytes_per_sec: f64,
}

#[derive(Debug, Clone)]
pub struct WindowDetector {
    /// Minimum samples for a sustained claim.
    pub min_samples: usize,
    /// Max relative range (max-min)/median allowed for sustained (default 0.10).
    pub max_trend_fraction: f64,
}

impl Default for WindowDetector {
    fn default() -> Self {
        Self {
            min_samples: 5,
            max_trend_fraction: 0.10,
        }
    }
}

impl WindowDetector {
    pub fn classify(&self, samples: &[WindowSample]) -> WindowClass {
        if samples.is_empty() {
            return WindowClass::Inconclusive;
        }
        if samples.len() < self.min_samples {
            return WindowClass::Finite;
        }
        let mut vals: Vec<f64> = samples
            .iter()
            .map(|s| s.throughput_bytes_per_sec)
            .filter(|v| v.is_finite() && *v >= 0.0)
            .collect();
        if vals.len() < self.min_samples {
            return WindowClass::Inconclusive;
        }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = vals[vals.len() / 2];
        if median <= 0.0 {
            return WindowClass::Inconclusive;
        }
        let min = vals[0];
        let max = *vals.last().unwrap();
        let trend = (max - min) / median;
        if trend <= self.max_trend_fraction {
            WindowClass::Sustained
        } else {
            // Monotonic-ish large swing → inconclusive for sustained claims.
            WindowClass::Inconclusive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_run_is_finite() {
        let d = WindowDetector::default();
        let s = vec![
            WindowSample {
                throughput_bytes_per_sec: 1e8,
            };
            3
        ];
        assert_eq!(d.classify(&s), WindowClass::Finite);
    }

    #[test]
    fn flat_window_sustained() {
        let d = WindowDetector::default();
        let s: Vec<_> = (0..8)
            .map(|_| WindowSample {
                throughput_bytes_per_sec: 1e8,
            })
            .collect();
        assert_eq!(d.classify(&s), WindowClass::Sustained);
    }

    #[test]
    fn large_trend_inconclusive() {
        let d = WindowDetector::default();
        let s = vec![
            WindowSample {
                throughput_bytes_per_sec: 1e8,
            },
            WindowSample {
                throughput_bytes_per_sec: 1e8,
            },
            WindowSample {
                throughput_bytes_per_sec: 1e8,
            },
            WindowSample {
                throughput_bytes_per_sec: 1e8,
            },
            WindowSample {
                throughput_bytes_per_sec: 5e8,
            },
        ];
        assert_eq!(d.classify(&s), WindowClass::Inconclusive);
    }
}
