//! Qualified heap request dispatch (`HEAP_SPEC` §33.6 / HP-008).
//!
//! Under `heap-key-v1`, heap identity comes solely from the channel [`HeapCap`].
//! Token/RBAC fields are rejected. The hot path never consults an authority store.

use dingo_heap::{
    active_operation_ids, refresh_capability_or_terminate, HeapCap, Operation, OperationStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Public error code for heap-isolation failures on an established channel.
pub const HEAP_UNAVAILABLE: &str = "heap_unavailable";

/// Qualified common request envelope (`HEAP_SPEC` §33.6).
#[derive(Debug, Clone, Deserialize)]
pub struct HeapRpcRequest {
    /// Protocol major.
    pub v: u16,
    /// Connection-local correlation id.
    pub id: u64,
    /// Mutation operation id (UUID) when required; absent on pure reads.
    #[serde(default)]
    pub operation_id: Option<String>,
    /// Numeric registry op id.
    pub op_id: u16,
    /// Immutable collection id when required.
    #[serde(default)]
    pub collection_id: Option<String>,
    /// Immutable stream id when required.
    #[serde(default)]
    pub stream_id: Option<String>,
    /// Operation args object.
    #[serde(default)]
    pub args: Map<String, Value>,
    /// Legacy token — MUST be absent under heap-key-v1.
    #[serde(default)]
    pub token: Option<String>,
    /// Legacy collection name — not accepted on qualified envelope.
    #[serde(default)]
    pub collection: Option<String>,
}

/// Response envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeapRpcResponse {
    /// Protocol major.
    pub v: u16,
    /// Correlation id.
    pub id: u64,
    /// Success flag.
    pub ok: bool,
    /// Result object when ok.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Public error when not ok.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<HeapRpcError>,
}

/// Public error object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeapRpcError {
    /// Registered snake_case code.
    pub code: String,
    /// Whether the client may retry.
    pub retryable: bool,
}

/// Dispatch outcome for one request.
#[derive(Debug)]
pub enum HeapDispatchResult {
    /// Framed response body.
    Response(HeapRpcResponse),
}

/// Validate and dispatch one qualified request using the session capability.
///
/// Only HP-000 active public process ops (1–3) are executable until later
/// packages land §32.4 schemas for heap data operations.
pub fn dispatch_heap_request(cap: &HeapCap, raw: &[u8]) -> HeapDispatchResult {
    let unavailable = |id: u64| {
        HeapDispatchResult::Response(HeapRpcResponse {
            v: 1,
            id,
            ok: false,
            result: None,
            error: Some(HeapRpcError {
                code: HEAP_UNAVAILABLE.into(),
                retryable: false,
            }),
        })
    };

    let req: HeapRpcRequest = match serde_json::from_slice(raw) {
        Ok(r) => r,
        Err(_) => {
            return HeapDispatchResult::Response(HeapRpcResponse {
                v: 1,
                id: 0,
                ok: false,
                result: None,
                error: Some(HeapRpcError {
                    code: HEAP_UNAVAILABLE.into(),
                    retryable: false,
                }),
            });
        }
    };

    if req.v != 1 {
        return unavailable(req.id);
    }
    // Token / legacy collection fields are forbidden on the qualified path.
    if req.token.is_some() || req.collection.is_some() {
        return unavailable(req.id);
    }
    // Reject unknown envelope keys by re-parsing as a map.
    let map: Map<String, Value> = match serde_json::from_slice(raw) {
        Ok(m) => m,
        Err(_) => return unavailable(req.id),
    };
    const ALLOWED: &[&str] = &[
        "v",
        "id",
        "operation_id",
        "op_id",
        "collection_id",
        "stream_id",
        "args",
    ];
    for k in map.keys() {
        if !ALLOWED.contains(&k.as_str()) {
            return unavailable(req.id);
        }
    }

    if refresh_capability_or_terminate(cap).is_err() {
        return unavailable(req.id);
    }

    if !active_operation_ids().contains(&req.op_id) {
        return unavailable(req.id);
    }
    match Operation::status(req.op_id) {
        Ok(OperationStatus::Active) => {}
        _ => return unavailable(req.id),
    }

    // Public process ops: no subordinate IDs required.
    match req.op_id {
        1 => HeapDispatchResult::Response(HeapRpcResponse {
            v: 1,
            id: req.id,
            ok: true,
            result: Some(serde_json::json!({ "pong": true })),
            error: None,
        }),
        2 => HeapDispatchResult::Response(HeapRpcResponse {
            v: 1,
            id: req.id,
            ok: true,
            result: Some(serde_json::json!({ "live": true })),
            error: None,
        }),
        3 => HeapDispatchResult::Response(HeapRpcResponse {
            v: 1,
            id: req.id,
            ok: true,
            result: Some(serde_json::json!({ "ready": true })),
            error: None,
        }),
        _ => unavailable(req.id),
    }
}

/// True when the request registry lists `op_id` as active (HP-000 set).
pub fn request_registry_allows(op_id: u16) -> bool {
    active_operation_ids().contains(&op_id)
        && matches!(Operation::status(op_id), Ok(OperationStatus::Active))
}
