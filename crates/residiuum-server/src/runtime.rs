//! Bounded TCP server runtime (DEF-030).
//!
//! Replaces the sequential accept→handle loop with:
//! - one coordinated store owner per serve process
//! - a bounded set of connection worker threads
//! - connection admission limits and overload responses
//! - idle timeouts, cooperative drain, and mutation accounting
//!
//! Mutations still serialize through a single [`residiuum_store::Store`] under a
//! process-local mutex (exclusive writer ownership from DEF-020). Concurrent
//! connections keep the accept loop free so one slow client cannot starve
//! unrelated clients of network progress.

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Profile tag for the bounded single-node server (DEF-030).
pub const SERVER_PROFILE: &str = "residiuum-server-v1";

/// Default maximum simultaneous client connections.
pub const DEFAULT_MAX_CONNECTIONS: usize = 64;

/// Default idle read/write timeout for an established connection.
pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(120);

/// Default time to wait for in-flight connections after drain begins.
pub const DEFAULT_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

/// Operator-facing connection/admission limits for [`crate::ServeOptions`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLimits {
    /// Maximum simultaneous open client connections.
    pub max_connections: usize,
    /// Idle timeout applied to each connection's socket.
    pub idle_timeout: Duration,
    /// How long graceful shutdown waits for workers after stop-accept.
    pub drain_timeout: Duration,
}

impl Default for ServerLimits {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
            drain_timeout: DEFAULT_DRAIN_TIMEOUT,
        }
    }
}

impl ServerLimits {
    /// Draft defaults used by `residiuum serve` and library helpers.
    pub fn draft_defaults() -> Self {
        Self::default()
    }

    /// Clamp zero/overflow into safe minimums.
    pub fn normalized(self) -> Self {
        Self {
            max_connections: self.max_connections.max(1),
            idle_timeout: if self.idle_timeout.is_zero() {
                DEFAULT_IDLE_TIMEOUT
            } else {
                self.idle_timeout
            },
            drain_timeout: self.drain_timeout,
        }
    }
}

/// Shared counters and flags for one serve process (DEF-030).
#[derive(Debug)]
pub struct ServerRuntime {
    limits: ServerLimits,
    /// When true, stop accepting and refuse new RPCs on open connections.
    draining: AtomicBool,
    /// Optional external shutdown request (set by tests or signal handlers).
    shutdown: Arc<AtomicBool>,
    active_connections: AtomicUsize,
    /// Peak concurrent connections observed since start.
    peak_connections: AtomicUsize,
    /// Connections rejected because the admission limit was full.
    rejected_connections: AtomicU64,
    /// Mutations that began dispatch under this runtime.
    mutations_started: AtomicU64,
    /// Mutations that finished dispatch (success or typed error).
    mutations_finished: AtomicU64,
    /// Total connections accepted (admitted) since start.
    accepted_connections: AtomicU64,
}

impl ServerRuntime {
    /// Create a runtime with the given limits and optional external shutdown flag.
    pub fn new(limits: ServerLimits, shutdown: Option<Arc<AtomicBool>>) -> Arc<Self> {
        Arc::new(Self {
            limits: limits.normalized(),
            draining: AtomicBool::new(false),
            shutdown: shutdown.unwrap_or_else(|| Arc::new(AtomicBool::new(false))),
            active_connections: AtomicUsize::new(0),
            peak_connections: AtomicUsize::new(0),
            rejected_connections: AtomicU64::new(0),
            mutations_started: AtomicU64::new(0),
            mutations_finished: AtomicU64::new(0),
            accepted_connections: AtomicU64::new(0),
        })
    }

    /// Limits in effect for this process.
    pub fn limits(&self) -> &ServerLimits {
        &self.limits
    }

    /// Shared shutdown flag (tests may set this to stop the accept loop).
    pub fn shutdown_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown)
    }

    /// Request shutdown (stop accept + begin drain).
    pub fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.draining.store(true, Ordering::SeqCst);
    }

    /// Whether shutdown was requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    /// Whether the server is draining (no new work).
    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst) || self.is_shutdown_requested()
    }

    /// Begin drain without necessarily having an external shutdown flag set first.
    pub fn begin_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    /// Try to admit one more connection. Returns `false` when at capacity.
    pub fn try_admit_connection(&self) -> bool {
        if self.is_draining() {
            self.rejected_connections.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        let max = self.limits.max_connections;
        loop {
            let cur = self.active_connections.load(Ordering::SeqCst);
            if cur >= max {
                self.rejected_connections.fetch_add(1, Ordering::Relaxed);
                return false;
            }
            match self.active_connections.compare_exchange_weak(
                cur,
                cur + 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    self.accepted_connections.fetch_add(1, Ordering::Relaxed);
                    // Track peak.
                    let mut peak = self.peak_connections.load(Ordering::Relaxed);
                    while cur + 1 > peak {
                        match self.peak_connections.compare_exchange_weak(
                            peak,
                            cur + 1,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            Ok(_) => break,
                            Err(p) => peak = p,
                        }
                    }
                    return true;
                }
                Err(_) => continue,
            }
        }
    }

    /// Release an admitted connection slot (idempotent via guard drop).
    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }

    /// Current number of live connection workers.
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }

    /// Record that a mutating RPC began.
    pub fn mutation_started(&self) {
        self.mutations_started.fetch_add(1, Ordering::Relaxed);
    }

    /// Record that a mutating RPC finished (success or error response path).
    pub fn mutation_finished(&self) {
        self.mutations_finished.fetch_add(1, Ordering::Relaxed);
    }

    /// Snapshot of counters for diagnostics / drain reports.
    pub fn stats(&self) -> ServerStats {
        ServerStats {
            active_connections: self.active_connections(),
            peak_connections: self.peak_connections.load(Ordering::Relaxed),
            rejected_connections: self.rejected_connections.load(Ordering::Relaxed),
            accepted_connections: self.accepted_connections.load(Ordering::Relaxed),
            mutations_started: self.mutations_started.load(Ordering::Relaxed),
            mutations_finished: self.mutations_finished.load(Ordering::Relaxed),
            draining: self.is_draining(),
            max_connections: self.limits.max_connections,
        }
    }

    /// Wait until no connections remain or `drain_timeout` elapses.
    ///
    /// Returns `true` when all connections closed in time.
    pub fn wait_for_idle(&self) -> bool {
        let deadline = std::time::Instant::now() + self.limits.drain_timeout;
        while self.active_connections() > 0 {
            if std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        true
    }
}

/// Point-in-time server counters (DEF-030 diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerStats {
    /// Live connection workers.
    pub active_connections: usize,
    /// High-water mark for concurrent connections.
    pub peak_connections: usize,
    /// Connections refused due to limit or drain.
    pub rejected_connections: u64,
    /// Connections successfully admitted.
    pub accepted_connections: u64,
    /// Mutating RPCs that entered dispatch.
    pub mutations_started: u64,
    /// Mutating RPCs that left dispatch.
    pub mutations_finished: u64,
    /// Whether drain/shutdown is active.
    pub draining: bool,
    /// Configured admission limit.
    pub max_connections: usize,
}

/// RAII guard that releases a connection slot on drop.
pub struct ConnectionGuard {
    runtime: Arc<ServerRuntime>,
}

impl ConnectionGuard {
    /// Call only after a successful [`ServerRuntime::try_admit_connection`].
    pub fn new(runtime: Arc<ServerRuntime>) -> Self {
        Self { runtime }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.runtime.connection_closed();
    }
}

/// RAII guard that pairs mutation_started / mutation_finished.
pub struct MutationGuard {
    runtime: Arc<ServerRuntime>,
}

impl MutationGuard {
    /// Begin accounting for one mutating RPC.
    pub fn new(runtime: Arc<ServerRuntime>) -> Self {
        runtime.mutation_started();
        Self { runtime }
    }
}

impl Drop for MutationGuard {
    fn drop(&mut self) {
        self.runtime.mutation_finished();
    }
}

/// Whether the RPC name mutates store state (needs mutation accounting).
pub fn is_mutating_op(op: &str) -> bool {
    matches!(
        op,
        "put"
            | "put_bytes"
            | "delete"
            | "index_create"
            | "index_drop"
            | "index_rebuild"
            | "purge"
            | "tier_move"
            | "force_reconfig"
            | "salvage_export"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admit_and_reject_at_capacity() {
        let rt = ServerRuntime::new(
            ServerLimits {
                max_connections: 2,
                ..ServerLimits::default()
            },
            None,
        );
        assert!(rt.try_admit_connection());
        assert!(rt.try_admit_connection());
        assert!(!rt.try_admit_connection());
        assert_eq!(rt.active_connections(), 2);
        assert_eq!(rt.stats().rejected_connections, 1);
        rt.connection_closed();
        assert!(rt.try_admit_connection());
    }

    #[test]
    fn drain_rejects_new_admissions() {
        let rt = ServerRuntime::new(ServerLimits::default(), None);
        rt.begin_drain();
        assert!(!rt.try_admit_connection());
        assert!(rt.is_draining());
    }

    #[test]
    fn mutation_accounting_pairs() {
        let rt = ServerRuntime::new(ServerLimits::default(), None);
        {
            let _g = MutationGuard::new(Arc::clone(&rt));
            assert_eq!(rt.stats().mutations_started, 1);
            assert_eq!(rt.stats().mutations_finished, 0);
        }
        assert_eq!(rt.stats().mutations_finished, 1);
    }
}
