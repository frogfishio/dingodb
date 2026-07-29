//! Heap-scoped auth audit diagnostics (`HEAP_SPEC` §33 / HP-008).
//!
//! Internal causes never appear on the wire. The public reject shape is always
//! identical; this sink records bounded evidence for operators only.

use crate::heap_auth::HeapAuthInternalCause;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Default bound on retained audit records.
pub const DEFAULT_HEAP_AUDIT_CAPACITY: usize = 256;

/// One heap-auth audit event (never serialized to clients).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapAuthAuditEvent {
    /// Uniform reject with internal cause.
    Reject {
        /// Fail-closed cause.
        cause: HeapAuthInternalCause,
    },
    /// Successful welcome for a heap (canonical UUID string).
    Welcome {
        /// Heap id display form.
        heap_id: String,
    },
}

/// Bounded in-memory audit log for heap-key handshake diagnostics.
#[derive(Debug)]
pub struct HeapAuthAuditLog {
    inner: Mutex<VecDeque<HeapAuthAuditEvent>>,
    capacity: usize,
}

impl Default for HeapAuthAuditLog {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_HEAP_AUDIT_CAPACITY)
    }
}

impl HeapAuthAuditLog {
    /// Create with a fixed retention bound.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(VecDeque::with_capacity(capacity.min(4096))),
            capacity: capacity.max(1).min(4096),
        }
    }

    /// Record a reject (audit only).
    pub fn record_reject(&self, cause: HeapAuthInternalCause) {
        self.push(HeapAuthAuditEvent::Reject { cause });
    }

    /// Record a successful welcome (audit only).
    pub fn record_welcome(&self, heap_id: &str) {
        self.push(HeapAuthAuditEvent::Welcome {
            heap_id: heap_id.to_string(),
        });
    }

    /// Snapshot current events (oldest first).
    pub fn snapshot(&self) -> Vec<HeapAuthAuditEvent> {
        self.inner
            .lock()
            .expect("heap audit lock")
            .iter()
            .cloned()
            .collect()
    }

    /// Number of retained events.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("heap audit lock").len()
    }

    /// Whether the log is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn push(&self, event: HeapAuthAuditEvent) {
        let mut g = self.inner.lock().expect("heap audit lock");
        while g.len() >= self.capacity {
            g.pop_front();
        }
        g.push_back(event);
    }
}
