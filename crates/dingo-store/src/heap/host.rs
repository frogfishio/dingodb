//! Store host with no unscoped data plane (`HEAP_SPEC` HP-003).

use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use dingo_heap::HeapCap;
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

    /// Bind a validated [`HeapCap`] into a heap-scoped façade.
    pub fn open_heap(&self, cap: HeapCap) -> HeapStore {
        HeapStore::from_host(Arc::clone(&self.physical), cap)
    }

    /// Store root path (operational metadata only).
    pub fn root(&self) -> &Path {
        &self.root
    }
}
