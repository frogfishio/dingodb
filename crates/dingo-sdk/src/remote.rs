//! Line-delimited JSON protocol for `dingo://` remote access (Stages 7 + 8d).
//!
//! Server and client share the same request/response shapes. Transport is TCP;
//! each message is one UTF-8 JSON object terminated by `\n`.
//!
//! Supported ops include put/get/delete/scan, **history**, secondary index
//! list/create/drop/rebuild, **get_payload** (chunk completeness), **find**
//! (server-side filter with index acceleration), and **directory** (Stage 8d
//! partition route snapshot for client caches). Connection-only concerns (auth
//! token, deadlines, connect retries) live on [`ConnectOptions`] /
//! [`ServeOptions`] — not on the collection API (DX_SPEC §4.2).

use crate::collection::find_on_store;
use crate::directory_cache::{AssignmentWire, DirectorySnapshot};
use crate::error::Error;
use crate::filter::{Filter, QueryBudget, QueryOptions, SortOrder};
use crate::history::{KeyHistory, Version};
use crate::indexes::{create_index_on_store, mark_indexes_stale, IndexInfo};
use crate::receipt::{DeleteReceipt, PutOptions, WriteReceipt};
use crate::subject::{
    collection_prefix, decode_subject, encode_subject, validate_collection_name, validate_key,
};
use crate::value::{decode_bytes, decode_json, encode_bytes, encode_json};
use dingo_cluster::{DEFAULT_VIRTUAL_PARTITIONS, HASH_PROFILE_BLAKE3_MOD};
use dingo_store::{
    ByteRange, DurabilityMode, IndexState, LogicalExtent, PayloadResult, Store,
    WriteReceipt as StoreWriteReceipt,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::Path;
use std::str;
use std::sync::atomic::{AtomicU64, Ordering};
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
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            auth_token: None,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_connect_attempts: 3,
            retry_backoff: Duration::from_millis(50),
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
}

/// Server options for `dingo serve` / [`serve_store_with`].
#[derive(Debug, Clone, Default)]
pub struct ServeOptions {
    /// When set, every RPC must carry a matching `token` field.
    pub auth_token: Option<String>,
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
        })
    }
}

/// Client connection to a `dingo serve` process.
pub struct RemoteClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    next_id: AtomicU64,
    endpoint: String,
    options: ConnectOptions,
    /// Cached store id from the first `store_info` after connect.
    store_id: [u8; 16],
}

impl RemoteClient {
    /// Connect to `host:port` with default [`ConnectOptions`].
    pub fn connect(addr: &str, endpoint: String) -> Result<Self, Error> {
        Self::connect_with(addr, endpoint, ConnectOptions::default())
    }

    /// Connect with explicit auth / deadline / retry options.
    pub fn connect_with(
        addr: &str,
        endpoint: String,
        options: ConnectOptions,
    ) -> Result<Self, Error> {
        let stream = tcp_connect_with_retry(addr, &options)?;
        stream
            .set_read_timeout(Some(options.request_timeout))
            .map_err(Error::from_io)?;
        stream
            .set_write_timeout(Some(options.request_timeout))
            .map_err(Error::from_io)?;
        let reader = BufReader::new(stream.try_clone().map_err(Error::from_io)?);
        let mut client = Self {
            stream,
            reader,
            next_id: AtomicU64::new(1),
            endpoint,
            options,
            store_id: [0u8; 16],
        };
        // Immediate store_info validates auth token, proves protocol, caches store id.
        let (_path, sid_hex, _n) = client.store_info()?;
        client.store_id = parse_hex16(Some(&sid_hex)).unwrap_or([0u8; 16]);
        Ok(client)
    }

    /// Endpoint string used for errors and display.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Connection options used for this client.
    pub fn options(&self) -> &ConnectOptions {
        &self.options
    }

    /// Store identifier reported by the server at connect time.
    pub fn store_id(&self) -> [u8; 16] {
        self.store_id
    }

    fn call(&mut self, mut req: RpcRequest) -> Result<RpcResponse, Error> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        req.id = id;
        if req.token.is_none() {
            req.token = self.options.auth_token.clone();
        }
        let mut line = serde_json::to_string(&req).map_err(|e| Error::Internal(e.to_string()))?;
        line.push('\n');
        self.stream
            .write_all(line.as_bytes())
            .map_err(Error::from_io)?;
        self.stream.flush().map_err(Error::from_io)?;

        let mut resp_line = String::new();
        let n = self
            .reader
            .read_line(&mut resp_line)
            .map_err(Error::from_io)?;
        if n == 0 {
            return Err(Error::Internal(format!(
                "remote closed connection: {}",
                self.endpoint
            )));
        }
        let resp: RpcResponse = serde_json::from_str(resp_line.trim()).map_err(|e| {
            Error::Internal(format!("invalid rpc response from {}: {e}", self.endpoint))
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
                _ => Error::Remote { code, message },
            });
        }
        Ok(resp)
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

    /// Put JSON under collection/key.
    pub fn put_json(
        &mut self,
        collection: &str,
        key: &str,
        value: &JsonValue,
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "put".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            json: Some(value.clone()),
            durability: Some(durability_name(options.durability).into()),
            ..base_req("put")
        })?;
        Ok(write_receipt_from_resp(key, &resp, options.durability)?)
    }

    /// Get JSON for collection/key.
    pub fn get_json(&mut self, collection: &str, key: &str) -> Result<Option<JsonValue>, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "get".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            ..base_req("get")
        })?;
        if resp.found == Some(false) {
            return Ok(None);
        }
        Ok(resp.value)
    }

    /// Delete collection/key.
    pub fn delete(
        &mut self,
        collection: &str,
        key: &str,
        options: PutOptions,
    ) -> Result<DeleteReceipt, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "delete".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            durability: Some(durability_name(options.durability).into()),
            ..base_req("delete")
        })?;
        Ok(delete_receipt_from_resp(key, &resp, options.durability)?)
    }

    /// Put raw bytes (base64 on the wire).
    pub fn put_bytes(
        &mut self,
        collection: &str,
        key: &str,
        bytes: &[u8],
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "put_bytes".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            bytes_b64: Some(b64_encode(bytes)),
            durability: Some(durability_name(options.durability).into()),
            ..base_req("put_bytes")
        })?;
        Ok(write_receipt_from_resp(key, &resp, options.durability)?)
    }

    /// Get raw bytes.
    pub fn get_bytes(&mut self, collection: &str, key: &str) -> Result<Option<Vec<u8>>, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "get_bytes".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            ..base_req("get_bytes")
        })?;
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

    /// Per-key history (DX_SPEC §10.1).
    pub fn history(&mut self, collection: &str, key: &str) -> Result<KeyHistory, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "history".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            ..base_req("history")
        })?;
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

    /// Completeness-aware payload read (chunked values; FORMAT_SPEC §8).
    pub fn get_payload(
        &mut self,
        collection: &str,
        key: &str,
    ) -> Result<Option<PayloadResult>, Error> {
        validate_collection_name(collection)?;
        validate_key(key)?;
        let resp = self.call(RpcRequest {
            op: "get_payload".into(),
            collection: Some(collection.into()),
            key: Some(key.into()),
            ..base_req("get_payload")
        })?;
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
        resp.directory.ok_or_else(|| {
            Error::Internal("directory response missing directory payload".into())
        })
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
        let max_docs = options
            .budget
            .as_ref()
            .and_then(|b| b.max_docs_scanned);
        let resp = self.call(RpcRequest {
            op: "find".into(),
            collection: Some(collection.into()),
            json: Some(filter.to_json()),
            limit: options.limit,
            force_scan: if options.force_scan {
                Some(true)
            } else {
                None
            },
            order_field,
            order_dir,
            max_docs_scanned: max_docs,
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
    }
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
    fallback: DurabilityMode,
) -> Result<WriteReceipt, Error> {
    Ok(WriteReceipt {
        key: resp.key.clone().unwrap_or_else(|| key.to_string()),
        event_id: parse_hex16(resp.event_id.as_deref())?,
        version: parse_hex16(resp.version.as_deref())?,
        acknowledgement: parse_durability(resp.acknowledgement.as_deref()).unwrap_or(fallback),
        committed: resp.committed.unwrap_or(true),
        store_id: parse_hex16(resp.store_id.as_deref())?,
        segment_id: parse_hex16(resp.segment_id.as_deref())?,
    })
}

fn delete_receipt_from_resp(
    key: &str,
    resp: &RpcResponse,
    fallback: DurabilityMode,
) -> Result<DeleteReceipt, Error> {
    Ok(DeleteReceipt {
        key: resp.key.clone().unwrap_or_else(|| key.to_string()),
        removed: resp.removed.unwrap_or(false),
        event_id: parse_hex16(resp.event_id.as_deref())?,
        version: parse_hex16(resp.version.as_deref())?,
        acknowledgement: parse_durability(resp.acknowledgement.as_deref()).unwrap_or(fallback),
        committed: resp.committed.unwrap_or(true),
        store_id: parse_hex16(resp.store_id.as_deref())?,
        segment_id: parse_hex16(resp.segment_id.as_deref())?,
    })
}

fn parse_hex16(s: Option<&str>) -> Result<[u8; 16], Error> {
    let Some(s) = s else {
        return Ok([0u8; 16]);
    };
    if s.len() != 32 {
        return Err(Error::Internal(format!(
            "expected 32 hex chars for id, got {}",
            s.len()
        )));
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| Error::Internal(format!("invalid hex id: {e}")))?;
        out[i] = byte;
    }
    Ok(out)
}

fn fill_receipt_fields(resp: &mut RpcResponse, receipt: &StoreWriteReceipt) {
    resp.event_id = Some(hex16(&receipt.event_id));
    resp.version = Some(hex16(&receipt.item_id));
    resp.store_id = Some(hex16(&receipt.store_id));
    resp.segment_id = Some(hex16(&receipt.segment_id));
    resp.acknowledgement = Some(durability_name(receipt.durability).into());
    resp.committed = Some(true);
}

/// Serve one store over TCP until the listener ends (Stage 7).
///
/// Handles connections sequentially (single-threaded MVP). No auth required.
pub fn serve_store(store_path: impl AsRef<Path>, bind: &str) -> Result<(), Error> {
    serve_store_with(store_path, bind, ServeOptions::default())
}

/// Serve with explicit auth / server options.
pub fn serve_store_with(
    store_path: impl AsRef<Path>,
    bind: &str,
    options: ServeOptions,
) -> Result<(), Error> {
    let path = store_path.as_ref().to_path_buf();
    let listener = TcpListener::bind(bind).map_err(Error::from_io)?;
    for conn in listener.incoming() {
        let stream = conn.map_err(Error::from_io)?;
        let mut store = Store::open(&path)?;
        if let Err(e) = handle_connection_with(&mut store, stream, options.clone()) {
            eprintln!("dingo serve connection error: {e}");
        }
    }
    Ok(())
}

/// Serve a single already-open TCP client (used by tests and `serve_store`).
pub fn handle_connection(store: &mut Store, stream: TcpStream) -> Result<(), Error> {
    handle_connection_with(store, stream, ServeOptions::default())
}

/// Handle one client with server options (auth token).
pub fn handle_connection_with(
    store: &mut Store,
    stream: TcpStream,
    options: ServeOptions,
) -> Result<(), Error> {
    stream
        .set_read_timeout(Some(Duration::from_secs(120)))
        .map_err(Error::from_io)?;
    let mut reader = BufReader::new(stream.try_clone().map_err(Error::from_io)?);
    let mut writer = stream;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).map_err(Error::from_io)?;
        if n == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let req: RpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                write_resp(
                    &mut writer,
                    RpcResponse {
                        id: 0,
                        ok: false,
                        error: Some(format!("bad request: {e}")),
                        code: Some("validation_failed".into()),
                        ..empty_resp(0)
                    },
                )?;
                continue;
            }
        };
        if let Some(required) = options.auth_token.as_deref() {
            let presented = req.token.as_deref().unwrap_or("");
            if presented != required {
                write_resp(
                    &mut writer,
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
        let resp = match dispatch(store, &req) {
            Ok(r) => r,
            Err(e) => RpcResponse {
                id: req.id,
                ok: false,
                error: Some(e.to_string()),
                code: Some(e.code().as_str().into()),
                ..empty_resp(req.id)
            },
        };
        write_resp(&mut writer, resp)?;
    }
    Ok(())
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

fn write_resp(w: &mut TcpStream, resp: RpcResponse) -> Result<(), Error> {
    let mut line = serde_json::to_string(&resp).map_err(|e| Error::Internal(e.to_string()))?;
    line.push('\n');
    w.write_all(line.as_bytes()).map_err(Error::from_io)?;
    w.flush().map_err(Error::from_io)?;
    Ok(())
}

fn dispatch(store: &mut Store, req: &RpcRequest) -> Result<RpcResponse, Error> {
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
            let mode =
                parse_durability(req.durability.as_deref()).unwrap_or(DurabilityMode::Durable);
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            let body = encode_json(json)?;
            let receipt = store.put(subject_str, &body, mode)?;
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
            let removed = store.get(subject_str)?.is_some();
            let receipt = store.delete(subject_str, mode)?;
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
            let mode =
                parse_durability(req.durability.as_deref()).unwrap_or(DurabilityMode::Durable);
            let subject = encode_subject(coll, key)?;
            let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
            let body = encode_bytes(&bytes);
            let receipt = store.put(subject_str, &body, mode)?;
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
                let value = match decode_json(&body) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
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
            let budget = req.max_docs_scanned.map(|n| QueryBudget {
                max_docs_scanned: Some(n),
            });
            let options = QueryOptions {
                limit: req.limit,
                order_by,
                budget,
                force_scan: req.force_scan.unwrap_or(false),
                allow_partial_coverage: false,
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
            vec![
                "a:1".to_string(),
                "b:2".to_string(),
                "c:3".to_string()
            ]
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
        assert_eq!(parse_hex16(None).unwrap(), [0u8; 16]);
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
