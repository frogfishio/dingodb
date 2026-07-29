//! Resident heap slot registry for the qualified data-plane hot path (HP-008).
//!
//! Lookups resolve only in-memory [`HeapSlot`] entries. This module MUST NOT
//! open or query a master authority store.

use dingo_heap::{HeapId, HeapSlot};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory map of published heaps available to the data server.
#[derive(Debug, Default)]
pub struct ResidentHeapRegistry {
    inner: RwLock<HashMap<HeapId, ResidentHeap>>,
}

/// One resident heap entry.
#[derive(Clone)]
pub struct ResidentHeap {
    /// Atomic security snapshot.
    pub slot: Arc<HeapSlot>,
    /// Optional human-facing name (checked only after authority succeeds).
    pub display_name: Option<String>,
}

impl std::fmt::Debug for ResidentHeap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResidentHeap")
            .field("display_name", &self.display_name)
            .finish_non_exhaustive()
    }
}

impl ResidentHeapRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a resident heap (reload / publish path only).
    pub fn insert(&self, heap_id: HeapId, entry: ResidentHeap) {
        self.inner
            .write()
            .expect("registry lock")
            .insert(heap_id, entry);
    }

    /// Resolve by heap id. Misses are fail-closed at the handshake layer.
    pub fn get(&self, heap_id: &HeapId) -> Option<ResidentHeap> {
        self.inner
            .read()
            .expect("registry lock")
            .get(heap_id)
            .cloned()
    }

    /// Number of resident heaps (diagnostics only).
    pub fn len(&self) -> usize {
        self.inner.read().expect("registry lock").len()
    }

    /// Whether the registry has no heaps.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
