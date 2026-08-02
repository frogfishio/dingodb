//! Store host with no unscoped data plane (`HEAP_SPEC` HP-003).

use crate::adaptive_write::{
    AdaptiveWriteError, AdaptiveWriteHandle, AdaptiveWriteMode, AdaptiveWritePolicy,
    AdaptiveWriteStatus,
};
use crate::error::StoreError;
use crate::kernel::PhysicalStore;
use residiuum_heap::HeapCap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::HeapStore;

/// Deployment-level host. Exposes no get/put/scan of application data.
pub struct StoreHost {
    physical: Arc<Mutex<PhysicalStore>>,
    root: PathBuf,
    /// Optional AWO handle (None for ordinary create/open).
    adaptive: Option<AdaptiveWriteHandle>,
}

impl StoreHost {
    /// Open an existing store directory as a host.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        Ok(Self {
            physical: Arc::new(Mutex::new(PhysicalStore::open(path)?)),
            root: path.to_path_buf(),
            adaptive: None,
        })
    }

    /// Create a new store directory as a host.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        Ok(Self {
            physical: Arc::new(Mutex::new(PhysicalStore::create(path)?)),
            root: path.to_path_buf(),
            adaptive: None,
        })
    }

    /// Create with an adaptive-write policy (plan §5).
    ///
    /// **Default product posture:** [`AdaptiveWriteMode::Disabled`] — identical
    /// mutation semantics to [`Self::create`]. Static/Adaptive attach a lease
    /// and warm cookers; batch coalescing residual until coordinator depth.
    pub fn create_with_adaptive_write(
        path: impl AsRef<Path>,
        policy: AdaptiveWritePolicy,
    ) -> Result<Self, StoreError> {
        let mut host = Self::create(path)?;
        host.attach_adaptive_write(policy)?;
        Ok(host)
    }

    /// Open with an adaptive-write policy (plan §5).
    pub fn open_with_adaptive_write(
        path: impl AsRef<Path>,
        policy: AdaptiveWritePolicy,
    ) -> Result<Self, StoreError> {
        let mut host = Self::open(path)?;
        host.attach_adaptive_write(policy)?;
        Ok(host)
    }

    /// Attach or replace adaptive-write policy on this host.
    pub fn attach_adaptive_write(
        &mut self,
        policy: AdaptiveWritePolicy,
    ) -> Result<(), StoreError> {
        // Detach prior handle if any.
        if let Some(prev) = self.adaptive.take() {
            let mut guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::CorruptMeta("store lock poisoned"))?;
            prev.detach(&mut guard);
        }
        policy.validate().map_err(|e| {
            StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("awo policy: {e:?}"),
            ))
        })?;
        let handle = match policy.mode {
            AdaptiveWriteMode::Disabled => AdaptiveWriteHandle::disabled(policy).map_err(|e| {
                StoreError::Io(std::io::Error::other(format!("awo disabled attach: {e:?}")))
            })?,
            AdaptiveWriteMode::Static | AdaptiveWriteMode::Adaptive => {
                let mut guard = self
                    .physical
                    .lock()
                    .map_err(|_| StoreError::CorruptMeta("store lock poisoned"))?;
                AdaptiveWriteHandle::start_static(policy, &mut guard).map_err(|e| match e {
                    AdaptiveWriteError::WriterPoisoned { .. } => {
                        StoreError::AdaptiveWriterPoisoned
                    }
                    other => StoreError::Io(std::io::Error::other(format!("awo start: {other:?}"))),
                })?
            }
        };
        self.adaptive = Some(handle);
        Ok(())
    }

    /// Share an already-opened physical store (e.g. server exclusive writer).
    ///
    /// Used by the qualified serve path so HeapKey sessions and legacy token
    /// paths do not double-open the writer lock.
    pub fn from_shared(physical: Arc<Mutex<PhysicalStore>>, root: impl Into<PathBuf>) -> Self {
        Self {
            physical,
            root: root.into(),
            adaptive: None,
        }
    }

    /// Bind a validated [`HeapCap`] into a heap-scoped façade.
    ///
    /// When this host has an adaptive-write lease active, the heap routes
    /// put/delete through [`AdaptiveWriteHandle`] (AWO-3).
    pub fn open_heap(&self, cap: HeapCap) -> HeapStore {
        HeapStore::from_host_with_adaptive(
            Arc::clone(&self.physical),
            cap,
            self.adaptive.clone(),
        )
    }

    /// Shared physical store handle (for process-local host reuse).
    pub fn physical(&self) -> Arc<Mutex<PhysicalStore>> {
        Arc::clone(&self.physical)
    }

    /// Adaptive-write handle if attached via `*_with_adaptive_write`.
    pub fn adaptive_write(&self) -> Option<&AdaptiveWriteHandle> {
        self.adaptive.as_ref()
    }

    /// Status when AWO is attached; `None` for ordinary create/open.
    pub fn adaptive_write_status(&self) -> Option<AdaptiveWriteStatus> {
        self.adaptive.as_ref().map(|h| h.status())
    }

    /// Drain adaptive admits until idle or deadline (no-op when disabled/absent).
    pub fn drain_writes(&self, deadline: Instant) -> Result<(), AdaptiveWriteError> {
        match &self.adaptive {
            Some(h) => h.drain_writes(deadline),
            None => Ok(()),
        }
    }

    /// Store root path (operational metadata only).
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl Drop for StoreHost {
    fn drop(&mut self) {
        if let Some(h) = self.adaptive.take() {
            if let Ok(mut guard) = self.physical.lock() {
                h.detach(&mut guard);
            }
        }
    }
}