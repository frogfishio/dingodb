//! Database handle: `Dingo::open` / `Dingo::connect` / `Dingo::open_cluster` (DX_SPEC §4).

use crate::cluster_backend::ClusterBackend;
use crate::collection::Collection;
use crate::error::Error;
use crate::remote::{parse_dingo_url, ConnectOptions, RemoteClient};
use crate::subject::validate_collection_name;
use dingo_cluster::ClusterConfig;
use dingo_store::Store;
use std::path::{Path, PathBuf};

/// Embedded, remote, or clustered DingoDB database handle.
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
/// // or with auth / deadlines / connect retry:
/// let mut db = Dingo::connect_with(
///     "dingo://localhost:7434/app",
///     ConnectOptions::new().auth_token("secret"),
/// )?;
/// ```
///
/// Cluster (Stage 8d) — same collection API; partition routes are cached client-side:
/// ```ignore
/// let mut db = Dingo::create_cluster(
///     dingo_cluster::ClusterConfig::dependable_local("./cluster")
/// )?;
/// // or open an existing cluster root:
/// let mut db = Dingo::open_cluster("./cluster")?;
/// db.collection("users")?.put("user-42", &serde_json::json!({"name": "Alice"}))?;
/// ```
pub struct Dingo {
    pub(crate) backend: Backend,
}

pub(crate) enum Backend {
    Local(Store),
    Remote(RemoteClient),
    Cluster(ClusterBackend),
}

impl Dingo {
    /// Open an existing store at `path`, or create one with safe defaults.
    ///
    /// If the path does not exist, it is created. If it exists as a DingoDB
    /// store, it is opened (using the optional index cache when valid).
    ///
    /// Writer opens take an exclusive store lock (DEF-020). A second writer —
    /// including `dingo serve` while an embedded handle is open — fails until
    /// the first handle is dropped. Use [`Self::open_inspect`] for concurrent
    /// read-only doctor/parity checks.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            backend: Backend::Local(Store::open(path)?),
        })
    }

    /// Open an **existing** store for read-only inspection (no writer lock).
    ///
    /// Suitable while another process holds the exclusive writer (for example
    /// `dingo serve`). Mutations fail because no active writer is opened.
    pub fn open_inspect(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            backend: Backend::Local(Store::open_inspect(path)?),
        })
    }

    /// Connect to a remote `dingo serve` endpoint (`dingo://host:port[/label]`).
    ///
    /// Uses default [`ConnectOptions`] (no auth token, 5s connect / 30s request
    /// deadlines, 3 connect attempts). Prefer [`Self::connect_with`] when the
    /// server requires a token or custom deadlines.
    ///
    /// The optional path label is informational only for Stage 7 (the server
    /// process already binds a store directory). Transport is TCP line-delimited
    /// JSON.
    pub fn connect(url: impl AsRef<str>) -> Result<Self, Error> {
        Self::connect_with(url, ConnectOptions::default())
    }

    /// Connect with explicit connection options (authn, deadlines, retry).
    ///
    /// Application put/get APIs stay the same; only the transport policy changes
    /// (DX_SPEC §4.2). Multi-seed URLs (`dingo://h1:p1,h2:p2/app`) try seeds in
    /// order and use the first that accepts a connection; the client may then
    /// fetch a `directory` snapshot for route caching (Stage 8d).
    pub fn connect_with(url: impl AsRef<str>, options: ConnectOptions) -> Result<Self, Error> {
        let url = url.as_ref();
        let parsed = parse_dingo_url(url)?;
        if parsed.seeds.is_empty() {
            return Err(Error::ValidationMsg("empty dingo:// URL".into()));
        }
        let mut last_err: Option<Error> = None;
        for hostport in &parsed.seeds {
            match RemoteClient::connect_with(hostport, url.to_string(), options.clone()) {
                Ok(client) => {
                    return Ok(Self {
                        backend: Backend::Remote(client),
                    });
                }
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err
            .unwrap_or_else(|| Error::Internal("connect failed with no seed errors".into())))
    }

    /// Create a new multi-node cluster and return a SDK handle (Stage 8d).
    ///
    /// Ordinary collection put/get/delete use the same API as embedded/server;
    /// the client caches partition routes and refreshes on stale placement
    /// (CLUSTER_SPEC §13).
    pub fn create_cluster(cfg: ClusterConfig) -> Result<Self, Error> {
        Ok(Self {
            backend: Backend::Cluster(ClusterBackend::create(cfg)?),
        })
    }

    /// Open an existing cluster root directory as a SDK handle (Stage 8d).
    pub fn open_cluster(root: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            backend: Backend::Cluster(ClusterBackend::open(root)?),
        })
    }

    /// Whether this handle is a remote connection (single-node or multi-seed).
    pub fn is_remote(&self) -> bool {
        matches!(self.backend, Backend::Remote(_))
    }

    /// Whether this handle is an in-process cluster.
    pub fn is_cluster(&self) -> bool {
        matches!(self.backend, Backend::Cluster(_))
    }

    /// Borrow the cluster backend (Stage 8d tests / ops).
    pub fn cluster_backend_mut(&mut self) -> Result<&mut ClusterBackend, Error> {
        match &mut self.backend {
            Backend::Cluster(c) => Ok(c),
            _ => Err(Error::RemoteUnsupported("cluster_backend_mut")),
        }
    }

    /// Filesystem root of this store (embedded or cluster root).
    pub fn path(&self) -> Option<&Path> {
        match &self.backend {
            Backend::Local(s) => Some(s.path()),
            Backend::Remote(_) => None,
            Backend::Cluster(c) => Some(c.root()),
        }
    }

    /// Store / cluster identifier (16 bytes).
    pub fn store_id(&self) -> [u8; 16] {
        match &self.backend {
            Backend::Local(s) => s.store_id(),
            Backend::Remote(c) => c.store_id(),
            Backend::Cluster(c) => c.store_id(),
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
    /// Remote issues a `store_info` RPC. Cluster scans online partitions.
    pub fn live_count(&mut self) -> Result<usize, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s.live_count()),
            Backend::Remote(c) => {
                let (_path, _id, n) = c.store_info()?;
                Ok(n)
            }
            Backend::Cluster(c) => c.live_count_approx(),
        }
    }

    /// Rebuild the primary index from segment files (catalog-free). Embedded only.
    pub fn rebuild_index(&mut self) -> Result<(), Error> {
        match &mut self.backend {
            Backend::Local(s) => {
                s.rebuild_index()?;
                Ok(())
            }
            Backend::Remote(_) | Backend::Cluster(_) => {
                Err(Error::RemoteUnsupported("rebuild_index"))
            }
        }
    }

    /// Rebuild derived collection catalogs from the primary index. Embedded only.
    pub fn rebuild_catalogs(&mut self) -> Result<(), Error> {
        match &mut self.backend {
            Backend::Local(s) => {
                s.rebuild_catalogs()?;
                Ok(())
            }
            Backend::Remote(_) | Backend::Cluster(_) => {
                Err(Error::RemoteUnsupported("rebuild_catalogs"))
            }
        }
    }

    /// Collection names known from the derived catalog (sorted).
    pub fn list_collections(&mut self) -> Result<Vec<String>, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s.list_collections()),
            Backend::Remote(c) => c.list_collections(),
            Backend::Cluster(c) => c.list_collections(),
        }
    }

    /// Compact live state into a new sealed segment (sources retained). Embedded only.
    pub fn compact_live(&mut self) -> Result<dingo_store::CompactReport, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s.compact_live()?),
            Backend::Remote(_) | Backend::Cluster(_) => {
                Err(Error::RemoteUnsupported("compact_live"))
            }
        }
    }

    /// Write a derived checkpoint with declared coverage. Embedded only.
    pub fn checkpoint(
        &self,
        coverage: &str,
    ) -> Result<(dingo_store::CheckpointMeta, PathBuf), Error> {
        match &self.backend {
            Backend::Local(s) => Ok(s.checkpoint(coverage)?),
            Backend::Remote(_) | Backend::Cluster(_) => Err(Error::RemoteUnsupported("checkpoint")),
        }
    }

    /// Access the underlying store (embedded only).
    pub fn store(&self) -> Result<&Store, Error> {
        match &self.backend {
            Backend::Local(s) => Ok(s),
            Backend::Remote(_) | Backend::Cluster(_) => Err(Error::RemoteUnsupported("store()")),
        }
    }

    /// Mutable access to the underlying store (embedded only).
    pub fn store_mut(&mut self) -> Result<&mut Store, Error> {
        match &mut self.backend {
            Backend::Local(s) => Ok(s),
            Backend::Remote(_) | Backend::Cluster(_) => {
                Err(Error::RemoteUnsupported("store_mut()"))
            }
        }
    }

    /// Path buffer for callers that need an owned root (embedded or cluster).
    pub fn path_buf(&self) -> Option<PathBuf> {
        self.path().map(|p| p.to_path_buf())
    }
}
