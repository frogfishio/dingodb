//! PQH-3: metrics, probes, histograms, and result kernel.
//!
//! Fixed-bucket latency histograms, monotonic stage clocks, deterministic
//! sampling, bounded per-thread aggregation, probe modes, host sampler
//! interface, and post-timing artifact writers.

mod aggregate;
mod clock;
mod counters;
mod histogram;
mod probe;
mod result;
mod sample;
mod sampler;
mod writer;

pub use aggregate::{merge_thread_aggregates, ThreadAggregate};
pub use clock::{MonotonicClock, StageTimestamp, TimestampError};
pub use counters::{CounterId, CounterSet, COUNTER_IDS};
pub use histogram::{LatencyHistogram, Percentiles, HISTOGRAM_BUCKETS};
pub use probe::{InstrumentationBudget, ProbeMode, ProbeSession};
pub use result::{MetricMap, ResultKernel, StageResiduals};
pub use sample::{should_sample, SamplerConfig, DEFAULT_SAMPLE_RATE};
pub use sampler::{HostSample, HostSampler, NullHostSampler, ProcessSample};
pub use writer::{hash_bytes, write_histograms_json, write_result_json, write_timeseries_ndjson};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("metrics: {0}")]
    Msg(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("timestamp: {0}")]
    Timestamp(#[from] TimestampError),
}
