//! Recovery façade emitting Known/Unknown/Conflict packages.

#![allow(dead_code)] // Wired when HP-009 mints RecoveryCap; ownership helpers are usable.

use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use residiuum_format::{agree_ownership, parse_ownership_envelope, OwnershipEvidence};
use residiuum_heap::RecoveryCap;
use std::sync::{Arc, Mutex};

/// Recovery-plane store view.
pub struct RecoveryStore {
    _physical: Arc<Mutex<PhysicalStore>>,
    _cap: RecoveryCap,
}

impl RecoveryStore {
    /// Construct from host internals.
    pub(crate) fn new(physical: Arc<Mutex<PhysicalStore>>, cap: RecoveryCap) -> Self {
        Self {
            _physical: physical,
            _cap: cap,
        }
    }

    /// Classify ownership evidence from an envelope (no heap reassignment).
    pub fn classify_envelope(&self, envelope: &[u8]) -> Result<OwnershipEvidence, StoreError> {
        parse_ownership_envelope(envelope)
            .map_err(|e| StoreError::HeapCapability(format!("ownership: {e}")))
    }

    /// Agree two claims; surfaces conflict without choosing a winner.
    pub fn agree(
        &self,
        a: &OwnershipEvidence,
        b: &OwnershipEvidence,
    ) -> Result<OwnershipEvidence, StoreError> {
        agree_ownership(a, b).map_err(|e| StoreError::HeapCapability(format!("ownership: {e}")))
    }
}
