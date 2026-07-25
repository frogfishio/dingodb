//! DingoDB collection SDK (Stages 4 + 6 + 7 + 8d–8e + product freezes).
//!
//! Ordinary application surface: open a store directory, connect to a server,
//! or open a multi-node cluster; name a collection; put/get/delete JSON or
//! bytes; filter JSON documents; manage secondary indexes; inspect per-key
//! history — without learning SDA.
//!
//! Normative: DX_SPEC §§1–10, §14; DELIVERY_PLAN Stages 4, 6, 7, and 8d–8e;
//! CLUSTER_SPEC §13 (client routing / directory cache), §17 (query coverage).
//!
//! **Product freeze:** [`SDK_API_VERSION`] labels the collection API surface
//! after Stage 4 + 7 embedded/server parity (DELIVERY_PLAN §7).

#![deny(missing_docs)]

/// Collection SDK API freeze label (DELIVERY_PLAN §7: Collection SDK 1.0).
///
/// Stages 4 + 7 parity are met: embedded, `dingo serve`, and cluster handles
/// share put/get/delete/scan/find/history/indexes. Patch releases may fix bugs;
/// breaking collection API changes require a major bump of this label.
pub const SDK_API_VERSION: &str = "1.0";

mod cluster_backend;
mod collection;
mod dingo;
mod directory_cache;
mod error;
mod filter;
mod history;
mod indexes;
mod receipt;
mod remote;
mod subject;
mod value;

pub use cluster_backend::{ClusterBackend, ClusterFindResult};
pub use collection::Collection;
pub use dingo::Dingo;
/// Re-export cluster coverage / scan types for Stage 8e callers.
pub use dingo_cluster::{Coverage, FindResult, ScanOptions};
pub use directory_cache::{AssignmentWire, CachedRoute, ClientDirectoryCache, DirectorySnapshot};
pub use error::{Error, ErrorCode};
pub use filter::{FieldBuilder, Filter, Pred, QueryBudget, QueryBuilder, QueryOptions, SortOrder};
pub use history::{KeyHistory, Version};
pub use indexes::{IndexInfo, Indexes};
pub use receipt::{DeleteReceipt, PutOptions, WriteReceipt};
pub use remote::{
    handle_connection, handle_connection_with, parse_dingo_url, serve_cluster_node, serve_store,
    serve_store_with, ConnectOptions, ParsedDingoUrl, RemoteClient, ServeOptions, DEFAULT_PORT,
};
pub use subject::{
    collection_prefix, decode_subject, encode_subject, validate_collection_name, validate_key,
    MAX_COLLECTION_NAME_LEN, MAX_KEY_LEN,
};

/// Re-export cluster config for [`Dingo::create_cluster`].
pub use dingo_cluster::ClusterConfig;

/// Re-export durability modes used on receipts and put options.
pub use dingo_store::DurabilityMode;
/// Re-export index lifecycle states (DX_SPEC §8.2).
pub use dingo_store::IndexState;
/// Re-export chunked payload completeness (FORMAT_SPEC §8).
pub use dingo_store::PayloadResult;

/// Build a `serde_json::Value` from a JSON literal (re-export for examples/tests).
pub use serde_json::json;

/// JSON value type used by [`Collection::get`] and filters.
pub type JsonValue = serde_json::Value;
