//! Database handle: `Dingo::open` (DX_SPEC §4.1).

use crate::collection::Collection;
use crate::error::Error;
use crate::subject::validate_collection_name;
use dingo_store::Store;
use std::path::{Path, PathBuf};

/// Embedded DingoDB database handle.
///
/// ```ignore
/// let mut db = Dingo::open("./app.dingo")?;
/// let mut users = db.collection("users")?;
/// users.put("user-42", &serde_json::json!({"name": "Alice"}))?;
/// ```
pub struct Dingo {
    store: Store,
}

impl Dingo {
    /// Open an existing store at `path`, or create one with safe defaults.
    ///
    /// If the path does not exist, it is created. If it exists as a DingoDB
    /// store, it is opened (using the optional index cache when valid).
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            store: Store::open(path)?,
        })
    }

    /// Filesystem root of this store.
    pub fn path(&self) -> &Path {
        self.store.path()
    }

    /// Store identifier (16 bytes).
    pub fn store_id(&self) -> [u8; 16] {
        self.store.store_id()
    }

    /// Borrow a named collection handle.
    ///
    /// Collection access is lazy: no disk mutation occurs until the first write.
    pub fn collection(&mut self, name: impl Into<String>) -> Result<Collection<'_>, Error> {
        let name = name.into();
        validate_collection_name(&name)?;
        Ok(Collection::new(&mut self.store, name))
    }

    /// Number of live subjects across all collections (store-level count).
    pub fn live_count(&self) -> usize {
        self.store.live_count()
    }

    /// Rebuild the primary index from segment files (catalog-free).
    pub fn rebuild_index(&mut self) -> Result<(), Error> {
        self.store.rebuild_index()?;
        Ok(())
    }

    /// Rebuild derived collection catalogs from the primary index.
    pub fn rebuild_catalogs(&mut self) -> Result<(), Error> {
        self.store.rebuild_catalogs()?;
        Ok(())
    }

    /// Collection names known from the derived catalog (sorted).
    pub fn list_collections(&self) -> Vec<String> {
        self.store.list_collections()
    }

    /// Compact live state into a new sealed segment (sources retained).
    pub fn compact_live(&mut self) -> Result<dingo_store::CompactReport, Error> {
        Ok(self.store.compact_live()?)
    }

    /// Write a derived checkpoint with declared coverage.
    pub fn checkpoint(
        &self,
        coverage: &str,
    ) -> Result<(dingo_store::CheckpointMeta, PathBuf), Error> {
        Ok(self.store.checkpoint(coverage)?)
    }

    /// Access the underlying store (advanced / Stage 6 operator paths).
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Mutable access to the underlying store.
    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }

    /// Path buffer for callers that need an owned root.
    pub fn path_buf(&self) -> PathBuf {
        self.store.path().to_path_buf()
    }
}
