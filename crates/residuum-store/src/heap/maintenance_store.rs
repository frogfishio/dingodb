//! Local maintenance façade (non-serializable maintenance cap).

#![allow(dead_code)] // Wired when HP-009 mints HeapMaintenanceCap.

use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use residuum_heap::HeapMaintenanceCap;
use std::sync::{Arc, Mutex};

/// Maintenance operations requiring a local [`HeapMaintenanceCap`].
pub struct MaintenanceStore {
    _physical: Arc<Mutex<PhysicalStore>>,
    _cap: HeapMaintenanceCap,
}

impl MaintenanceStore {
    /// Construct from host internals (crate-local).
    pub(crate) fn new(physical: Arc<Mutex<PhysicalStore>>, cap: HeapMaintenanceCap) -> Self {
        Self {
            _physical: physical,
            _cap: cap,
        }
    }

    /// Placeholder until HP-009 lifecycle wiring.
    pub fn ping_maintenance(&self) -> Result<(), StoreError> {
        Ok(())
    }
}
