//! Framed, versioned JSON RPC for `dingo://` remote access (Stages 7 + 8d, DEF-031).
//!
//! Server and client share the same request/response shapes. Transport is TCP.
//! **Production profile** (`dingo-rpc-v1`):
//! 1. length-prefixed frames (`u32` BE + UTF-8 JSON payload);
//! 2. explicit hello/welcome handshake with feature negotiation;
//! 3. application [`RpcRequest`] / [`RpcResponse`] only after session setup.
//!
//! **Diagnostic profile:** optional newline-delimited JSON when both sides set
//! `diagnostic_line_protocol` (human debugging with `nc`; not for production).
//!
//! Supported ops include put/get/delete/scan, **history**, secondary index
//! list/create/drop/rebuild, **get_payload** (chunk completeness), **find**
//! (server-side filter with index acceleration), and **directory** (Stage 8d
//! partition route snapshot for client caches). Connection-only concerns (auth
//! token, deadlines, connect retries, wire profile) live on [`ConnectOptions`] /
//! [`ServeOptions`] — not on the collection API (DX_SPEC §4.2).

use crate::collection::find_on_store;
use crate::directory_cache::{AssignmentWire, ClientDirectoryCache, DirectorySnapshot};
use crate::error::Error;
use crate::filter::{Filter, QueryBudget, QueryOptions, SortOrder};
use crate::history::{KeyHistory, Version};
use crate::indexes::{create_index_on_store, mark_indexes_stale, IndexInfo};
use crate::protocol::{
    self, client_handshake, server_handshake, write_json_frame, write_reject_frame,
    NegotiatedSession, PROTOCOL_PROFILE,
};
use crate::receipt::{DeleteReceipt, PutOptions, WriteReceipt};
use crate::subject::{
    collection_prefix, decode_subject, encode_subject, validate_collection_name, validate_key,
};
use crate::value::{decode_bytes, decode_json, encode_bytes, encode_json};
use dingo_cluster::{DEFAULT_VIRTUAL_PARTITIONS, HASH_PROFILE_BLAKE3_MOD};
use dingo_store::{
    random_id, ByteRange, DurabilityMode, IndexState, LogicalExtent, PayloadResult, Store,
    WriteReceipt as StoreWriteReceipt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::server::{
    is_mutating_op, ConnectionGuard, MutationGuard, ServerLimits, ServerRuntime, SERVER_PROFILE,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::str;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Default TCP port from DX_SPEC server examples (`dingo://localhost:7434/...`).
pub const DEFAULT_PORT: u16 = 7434;

/// Client connection options (DX_SPEC §4.2: authn, deadlines, retry).
///
/// These do **not** change the collection put/get API — only how `connect` and
/// subsequent RPCs behave on the wire.
#[derive(Debug, Clone)]
pub struct ConnectOptions {
    /// Optional shared secret sent on every RPC as `token`.
    pub auth_token: Option<String>,
    /// Timeout for establishing the TCP connection (per attempt).
    pub connect_timeout: Duration,
    /// Read/write timeout applied to each request/response exchange.
    pub request_timeout: Duration,
    /// How many times to retry a failed connect (including the first try).
    pub max_connect_attempts: u32,
    /// Sleep between connect attempts.
    pub retry_backoff: Duration,
    /// Use legacy newline-delimited JSON (diagnostic only; DEF-031).
    ///
    /// Requires the server to also enable
    /// [`ServeOptions::diagnostic_line_protocol`]. Production clients leave
    /// this `false` and perform the framed `dingo-rpc-v1` handshake.
    pub diagnostic_line_protocol: bool,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            auth_token: None,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_connect_attempts: 3,
            retry_backoff: Duration::from_millis(50),
            diagnostic_line_protocol: false,
        }
    }
}

impl ConnectOptions {
    /// Default options (no auth, 5s connect / 30s request, 3 attempts).
    pub fn new() -> Self {
        Self::default()
    }

    /// Require this shared token on the server (`ServeOptions::auth_token`).
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Per-attempt TCP connect timeout.
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }

    /// Per-RPC read/write deadline.
    pub fn request_timeout(mut self, d: Duration) -> Self {
        self.request_timeout = d;
        self
    }

    /// Connect attempts including the first try (minimum 1).
    pub fn max_connect_attempts(mut self, n: u32) -> Self {
        self.max_connect_attempts = n.max(1);
        self
    }

    /// Delay between failed connect attempts.
    pub fn retry_backoff(mut self, d: Duration) -> Self {
        self.retry_backoff = d;
        self
    }

    /// Enable legacy line-delimited JSON (must match server diagnostic mode).
    pub fn diagnostic_line_protocol(mut self, enabled: bool) -> Self {
        self.diagnostic_line_protocol = enabled;
        self
    }
}

/// Server options for `dingo serve` / [`serve_store_with`] / cluster node serve.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// When set, every RPC must carry a matching `token` field.
    pub auth_token: Option<String>,
    /// When set, `directory` RPC returns this snapshot (network multi-node serve).
    ///
    /// Single-node `dingo serve` leaves this unset and synthesizes an all-local
    /// directory. Cluster nodes (`dingo serve-cluster`) pass real placement +
    /// `endpoints.json` so clients can cache routes (CLUSTER_SPEC §13).
    ///
    /// When [`Self::cluster_root`] is also set, each `directory` RPC reloads
    /// `endpoints.json` so late-joining nodes appear without restarting peers.
    pub directory: Option<DirectorySnapshot>,
    /// Dense node index this process represents (informational; for logs/tests).
    pub node_index: Option<u32>,
    /// Cluster root directory for live `endpoints.json` reload on `directory` RPC.
    pub cluster_root: Option<std::path::PathBuf>,
    /// Allow non-loopback plaintext binds (development only; DEF-002).
    ///
    /// Defaults to `false`. TLS is not implemented yet; public binds require
    /// this explicit opt-in (CLI: `--allow-insecure-bind`).
    pub allow_insecure_bind: bool,
    /// Acknowledge that network `serve-cluster` is experimental (DEF-002).
    ///
    /// Required by [`serve_cluster_node`]. Quorum replication over TCP is not
    /// implemented; this flag only unlocks the routing/advertise prototype.
    pub experimental_network_cluster: bool,
    /// Skip the structured stderr startup report (when the CLI already printed it).
    pub suppress_startup_report: bool,
    /// Connection admission, idle timeout, and drain bounds (DEF-030).
    pub server_limits: ServerLimits,
    /// Optional external shutdown flag. When set true, the accept loop stops
    /// admitting work and drains in-flight connections (DEF-030).
    pub shutdown: Option<Arc<AtomicBool>>,
    /// Accept legacy newline-delimited JSON without handshake (DEF-031 diagnostic).
    ///
    /// Defaults to `false`. Production servers require the framed
    /// `dingo-rpc-v1` hello/welcome exchange. Enable only for local debugging.
    pub diagnostic_line_protocol: bool,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            auth_token: None,
            directory: None,
            node_index: None,
            cluster_root: None,
            allow_insecure_bind: false,
            experimental_network_cluster: false,
            suppress_startup_report: false,
            server_limits: ServerLimits::draft_defaults(),
            shutdown: None,
            diagnostic_line_protocol: false,
        }
    }
}

impl ServeOptions {
    /// No authentication required.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require this shared token on every request.
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Advertise a cluster placement directory on `directory` RPC.
    pub fn directory(mut self, snapshot: DirectorySnapshot) -> Self {
        self.directory = Some(snapshot);
        self
    }

    /// Record which cluster node index this process is serving.
    pub fn node_index(mut self, index: u32) -> Self {
        self.node_index = Some(index);
        self
    }

    /// Reload `endpoints.json` from this cluster root on every `directory` RPC.
    pub fn cluster_root(mut self, root: impl Into<std::path::PathBuf>) -> Self {
        self.cluster_root = Some(root.into());
        self
    }

    /// Allow binding to a non-loopback address without TLS (development only).
    pub fn allow_insecure_bind(mut self, allow: bool) -> Self {
        self.allow_insecure_bind = allow;
        self
    }

    /// Opt into experimental network cluster serve (routing/advertise only).
    pub fn experimental_network_cluster(mut self, enabled: bool) -> Self {
        self.experimental_network_cluster = enabled;
        self
    }

    /// Do not print the structured serve startup report (caller already did).
    pub fn suppress_startup_report(mut self, suppress: bool) -> Self {
        self.suppress_startup_report = suppress;
        self
    }

    /// Set connection / drain limits for the bounded server (DEF-030).
    pub fn server_limits(mut self, limits: ServerLimits) -> Self {
        self.server_limits = limits;
        self
    }

    /// Maximum simultaneous client connections (shorthand for server limits).
    pub fn max_connections(mut self, n: usize) -> Self {
        self.server_limits.max_connections = n.max(1);
        self
    }

    /// Idle socket timeout for established connections.
    pub fn idle_timeout(mut self, d: Duration) -> Self {
        self.server_limits.idle_timeout = d;
        self
    }

    /// How long graceful shutdown waits for in-flight connections.
    pub fn drain_timeout(mut self, d: Duration) -> Self {
        self.server_limits.drain_timeout = d;
        self
    }

    /// External shutdown signal shared with the accept loop (DEF-030).
    pub fn shutdown_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.shutdown = Some(flag);
        self
    }

    /// Enable legacy line-delimited JSON (diagnostic / `nc` debugging only).
    pub fn diagnostic_line_protocol(mut self, enabled: bool) -> Self {
        self.diagnostic_line_protocol = enabled;
        self
    }
}

/// Wire request (one JSON line).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Correlation id (echoed in the response).
    pub id: u64,
    /// Operation name.
    pub op: String,
    /// Collection name when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    /// Application key when applicable (also used as index name for index_* ops).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// JSON document body for put.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json: Option<JsonValue>,
    /// Base64-encoded bytes for put_bytes/get_bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_b64: Option<String>,
    /// Optional result limit for scans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Durability mode name: `memory` | `buffered` | `durable` (default durable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub durability: Option<String>,
    /// Optional shared auth token (connection option; DX_SPEC §4.2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Field paths for index_create.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,
    /// When true, force a full collection scan for `find` (skip indexes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force_scan: Option<bool>,
    /// Order-by field path for `find`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_field: Option<String>,
    /// Order direction for `find`: `asc` | `desc`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_dir: Option<String>,
    /// Max documents the server may examine for `find` (query budget).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_docs_scanned: Option<usize>,
    /// Max payload bytes the server may examine for `find` (DEF-029).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bytes_scanned: Option<u64>,
    /// Max approximate result materialisation bytes for `find` (DEF-029).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_result_bytes: Option<u64>,
    /// Client-generated operation id (32 hex chars) for idempotent mutations (DEF-010).
    ///
    /// Required on put/delete/put_bytes from modern clients. Retries reuse the
    /// same id; reuse with different content yields `consistency_violation`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
}

/// Wire response (one JSON line).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Correlation id from the request.
    pub id: u64,
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Error message when `ok` is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Stable error code when `ok` is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// JSON value for get.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<JsonValue>,
    /// Whether a get found a value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<bool>,
    /// Whether delete removed a visible value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed: Option<bool>,
    /// Application key from write/delete receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Committed flag from write receipt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub committed: Option<bool>,
    /// Achieved durability mode name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledgement: Option<String>,
    /// List of keys or collection names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<Vec<String>>,
    /// List of (key, json) pairs from scan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<ScanRow>>,
    /// Base64 bytes for get_bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_b64: Option<String>,
    /// Hex store id for store_info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_id: Option<String>,
    /// Store path string for store_info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Live subject count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live_count: Option<usize>,
    /// Hex event id from write/delete receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Hex item/version id from write/delete receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Hex segment id from write/delete receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
    /// Per-key history versions (history op).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versions: Option<Vec<HistoryVersionRow>>,
    /// Whether history stream observed salvage holes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_known_holes: Option<bool>,
    /// Secondary index metadata rows (index_list / index_create / index_rebuild).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexes: Option<Vec<IndexInfoRow>>,
    /// Payload completeness: `complete` | `partial` | `unavailable` | `conflicting`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_status: Option<String>,
    /// BLAKE3-256 content hash as 64 hex chars (partial/unavailable payloads).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash_hex: Option<String>,
    /// Missing chunk indices for partial payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<Vec<u32>>,
    /// Declared total chunk count (unavailable payloads).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_chunks: Option<u32>,
    /// Conflicting chunk index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_index: Option<u32>,
    /// Surviving chunk bodies for partial payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub present_chunks: Option<Vec<PresentChunkRow>>,
    /// Extent map for partial payloads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extents: Option<Vec<ExtentRow>>,
    /// Partition directory snapshot (`directory` op; Stage 8d).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directory: Option<DirectorySnapshot>,
}

/// One scan row on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRow {
    /// Application key.
    pub key: String,
    /// JSON document.
    pub value: JsonValue,
}

/// One history version on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryVersionRow {
    /// `put` or `delete`.
    pub kind: String,
    /// Hex event id.
    pub event_id: String,
    /// Hex item lineage id.
    pub item_id: String,
    /// Hex segment id.
    pub segment_id: String,
    /// JSON document when the put stored JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub json: Option<JsonValue>,
    /// Optional typed store body (base64).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_b64: Option<String>,
    /// Known salvage hole before this event.
    #[serde(default)]
    pub known_gap_before: bool,
}

/// Secondary index metadata on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfoRow {
    /// Index name.
    pub name: String,
    /// Collection name.
    pub collection: String,
    /// Indexed field paths.
    pub fields: Vec<String>,
    /// Lifecycle state snake_case name.
    pub state: String,
    /// Number of postings.
    pub entry_count: u64,
    /// Whether the index claims complete coverage.
    pub complete_coverage: bool,
    /// Failure / partial detail (DEF-027); empty when healthy.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub failure_reason: String,
    /// Hex build id (DEF-027).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub build_id_hex: String,
}

/// One present chunk body on the wire (`get_payload` partial).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentChunkRow {
    /// Chunk index.
    pub index: u32,
    /// Base64 chunk body.
    pub bytes_b64: String,
}

/// One logical extent on the wire (`get_payload` partial).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtentRow {
    /// Chunk index.
    pub index: u32,
    /// Inclusive logical start offset.
    pub start: u64,
    /// Exclusive logical end offset.
    pub end: u64,
    /// Whether the chunk body is present.
    pub present: bool,
}

impl IndexInfoRow {
    fn from_info(info: &IndexInfo) -> Self {
        Self {
            name: info.name.clone(),
            collection: info.collection.clone(),
            fields: info.fields.clone(),
            state: info.state.as_str().into(),
            entry_count: info.entry_count,
            complete_coverage: info.complete_coverage,
            failure_reason: info.failure_reason.clone(),
            build_id_hex: info.build_id_hex.clone(),
        }
    }

    fn into_info(self) -> Result<IndexInfo, Error> {
        let state = IndexState::parse(&self.state).ok_or_else(|| {
            Error::Internal(format!("unknown index state from remote: {}", self.state))
        })?;
        Ok(IndexInfo {
            name: self.name,
            collection: self.collection,
            fields: self.fields,
            state,
            entry_count: self.entry_count,
            complete_coverage: self.complete_coverage,
            failure_reason: self.failure_reason,
            build_id_hex: self.build_id_hex,
        })
    }
}

/// Maximum transport retries after a directory refresh (multi-hop polish).
const MAX_ROUTE_TRANSPORT_RETRIES: u32 = 3;

/// Client connection to a `dingo serve` / `dingo serve-cluster` process.
///
/// After connect the client fetches a partition [`DirectorySnapshot`] and, for
/// keyed ops, routes to the advertised leader `host:port` (CLUSTER_SPEC §13).
/// On transport failure the directory is refreshed and the op is retried.
pub struct RemoteClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    next_id: AtomicU64,
    /// Logical URL (`dingo://…`) for display / errors.
    endpoint: String,
    /// TCP address of the current connection (`host:port`).
    addr: String,
    options: ConnectOptions,
    /// Cached store id from the first `store_info` after connect.
    store_id: [u8; 16],
    /// Client route cache (populated after connect via `directory` RPC).
    directory: Option<ClientDirectoryCache>,
    /// How many times we reconnected to a different leader for routing.
    route_hops: u64,
    /// How many times we refreshed the directory after transport failure.
    directory_refreshes: u64,
    /// Negotiated max frame size (framed profile) or host line limit (diagnostic).
    max_frame: usize,
    /// Snapshot of negotiated features (empty in diagnostic line mode).
    session: Option<NegotiatedSession>,
}

impl RemoteClient {
    /// Connect to `host:port` with default [`ConnectOptions`].
    pub fn connect(addr: &str, endpoint: String) -> Result<Self, Error> {
        Self::connect_with(addr, endpoint, ConnectOptions::default())
    }

    /// Connect with explicit auth / deadline / retry options.
    ///
    /// On success, loads a partition directory snapshot for multi-hop routing
    /// when the server advertises real endpoints (`dingo serve-cluster`).
    pub fn connect_with(
        addr: &str,
        endpoint: String,
        options: ConnectOptions,
    ) -> Result<Self, Error> {
        let mut client = Self::connect_raw(addr, endpoint, options)?;
        // Best-effort directory load: single-node servers return a synthetic
        // map; cluster nodes return placement + endpoints.json.
        let _ = client.refresh_directory();
        Ok(client)
    }

    fn connect_raw(addr: &str, endpoint: String, options: ConnectOptions) -> Result<Self, Error> {
        let stream = tcp_connect_with_retry(addr, &options)?;
        stream
            .set_read_timeout(Some(options.request_timeout))
            .map_err(Error::from_io)?;
        stream
            .set_write_timeout(Some(options.request_timeout))
            .map_err(Error::from_io)?;
        let mut reader = BufReader::new(stream.try_clone().map_err(Error::from_io)?);
        let mut stream = stream;
        let (max_frame, session) = if options.diagnostic_line_protocol {
            (crate::resource::host_limits().max_rpc_line_bytes, None)
        } else {
            let session = client_handshake(&mut reader, &mut stream)?;
            (session.max_frame, Some(session))
        };
        let mut client = Self {
            stream,
            reader,
            next_id: AtomicU64::new(1),
            endpoint,
            addr: addr.to_string(),
            options,
            store_id: [0u8; 16],
            directory: None,
            route_hops: 0,
            directory_refreshes: 0,
            max_frame,
            session,
        };
        // Immediate store_info validates auth token, proves protocol, caches store id.
        let (_path, sid_hex, _n) = client.store_info()?;
        client.store_id = require_hex16(Some(sid_hex.as_str()), "store_info.store_id")?;
        if client.store_id == [0u8; 16] {
            return Err(Error::ProtocolViolation(
                "store_info.store_id must not be the zero id".into(),
            ));
        }
        Ok(client)
    }

    /// Endpoint string used for errors and display.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Current TCP address (`host:port`).
    pub fn addr(&self) -> &str {
        &self.addr
    }

    /// Connection options used for this client.
    pub fn options(&self) -> &ConnectOptions {
        &self.options
    }

    /// Store identifier reported by the server at connect time.
    pub fn store_id(&self) -> [u8; 16] {
        self.store_id
    }

    /// Borrow the client directory cache when loaded.
    pub fn directory_cache(&self) -> Option<&ClientDirectoryCache> {
        self.directory.as_ref()
    }

    /// Number of multi-hop reconnects performed for routing.
    pub fn route_hops(&self) -> u64 {
        self.route_hops
    }

    /// Number of directory refreshes after transport failure.
    pub fn directory_refreshes(&self) -> u64 {
        self.directory_refreshes
    }

    /// Fetch and install a fresh partition directory snapshot.
    pub fn refresh_directory(&mut self) -> Result<&ClientDirectoryCache, Error> {
        let snap = self.fetch_directory()?;
        self.directory = Some(ClientDirectoryCache::from_snapshot(&snap));
        self.directory_refreshes = self.directory_refreshes.saturating_add(1);
        Ok(self.directory.as_ref().expect("just set"))
    }

    /// Negotiated session after framed handshake (None in diagnostic line mode).
    pub fn session(&self) -> Option<&NegotiatedSession> {
        self.session.as_ref()
    }

    /// Reconnect this client to a different `host:port`, keeping directory cache.
    pub fn reconnect(&mut self, addr: &str) -> Result<(), Error> {
        if addr == self.addr {
            return Ok(());
        }
        let stream = tcp_connect_with_retry(addr, &self.options)?;
        stream
            .set_read_timeout(Some(self.options.request_timeout))
            .map_err(Error::from_io)?;
        stream
            .set_write_timeout(Some(self.options.request_timeout))
            .map_err(Error::from_io)?;
        let mut reader = BufReader::new(stream.try_clone().map_err(Error::from_io)?);
        let mut stream = stream;
        if self.options.diagnostic_line_protocol {
            self.max_frame = crate::resource::host_limits().max_rpc_line_bytes;
            self.session = None;
        } else {
            let session = client_handshake(&mut reader, &mut stream)?;
            self.max_frame = session.max_frame;
            self.session = Some(session);
        }
        self.stream = stream;
        self.reader = reader;
        self.addr = addr.to_string();
        self.route_hops = self.route_hops.saturating_add(1);
        // Refresh store id on the new node (each node has its own store).
        // Fail closed on missing/malformed identity (DEF-014); multi-hop may
        // legitimately change store_id when routing to a different node.
        let resp = self.call_on_current(base_req("store_info"))?;
        let sid_hex = resp.store_id.as_deref().ok_or_else(|| {
            Error::ProtocolViolation("store_info response missing store_id".into())
        })?;
        self.store_id = require_hex16(Some(sid_hex), "store_info.store_id")?;
        if self.store_id == [0u8; 16] {
            return Err(Error::ProtocolViolation(
                "store_info.store_id must not be the zero id".into(),
            ));
        }
        Ok(())
    }

    /// Route a keyed op to the cached partition leader when endpoints are known.
    fn ensure_route_for_key(&mut self, collection: &str, key: &str) -> Result<(), Error> {
        let subject = encode_subject(collection, key)?;
        let target = {
            let Some(cache) = self.directory.as_ref() else {
                return Ok(());
            };
            let Some(route) = cache.route(&subject) else {
                return Ok(());
            };
            let Some(ep) = cache.endpoint(route.leader) else {
                return Ok(());
            };
            if ep.is_empty() || ep == self.addr {
                return Ok(());
            }
            ep.to_string()
        };
        self.reconnect(&target)
    }

    /// Try a keyed RPC with multi-hop routing and transport refresh.
    fn call_keyed(
        &mut self,
        collection: &str,
        key: &str,
        req: RpcRequest,
    ) -> Result<RpcResponse, Error> {
        let mut last_err: Option<Error> = None;
        for attempt in 0..MAX_ROUTE_TRANSPORT_RETRIES {
            // Best-effort hop to the cached leader; fall through on failure.
            let _ = self.ensure_route_for_key(collection, key);
            match self.call_on_current(req.clone()) {
                Ok(r) => return Ok(r),
                Err(e) if is_transport_error(&e) && attempt + 1 < MAX_ROUTE_TRANSPORT_RETRIES => {
                    // Mark partition stale, refresh directory, try again.
                    if let Ok(subject) = encode_subject(collection, key) {
                        if let Some(cache) = self.directory.as_mut() {
                            let p = cache.partition_of(&subject);
                            cache.mark_stale(p);
                        }
                    }
                    let _ = self.refresh_directory();
                    last_err = Some(e);
                }
                Err(e) => return Err(e),
            }
        }
        Err(last_err.unwrap_or_else(|| Error::Internal("route retries exhausted".into())))
    }

    fn call(&mut self, req: RpcRequest) -> Result<RpcResponse, Error> {
        self.call_on_current(req)
    }

    fn call_on_current(&mut self, mut req: RpcRequest) -> Result<RpcResponse, Error> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        req.id = id;
        if req.token.is_none() {
            req.token = self.options.auth_token.clone();
        }
        let payload = serde_json::to_vec(&req).map_err(|e| Error::Internal(e.to_string()))?;
        // DEF-029 / DEF-031: fail closed on oversized outbound messages before write.
        if payload.len() > self.max_frame {
            return Err(Error::ResourceLimit(format!(
                "rpc request {} bytes exceeds negotiated max_frame {}",
                payload.len(),
                self.max_frame
            )));
        }
        if self.options.diagnostic_line_protocol {
            self.stream
                .write_all(&payload)
                .map_err(Error::from_io)?;
            self.stream.write_all(b"\n").map_err(Error::from_io)?;
            self.stream.flush().map_err(Error::from_io)?;
            let mut resp_line = String::new();
            let n = self
                .reader
                .read_line(&mut resp_line)
                .map_err(Error::from_io)?;
            if n == 0 {
                return Err(Error::Internal(format!(
                    "remote closed connection: {} ({})",
                    self.endpoint, self.addr
                )));
            }
            return decode_rpc_response(resp_line.trim().as_bytes(), &self.endpoint, &self.addr);
        }

        write_json_frame(&mut self.stream, &req)?;
        let resp_bytes = protocol::read_frame(&mut self.reader, self.max_frame)?.ok_or_else(|| {
            Error::Internal(format!(
                "remote closed connection: {} ({})",
                self.endpoint, self.addr
            ))
        })?;
        decode_rpc_response(&resp_bytes, &self.endpoint, &self.addr)
    }

    /// Ping the server.
    pub fn ping(&mut self) -> Result<(), Error> {
        let _ = self.call(base_req("ping"))?;
        Ok(())
    }

    /// Store path / id summary.
    pub fn store_info(&mut self) -> Result<(String, String, usize), Error> {
        let resp = self.call(base_req("store_info"))?;
        Ok((
            resp.path.unwrap_or_default(),
            resp.store_id.unwrap_or_default(),
            resp.live_count.unwrap_or(0),
        ))
    }

    /// List collection names.
    pub fn list_collections(&mut self) -> Result<Vec<String>, Error> {
        let resp = self.call(base_req("list_collections"))?;
        Ok(resp.keys.unwrap_or_default())
    }

    /// Put JSON under collection/key (multi-hop: routes to partition leader).
    pub fn put_json(
        &mut self,
        collection: &str,
        key: &str,
        value: &JsonValue,
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        // Mint once so call_keyed transport retries stay idempotent (DEF-010).
        let operation_id = Some(hex16(&client_operation_id()?));
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "put".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                json: Some(value.clone()),
                durability: Some(durability_name(options.durability).into()),
                operation_id,
                ..base_req("put")
            },
        )?;
        Ok(write_receipt_from_resp(key, &resp, options.durability)?)
    }

    /// Get JSON for collection/key (multi-hop: routes to partition leader).
    pub fn get_json(&mut self, collection: &str, key: &str) -> Result<Option<JsonValue>, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "get".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                ..base_req("get")
            },
        )?;
        if resp.found == Some(false) {
            return Ok(None);
        }
        Ok(resp.value)
    }

    /// Delete collection/key (multi-hop: routes to partition leader).
    pub fn delete(
        &mut self,
        collection: &str,
        key: &str,
        options: PutOptions,
    ) -> Result<DeleteReceipt, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let operation_id = Some(hex16(&client_operation_id()?));
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "delete".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                durability: Some(durability_name(options.durability).into()),
                operation_id,
                ..base_req("delete")
            },
        )?;
        Ok(delete_receipt_from_resp(key, &resp, options.durability)?)
    }

    /// Put raw bytes (base64 on the wire; multi-hop routed).
    pub fn put_bytes(
        &mut self,
        collection: &str,
        key: &str,
        bytes: &[u8],
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let operation_id = Some(hex16(&client_operation_id()?));
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "put_bytes".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                bytes_b64: Some(b64_encode(bytes)),
                durability: Some(durability_name(options.durability).into()),
                operation_id,
                ..base_req("put_bytes")
            },
        )?;
        Ok(write_receipt_from_resp(key, &resp, options.durability)?)
    }

    /// Get raw bytes (multi-hop routed).
    pub fn get_bytes(&mut self, collection: &str, key: &str) -> Result<Option<Vec<u8>>, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "get_bytes".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                ..base_req("get_bytes")
            },
        )?;
        if resp.found == Some(false) {
            return Ok(None);
        }
        match resp.bytes_b64 {
            None => Ok(None),
            Some(s) => Ok(Some(b64_decode(&s)?)),
        }
    }

    /// List keys in a collection.
    pub fn list_keys(&mut self, collection: &str) -> Result<Vec<String>, Error> {
        validate_collection_name(collection)?;
        let resp = self.call(RpcRequest {
            op: "list_keys".into(),
            collection: Some(collection.into()),
            ..base_req("list_keys")
        })?;
        Ok(resp.keys.unwrap_or_default())
    }

    /// Scan JSON rows (optional limit).
    pub fn scan_json(
        &mut self,
        collection: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        validate_collection_name(collection)?;
        let resp = self.call(RpcRequest {
            op: "scan_json".into(),
            collection: Some(collection.into()),
            limit,
            ..base_req("scan_json")
        })?;
        Ok(resp
            .rows
            .unwrap_or_default()
            .into_iter()
            .map(|r| (r.key, r.value))
            .collect())
    }

    /// Per-key history (DX_SPEC §10.1; multi-hop routed).
    pub fn history(&mut self, collection: &str, key: &str) -> Result<KeyHistory, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "history".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                ..base_req("history")
            },
        )?;
        let versions = resp
            .versions
            .unwrap_or_default()
            .into_iter()
            .map(history_version_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(KeyHistory {
            key: resp.key.unwrap_or_else(|| key.to_string()),
            versions,
            has_known_holes: resp.has_known_holes.unwrap_or(false),
        })
    }

    /// List secondary indexes on a collection.
    pub fn index_list(&mut self, collection: &str) -> Result<Vec<IndexInfo>, Error> {
        validate_collection_name(collection)?;
        let resp = self.call(RpcRequest {
            op: "index_list".into(),
            collection: Some(collection.into()),
            ..base_req("index_list")
        })?;
        resp.indexes
            .unwrap_or_default()
            .into_iter()
            .map(IndexInfoRow::into_info)
            .collect()
    }

    /// Create (or rebuild-by-create) a secondary field index.
    pub fn index_create(
        &mut self,
        collection: &str,
        name: &str,
        fields: &[&str],
    ) -> Result<IndexInfo, Error> {
        validate_collection_name(collection)?;
        if name.is_empty() {
            return Err(Error::InvalidKey("index name empty"));
        }
        let resp = self.call(RpcRequest {
            op: "index_create".into(),
            collection: Some(collection.into()),
            key: Some(name.into()),
            fields: Some(fields.iter().map(|s| (*s).to_string()).collect()),
            ..base_req("index_create")
        })?;
        let mut rows = resp.indexes.unwrap_or_default();
        let row = rows.pop().ok_or_else(|| {
            Error::Internal("index_create response missing index metadata".into())
        })?;
        row.into_info()
    }

    /// Drop a secondary index by name.
    pub fn index_drop(&mut self, collection: &str, name: &str) -> Result<(), Error> {
        validate_collection_name(collection)?;
        let _ = self.call(RpcRequest {
            op: "index_drop".into(),
            collection: Some(collection.into()),
            key: Some(name.into()),
            ..base_req("index_drop")
        })?;
        Ok(())
    }

    /// Rebuild an existing secondary index from live data.
    pub fn index_rebuild(&mut self, collection: &str, name: &str) -> Result<IndexInfo, Error> {
        validate_collection_name(collection)?;
        let resp = self.call(RpcRequest {
            op: "index_rebuild".into(),
            collection: Some(collection.into()),
            key: Some(name.into()),
            ..base_req("index_rebuild")
        })?;
        let mut rows = resp.indexes.unwrap_or_default();
        let row = rows.pop().ok_or_else(|| {
            Error::Internal("index_rebuild response missing index metadata".into())
        })?;
        row.into_info()
    }

    /// Completeness-aware payload read (chunked values; FORMAT_SPEC §8; multi-hop).
    pub fn get_payload(
        &mut self,
        collection: &str,
        key: &str,
    ) -> Result<Option<PayloadResult>, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call_keyed(
            collection,
            key,
            RpcRequest {
                op: "get_payload".into(),
                collection: Some(collection.into()),
                key: Some(key.into()),
                ..base_req("get_payload")
            },
        )?;
        if resp.found == Some(false) {
            return Ok(None);
        }
        Ok(Some(payload_from_resp(&resp)?))
    }

    /// Fetch the server's partition directory snapshot (Stage 8d).
    ///
    /// Single-node servers advertise themselves as leader of every virtual
    /// partition so clients can cache routes uniformly with multi-node clusters.
    pub fn fetch_directory(&mut self) -> Result<DirectorySnapshot, Error> {
        let resp = self.call(base_req("directory"))?;
        resp.directory
            .ok_or_else(|| Error::Internal("directory response missing directory payload".into()))
    }

    /// Server-side find with optional index acceleration.
    pub fn find(
        &mut self,
        collection: &str,
        filter: &Filter,
        options: &QueryOptions,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        validate_collection_name(collection)?;
        let (order_field, order_dir) = match &options.order_by {
            Some((f, SortOrder::Asc)) => (Some(f.clone()), Some("asc".into())),
            Some((f, SortOrder::Desc)) => (Some(f.clone()), Some("desc".into())),
            None => (None, None),
        };
        let max_docs = options.budget.as_ref().and_then(|b| b.max_docs_scanned);
        let max_bytes = options.budget.as_ref().and_then(|b| b.max_bytes_scanned);
        let max_result = options.budget.as_ref().and_then(|b| b.max_result_bytes);
        let resp = self.call(RpcRequest {
            op: "find".into(),
            collection: Some(collection.into()),
            json: Some(filter.to_json()),
            limit: options.limit,
            force_scan: if options.force_scan { Some(true) } else { None },
            order_field,
            order_dir,
            max_docs_scanned: max_docs,
            max_bytes_scanned: max_bytes,
            max_result_bytes: max_result,
            ..base_req("find")
        })?;
        Ok(resp
            .rows
            .unwrap_or_default()
            .into_iter()
            .map(|r| (r.key, r.value))
            .collect())
    }
}

fn history_version_from_row(row: HistoryVersionRow) -> Result<Version, Error> {
    let kind = match row.kind.as_str() {
        "put" => "put",
        "delete" => "delete",
        other => {
            return Err(Error::Internal(format!(
                "unknown history kind from remote: {other}"
            )))
        }
    };
    let body = match row.body_b64 {
        Some(s) => Some(b64_decode(&s)?),
        None => None,
    };
    Ok(Version {
        kind,
        event_id: row.event_id,
        item_id: row.item_id,
        segment_id: row.segment_id,
        json: row.json,
        body,
        known_gap_before: row.known_gap_before,
    })
}

fn base_req(op: &str) -> RpcRequest {
    RpcRequest {
        id: 0,
        op: op.into(),
        collection: None,
        key: None,
        json: None,
        bytes_b64: None,
        limit: None,
        durability: None,
        token: None,
        fields: None,
        force_scan: None,
        order_field: None,
        order_dir: None,
        max_docs_scanned: None,
        max_bytes_scanned: None,
        max_result_bytes: None,
        operation_id: None,
    }
}

/// Mint a fresh client operation id via OS CSPRNG (DEF-025; fail closed).
fn client_operation_id() -> Result<[u8; 16], Error> {
    random_id().map_err(Error::from)
}

fn payload_from_resp(resp: &RpcResponse) -> Result<PayloadResult, Error> {
    let status = resp
        .payload_status
        .as_deref()
        .ok_or_else(|| Error::Internal("get_payload response missing payload_status".into()))?;
    match status {
        "complete" => {
            let b64 = resp
                .bytes_b64
                .as_deref()
                .ok_or_else(|| Error::Internal("complete payload missing bytes_b64".into()))?;
            Ok(PayloadResult::Complete {
                body: b64_decode(b64)?,
            })
        }
        "partial" => {
            let content_hash = parse_hex32(resp.content_hash_hex.as_deref())?;
            let missing = resp.missing.clone().unwrap_or_default();
            let extents = resp
                .extents
                .as_ref()
                .map(|rows| {
                    rows.iter()
                        .map(|e| LogicalExtent {
                            index: e.index,
                            range: ByteRange::new(e.start, e.end),
                            present: e.present,
                        })
                        .collect()
                })
                .unwrap_or_default();
            let mut present_bodies = Vec::new();
            for row in resp.present_chunks.as_ref().into_iter().flatten() {
                present_bodies.push((row.index, b64_decode(&row.bytes_b64)?));
            }
            Ok(PayloadResult::Partial {
                extents,
                missing,
                content_hash,
                present_bodies,
            })
        }
        "unavailable" => {
            let content_hash = parse_hex32(resp.content_hash_hex.as_deref())?;
            Ok(PayloadResult::Unavailable {
                content_hash,
                total_chunks: resp.total_chunks.unwrap_or(0),
            })
        }
        "conflicting" => Ok(PayloadResult::Conflicting {
            index: resp.conflict_index.unwrap_or(0),
        }),
        other => Err(Error::Internal(format!(
            "unknown payload_status from remote: {other}"
        ))),
    }
}

fn payload_to_resp(id: u64, result: &PayloadResult) -> Result<RpcResponse, Error> {
    match result {
        PayloadResult::Complete { body } => Ok(RpcResponse {
            id,
            ok: true,
            found: Some(true),
            payload_status: Some("complete".into()),
            bytes_b64: Some(b64_encode(body)),
            ..empty_resp(id)
        }),
        PayloadResult::Partial {
            extents,
            missing,
            content_hash,
            present_bodies,
        } => {
            let present_chunks = present_bodies
                .iter()
                .map(|(idx, body)| PresentChunkRow {
                    index: *idx,
                    bytes_b64: b64_encode(body),
                })
                .collect();
            let extent_rows = extents
                .iter()
                .map(|e| ExtentRow {
                    index: e.index,
                    start: e.range.start,
                    end: e.range.end,
                    present: e.present,
                })
                .collect();
            Ok(RpcResponse {
                id,
                ok: true,
                found: Some(true),
                payload_status: Some("partial".into()),
                content_hash_hex: Some(hex32(content_hash)),
                missing: Some(missing.clone()),
                present_chunks: Some(present_chunks),
                extents: Some(extent_rows),
                ..empty_resp(id)
            })
        }
        PayloadResult::Unavailable {
            content_hash,
            total_chunks,
        } => Ok(RpcResponse {
            id,
            ok: true,
            found: Some(true),
            payload_status: Some("unavailable".into()),
            content_hash_hex: Some(hex32(content_hash)),
            total_chunks: Some(*total_chunks),
            ..empty_resp(id)
        }),
        PayloadResult::Conflicting { index } => Ok(RpcResponse {
            id,
            ok: true,
            found: Some(true),
            payload_status: Some("conflicting".into()),
            conflict_index: Some(*index),
            ..empty_resp(id)
        }),
    }
}

fn parse_hex32(s: Option<&str>) -> Result<[u8; 32], Error> {
    let Some(s) = s else {
        return Ok([0u8; 32]);
    };
    if s.len() != 64 {
        return Err(Error::Internal(format!(
            "expected 64 hex chars for content hash, got {}",
            s.len()
        )));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| Error::Internal(format!("invalid content hash hex: {e}")))?;
        out[i] = byte;
    }
    Ok(out)
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn write_receipt_from_resp(
    key: &str,
    resp: &RpcResponse,
    _fallback: DurabilityMode,
) -> Result<WriteReceipt, Error> {
    // DEF-014: never invent stronger guarantees than the server proved.
    let _ = _fallback;
    let acknowledgement = require_durability(resp.acknowledgement.as_deref(), "acknowledgement")?;
    let committed = resp.committed.ok_or_else(|| {
        Error::ProtocolViolation("write receipt missing required field `committed`".into())
    })?;
    let event_id = require_hex16(resp.event_id.as_deref(), "event_id")?;
    let version = require_hex16(resp.version.as_deref(), "version")?;
    let store_id = require_hex16(resp.store_id.as_deref(), "store_id")?;
    let segment_id = require_hex16(resp.segment_id.as_deref(), "segment_id")?;
    if event_id == [0u8; 16] {
        return Err(Error::ProtocolViolation(
            "write receipt event_id must not be the zero id".into(),
        ));
    }
    Ok(WriteReceipt {
        // Request key is authoritative when the server omits the echo.
        key: resp.key.clone().unwrap_or_else(|| key.to_string()),
        event_id,
        version,
        acknowledgement,
        committed,
        store_id,
        segment_id,
    })
}

fn delete_receipt_from_resp(
    key: &str,
    resp: &RpcResponse,
    _fallback: DurabilityMode,
) -> Result<DeleteReceipt, Error> {
    let _ = _fallback;
    let acknowledgement = require_durability(resp.acknowledgement.as_deref(), "acknowledgement")?;
    let committed = resp.committed.ok_or_else(|| {
        Error::ProtocolViolation("delete receipt missing required field `committed`".into())
    })?;
    let removed = resp.removed.ok_or_else(|| {
        Error::ProtocolViolation("delete receipt missing required field `removed`".into())
    })?;
    let event_id = require_hex16(resp.event_id.as_deref(), "event_id")?;
    let version = require_hex16(resp.version.as_deref(), "version")?;
    let store_id = require_hex16(resp.store_id.as_deref(), "store_id")?;
    let segment_id = require_hex16(resp.segment_id.as_deref(), "segment_id")?;
    if event_id == [0u8; 16] {
        return Err(Error::ProtocolViolation(
            "delete receipt event_id must not be the zero id".into(),
        ));
    }
    Ok(DeleteReceipt {
        key: resp.key.clone().unwrap_or_else(|| key.to_string()),
        removed,
        event_id,
        version,
        acknowledgement,
        committed,
        store_id,
        segment_id,
    })
}

/// Require a present, well-formed 16-byte hex id (DEF-014: no zero defaults).
fn require_hex16(s: Option<&str>, field: &str) -> Result<[u8; 16], Error> {
    let Some(s) = s else {
        return Err(Error::ProtocolViolation(format!(
            "missing required id field `{field}`"
        )));
    };
    parse_hex16_strict(s, field)
}

fn parse_hex16_strict(s: &str, field: &str) -> Result<[u8; 16], Error> {
    if s.len() != 32 {
        return Err(Error::ProtocolViolation(format!(
            "expected 32 hex chars for `{field}`, got {}",
            s.len()
        )));
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|e| {
            Error::ProtocolViolation(format!("invalid hex id for `{field}`: {e}"))
        })?;
        out[i] = byte;
    }
    Ok(out)
}

/// Parse optional hex id for non-receipt diagnostics (legacy tests only).
#[cfg(test)]
fn parse_hex16(s: Option<&str>) -> Result<[u8; 16], Error> {
    match s {
        None => Err(Error::ProtocolViolation("missing hex id".into())),
        Some(s) => parse_hex16_strict(s, "id"),
    }
}

fn require_durability(s: Option<&str>, field: &str) -> Result<DurabilityMode, Error> {
    match parse_durability(s) {
        Some(m) => Ok(m),
        None => Err(Error::ProtocolViolation(format!(
            "missing or unknown durability field `{field}` (got {s:?})"
        ))),
    }
}

fn fill_receipt_fields(resp: &mut RpcResponse, receipt: &StoreWriteReceipt) {
    resp.event_id = Some(hex16(&receipt.event_id));
    resp.version = Some(hex16(&receipt.item_id));
    resp.store_id = Some(hex16(&receipt.store_id));
    resp.segment_id = Some(hex16(&receipt.segment_id));
    resp.acknowledgement = Some(durability_name(receipt.durability).into());
    resp.committed = Some(true);
}

/// Outcome of client operation_id lookup before a mutation (DEF-010).
enum DedupOutcome {
    /// Exact retry — return the original receipt without writing again.
    Replay(StoreWriteReceipt),
    /// New operation (optional op_id when client omitted it).
    Fresh {
        op_id: Option<[u8; 16]>,
        content_hash: [u8; 32],
    },
}

/// Look up a client operation id; reject content-mismatched reuse.
fn prepare_mutation_dedup(
    store: &Store,
    operation_id_hex: Option<&str>,
    op: &str,
    collection: &str,
    key: &str,
    payload: &[u8],
) -> Result<DedupOutcome, Error> {
    let content_hash = dingo_store::content_identity(op, collection, key, payload);
    let Some(hex) = operation_id_hex else {
        return Ok(DedupOutcome::Fresh {
            op_id: None,
            content_hash,
        });
    };
    let op_id = parse_hex16_strict(hex, "operation_id")?;
    match store.resolve_write_dedup(&op_id, &content_hash)? {
        Some(prior) => Ok(DedupOutcome::Replay(prior)),
        None => Ok(DedupOutcome::Fresh {
            op_id: Some(op_id),
            content_hash,
        }),
    }
}

/// Serve one store over TCP until shutdown or the listener fails (Stage 7 + DEF-030).
///
/// Opens the store **once** (exclusive writer ownership), then admits a bounded
/// number of concurrent connection workers. No auth required by default.
pub fn serve_store(store_path: impl AsRef<Path>, bind: &str) -> Result<(), Error> {
    serve_store_with(store_path, bind, ServeOptions::default())
}

/// Serve with explicit auth / server options.
///
/// Enforces the plaintext bind policy (DEF-002): non-loopback addresses require
/// [`ServeOptions::allow_insecure_bind`]. Emits a structured startup report to
/// stderr that never claims network quorum durability for single-node serve.
///
/// **Bounded server (DEF-030):** one store owner, worker threads per connection,
/// connection admission limits, idle timeouts, overload responses, and optional
/// graceful drain via [`ServeOptions::shutdown_flag`].
pub fn serve_store_with(
    store_path: impl AsRef<Path>,
    bind: &str,
    options: ServeOptions,
) -> Result<(), Error> {
    crate::bind_policy::validate_plaintext_bind(bind, options.allow_insecure_bind)?;
    let path = store_path.as_ref().to_path_buf();
    // Cluster serve prints its own report; single-node prints unless suppressed.
    if !options.suppress_startup_report
        && options.directory.is_none()
        && options.cluster_root.is_none()
    {
        crate::bind_policy::ServeStartupReport::single_node(
            path.display().to_string(),
            bind,
            options.auth_token.is_some(),
            options.allow_insecure_bind,
        )
        .emit_stderr();
        eprintln!(
            "dingo serve: profile={SERVER_PROFILE} protocol={PROTOCOL_PROFILE} \
             max_connections={} idle_timeout_ms={} diagnostic_line={}",
            options.server_limits.max_connections,
            options.server_limits.idle_timeout.as_millis(),
            options.diagnostic_line_protocol
        );
    }

    // One coordinated store owner for the whole process (DEF-020 + DEF-030).
    let store = Arc::new(Mutex::new(Store::open(&path)?));
    let runtime = ServerRuntime::new(options.server_limits.clone(), options.shutdown.clone());
    let listener = TcpListener::bind(bind).map_err(Error::from_io)?;
    // Non-blocking accept so the loop can observe shutdown without a stuck client.
    listener
        .set_nonblocking(true)
        .map_err(Error::from_io)?;

    serve_accept_loop(listener, store, runtime, options)
}

/// Bounded accept loop: admit connections as worker threads until shutdown.
fn serve_accept_loop(
    listener: TcpListener,
    store: Arc<Mutex<Store>>,
    runtime: Arc<ServerRuntime>,
    options: ServeOptions,
) -> Result<(), Error> {
    loop {
        if runtime.is_shutdown_requested() {
            break;
        }
        match listener.accept() {
            Ok((stream, _addr)) => {
                // Accepted fds inherit non-blocking from the listener on some
                // platforms; workers use blocking reads with idle timeouts.
                if let Err(e) = stream.set_nonblocking(false) {
                    eprintln!("dingo serve: set_nonblocking(false) failed: {e}");
                    continue;
                }
                if !runtime.try_admit_connection() {
                    // Overload / drain: reply once and drop (backpressure).
                    let max = runtime.limits().max_connections;
                    let reason = if runtime.is_draining() {
                        "server draining; connection refused".to_string()
                    } else {
                        format!("connection limit exceeded (max {max})")
                    };
                    let _ = reject_connection(stream, &reason);
                    continue;
                }
                let guard = ConnectionGuard::new(Arc::clone(&runtime));
                let store_c = Arc::clone(&store);
                let opts_c = options.clone();
                let runtime_c = Arc::clone(&runtime);
                // Detach worker: accept loop must not wait on client I/O.
                // On spawn failure the ConnectionGuard drops here and releases the slot.
                thread::Builder::new()
                    .name("dingo-serve-conn".into())
                    .spawn(move || {
                        let _guard = guard;
                        if let Err(e) =
                            handle_connection_shared(store_c, stream, opts_c, runtime_c)
                        {
                            eprintln!("dingo serve connection error: {e}");
                        }
                    })
                    .map_err(Error::from_io)?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(Error::from_io(e)),
        }
    }

    // Graceful drain: stop new admissions, wait for workers, report mutation outcome.
    runtime.begin_drain();
    let idle = runtime.wait_for_idle();
    let stats = runtime.stats();
    eprintln!(
        "dingo serve: drain {} active={} peak={} rejected={} accepted={} \
         mutations_started={} mutations_finished={}",
        if idle { "complete" } else { "timeout" },
        stats.active_connections,
        stats.peak_connections,
        stats.rejected_connections,
        stats.accepted_connections,
        stats.mutations_started,
        stats.mutations_finished
    );
    if !idle {
        return Err(Error::ResourceLimit(format!(
            "graceful drain timed out with {} connection(s) still active; \
             mutations_started={} mutations_finished={}",
            stats.active_connections, stats.mutations_started, stats.mutations_finished
        )));
    }
    if stats.mutations_started != stats.mutations_finished {
        return Err(Error::Internal(format!(
            "drain accounting mismatch: mutations_started={} mutations_finished={}",
            stats.mutations_started, stats.mutations_finished
        )));
    }
    Ok(())
}

/// Write an unsolicited overload/drain framed reject and close the socket.
///
/// Emitted before the worker admits the connection (and before application
/// RPCs). Framed clients parse this as a handshake reject with
/// `code=resource_limit`. Diagnostic servers also emit a line JSON body so
/// simple tools can still observe the refusal.
fn reject_connection(mut stream: TcpStream, reason: &str) -> Result<(), Error> {
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    // Prefer framed reject (production clients expect a frame on first read
    // after their hello, or as an unsolicited first frame on overload).
    let _ = write_reject_frame(&mut stream, "resource_limit", reason);
    Ok(())
}

fn decode_rpc_response(bytes: &[u8], endpoint: &str, addr: &str) -> Result<RpcResponse, Error> {
    let resp: RpcResponse = serde_json::from_slice(bytes).map_err(|e| {
        Error::Internal(format!(
            "invalid rpc response from {endpoint} ({addr}): {e}"
        ))
    })?;
    if !resp.ok {
        let code = resp.code.unwrap_or_else(|| "internal".into());
        let message = resp
            .error
            .unwrap_or_else(|| "remote operation failed".into());
        return Err(match code.as_str() {
            "authentication_failed" => Error::AuthenticationFailed(message),
            "deadline_exceeded" => Error::DeadlineExceeded(message),
            "query_budget_required" => Error::QueryBudgetRequired(message),
            "query_invalid" => Error::QueryInvalid(message),
            "consistency_violation" => Error::ConsistencyViolation(message),
            "coverage_incomplete" => Error::CoverageIncomplete(message),
            "resource_limit" => Error::ResourceLimit(message),
            "protocol_violation" => Error::ProtocolViolation(message),
            _ => Error::Remote { code, message },
        });
    }
    Ok(resp)
}

/// Serve one node of a multi-node cluster root over TCP (network follow-on).
///
/// Opens `cluster_root/nodes/node-{node_index}`, upserts `bind` into
/// `endpoints.json`, and advertises the live [`PartitionDirectory`] plus
/// endpoints on every `directory` RPC so multi-seed clients can cache routes.
///
/// Writes still apply to **this node's store only** in this slice (single-node
/// RPC dispatch). In-process quorum remains `Dingo::open_cluster`; multi-hop
/// Raft over the network continues to harden on this advertise path.
///
/// **Experimental (DEF-002):** requires
/// [`ServeOptions::experimental_network_cluster`]. This is a routing and
/// endpoint-advertisement prototype, **not** network quorum replication.
pub fn serve_cluster_node(
    cluster_root: impl AsRef<Path>,
    node_index: u32,
    bind: &str,
    options: ServeOptions,
) -> Result<(), Error> {
    if !options.experimental_network_cluster {
        return Err(Error::ValidationMsg(
            "serve-cluster is experimental: pass ServeOptions::experimental_network_cluster(true) \
             or CLI --experimental-network-cluster. Network quorum replication is not implemented; \
             writes apply to this node only (DEF-002)."
                .into(),
        ));
    }
    crate::bind_policy::validate_plaintext_bind(bind, options.allow_insecure_bind)?;

    let root = cluster_root.as_ref();
    let meta = dingo_cluster::ClusterMeta::load(root)
        .map_err(|e| Error::Internal(format!("cluster root {}: {e}", root.display())))?;
    if node_index >= meta.node_count {
        return Err(Error::ValidationMsg(format!(
            "node index {node_index} out of range (cluster has {} nodes)",
            meta.node_count
        )));
    }

    let endpoints = dingo_cluster::upsert_endpoint(root, node_index, bind)
        .map_err(|e| Error::Internal(format!("endpoints.json: {e}")))?;

    let directory = dingo_cluster::PartitionDirectory::load(root)
        .map_err(|e| Error::Internal(format!("placement.json: {e}")))?
        .ok_or_else(|| {
            Error::Internal(format!("missing placement.json under {}", root.display()))
        })?;

    let snapshot = DirectorySnapshot::from_directory(&directory, endpoints);
    let opts = options
        .directory(snapshot)
        .node_index(node_index)
        .cluster_root(root);

    let store_path = dingo_cluster::node_store_path(root, node_index);
    if !store_path.join("store-info").is_dir() {
        return Err(Error::Internal(format!(
            "node store missing at {} (expected store-info/)",
            store_path.display()
        )));
    }

    if !opts.suppress_startup_report {
        crate::bind_policy::ServeStartupReport::cluster_node(
            root.display().to_string(),
            bind,
            opts.auth_token.is_some(),
            opts.allow_insecure_bind,
            node_index,
        )
        .emit_stderr();
        eprintln!(
            "dingo serve-cluster: root={} node={node_index} store={} bind={bind} nodes={}",
            root.display(),
            store_path.display(),
            meta.node_count
        );
    }
    // Avoid a second single-node-style report inside serve_store_with.
    let opts = opts.suppress_startup_report(true);
    serve_store_with(store_path, bind, opts)
}

/// Serve a single already-open TCP client (used by tests and `serve_store`).
pub fn handle_connection(store: &mut Store, stream: TcpStream) -> Result<(), Error> {
    handle_connection_with(store, stream, ServeOptions::default())
}

/// Handle one client with server options (auth token).
///
/// Single-threaded helper for tests that own a exclusive [`Store`]. Production
/// `serve_store_with` uses [`handle_connection_shared`] so many clients share
/// one store owner under a mutex.
pub fn handle_connection_with(
    store: &mut Store,
    stream: TcpStream,
    options: ServeOptions,
) -> Result<(), Error> {
    // Local single-connection path: no shared runtime accounting.
    let runtime = ServerRuntime::new(options.server_limits.clone(), None);
    // Pretend admitted so drain checks are no-ops.
    let _ = runtime.try_admit_connection();
    let _guard = ConnectionGuard::new(Arc::clone(&runtime));
    connection_loop(store, stream, &options, &runtime)
}

/// Handle one client against a shared store owner (DEF-030 worker path).
///
/// Holds the store mutex only while dispatching an RPC — never across socket
/// reads/writes — so one slow peer cannot block unrelated clients' store access
/// during network I/O.
pub fn handle_connection_shared(
    store: Arc<Mutex<Store>>,
    stream: TcpStream,
    options: ServeOptions,
    runtime: Arc<ServerRuntime>,
) -> Result<(), Error> {
    let idle = options.server_limits.idle_timeout;
    stream
        .set_read_timeout(Some(idle))
        .map_err(Error::from_io)?;
    stream
        .set_write_timeout(Some(idle))
        .map_err(Error::from_io)?;
    let mut reader = BufReader::new(stream.try_clone().map_err(Error::from_io)?);
    let mut writer = stream;

    let max_frame = if options.diagnostic_line_protocol {
        crate::resource::host_limits().max_rpc_line_bytes
    } else {
        match server_handshake(&mut reader, &mut writer) {
            Ok(session) => session.max_frame,
            Err(e) => {
                // Handshake already wrote a reject when possible.
                return Err(e);
            }
        }
    };

    loop {
        if runtime.is_draining() {
            // Finish after the current request; refuse further work cleanly.
        }
        let req = match read_rpc_request(&mut reader, &mut writer, max_frame, &options) {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(ReadRpc::Idle) => break,
            Err(ReadRpc::Fatal(e)) => return Err(e),
            Err(ReadRpc::Continue) => continue,
            Err(ReadRpc::Close) => break,
        };
        if runtime.is_draining() {
            write_rpc_response(
                &mut writer,
                &options,
                RpcResponse {
                    id: req.id,
                    ok: false,
                    error: Some("server draining; request refused".into()),
                    code: Some("resource_limit".into()),
                    ..empty_resp(req.id)
                },
            )?;
            break;
        }
        if let Some(required) = options.auth_token.as_deref() {
            let presented = req.token.as_deref().unwrap_or("");
            if presented != required {
                write_rpc_response(
                    &mut writer,
                    &options,
                    RpcResponse {
                        id: req.id,
                        ok: false,
                        error: Some("invalid or missing auth token".into()),
                        code: Some("authentication_failed".into()),
                        ..empty_resp(req.id)
                    },
                )?;
                continue;
            }
        }
        let resp = {
            let _mutation = if is_mutating_op(&req.op) {
                Some(MutationGuard::new(Arc::clone(&runtime)))
            } else {
                None
            };
            // Serialize store access; release before writing the response.
            let mut guard = store.lock().map_err(|_| {
                Error::Internal("store mutex poisoned".into())
            })?;
            match dispatch(&mut guard, &req, &options) {
                Ok(r) => r,
                Err(e) => RpcResponse {
                    id: req.id,
                    ok: false,
                    error: Some(e.to_string()),
                    code: Some(e.code().as_str().into()),
                    ..empty_resp(req.id)
                },
            }
        };
        write_rpc_response(&mut writer, &options, resp)?;
    }
    Ok(())
}

/// Exclusive-store connection loop used by [`handle_connection_with`].
fn connection_loop(
    store: &mut Store,
    stream: TcpStream,
    options: &ServeOptions,
    runtime: &ServerRuntime,
) -> Result<(), Error> {
    let idle = options.server_limits.idle_timeout;
    stream
        .set_read_timeout(Some(idle))
        .map_err(Error::from_io)?;
    stream
        .set_write_timeout(Some(idle))
        .map_err(Error::from_io)?;
    let mut reader = BufReader::new(stream.try_clone().map_err(Error::from_io)?);
    let mut writer = stream;

    let max_frame = if options.diagnostic_line_protocol {
        crate::resource::host_limits().max_rpc_line_bytes
    } else {
        server_handshake(&mut reader, &mut writer)?.max_frame
    };

    loop {
        let req = match read_rpc_request(&mut reader, &mut writer, max_frame, options) {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(ReadRpc::Idle) => break,
            Err(ReadRpc::Fatal(e)) => return Err(e),
            Err(ReadRpc::Continue) => continue,
            Err(ReadRpc::Close) => break,
        };
        if let Some(required) = options.auth_token.as_deref() {
            let presented = req.token.as_deref().unwrap_or("");
            if presented != required {
                write_rpc_response(
                    &mut writer,
                    options,
                    RpcResponse {
                        id: req.id,
                        ok: false,
                        error: Some("invalid or missing auth token".into()),
                        code: Some("authentication_failed".into()),
                        ..empty_resp(req.id)
                    },
                )?;
                continue;
            }
        }
        // Exclusive test path: no process-wide mutation accounting (runtime is
        // local to this connection). Shared workers use MutationGuard.
        let _ = runtime;
        let resp = match dispatch(store, &req, options) {
            Ok(r) => r,
            Err(e) => RpcResponse {
                id: req.id,
                ok: false,
                error: Some(e.to_string()),
                code: Some(e.code().as_str().into()),
                ..empty_resp(req.id)
            },
        };
        write_rpc_response(&mut writer, options, resp)?;
    }
    Ok(())
}

/// Outcome of reading one application RPC from the wire.
enum ReadRpc {
    /// Idle timeout on the socket.
    Idle,
    /// Soft error already answered; keep the connection.
    Continue,
    /// Drop the connection (adversarial / oversized).
    Close,
    /// Hard transport failure.
    Fatal(Error),
}

fn read_rpc_request(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
    max_frame: usize,
    options: &ServeOptions,
) -> Result<Option<RpcRequest>, ReadRpc> {
    if options.diagnostic_line_protocol {
        let mut line = String::new();
        let n = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                return Err(ReadRpc::Idle);
            }
            Err(e) => return Err(ReadRpc::Fatal(Error::from_io(e))),
        };
        if n == 0 {
            return Ok(None);
        }
        if let Err(e) = crate::resource::check_rpc_line_len(line.len(), &crate::resource::host_limits())
        {
            let _ = write_rpc_response(
                writer,
                options,
                RpcResponse {
                    id: 0,
                    ok: false,
                    error: Some(e.to_string()),
                    code: Some(e.code().as_str().into()),
                    ..empty_resp(0)
                },
            );
            return Err(ReadRpc::Close);
        }
        let line = line.trim();
        if line.is_empty() {
            return Err(ReadRpc::Continue);
        }
        match serde_json::from_str(line) {
            Ok(r) => Ok(Some(r)),
            Err(e) => {
                let _ = write_rpc_response(
                    writer,
                    options,
                    RpcResponse {
                        id: 0,
                        ok: false,
                        error: Some(format!("bad request: {e}")),
                        code: Some("validation_failed".into()),
                        ..empty_resp(0)
                    },
                );
                Err(ReadRpc::Continue)
            }
        }
    } else {
        let bytes = match protocol::read_frame(reader, max_frame) {
            Ok(Some(b)) => b,
            Ok(None) => return Ok(None),
            Err(e)
                if matches!(
                    e,
                    Error::DeadlineExceeded(_)
                ) =>
            {
                return Err(ReadRpc::Idle);
            }
            Err(e) if e.code() == crate::ErrorCode::ResourceLimit => {
                let _ = write_rpc_response(
                    writer,
                    options,
                    RpcResponse {
                        id: 0,
                        ok: false,
                        error: Some(e.to_string()),
                        code: Some("resource_limit".into()),
                        ..empty_resp(0)
                    },
                );
                return Err(ReadRpc::Close);
            }
            Err(e) => return Err(ReadRpc::Fatal(e)),
        };
        match serde_json::from_slice(&bytes) {
            Ok(r) => Ok(Some(r)),
            Err(e) => {
                let _ = write_rpc_response(
                    writer,
                    options,
                    RpcResponse {
                        id: 0,
                        ok: false,
                        error: Some(format!("bad request: {e}")),
                        code: Some("validation_failed".into()),
                        ..empty_resp(0)
                    },
                );
                Err(ReadRpc::Continue)
            }
        }
    }
}

fn write_rpc_response(
    w: &mut TcpStream,
    options: &ServeOptions,
    resp: RpcResponse,
) -> Result<(), Error> {
    if options.diagnostic_line_protocol {
        write_resp_line(w, resp)
    } else {
        write_json_frame(w, &resp)
    }
}

fn empty_resp(id: u64) -> RpcResponse {
    RpcResponse {
        id,
        ok: true,
        error: None,
        code: None,
        value: None,
        found: None,
        removed: None,
        key: None,
        committed: None,
        acknowledgement: None,
        keys: None,
        rows: None,
        bytes_b64: None,
        store_id: None,
        path: None,
        live_count: None,
        event_id: None,
        version: None,
        segment_id: None,
        versions: None,
        has_known_holes: None,
        indexes: None,
        payload_status: None,
        content_hash_hex: None,
        missing: None,
        total_chunks: None,
        conflict_index: None,
        present_chunks: None,
        extents: None,
        directory: None,
    }
}

/// Whether an error is a transport / connection failure worth re-routing.
fn is_transport_error(err: &Error) -> bool {
    match err {
        Error::DeadlineExceeded(_) | Error::Io(_) => true,
        Error::Internal(msg) => {
            msg.contains("remote closed")
                || msg.contains("Connection refused")
                || msg.contains("Broken pipe")
                || msg.contains("os error")
                || msg.contains("timed out")
                || msg.contains("reset")
        }
        Error::Store(se) if se.is_io() => true,
        _ => {
            let s = err.to_string();
            s.contains("Connection refused")
                || s.contains("Broken pipe")
                || s.contains("Connection reset")
        }
    }
}

fn tcp_connect_with_retry(addr: &str, options: &ConnectOptions) -> Result<TcpStream, Error> {
    let attempts = options.max_connect_attempts.max(1);
    let mut last_err: Option<Error> = None;
    for attempt in 0..attempts {
        match tcp_connect_once(addr, options.connect_timeout) {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                last_err = Some(e);
                if attempt + 1 < attempts {
                    thread::sleep(options.retry_backoff);
                }
            }
        }
    }
    Err(last_err
        .unwrap_or_else(|| Error::Internal(format!("connect failed with no attempts: {addr}"))))
}

fn tcp_connect_once(addr: &str, timeout: Duration) -> Result<TcpStream, Error> {
    let mut addrs = addr.to_socket_addrs().map_err(Error::from_io)?;
    let mut last_io: Option<std::io::Error> = None;
    for sa in addrs.by_ref() {
        match TcpStream::connect_timeout(&sa, timeout) {
            Ok(s) => return Ok(s),
            Err(e) => last_io = Some(e),
        }
    }
    Err(Error::from_io(last_io.unwrap_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("no addresses resolved for {addr}"),
        )
    })))
}

fn write_resp_line(w: &mut TcpStream, resp: RpcResponse) -> Result<(), Error> {
    let mut line = serde_json::to_string(&resp).map_err(|e| Error::Internal(e.to_string()))?;
    line.push('\n');
    w.write_all(line.as_bytes()).map_err(Error::from_io)?;
    w.flush().map_err(Error::from_io)?;
    Ok(())
}

fn dispatch(
    store: &mut Store,
    req: &RpcRequest,
    options: &ServeOptions,
) -> Result<RpcResponse, Error> {
    let id = req.id;
    match req.op.as_str() {
        "ping" => Ok(RpcResponse {
            id,
            ok: true,
            ..empty_resp(id)
        }),
        "store_info" => Ok(RpcResponse {
            id,
            ok: true,
            path: Some(store.path().display().to_string()),
            store_id: Some(hex16(&store.store_id())),
            live_count: Some(store.live_count()),
            ..empty_resp(id)
        }),
        "directory" => {
            // Prefer operator-supplied cluster snapshot (serve-cluster).
            if let Some(mut dir) = options.directory.clone() {
                // Live reload endpoints so nodes that join after this process
                // started are visible without restarting every peer.
                if let Some(root) = options.cluster_root.as_ref() {
                    if let Ok(eps) = dingo_cluster::load_endpoints(root) {
                        dir.endpoints = eps;
                    }
                }
                return Ok(RpcResponse {
                    id,
                    ok: true,
                    directory: Some(dir),
                    ..empty_resp(id)
                });
            }
            // Single-node: all virtual partitions led by node 0 (this server).
            // Clients cache this and may refresh on stale_epoch (CLUSTER_SPEC §13).
            let n = DEFAULT_VIRTUAL_PARTITIONS;
            let mut assignments = Vec::with_capacity(n as usize);
            for p in 0..n {
                assignments.push(AssignmentWire {
                    partition: p,
                    replicas: vec![0],
                    leader: 0,
                    term: 1,
                    placement_epoch: 1,
                });
            }
            let mut endpoints = HashMap::new();
            // Endpoint is informational for single-node; client already connected.
            endpoints.insert(0, String::new());
            Ok(RpcResponse {
                id,
                ok: true,
                directory: Some(DirectorySnapshot {
                    virtual_partitions: n,
                    hash_profile: HASH_PROFILE_BLAKE3_MOD.to_string(),
                    placement_epoch: 1,
                    assignments,
                    endpoints,
                }),
                ..empty_resp(id)
            })
        }
        "list_collections" => Ok(RpcResponse {
            id,
            ok: true,
            keys: Some(store.list_collections()),
            ..empty_resp(id)
        }),
        "put" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let json = req
                .json
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("put requires json".into()))?;
            // DEF-029: host limits on depth and payload size (server-side).
            let limits = crate::resource::host_limits();
            crate::resource::check_json_depth(json, &limits)?;
            let mode =
                parse_durability(req.durability.as_deref()).unwrap_or(DurabilityMode::Durable);
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            let body = encode_json(json)?;
            crate::resource::check_payload_len(body.len(), &limits)?;
            let dedup = prepare_mutation_dedup(
                store,
                req.operation_id.as_deref(),
                "put",
                coll,
                key,
                &body,
            )?;
            let receipt = match dedup {
                DedupOutcome::Replay(r) => r,
                DedupOutcome::Fresh { op_id, content_hash } => {
                    let receipt = store.put(subject_str, &body, mode)?;
                    if let Some(op_id) = op_id {
                        store.record_write_dedup(op_id, content_hash, &receipt)?;
                    }
                    receipt
                }
            };
            let _ = mark_indexes_stale(store, coll);
            let mut resp = RpcResponse {
                id,
                ok: true,
                key: Some(key.to_string()),
                ..empty_resp(id)
            };
            fill_receipt_fields(&mut resp, &receipt);
            Ok(resp)
        }
        "get" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            match store.get(subject_str)? {
                None => Ok(RpcResponse {
                    id,
                    ok: true,
                    found: Some(false),
                    ..empty_resp(id)
                }),
                Some(body) => Ok(RpcResponse {
                    id,
                    ok: true,
                    found: Some(true),
                    value: Some(decode_json(&body)?),
                    ..empty_resp(id)
                }),
            }
        }
        "delete" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let mode =
                parse_durability(req.durability.as_deref()).unwrap_or(DurabilityMode::Durable);
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            let dedup = prepare_mutation_dedup(
                store,
                req.operation_id.as_deref(),
                "delete",
                coll,
                key,
                &[],
            )?;
            let (receipt, removed) = match dedup {
                DedupOutcome::Replay(r) => {
                    // Exact retry: subject is already gone after the first delete.
                    (r, false)
                }
                DedupOutcome::Fresh { op_id, content_hash } => {
                    let removed = store.get(subject_str)?.is_some();
                    let receipt = store.delete(subject_str, mode)?;
                    if let Some(op_id) = op_id {
                        store.record_write_dedup(op_id, content_hash, &receipt)?;
                    }
                    (receipt, removed)
                }
            };
            let _ = mark_indexes_stale(store, coll);
            let mut resp = RpcResponse {
                id,
                ok: true,
                key: Some(key.to_string()),
                removed: Some(removed),
                ..empty_resp(id)
            };
            fill_receipt_fields(&mut resp, &receipt);
            Ok(resp)
        }
        "put_bytes" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let b64 = req
                .bytes_b64
                .as_deref()
                .ok_or_else(|| Error::QueryInvalid("put_bytes requires bytes_b64".into()))?;
            let bytes = b64_decode(b64)?;
            let limits = crate::resource::host_limits();
            crate::resource::check_payload_len(bytes.len().saturating_add(1), &limits)?;
            let mode =
                parse_durability(req.durability.as_deref()).unwrap_or(DurabilityMode::Durable);
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            let body = encode_bytes(&bytes);
            let dedup = prepare_mutation_dedup(
                store,
                req.operation_id.as_deref(),
                "put_bytes",
                coll,
                key,
                &body,
            )?;
            let receipt = match dedup {
                DedupOutcome::Replay(r) => r,
                DedupOutcome::Fresh { op_id, content_hash } => {
                    let receipt = store.put(subject_str, &body, mode)?;
                    if let Some(op_id) = op_id {
                        store.record_write_dedup(op_id, content_hash, &receipt)?;
                    }
                    receipt
                }
            };
            let _ = mark_indexes_stale(store, coll);
            let mut resp = RpcResponse {
                id,
                ok: true,
                key: Some(key.to_string()),
                ..empty_resp(id)
            };
            fill_receipt_fields(&mut resp, &receipt);
            Ok(resp)
        }
        "get_bytes" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            match store.get(subject_str)? {
                None => Ok(RpcResponse {
                    id,
                    ok: true,
                    found: Some(false),
                    ..empty_resp(id)
                }),
                Some(body) => Ok(RpcResponse {
                    id,
                    ok: true,
                    found: Some(true),
                    bytes_b64: Some(b64_encode(&decode_bytes(&body)?)),
                    ..empty_resp(id)
                }),
            }
        }
        "list_keys" => {
            let coll = require_coll(req)?;
            let prefix = collection_prefix(coll)?;
            let mut keys = Vec::new();
            for (subject, _body) in store.live_entries() {
                if !subject.starts_with(&prefix) {
                    continue;
                }
                if let Some((c, key)) = decode_subject(subject) {
                    if c == coll {
                        keys.push(key.to_string());
                    }
                }
            }
            keys.sort();
            Ok(RpcResponse {
                id,
                ok: true,
                keys: Some(keys),
                ..empty_resp(id)
            })
        }
        "scan_json" => {
            let coll = require_coll(req)?;
            let prefix = collection_prefix(coll)?;
            let logical = store.live_logical_entries()?;
            let mut rows = Vec::new();
            for (subject, body) in logical {
                if !subject.starts_with(&prefix) {
                    continue;
                }
                let Some((c, key)) = decode_subject(&subject) else {
                    continue;
                };
                if c != coll {
                    continue;
                }
                // DEF-012: never convert decode failure into a shorter success.
                let value = decode_json(&body).map_err(|e| {
                    Error::CoverageIncomplete(format!(
                        "scan_json: key {key:?} failed JSON decode: {e}"
                    ))
                })?;
                rows.push(ScanRow {
                    key: key.to_string(),
                    value,
                });
                if let Some(limit) = req.limit {
                    if rows.len() >= limit {
                        break;
                    }
                }
            }
            rows.sort_by(|a, b| a.key.cmp(&b.key));
            Ok(RpcResponse {
                id,
                ok: true,
                rows: Some(rows),
                ..empty_resp(id)
            })
        }
        "history" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            let hist = store.history(subject_str)?;
            let projected = KeyHistory::from_store(key.to_string(), hist)?;
            let versions = projected
                .versions
                .into_iter()
                .map(|v| HistoryVersionRow {
                    kind: v.kind.into(),
                    event_id: v.event_id,
                    item_id: v.item_id,
                    segment_id: v.segment_id,
                    json: v.json,
                    body_b64: v.body.as_deref().map(b64_encode),
                    known_gap_before: v.known_gap_before,
                })
                .collect();
            Ok(RpcResponse {
                id,
                ok: true,
                key: Some(projected.key),
                versions: Some(versions),
                has_known_holes: Some(projected.has_known_holes),
                ..empty_resp(id)
            })
        }
        "index_list" => {
            let coll = require_coll(req)?;
            let indexes = store
                .list_secondary_indexes(coll)?
                .iter()
                .map(|i| IndexInfoRow::from_info(&IndexInfo::from_store(i)))
                .collect();
            Ok(RpcResponse {
                id,
                ok: true,
                indexes: Some(indexes),
                ..empty_resp(id)
            })
        }
        "index_create" => {
            let coll = require_coll(req)?;
            let name = require_key(req)?;
            let fields = req
                .fields
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("index_create requires fields".into()))?;
            if fields.is_empty() {
                return Err(Error::QueryInvalid(
                    "index requires at least one field".into(),
                ));
            }
            let field_refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
            let info = create_index_on_store(store, coll, name, &field_refs)?;
            Ok(RpcResponse {
                id,
                ok: true,
                indexes: Some(vec![IndexInfoRow::from_info(&info)]),
                ..empty_resp(id)
            })
        }
        "index_drop" => {
            let coll = require_coll(req)?;
            let name = require_key(req)?;
            store.delete_secondary_index(coll, name)?;
            Ok(RpcResponse {
                id,
                ok: true,
                ..empty_resp(id)
            })
        }
        "index_rebuild" => {
            let coll = require_coll(req)?;
            let name = require_key(req)?;
            let existing = store
                .load_secondary_index(coll, name)?
                .ok_or_else(|| Error::QueryInvalid(format!("index not found: {name}")))?;
            let fields: Vec<&str> = existing.meta.fields.iter().map(|s| s.as_str()).collect();
            let info = create_index_on_store(store, coll, name, &fields)?;
            Ok(RpcResponse {
                id,
                ok: true,
                indexes: Some(vec![IndexInfoRow::from_info(&info)]),
                ..empty_resp(id)
            })
        }
        "get_payload" => {
            let coll = require_coll(req)?;
            let key = require_key(req)?;
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            match store.get_payload(subject_str)? {
                None => Ok(RpcResponse {
                    id,
                    ok: true,
                    found: Some(false),
                    ..empty_resp(id)
                }),
                Some(result) => payload_to_resp(id, &result),
            }
        }
        "find" => {
            let coll = require_coll(req)?;
            let filter_json = req
                .json
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("find requires json filter object".into()))?;
            let filter = Filter::from_json(filter_json)?;
            let order_by = match (
                req.order_field.as_deref(),
                req.order_dir.as_deref().unwrap_or("asc"),
            ) {
                (Some(f), "desc") => Some((f.to_string(), SortOrder::Desc)),
                (Some(f), _) => Some((f.to_string(), SortOrder::Asc)),
                (None, _) => None,
            };
            let budget = if req.max_docs_scanned.is_some()
                || req.max_bytes_scanned.is_some()
                || req.max_result_bytes.is_some()
            {
                Some(QueryBudget {
                    max_docs_scanned: req.max_docs_scanned,
                    max_bytes_scanned: req.max_bytes_scanned,
                    max_result_bytes: req.max_result_bytes,
                })
            } else {
                None
            };
            let options = QueryOptions {
                limit: req.limit,
                order_by,
                budget,
                force_scan: req.force_scan.unwrap_or(false),
                allow_partial_coverage: false,
                cancel: None,
            };
            let rows = find_on_store(store, coll, &filter, &options)?;
            Ok(RpcResponse {
                id,
                ok: true,
                rows: Some(
                    rows.into_iter()
                        .map(|(key, value)| ScanRow { key, value })
                        .collect(),
                ),
                ..empty_resp(id)
            })
        }
        other => Err(Error::QueryInvalid(format!("unknown op: {other}"))),
    }
}

fn require_coll(req: &RpcRequest) -> Result<&str, Error> {
    let c = req
        .collection
        .as_deref()
        .ok_or(Error::InvalidCollectionName("missing collection"))?;
    validate_collection_name(c)?;
    Ok(c)
}

fn require_key(req: &RpcRequest) -> Result<&str, Error> {
    let k = req.key.as_deref().ok_or(Error::InvalidKey("missing key"))?;
    validate_key(k)?;
    Ok(k)
}

fn durability_name(mode: DurabilityMode) -> &'static str {
    mode.as_str()
}

fn parse_durability(s: Option<&str>) -> Option<DurabilityMode> {
    match s? {
        "memory" => Some(DurabilityMode::Memory),
        "buffered" => Some(DurabilityMode::Buffered),
        "durable" => Some(DurabilityMode::Durable),
        _ => None,
    }
}

fn hex16(id: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in id {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn b64_encode(bytes: &[u8]) -> String {
    // Minimal base64 (std-only) for Stage 7 wire encoding.
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = if i + 1 < bytes.len() {
            bytes[i + 1] as u32
        } else {
            0
        };
        let b2 = if i + 2 < bytes.len() {
            bytes[i + 2] as u32
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(T[((triple >> 18) & 63) as usize] as char);
        out.push(T[((triple >> 12) & 63) as usize] as char);
        if i + 1 < bytes.len() {
            out.push(T[((triple >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < bytes.len() {
            out.push(T[(triple & 63) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

fn b64_decode(s: &str) -> Result<Vec<u8>, Error> {
    fn val(c: u8) -> Result<u8, Error> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(Error::QueryInvalid("invalid base64".into())),
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err(Error::QueryInvalid("invalid base64 length".into()));
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut i = 0;
    while i < bytes.len() {
        let a = val(bytes[i])?;
        let b = val(bytes[i + 1])?;
        let c = if bytes[i + 2] == b'=' {
            0
        } else {
            val(bytes[i + 2])?
        };
        let d = if bytes[i + 3] == b'=' {
            0
        } else {
            val(bytes[i + 3])?
        };
        let triple = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
        out.push(((triple >> 16) & 0xff) as u8);
        if bytes[i + 2] != b'=' {
            out.push(((triple >> 8) & 0xff) as u8);
        }
        if bytes[i + 3] != b'=' {
            out.push((triple & 0xff) as u8);
        }
        i += 4;
    }
    Ok(out)
}

/// Parsed `dingo://` URL (Stage 7 single-seed; Stage 8d multi-seed).
///
/// Forms:
/// - `dingo://host:port[/label]`
/// - `dingo://h1:p1,h2:p2,h3:p3[/label]` (comma-separated seeds)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDingoUrl {
    /// Seed endpoints as `host:port` (at least one).
    pub seeds: Vec<String>,
    /// Optional path label (informational for Stage 7/8d).
    pub label: Option<String>,
}

/// Parse a `dingo://host:port[/path]` or multi-seed URL.
pub fn parse_dingo_url(url: &str) -> Result<ParsedDingoUrl, Error> {
    let rest = url
        .strip_prefix("dingo://")
        .ok_or_else(|| Error::ValidationMsg("URL must start with dingo://".into()))?;
    if rest.is_empty() {
        return Err(Error::ValidationMsg("empty dingo:// URL".into()));
    }
    let (hosts_part, label) = match rest.split_once('/') {
        Some((hp, path)) => {
            let label = path.trim_matches('/');
            (
                hp,
                if label.is_empty() {
                    None
                } else {
                    Some(label.to_string())
                },
            )
        }
        None => (rest, None),
    };
    if hosts_part.is_empty() {
        return Err(Error::ValidationMsg("dingo:// URL missing host".into()));
    }
    let mut seeds = Vec::new();
    for part in hosts_part.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let hostport = if part.contains(':') {
            part.to_string()
        } else {
            format!("{part}:{DEFAULT_PORT}")
        };
        seeds.push(hostport);
    }
    if seeds.is_empty() {
        return Err(Error::ValidationMsg("dingo:// URL has no seeds".into()));
    }
    Ok(ParsedDingoUrl { seeds, label })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64_roundtrip() {
        let samples: &[&[u8]] = &[b"", b"f", b"fo", b"foo", b"foob", b"fooba", b"foobar"];
        for s in samples {
            let enc = b64_encode(s);
            let dec = b64_decode(&enc).unwrap();
            assert_eq!(&dec, s);
        }
    }

    #[test]
    fn parse_url() {
        let p = parse_dingo_url("dingo://localhost:7434/app").unwrap();
        assert_eq!(p.seeds, vec!["localhost:7434".to_string()]);
        assert_eq!(p.label.as_deref(), Some("app"));
        let p = parse_dingo_url("dingo://127.0.0.1").unwrap();
        assert_eq!(p.seeds, vec!["127.0.0.1:7434".to_string()]);
        assert!(p.label.is_none());
        let p = parse_dingo_url("dingo://a:1,b:2,c:3/app").unwrap();
        assert_eq!(
            p.seeds,
            vec!["a:1".to_string(), "b:2".to_string(), "c:3".to_string()]
        );
        assert_eq!(p.label.as_deref(), Some("app"));
    }

    #[test]
    fn hex16_roundtrip() {
        let id = [
            0xde, 0xad, 0xbe, 0xef, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
        ];
        let s = hex16(&id);
        assert_eq!(s.len(), 32);
        assert_eq!(parse_hex16(Some(&s)).unwrap(), id);
        assert!(parse_hex16(None).is_err());
    }

    #[test]
    fn write_receipt_fails_closed_without_committed() {
        let resp = RpcResponse {
            id: 1,
            ok: true,
            event_id: Some(hex16(&[1u8; 16])),
            version: Some(hex16(&[2u8; 16])),
            store_id: Some(hex16(&[3u8; 16])),
            segment_id: Some(hex16(&[4u8; 16])),
            acknowledgement: Some("durable".into()),
            committed: None,
            key: Some("k".into()),
            ..empty_resp(1)
        };
        let err = write_receipt_from_resp("k", &resp, DurabilityMode::Durable).unwrap_err();
        assert_eq!(err.code(), crate::ErrorCode::ProtocolViolation);
    }

    #[test]
    fn write_receipt_rejects_optimistic_durability_fallback() {
        let resp = RpcResponse {
            id: 1,
            ok: true,
            event_id: Some(hex16(&[1u8; 16])),
            version: Some(hex16(&[2u8; 16])),
            store_id: Some(hex16(&[3u8; 16])),
            segment_id: Some(hex16(&[4u8; 16])),
            acknowledgement: None,
            committed: Some(true),
            key: Some("k".into()),
            ..empty_resp(1)
        };
        let err = write_receipt_from_resp("k", &resp, DurabilityMode::Durable).unwrap_err();
        assert_eq!(err.code(), crate::ErrorCode::ProtocolViolation);
    }

    #[test]
    fn write_receipt_requires_event_id() {
        let resp = RpcResponse {
            id: 1,
            ok: true,
            event_id: None,
            version: Some(hex16(&[2u8; 16])),
            store_id: Some(hex16(&[3u8; 16])),
            segment_id: Some(hex16(&[4u8; 16])),
            acknowledgement: Some("durable".into()),
            committed: Some(true),
            key: Some("k".into()),
            ..empty_resp(1)
        };
        let err = write_receipt_from_resp("k", &resp, DurabilityMode::Durable).unwrap_err();
        assert_eq!(err.code(), crate::ErrorCode::ProtocolViolation);
    }

    #[test]
    fn connect_options_builder() {
        let o = ConnectOptions::new()
            .auth_token("secret")
            .connect_timeout(Duration::from_millis(100))
            .request_timeout(Duration::from_secs(2))
            .max_connect_attempts(5)
            .retry_backoff(Duration::from_millis(10));
        assert_eq!(o.auth_token.as_deref(), Some("secret"));
        assert_eq!(o.max_connect_attempts, 5);
        assert_eq!(o.connect_timeout, Duration::from_millis(100));
    }
}
