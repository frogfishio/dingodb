//! Store host with no unscoped data plane (`HEAP_SPEC` HP-003).

use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use residuum_heap::HeapCap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::HeapStore;

/// Deployment-level host. Exposes no get/put/scan of application data.
pub struct StoreHost {
    physical: Arc<Mutex<PhysicalStore>>,
    root: PathBuf,
}

impl StoreHost {
    /// Open an existing store directory as a host.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        Ok(Self {
            physical: Arc::new(Mutex::new(PhysicalStore::open(path)?)),
            root: path.to_path_buf(),
        })
    }

    /// Create a new store directory as a host.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        Ok(Self {
            physical: Arc::new(Mutex::new(PhysicalStore::create(path)?)),
            root: path.to_path_buf(),
        })
    }

    /// Share an already-opened physical store (e.g. server exclusive writer).
    ///
    /// Used by the qualified serve path so HeapKey sessions and legacy token
    /// paths do not double-open the writer lock.
    pub fn from_shared(physical: Arc<Mutex<PhysicalStore>>, root: impl Into<PathBuf>) -> Self {
        Self {
            physical,
            root: root.into(),
        }
    }

    /// Bind a validated [`HeapCap`] into a heap-scoped façade.
    pub fn open_heap(&self, cap: HeapCap) -> HeapStore {
        HeapStore::from_host(Arc::clone(&self.physical), cap)
    }

    /// Shared physical store handle (for process-local host reuse).
    pub fn physical(&self) -> Arc<Mutex<PhysicalStore>> {
        Arc::clone(&self.physical)
    }

    /// Store root path (operational metadata only).
    pub fn root(&self) -> &Path {
        &self.root
    }
}