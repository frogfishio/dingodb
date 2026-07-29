//! Heap-scoped data façade.

use crate::durability::DurabilityMode;
use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use crate::store::WriteReceipt;
use dingo_format::decode_subject_v2;
use dingo_heap::{refresh_capability_or_terminate, HeapCap};
use std::sync::{Arc, Mutex};

/// Capability-gated heap store. All methods re-check capability liveness.
pub struct HeapStore {
    physical: Arc<Mutex<PhysicalStore>>,
    cap: HeapCap,
}

impl HeapStore {
    pub(super) fn from_host(physical: Arc<Mutex<PhysicalStore>>, cap: HeapCap) -> Self {
        Self { physical, cap }
    }

    /// Bound heap capability.
    pub fn capability(&self) -> &HeapCap {
        &self.cap
    }

    fn gate(&self) -> Result<(), StoreError> {
        refresh_capability_or_terminate(&self.cap)
            .map_err(|e| StoreError::HeapCapability(e.to_string()))
    }

    /// Reject SubjectV2 keys that name a different heap than this capability.
    fn check_subject_heap(&self, subject: &str) -> Result<(), StoreError> {
        let bytes = subject.as_bytes();
        if bytes.first() != Some(&0x02) {
            return Ok(());
        }
        let sv2 = decode_subject_v2(bytes)
            .map_err(|e| StoreError::HeapAdmit(format!("subject v2: {e}")))?;
        if sv2.heap_id != self.cap.heap_id().as_bytes() {
            return Err(StoreError::HeapAdmit("subject heap mismatch".into()));
        }
        Ok(())
    }

    /// Put under a subject within the bound heap.
    pub fn put(&self, subject: &str, value: &[u8]) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.check_subject_heap(subject)?;
        let mut guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.put(subject, value, DurabilityMode::Durable)
    }

    /// Get by subject within the bound heap.
    pub fn get(&self, subject: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.gate()?;
        self.check_subject_heap(subject)?;
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.get(subject)
    }

    /// Delete by subject within the bound heap.
    pub fn delete(&self, subject: &str) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.check_subject_heap(subject)?;
        let mut guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.delete(subject, DurabilityMode::Durable)
    }
}
