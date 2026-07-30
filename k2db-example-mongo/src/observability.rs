// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::fmt::{Debug, Formatter};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use serde_json::Value;

/// Query lifecycle hooks that mirror the TypeScript before/after query contract.
///
/// ```
/// use k2db::QueryHooks;
///
/// let hooks = QueryHooks::default()
///     .with_before_query(|op, details| {
///         let _ = (op, details);
///     })
///     .with_after_query(|op, details, duration_ms| {
///         let _ = (op, details, duration_ms);
///     });
///
/// assert!(hooks.has_before_query());
/// assert!(hooks.has_after_query());
/// ```
pub type BeforeQueryHook = Arc<dyn Fn(&str, &Value) + Send + Sync>;
pub type AfterQueryHook = Arc<dyn Fn(&str, &Value, u64) + Send + Sync>;

#[derive(Clone, Default)]
pub struct QueryHooks {
    before_query: Option<BeforeQueryHook>,
    after_query: Option<AfterQueryHook>,
}

impl QueryHooks {
    /// Registers a hook that runs before a database operation starts.
    pub fn with_before_query<F>(mut self, hook: F) -> Self
    where
        F: Fn(&str, &Value) + Send + Sync + 'static,
    {
        self.before_query = Some(Arc::new(hook));
        self
    }

    /// Registers a hook that runs after a database operation finishes.
    pub fn with_after_query<F>(mut self, hook: F) -> Self
    where
        F: Fn(&str, &Value, u64) + Send + Sync + 'static,
    {
        self.after_query = Some(Arc::new(hook));
        self
    }

    pub(crate) fn emit_before_query(&self, op: &str, details: &Value) {
        if let Some(hook) = &self.before_query {
            let _ = catch_unwind(AssertUnwindSafe(|| hook(op, details)));
        }
    }

    pub(crate) fn emit_after_query(&self, op: &str, details: &Value, duration_ms: u64) {
        if let Some(hook) = &self.after_query {
            let _ = catch_unwind(AssertUnwindSafe(|| hook(op, details, duration_ms)));
        }
    }

    pub fn has_before_query(&self) -> bool {
        self.before_query.is_some()
    }

    pub fn has_after_query(&self) -> bool {
        self.after_query.is_some()
    }
}

impl Debug for QueryHooks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryHooks")
            .field("before_query", &self.before_query.is_some())
            .field("after_query", &self.after_query.is_some())
            .finish()
    }
}

struct OwnedSink(Box<dyn ratatouille::Sink + Send>);

impl ratatouille::Sink for OwnedSink {
    fn write_line(&mut self, line: &str) {
        self.0.write_line(line);
    }
}

#[derive(Clone)]
/// Thin adapter over the canonical Ratatouille logger used by k2db.
///
/// Ratatouille denies all topics unless the logger filter allows them.
/// Set `filter` to `Some("*".to_owned())` or a narrower allow-list if you
/// expect `k2db:debug` and `k2db:error` events to be emitted.
///
/// ```
/// use k2db::{RatatouilleLogger, ratatouille};
///
/// let logger = RatatouilleLogger::stdout(ratatouille::LoggerConfig {
///     filter: Some("*".to_owned()),
///     ..Default::default()
/// });
///
/// let stats = logger.stats();
/// assert_eq!(stats.emitted, 0);
/// ```
pub struct RatatouilleLogger {
    inner: Arc<Mutex<ratatouille::Logger<OwnedSink>>>,
}

impl RatatouilleLogger {
    /// Creates a logger that writes through Ratatouille's stdout sink.
    pub fn stdout(config: ratatouille::LoggerConfig) -> Self {
        Self::with_sink(config, ratatouille::StdoutSink)
    }

    /// Creates a logger using a caller-provided Ratatouille sink.
    pub fn with_sink<S>(config: ratatouille::LoggerConfig, sink: S) -> Self
    where
        S: ratatouille::Sink + Send + 'static,
    {
        Self {
            inner: Arc::new(Mutex::new(ratatouille::Logger::with_sink(
                config,
                OwnedSink(Box::new(sink)),
            ))),
        }
    }

    pub fn log(&self, topic: &str, message: &str) -> ratatouille::EmitResult {
        let mut logger = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        logger.log(topic, message)
    }

    pub fn stats(&self) -> ratatouille::Stats {
        let logger = self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        logger.stats()
    }
}

impl Debug for RatatouilleLogger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RatatouilleLogger").finish_non_exhaustive()
    }
}