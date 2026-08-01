//! APP-0 public Rust application surface (`residiuum-rust-app-v1`).
//!
//! These types freeze the **names and fields** from
//! `doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md` §5 / §10.
//! Method bodies that require storage/wire activation land in APP-1…APP-8;
//! this module must compile so implementers share one contract.
//!
//! Normative companions: `spec/app/v1/`, `spec/heap/rpc-v1/collection_create.*`,
//! `spec/heap/rpc-v1/rql_query.*`.

use crate::error::Error;
use crate::heap::{Heap, HeapCollection};
use crate::receipt::{DeleteReceipt, PutOptions, WriteReceipt};
use crate::remote_heap::RemoteHeap;
use residiuum_heap::{CollectionId, HeapId};
use residiuum_store::DurabilityMode;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Profile label for the public Rust application façade.
pub const RUST_APP_PROFILE: &str = "residiuum-rust-app-v1";

/// RQL Application Core source profile (serialized value is frozen Class C).
pub const RQL_APP_CORE_PROFILE: &str = "rql-app-core-v1";

/// Canonical logical plan profile (serialized value is frozen Class C).
pub const RQL_PLAN_PROFILE: &str = "rql-plan-v1";

/// Authenticated continuation profile.
pub const CURSOR_PROFILE: &str = "residiuum-cursor-v1";

/// Shared predicate profile.
pub const PREDICATE_PROFILE: &str = "residiuum-predicate-v1";

/// Catalog listing entry for one collection in a Heap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionInfo {
    /// Owning Heap.
    pub heap_id: HeapId,
    /// Immutable collection identity.
    pub collection_id: CollectionId,
    /// Canonical display name.
    pub name: String,
    /// Descriptor content hash (32 bytes).
    pub descriptor_hash: [u8; 32],
}

/// Options for collection create (CORE plan §5 / §6).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateCollectionOptions {
    /// Stable client operation id (16 bytes). Generated when `None`.
    pub operation_id: Option<[u8; 16]>,
}

/// Successful create returns a bound handle plus receipt.
#[derive(Debug)]
pub struct CreateCollectionResult {
    /// Open handle for the new collection (identity only until APP-1 wires storage).
    pub collection: CollectionClient,
    /// Typed create receipt.
    pub receipt: CollectionCreateReceipt,
}

/// Receipt for `create_collection` (wire op 106).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionCreateReceipt {
    /// Stable receipt id (16 bytes).
    pub receipt_id: [u8; 16],
    /// Always create-collection for this receipt kind.
    pub operation: AdminOperation,
    /// Owning Heap.
    pub heap_id: HeapId,
    /// New collection id.
    pub collection_id: CollectionId,
    /// Descriptor hash at create.
    pub descriptor_hash: [u8; 32],
    /// Wall clock at create (source of truth for replay equality).
    pub created_at: SystemTime,
}

/// Admin operation discriminant on receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminOperation {
    /// Collection create.
    CreateCollection,
}

/// Opaque query id (16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QueryId(pub [u8; 16]);

/// Named RQL / builder parameters.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Parameters {
    /// Name → JSON value.
    pub values: BTreeMap<String, serde_json::Value>,
}

/// Options for `rql` / `explain_rql` / page execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryRunOptions {
    /// Page size (1..=4096). Default 64 when `None`.
    pub page_size: Option<u32>,
    /// Coverage policy.
    pub coverage: CoveragePolicy,
    /// Consistency mode.
    pub consistency: ConsistencyMode,
    /// Optional budgets.
    pub budget: Option<QueryBudget>,
    /// When true, explain only (op 118 `explain: true`).
    pub explain: bool,
}

impl Default for QueryRunOptions {
    fn default() -> Self {
        Self {
            page_size: None,
            coverage: CoveragePolicy::Complete,
            consistency: ConsistencyMode::Available,
            budget: None,
            explain: false,
        }
    }
}

/// Coverage policy for Application Core queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoveragePolicy {
    /// Complete coverage required (default).
    Complete,
    /// Incomplete coverage allowed with evidence.
    IncompleteAllowed,
}

/// Consistency mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyMode {
    /// Available reads (default).
    Available,
    /// Current / linearizable when supported.
    Current,
}

/// Scan / materialization budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryBudget {
    /// Max documents examined.
    pub max_documents: Option<u64>,
    /// Max bytes read.
    pub max_bytes: Option<u64>,
    /// Max result payload bytes.
    pub max_result_bytes: Option<u64>,
}

impl QueryBudget {
    /// Document-count budget only.
    pub fn documents(max_documents: u64) -> Self {
        Self {
            max_documents: Some(max_documents),
            max_bytes: None,
            max_result_bytes: None,
        }
    }
}

/// One page of query results (CORE plan §10).
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPage {
    /// Query instance id.
    pub query_id: QueryId,
    /// Canonical plan hash.
    pub plan_hash: [u8; 32],
    /// Heap binding.
    pub heap_id: HeapId,
    /// Collection binding.
    pub collection_id: CollectionId,
    /// Rows for this page.
    pub rows: Vec<QueryRow>,
    /// Authenticated continuation when not exhausted.
    pub next: Option<Continuation>,
    /// No further logical rows.
    pub exhausted: bool,
    /// Coverage evidence.
    pub coverage: CoverageEvidence,
    /// Consistency evidence.
    pub consistency: ConsistencyEvidence,
    /// Remaining total limit across pages.
    pub remaining_limit: Option<u64>,
    /// Explicit hole evidence.
    pub known_holes: Vec<HoleEvidence>,
}

/// One projected row.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRow {
    /// Immutable document key.
    pub key: String,
    /// Projected JSON value.
    pub value: serde_json::Value,
}

/// Opaque authenticated continuation (`residiuum-cursor-v1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Continuation {
    /// Opaque token bytes (never log).
    pub token: Vec<u8>,
}

/// Coverage evidence on a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageEvidence {
    /// Whether coverage is complete for the requested result.
    pub complete: bool,
    /// Policy in force.
    pub mode: CoveragePolicy,
}

/// Consistency evidence on a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyEvidence {
    /// Mode in force.
    pub mode: ConsistencyMode,
}

/// Hole / damage evidence (never silent null).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleEvidence {
    /// Stable machine code.
    pub code: String,
    /// Optional subject key.
    pub key: Option<String>,
}

/// Structured explain result (op 118 with `explain: true`).
#[derive(Debug, Clone, PartialEq)]
pub struct QueryExplanation {
    /// Plan profile label.
    pub plan_profile: String,
    /// Canonical plan hash.
    pub plan_hash: [u8; 32],
    /// Human/debug tree (shape only at APP-0).
    pub tree: serde_json::Value,
}

/// Sealed storage backend for [`HeapClient`] (APB-1 G1 / G1b).
///
/// Unbound remains the contract-only fixture.
enum HeapBackend {
    /// No storage — `from_id_for_contract` / APP-0 fixtures.
    Unbound,
    /// Embedded capability-gated [`Heap`].
    Embedded(Heap),
    /// Qualified remote session ([`RemoteHeap`]); shared with collection handles.
    Remote(Arc<Mutex<RemoteHeap>>),
}

/// Heap-bound application client (façade).
///
/// Bind storage with [`From<Heap>`] or [`From<RemoteHeap>`] (APB-1 G1/G1b).
/// Contract fixtures use [`HeapClient::from_id_for_contract`] (unbound, fail-closed).
pub struct HeapClient {
    heap_id: HeapId,
    backend: HeapBackend,
}

impl fmt::Debug for HeapClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let backend = match &self.backend {
            HeapBackend::Unbound => "unbound",
            HeapBackend::Embedded(_) => "embedded",
            HeapBackend::Remote(_) => "remote",
        };
        f.debug_struct("HeapClient")
            .field("heap_id", &self.heap_id)
            .field("backend", &backend)
            .finish()
    }
}

impl From<Heap> for HeapClient {
    fn from(heap: Heap) -> Self {
        Self {
            heap_id: heap.id(),
            backend: HeapBackend::Embedded(heap),
        }
    }
}

impl From<RemoteHeap> for HeapClient {
    fn from(remote: RemoteHeap) -> Self {
        Self {
            heap_id: remote.id(),
            backend: HeapBackend::Remote(Arc::new(Mutex::new(remote))),
        }
    }
}

impl HeapClient {
    /// Contract fixture constructor (does not open storage).
    pub fn from_id_for_contract(heap_id: HeapId) -> Self {
        Self {
            heap_id,
            backend: HeapBackend::Unbound,
        }
    }

    /// Bound Heap id.
    pub fn id(&self) -> HeapId {
        self.heap_id
    }

    /// Whether this client has a storage backend (not a contract fixture).
    pub fn is_bound(&self) -> bool {
        !matches!(self.backend, HeapBackend::Unbound)
    }

    /// Create a collection (APP-1 / APB-1).
    pub fn create_collection(
        &mut self,
        name: &str,
    ) -> Result<CreateCollectionResult, Error> {
        self.create_collection_with(name, CreateCollectionOptions::default())
    }

    /// Create with options (APP-1 / APB-1).
    pub fn create_collection_with(
        &mut self,
        name: &str,
        options: CreateCollectionOptions,
    ) -> Result<CreateCollectionResult, Error> {
        match &self.backend {
            HeapBackend::Unbound => Err(Error::Internal(
                "HeapClient unbound; construct with From<Heap|RemoteHeap> (APB-1)".into(),
            )),
            HeapBackend::Embedded(heap) => {
                let created = heap.create_collection_with(name, options.operation_id)?;
                Ok(create_result_from_embedded(created))
            }
            HeapBackend::Remote(remote) => {
                let mut guard = lock_remote(remote)?;
                let created = guard.create_collection(name, options.operation_id)?;
                create_result_from_remote(self.heap_id, Arc::clone(remote), created)
            }
        }
    }

    /// Open by canonical name (embedded: [`Heap::collection`]; remote: op 105).
    pub fn open_collection(&mut self, name: &str) -> Result<CollectionClient, Error> {
        match &self.backend {
            HeapBackend::Unbound => Err(Error::Internal(
                "HeapClient unbound; construct with From<Heap|RemoteHeap> (APB-1)".into(),
            )),
            HeapBackend::Embedded(heap) => {
                let hc = heap.collection(name)?;
                Ok(CollectionClient::from_embedded(hc))
            }
            HeapBackend::Remote(remote) => {
                let mut guard = lock_remote(remote)?;
                let cid_s = guard.collection_open(name)?;
                let collection_id = CollectionId::from_str(&cid_s)
                    .map_err(|e| Error::ProtocolViolation(format!("collection_id: {e}")))?;
                Ok(CollectionClient::from_remote(
                    self.heap_id,
                    collection_id,
                    name.to_string(),
                    Arc::clone(remote),
                    cid_s,
                ))
            }
        }
    }

    /// List collections (embedded catalog; remote op 110).
    ///
    /// Remote wire omits descriptor hashes today — listed remote rows use
    /// zeroed `descriptor_hash` until the wire grows the field.
    pub fn list_collections(&mut self) -> Result<Vec<CollectionInfo>, Error> {
        match &self.backend {
            HeapBackend::Unbound => Err(Error::Internal(
                "HeapClient unbound; construct with From<Heap|RemoteHeap> (APB-1)".into(),
            )),
            HeapBackend::Embedded(heap) => {
                let listed = heap.list_collections()?;
                Ok(listed
                    .into_iter()
                    .map(|e| CollectionInfo {
                        heap_id: e.heap_id,
                        collection_id: e.collection_id,
                        name: e.name,
                        descriptor_hash: e.descriptor_hash,
                    })
                    .collect())
            }
            HeapBackend::Remote(remote) => {
                let mut guard = lock_remote(remote)?;
                let listed = guard.list_collections()?;
                let mut out = Vec::with_capacity(listed.len());
                for (id_s, name) in listed {
                    let collection_id = CollectionId::from_str(&id_s)
                        .map_err(|e| Error::ProtocolViolation(format!("collection_id: {e}")))?;
                    out.push(CollectionInfo {
                        heap_id: self.heap_id,
                        collection_id,
                        name,
                        descriptor_hash: [0u8; 32],
                    });
                }
                Ok(out)
            }
        }
    }
}

fn lock_remote(remote: &Arc<Mutex<RemoteHeap>>) -> Result<std::sync::MutexGuard<'_, RemoteHeap>, Error> {
    remote
        .lock()
        .map_err(|_| Error::Internal("RemoteHeap mutex poisoned".into()))
}

fn create_result_from_embedded(
    created: crate::heap::CreatedCollection,
) -> CreateCollectionResult {
    let heap_id = created.collection.heap_id();
    let collection_id = created.collection.id();
    let created_at = UNIX_EPOCH + Duration::from_secs(created.created_at_unix_s);
    CreateCollectionResult {
        collection: CollectionClient::from_embedded(created.collection),
        receipt: CollectionCreateReceipt {
            receipt_id: created.receipt_id,
            operation: AdminOperation::CreateCollection,
            heap_id,
            collection_id,
            descriptor_hash: created.descriptor_hash,
            created_at,
        },
    }
}

fn create_result_from_remote(
    heap_id: HeapId,
    remote: Arc<Mutex<RemoteHeap>>,
    created: crate::remote_heap::RemoteCreatedCollection,
) -> Result<CreateCollectionResult, Error> {
    let collection_id = CollectionId::from_str(&created.collection_id)
        .map_err(|e| Error::ProtocolViolation(format!("collection_id: {e}")))?;
    let descriptor_hash = parse_hex32(&created.descriptor_hash)?;
    let receipt_id = if created.receipt_id.is_empty() {
        [0u8; 16]
    } else {
        parse_hex16(&created.receipt_id)?
    };
    Ok(CreateCollectionResult {
        collection: CollectionClient::from_remote(
            heap_id,
            collection_id,
            created.canonical_name,
            remote,
            created.collection_id,
        ),
        receipt: CollectionCreateReceipt {
            receipt_id,
            operation: AdminOperation::CreateCollection,
            heap_id,
            collection_id,
            descriptor_hash,
            created_at: SystemTime::now(),
        },
    })
}

fn parse_hex16(s: &str) -> Result<[u8; 16], Error> {
    let clean: String = s.chars().filter(|c| *c != '-').collect();
    if clean.len() != 32 {
        return Err(Error::ProtocolViolation(format!(
            "expected 16-byte hex, got len {}",
            clean.len()
        )));
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16)
            .map_err(|e| Error::ProtocolViolation(format!("hex: {e}")))?;
    }
    Ok(out)
}

fn parse_hex32(s: &str) -> Result<[u8; 32], Error> {
    let clean: String = s.chars().filter(|c| *c != '-').collect();
    if clean.len() != 64 {
        return Err(Error::ProtocolViolation(format!(
            "expected 32-byte hex, got len {}",
            clean.len()
        )));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&clean[i * 2..i * 2 + 2], 16)
            .map_err(|e| Error::ProtocolViolation(format!("hex: {e}")))?;
    }
    Ok(out)
}

/// Sealed collection backend (embedded or remote handle when bound).
enum CollectionBackend {
    /// Identity-only / contract fixture.
    Unbound,
    /// Embedded [`HeapCollection`] for data-plane methods.
    Embedded(HeapCollection),
    /// Remote collection: wire collection UUID + shared session.
    Remote {
        remote: Arc<Mutex<RemoteHeap>>,
        wire_collection_id: String,
    },
}

/// Collection-bound application client (façade).
pub struct CollectionClient {
    heap_id: HeapId,
    collection_id: CollectionId,
    name: String,
    backend: CollectionBackend,
}

impl fmt::Debug for CollectionClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let backend = match &self.backend {
            CollectionBackend::Unbound => "unbound",
            CollectionBackend::Embedded(_) => "embedded",
            CollectionBackend::Remote { .. } => "remote",
        };
        f.debug_struct("CollectionClient")
            .field("heap_id", &self.heap_id)
            .field("collection_id", &self.collection_id)
            .field("name", &self.name)
            .field("backend", &backend)
            .finish()
    }
}

impl CollectionClient {
    /// Contract fixture constructor (does not open storage).
    pub fn from_parts_for_contract(
        heap_id: HeapId,
        collection_id: CollectionId,
        name: impl Into<String>,
    ) -> Self {
        Self {
            heap_id,
            collection_id,
            name: name.into(),
            backend: CollectionBackend::Unbound,
        }
    }

    fn from_embedded(hc: HeapCollection) -> Self {
        Self {
            heap_id: hc.heap_id(),
            collection_id: hc.id(),
            name: hc.name().to_string(),
            backend: CollectionBackend::Embedded(hc),
        }
    }

    fn from_remote(
        heap_id: HeapId,
        collection_id: CollectionId,
        name: String,
        remote: Arc<Mutex<RemoteHeap>>,
        wire_collection_id: String,
    ) -> Self {
        Self {
            heap_id,
            collection_id,
            name,
            backend: CollectionBackend::Remote {
                remote,
                wire_collection_id,
            },
        }
    }

    /// Whether this client holds an embedded or remote collection handle.
    pub fn is_bound(&self) -> bool {
        !matches!(self.backend, CollectionBackend::Unbound)
    }

    /// Owning Heap.
    pub fn heap_id(&self) -> HeapId {
        self.heap_id
    }

    /// Immutable collection id.
    pub fn id(&self) -> CollectionId {
        self.collection_id
    }

    /// Canonical name (display).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// JSON put (APP-3 / APB-2 path; embedded + remote).
    pub fn put<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<WriteReceipt, Error> {
        match &self.backend {
            CollectionBackend::Unbound => Err(Error::Internal(
                "CollectionClient unbound; open via HeapClient::open_collection / create (APB-1)".into(),
            )),
            CollectionBackend::Embedded(hc) => hc.put(key, value),
            CollectionBackend::Remote {
                remote,
                wire_collection_id,
            } => {
                let json = serde_json::to_value(value)
                    .map_err(|e| Error::ValidationMsg(format!("serialize: {e}")))?;
                let mut guard = lock_remote(remote)?;
                let (event_id_s, version_s) =
                    guard.put_json(wire_collection_id, key, &json)?;
                Ok(write_receipt_from_remote_ids(key, &event_id_s, &version_s)?)
            }
        }
    }

    /// JSON put with options (APP-3). Remote ignores durability options today
    /// (server default); embedded honors [`PutOptions`].
    pub fn put_with<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        match &self.backend {
            CollectionBackend::Unbound => Err(Error::Internal(
                "CollectionClient unbound; open via HeapClient (APB-1)".into(),
            )),
            CollectionBackend::Embedded(hc) => hc.put_with(key, value, options),
            CollectionBackend::Remote { .. } => {
                // Wire put path has no durability field on this surface yet.
                let _ = options;
                self.put(key, value)
            }
        }
    }

    /// Bytes put (APP-3).
    pub fn put_bytes(&mut self, key: &str, value: &[u8]) -> Result<WriteReceipt, Error> {
        match &self.backend {
            CollectionBackend::Unbound => Err(Error::Internal(
                "CollectionClient unbound; open via HeapClient (APB-1)".into(),
            )),
            CollectionBackend::Embedded(hc) => hc.put_bytes(key, value),
            CollectionBackend::Remote {
                remote,
                wire_collection_id,
            } => {
                let mut guard = lock_remote(remote)?;
                let (event_id_s, version_s) =
                    guard.put_bytes(wire_collection_id, key, value)?;
                Ok(write_receipt_from_remote_ids(key, &event_id_s, &version_s)?)
            }
        }
    }

    /// JSON get (APP-3).
    pub fn get(&mut self, key: &str) -> Result<Option<serde_json::Value>, Error> {
        match &self.backend {
            CollectionBackend::Unbound => Err(Error::Internal(
                "CollectionClient unbound; open via HeapClient (APB-1)".into(),
            )),
            CollectionBackend::Embedded(hc) => hc.get(key),
            CollectionBackend::Remote {
                remote,
                wire_collection_id,
            } => {
                let mut guard = lock_remote(remote)?;
                guard.get_json(wire_collection_id, key)
            }
        }
    }

    /// Bytes get (APP-3).
    pub fn get_bytes(&mut self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        match &self.backend {
            CollectionBackend::Unbound => Err(Error::Internal(
                "CollectionClient unbound; open via HeapClient (APB-1)".into(),
            )),
            CollectionBackend::Embedded(hc) => hc.get_bytes(key),
            CollectionBackend::Remote {
                remote,
                wire_collection_id,
            } => {
                let mut guard = lock_remote(remote)?;
                guard.get_bytes(wire_collection_id, key)
            }
        }
    }

    /// Delete (APP-3).
    pub fn delete(&mut self, key: &str) -> Result<DeleteReceipt, Error> {
        match &self.backend {
            CollectionBackend::Unbound => Err(Error::Internal(
                "CollectionClient unbound; open via HeapClient (APB-1)".into(),
            )),
            CollectionBackend::Embedded(hc) => hc.delete(key),
            CollectionBackend::Remote {
                remote,
                wire_collection_id,
            } => {
                let mut guard = lock_remote(remote)?;
                let removed = guard.delete(wire_collection_id, key)?;
                // Remote delete returns only removed: bool today — receipt ids zeroed.
                Ok(DeleteReceipt {
                    key: key.to_string(),
                    removed,
                    event_id: [0u8; 16],
                    version: [0u8; 16],
                    acknowledgement: DurabilityMode::Durable,
                    committed: true,
                    store_id: [0u8; 16],
                    segment_id: [0u8; 16],
                })
            }
        }
    }

    /// RQL Application Core execution (APP-5…APP-7).
    pub fn rql(
        &mut self,
        _source: &str,
        _parameters: &Parameters,
        _options: QueryRunOptions,
    ) -> Result<QueryPage, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: rql activates in APP-5…APP-7".into(),
        ))
    }

    /// RQL explain (APP-5…APP-7).
    pub fn explain_rql(
        &mut self,
        _source: &str,
        _parameters: &Parameters,
        _options: QueryRunOptions,
    ) -> Result<QueryExplanation, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: explain_rql activates in APP-5…APP-7".into(),
        ))
    }
}

/// Map remote put event/version hex ids into a façade [`WriteReceipt`].
///
/// Wire put does not yet return store/segment ids; those fields are zeroed.
fn write_receipt_from_remote_ids(
    key: &str,
    event_id_s: &str,
    version_s: &str,
) -> Result<WriteReceipt, Error> {
    Ok(WriteReceipt {
        key: key.to_string(),
        event_id: parse_hex16(event_id_s).unwrap_or([0u8; 16]),
        version: parse_hex16(version_s).unwrap_or([0u8; 16]),
        acknowledgement: DurabilityMode::Durable,
        committed: true,
        store_id: [0u8; 16],
        segment_id: [0u8; 16],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_are_stable() {
        assert_eq!(RUST_APP_PROFILE, "residiuum-rust-app-v1");
        assert_eq!(RQL_APP_CORE_PROFILE, "rql-app-core-v1");
        assert_eq!(RQL_PLAN_PROFILE, "rql-plan-v1");
        assert_eq!(CURSOR_PROFILE, "residiuum-cursor-v1");
        assert_eq!(PREDICATE_PROFILE, "residiuum-predicate-v1");
    }

    fn v4(seed: u8) -> [u8; 16] {
        let mut b = [seed; 16];
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        if b == [0u8; 16] {
            b[0] = 1;
        }
        b
    }

    #[test]
    fn facade_constructors_compile() {
        let hid = HeapId::from_bytes(v4(1)).expect("heap id");
        let cid = CollectionId::from_bytes(v4(2)).expect("collection id");
        let mut heap = HeapClient::from_id_for_contract(hid);
        assert_eq!(heap.id(), hid);
        let col = CollectionClient::from_parts_for_contract(hid, cid, "orders");
        assert_eq!(col.name(), "orders");
        assert_eq!(col.id(), cid);
        let err = heap.create_collection("orders").unwrap_err();
        assert_eq!(err.code().as_str(), "internal");
    }
}