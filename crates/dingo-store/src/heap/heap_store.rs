//! Heap-scoped data façade.

use crate::durability::DurabilityMode;
use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use crate::store::WriteReceipt;
use dingo_format::{decode_subject_v2, SubjectObjectKind};
use dingo_heap::{refresh_capability_or_terminate, HeapCap, Rights};
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

    fn require_right(&self, required: Rights) -> Result<(), StoreError> {
        if self.cap.rights().contains(required) {
            Ok(())
        } else {
            Err(StoreError::HeapCapability(format!(
                "missing right {}",
                required.bits()
            )))
        }
    }

    /// Decode and validate a SubjectV2 buffer for this bound heap.
    ///
    /// Qualified heap data paths require SubjectV2 (version byte `0x02`). Legacy
    /// v1 string subjects are rejected so foreign-heap names cannot ride the
    /// flat keyspace.
    fn require_subject_v2(
        &self,
        subject: &[u8],
        expect_kind: Option<SubjectObjectKind>,
        expect_object_id: Option<&[u8; 16]>,
    ) -> Result<(), StoreError> {
        let sv2 = decode_subject_v2(subject)
            .map_err(|e| StoreError::HeapAdmit(format!("subject v2: {e}")))?;
        if sv2.heap_id != self.cap.heap_id().as_bytes() {
            return Err(StoreError::HeapAdmit("subject heap mismatch".into()));
        }
        if let Some(kind) = expect_kind {
            if sv2.object_kind != kind {
                return Err(StoreError::HeapAdmit("subject object kind mismatch".into()));
            }
        }
        if let Some(oid) = expect_object_id {
            if sv2.object_id != oid {
                return Err(StoreError::HeapAdmit("subject object id mismatch".into()));
            }
        }
        Ok(())
    }

    /// Put under a SubjectV2 key within the bound heap.
    pub fn put(&self, subject: &[u8], value: &[u8]) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.require_right(Rights::WRITE)?;
        self.require_subject_v2(subject, None, None)?;
        let mut guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.put_subject_bytes(subject, value, DurabilityMode::Durable)
    }

    /// Get by SubjectV2 key within the bound heap.
    pub fn get(&self, subject: &[u8]) -> Result<Option<Vec<u8>>, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        self.require_subject_v2(subject, None, None)?;
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.get_subject_bytes(subject)
    }

    /// Delete by SubjectV2 key within the bound heap.
    pub fn delete(&self, subject: &[u8]) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.require_right(Rights::WRITE)?;
        self.require_subject_v2(subject, None, None)?;
        let mut guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.delete_subject_bytes(subject, DurabilityMode::Durable)
    }

    /// Put under a collection-scoped SubjectV2 (object id + key must match).
    pub fn put_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
        value: &[u8],
    ) -> Result<WriteReceipt, StoreError> {
        let subject = dingo_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.put(&subject, value)
    }

    /// Get a collection-scoped SubjectV2 value.
    pub fn get_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let subject = dingo_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.get(&subject)
    }

    /// Delete a collection-scoped SubjectV2 value.
    pub fn delete_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
    ) -> Result<WriteReceipt, StoreError> {
        let subject = dingo_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.delete(&subject)
    }
}
