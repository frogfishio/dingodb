//! Replica façade.

#![allow(dead_code)] // Wired when HP-011 mints ReplicaCap.

use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use residuum_heap::ReplicaCap;
use std::sync::{Arc, Mutex};

/// Replica-owner gated store view.
pub struct ReplicaCapStore {
    _physical: Arc<Mutex<PhysicalStore>>,
    _cap: ReplicaCap,
}

impl ReplicaCapStore {
    /// Construct from host internals.
    pub(crate) fn new(physical: Arc<Mutex<PhysicalStore>>, cap: ReplicaCap) -> Self {
        Self {
            _physical: physical,
            _cap: cap,
        }
    }

    /// Placeholder until HP-011.
    pub fn ping_replica(&self) -> Result<(), StoreError> {
        Ok(())
    }
}
