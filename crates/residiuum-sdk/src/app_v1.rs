//! APP-0 public Rust application surface (`dingo-rust-app-v1`).
//!
//! These types freeze the **names and fields** from
//! `doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md` §5 / §10.
//! Method bodies that require storage/wire activation land in APP-1…APP-8;
//! this module must compile so implementers share one contract.
//!
//! Normative companions: `spec/app/v1/`, `spec/heap/rpc-v1/collection_create.*`,
//! `spec/heap/rpc-v1/dql_query.*`.

use crate::error::Error;
use crate::receipt::{DeleteReceipt, PutOptions, WriteReceipt};
use residiuum_heap::{CollectionId, HeapId};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::SystemTime;

/// Profile label for the public Rust application façade.
pub const RUST_APP_PROFILE: &str = "dingo-rust-app-v1";

/// RQL Application Core source profile (serialized value is frozen Class C).
pub const RQL_APP_CORE_PROFILE: &str = "dql-app-core-v1";

/// Canonical logical plan profile (serialized value is frozen Class C).
pub const RQL_PLAN_PROFILE: &str = "dql-plan-v1";

/// Authenticated continuation profile.
pub const CURSOR_PROFILE: &str = "dingo-cursor-v1";

/// Shared predicate profile.
pub const PREDICATE_PROFILE: &str = "dingo-predicate-v1";

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

/// Opaque authenticated continuation (`dingo-cursor-v1`).
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

/// Heap-bound application client (façade).
///
/// Construction and method wiring: APP-1…APP-3. Identity fields compile now.
#[derive(Debug)]
pub struct HeapClient {
    heap_id: HeapId,
}

impl HeapClient {
    /// Contract fixture constructor (does not open storage).
    pub fn from_id_for_contract(heap_id: HeapId) -> Self {
        Self { heap_id }
    }

    /// Bound Heap id.
    pub fn id(&self) -> HeapId {
        self.heap_id
    }

    /// Create a collection (APP-1).
    ///
    /// Prefer [`crate::Heap::create_collection`] on an embedded heap until this
    /// façade holds a backend (APP-2). Contract-only clients have no storage.
    pub fn create_collection(
        &mut self,
        _name: &str,
    ) -> Result<CreateCollectionResult, Error> {
        Err(Error::Internal(
            "HeapClient façade backend not bound; use Heap::create_collection (APP-1 embedded) or APP-2 From<Heap>".into(),
        ))
    }

    /// Create with options (APP-1).
    pub fn create_collection_with(
        &mut self,
        _name: &str,
        _options: CreateCollectionOptions,
    ) -> Result<CreateCollectionResult, Error> {
        Err(Error::Internal(
            "HeapClient façade backend not bound; use Heap::create_collection_with (APP-1 embedded)".into(),
        ))
    }

    /// Open by canonical name (APP-2 façade; embedded path is [`crate::Heap::collection`]).
    pub fn open_collection(&mut self, _name: &str) -> Result<CollectionClient, Error> {
        Err(Error::Internal(
            "HeapClient façade backend not bound; use Heap::collection".into(),
        ))
    }

    /// List collections (APP-2 façade; embedded path is [`crate::Heap::list_collections`]).
    pub fn list_collections(&mut self) -> Result<Vec<CollectionInfo>, Error> {
        Err(Error::Internal(
            "HeapClient façade backend not bound; use Heap::list_collections".into(),
        ))
    }
}

/// Collection-bound application client (façade).
#[derive(Debug)]
pub struct CollectionClient {
    heap_id: HeapId,
    collection_id: CollectionId,
    name: String,
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
        }
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

    /// JSON put (APP-3).
    pub fn put<T: Serialize>(
        &mut self,
        _key: &str,
        _value: &T,
    ) -> Result<WriteReceipt, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: put activates in APP-3".into(),
        ))
    }

    /// JSON put with options (APP-3).
    pub fn put_with<T: Serialize>(
        &mut self,
        _key: &str,
        _value: &T,
        _options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: put_with activates in APP-3".into(),
        ))
    }

    /// Bytes put (APP-3).
    pub fn put_bytes(&mut self, _key: &str, _value: &[u8]) -> Result<WriteReceipt, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: put_bytes activates in APP-3".into(),
        ))
    }

    /// JSON get (APP-3).
    pub fn get(&mut self, _key: &str) -> Result<Option<serde_json::Value>, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: get activates in APP-3".into(),
        ))
    }

    /// Bytes get (APP-3).
    pub fn get_bytes(&mut self, _key: &str) -> Result<Option<Vec<u8>>, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: get_bytes activates in APP-3".into(),
        ))
    }

    /// Delete (APP-3).
    pub fn delete(&mut self, _key: &str) -> Result<DeleteReceipt, Error> {
        Err(Error::Internal(
            "APP-0 contract surface: delete activates in APP-3".into(),
        ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_are_stable() {
        assert_eq!(RUST_APP_PROFILE, "dingo-rust-app-v1");
        assert_eq!(RQL_APP_CORE_PROFILE, "dql-app-core-v1");
        assert_eq!(RQL_PLAN_PROFILE, "dql-plan-v1");
        assert_eq!(CURSOR_PROFILE, "dingo-cursor-v1");
        assert_eq!(PREDICATE_PROFILE, "dingo-predicate-v1");
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