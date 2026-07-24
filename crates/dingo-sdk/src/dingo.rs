//! Database handle: `Dingo::open` / `Dingo::connect` (DX_SPEC §4.1, Stage 7).

use crate::collection::Collection;
use crate::error::Error;
use crate::remote::{parse_dingo_url, RemoteClient};
use crate::subject::validate_collection_name;
use dingo_store::Store;
use std::path::{Path, PathBuf};

/// Embedded or remote DingoDB database handle.
///
/// ```ignore
/// let mut db = Dingo::open("./app.dingo")?;
/// let mut users = db.collection("users")?;
/// users.put("user-42", &serde_json::json!({"name": "Alice"}))?;
/// ```
///
/// Remote (Stage 7):
/// ```ignore
/// let mut db = Dingo::connect("dingo://localhost:7434/app")?;
/// ```
pub struct Dingo {
    pub(crate) backend: Backend,
}

pub(crate) enum Backend {
    Local(Store),
    Remote(RemoteClient),
}

impl Dingo {
    /// Open an existing store at `path`, or create one with safe defaults.
    ///
    /// If the path does not exist, it is created. If it exists as a DingoDB
    /// store, it is opened (using the optional index cache when valid).
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            backend: Backend::Local(Store::open(path)?),
        })
    }

    /// Connect to a remote `dingo serve` endpoint (`dingo://host:port[/label]`).
    ///
    /// The optional path label is informational only for Stage 7 (the server
    /// process already binds a store directory). Connection options such as
    /// authn / deadline / retry are reserved for later; transport is TCP
    /// line-delimited JSON.
    pub fn connect(url: impl AsRef<str>) -> Result<Self, Error> {
        let url = url.as_ref();
        let (hostport, _label) = parse_dingo_url(url)?;
        let client = RemoteClient::connect(&hostport, url.to_string())?;
        Ok(Self {
            backend: Backend::Remote(client),
        })
    }

    /// Whether this handle is a remote connection.
    pub fn is_remote(&self) -> bool {
        matches!(self.backend, Backend::Remote(_))
    }

    /// Filesystem root of this store (embedded only).
    pub fn path(&self) -> Option<&Path> {
        match &self.backend {
            Backend::Local(s) => Some(s.path()),
            Backend::Remote(_) => None,
        }
    }

    /// Store identifier (16 bytes). Remote returns zeros until a future
    /// protocol extension surfaces the server store id on every handle.
    pub fn store_id(&self) -> [u8; 16] {
        match &self.backend {
            Backend::Local(s) => s.store_id(),
            Backend::Remote(_) => [0u8; 16],
        }
    }

    /// Borrow a named collection handle.
    ///
    /// Collection access is lazy: no disk mutation occurs until the first write
    /// (embedded). Remote sends RPCs on each method call.
    pub fn collection(&mut self, name: impl Into<String>) -> Result<Collection<'_>, Error> {
        let name = name.into();
        validate_collection_name(&name)?;
        Ok(Collection::new(&mut self.backend, name))
    }

    /// Number of live subjects across all collections (embedded).
    ///
    /// Remote issues a `store_info` RPC.
    pub fn live_count(&mut self) -> Result<usize, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s.live_count()),
            Backend::Remote(c) => {
                let (_path, _id, n) = c.store_info()?;
                Ok(n)
            }
        }
    }

    /// Rebuild the primary index from segment files (catalog-free). Embedded only.
    pub fn rebuild_index(&mut self) -> Result<(), Error> {
        match &mut self.backend {
            Backend::Local(s) => {
                s.rebuild_index()?;
                Ok(())
            }
            Backend::Remote(_) => Err(Error::RemoteUnsupported("rebuild_index")),
        }
    }

    /// Rebuild derived collection catalogs from the primary index. Embedded only.
    pub fn rebuild_catalogs(&mut self) -> Result<(), Error> {
        match &mut self.backend {
            Backend::Local(s) => {
                s.rebuild_catalogs()?;
                Ok(())
            }
            Backend::Remote(_) => Err(Error::RemoteUnsupported("rebuild_catalogs")),
        }
    }

    /// Collection names known from the derived catalog (sorted).
    pub fn list_collections(&mut self) -> Result<Vec<String>, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s.list_collections()),
            Backend::Remote(c) => c.list_collections(),
        }
    }

    /// Compact live state into a new sealed segment (sources retained). Embedded only.
    pub fn compact_live(&mut self) -> Result<dingo_store::CompactReport, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s.compact_live()?),
            Backend::Remote(_) => Err(Error::RemoteUnsupported("compact_live")),
        }
    }

    /// Write a derived checkpoint with declared coverage. Embedded only.
    pub fn checkpoint(
        &self,
        coverage: &str,
    ) -> Result<(dingo_store::CheckpointMeta, PathBuf), Error> {
        match &self.backend {
            Backend::Local(s) => Ok(s.checkpoint(coverage)?),
            Backend::Remote(_) => Err(Error::RemoteUnsupported("checkpoint")),
        }
    }

    /// Access the underlying store (embedded only).
    pub fn store(&self) -> Result<&Store, Error> {
        match &self.backend {
            Backend::Local(s) => Ok(s),
            Backend::Remote(_) => Err(Error::RemoteUnsupported("store()")),
        }
    }

    /// Mutable access to the underlying store (embedded only).
    pub fn store_mut(&mut self) -> Result<&mut Store, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s),
            Backend::Remote(_) => Err(Error::RemoteUnsupported("store_mut()")),
        }
    }

    /// Path buffer for callers that need an owned root (embedded only).
    pub fn path_buf(&self) -> Option<PathBuf> {
        self.path().map(|p| p.to_path_buf())
    }
}
