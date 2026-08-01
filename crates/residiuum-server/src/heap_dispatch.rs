//! Qualified heap request dispatch (`HEAP_SPEC` §33.6 / HP-008 / §32.4 data cut).
//!
//! Under `heap-key-v1`, heap identity comes solely from the channel [`HeapCap`].
//! Token/RBAC fields are rejected. Active ops: process 1–3 plus collection data
//! 105–106 / 110–112 / 114–118 / 120–122 and secondary indexes 130–133.
//!
//! Op **118** `rql_query` is active (APP-7 T6) — Application Core page execution
//! via store `list_keys`+`get`. Package accept remains principal-gated.

use residiuum_client::b64u_decode;
use residiuum_heap::{
    active_operation_ids, refresh_capability_or_terminate, CollectionId, HeapCap, HeapId, Operation,
    OperationStatus, Rights,
};
use residiuum_sdk::{
    execute_rql, explain_rql_source, AppQueryBudget, ConsistencyMode, Continuation, CoveragePolicy,
    DocScan, Error as SdkError, Filter, Parameters, Pred, QueryRunOptions,
};
use residiuum_store::{
    create_collection_idempotent, hex16, rebuild_object_entry_from_chain,
    try_load_collections_catalog, HeapMetaLayout, HeapStore, ObjectKind, StoreError, WriteReceipt,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

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

/// Store-backed context for §32.4 data operations.
pub struct HeapDataCtx<'a> {
    /// Capability-gated heap façade.
    pub store: &'a HeapStore,
    /// Meta layout root (catalogs under the store directory).
    pub layout: &'a HeapMetaLayout,
}

/// Validate and dispatch one qualified request using the session capability.
///
/// Process ops (1–3) need only the capability. Data ops require [`HeapDataCtx`].
pub fn dispatch_heap_request(cap: &HeapCap, raw: &[u8]) -> HeapDispatchResult {
    dispatch_heap_request_with(cap, raw, None)
}

/// Dispatch with optional store context for data plane ops.
pub fn dispatch_heap_request_with(
    cap: &HeapCap,
    raw: &[u8],
    data: Option<HeapDataCtx<'_>>,
) -> HeapDispatchResult {
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
    let ok = |id: u64, result: Value| {
        HeapDispatchResult::Response(HeapRpcResponse {
            v: 1,
            id,
            ok: true,
            result: Some(result),
            error: None,
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

    // Rights gate for data ops.
    // 131–133 require IndexAdmin (bootstrap cert includes bit 3).
    // 106 requires HeapAdmin (CORE plan §6.3).
    let required_rights = match req.op_id {
        1 | 2 | 3 => Rights::EMPTY,
        105 | 110 | 111 | 112 | 114 | 115 | 116 | 117 | 118 | 130 => Rights::READ,
        106 => Rights::HEAP_ADMIN,
        120 | 121 | 122 => Rights::WRITE,
        131 | 132 | 133 => Rights::INDEX_ADMIN,
        _ => return unavailable(req.id),
    };
    if required_rights != Rights::EMPTY && !cap.rights().contains(required_rights) {
        return unavailable(req.id);
    }

    match req.op_id {
        1 => ok(req.id, serde_json::json!({ "pong": true })),
        2 => ok(req.id, serde_json::json!({ "live": true })),
        3 => ok(req.id, serde_json::json!({ "ready": true })),
        105 => match data {
            Some(ctx) => dispatch_collection_open(req.id, &req.args, ctx),
            None => unavailable(req.id),
        },
        106 => match data {
            Some(ctx) => dispatch_collection_create(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        110 => match data {
            Some(ctx) => dispatch_list_collections(req.id, &req.args, ctx),
            None => unavailable(req.id),
        },
        111 => match data {
            Some(ctx) => dispatch_get(req.id, &req, ctx, false),
            None => unavailable(req.id),
        },
        112 => match data {
            Some(ctx) => dispatch_get(req.id, &req, ctx, true),
            None => unavailable(req.id),
        },
        114 => match data {
            Some(ctx) => dispatch_list_keys(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        115 => match data {
            Some(ctx) => dispatch_scan_json(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        116 => match data {
            Some(ctx) => dispatch_find(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        117 => match data {
            Some(ctx) => dispatch_history(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        118 => match data {
            Some(ctx) => dispatch_rql_query(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        120 => match data {
            Some(ctx) => dispatch_put(req.id, &req, ctx, false),
            None => unavailable(req.id),
        },
        121 => match data {
            Some(ctx) => dispatch_put(req.id, &req, ctx, true),
            None => unavailable(req.id),
        },
        122 => match data {
            Some(ctx) => dispatch_delete(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        130 => match data {
            Some(ctx) => dispatch_index_list(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        131 => match data {
            Some(ctx) => dispatch_index_create(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        132 => match data {
            Some(ctx) => dispatch_index_drop(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        133 => match data {
            Some(ctx) => dispatch_index_rebuild(req.id, &req, ctx),
            None => unavailable(req.id),
        },
        _ => unavailable(req.id),
    }
}

fn unavailable_id(id: u64) -> HeapDispatchResult {
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
}

fn ok_id(id: u64, result: Value) -> HeapDispatchResult {
    HeapDispatchResult::Response(HeapRpcResponse {
        v: 1,
        id,
        ok: true,
        result: Some(result),
        error: None,
    })
}

fn err_code(id: u64, code: &str) -> HeapDispatchResult {
    HeapDispatchResult::Response(HeapRpcResponse {
        v: 1,
        id,
        ok: false,
        result: None,
        error: Some(HeapRpcError {
            code: code.into(),
            retryable: false,
        }),
    })
}

fn require_string_arg(args: &Map<String, Value>, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn parse_operation_id_hex(s: &str) -> Option<[u8; 16]> {
    if s.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}

fn hex32(h: &[u8; 32]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}

fn dispatch_collection_create(
    id: u64,
    req: &HeapRpcRequest,
    ctx: HeapDataCtx<'_>,
) -> HeapDispatchResult {
    // Mutation requires envelope operation_id (CORE plan §6.1).
    let op_hex = match req.operation_id.as_deref() {
        Some(s) => s,
        None => return err_code(id, "validation_failed"),
    };
    let operation_id = match parse_operation_id_hex(op_hex) {
        Some(o) => o,
        None => return err_code(id, "validation_failed"),
    };
    let name = match require_string_arg(&req.args, "canonical_name") {
        Some(n) if !n.is_empty() && n.len() <= 256 => n,
        _ => return err_code(id, "validation_failed"),
    };
    if req.args.keys().any(|k| k != "canonical_name") {
        return err_code(id, "validation_failed");
    }
    if req.collection_id.is_some() || req.stream_id.is_some() {
        return err_code(id, "validation_failed");
    }
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    match create_collection_idempotent(ctx.layout, &heap_id, operation_id, &name) {
        Ok(created) => {
            let coll = match CollectionId::from_bytes_unchecked_nonzero(created.object_id) {
                Ok(c) => c,
                Err(_) => return unavailable_id(id),
            };
            let heap = match HeapId::from_bytes_unchecked_nonzero(heap_id) {
                Ok(h) => h,
                Err(_) => return unavailable_id(id),
            };
            let desc = hex32(&created.descriptor_hash);
            ok_id(
                id,
                serde_json::json!({
                    "collection_id": coll.to_string(),
                    "canonical_name": created.name,
                    "descriptor_hash": desc,
                    "receipt": {
                        "receipt_id": hex16(&created.receipt_id),
                        "operation": "create_collection",
                        "heap_id": heap.to_string(),
                        "object_id": coll.to_string(),
                        "descriptor_hash": desc,
                        "created_at": created.created_at,
                    },
                    "replayed": created.replayed,
                }),
            )
        }
        Err(StoreError::ConsistencyViolation(_)) => err_code(id, "consistency_violation"),
        Err(StoreError::HeapAdmit(ref msg))
            if msg.contains("name/alias conflict") || msg.contains("object id already exists") =>
        {
            err_code(id, "already_exists")
        }
        Err(StoreError::HeapAdmit(ref msg)) if msg.contains("empty") || msg.contains("too long") || msg.contains("NUL") => {
            err_code(id, "validation_failed")
        }
        Err(_) => unavailable_id(id),
    }
}

fn parse_collection_id(s: &str) -> Option<CollectionId> {
    s.parse().ok()
}

fn receipt_result(r: &WriteReceipt) -> Value {
    // Public OCC `version` is the establishing event id (APB-2 / DX §6.4).
    // Item lineage remains on history rows as `item_id`, not this field.
    serde_json::json!({
        "event_id": hex16(&r.event_id),
        "version": hex16(&r.event_id),
    })
}

fn dispatch_collection_open(
    id: u64,
    args: &Map<String, Value>,
    ctx: HeapDataCtx<'_>,
) -> HeapDispatchResult {
    let name = match require_string_arg(args, "name") {
        Some(n) if !n.is_empty() && n.len() <= 256 => n,
        _ => return unavailable_id(id),
    };
    // Reject unexpected args keys.
    if args.keys().any(|k| k != "name") {
        return unavailable_id(id);
    }
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    let entries = match try_load_collections_catalog(ctx.layout, &heap_id) {
        Ok(Some(c)) => c,
        Ok(None) => return unavailable_id(id),
        Err(_) => return unavailable_id(id),
    };
    let mut found: Option<([u8; 16], String)> = None;
    for entry in &entries {
        if entry.name == name {
            found = Some((entry.object_id, entry.name.clone()));
            break;
        }
    }
    let Some((oid, tip_name)) = found else {
        return unavailable_id(id);
    };
    let coll = match CollectionId::from_bytes_unchecked_nonzero(oid) {
        Ok(c) => c,
        Err(_) => return unavailable_id(id),
    };
    ok_id(
        id,
        serde_json::json!({
            "collection_id": coll.to_string(),
            "name": tip_name,
        }),
    )
}

fn dispatch_list_collections(
    id: u64,
    args: &Map<String, Value>,
    ctx: HeapDataCtx<'_>,
) -> HeapDispatchResult {
    if !args.is_empty() {
        return unavailable_id(id);
    }
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    let entries = match try_load_collections_catalog(ctx.layout, &heap_id) {
        Ok(Some(c)) => c,
        Ok(None) => Vec::new(),
        Err(_) => return unavailable_id(id),
    };
    let mut collections = Vec::new();
    for entry in entries {
        let Ok(coll) = CollectionId::from_bytes_unchecked_nonzero(entry.object_id) else {
            continue;
        };
        collections.push(serde_json::json!({
            "collection_id": coll.to_string(),
            "name": entry.name,
        }));
    }
    ok_id(id, serde_json::json!({ "collections": collections }))
}

fn limit_arg(args: &Map<String, Value>) -> Option<usize> {
    match args.get("limit") {
        None => Some(64),
        Some(Value::Number(n)) => n.as_u64().map(|u| u as usize).filter(|&u| (1..=4096).contains(&u)),
        _ => None,
    }
}

fn dispatch_list_keys(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let limit = match limit_arg(&req.args) {
        Some(l) => l,
        None => return unavailable_id(id),
    };
    let after = match req.args.get("after_key") {
        None => None,
        Some(Value::String(s)) if s.len() <= 2048 => Some(s.as_str()),
        _ => return unavailable_id(id),
    };
    for k in req.args.keys() {
        if k != "limit" && k != "after_key" {
            return unavailable_id(id);
        }
    }
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }
    match ctx.store.list_collection_keys(
        coll.as_bytes(),
        limit,
        after.map(|s| s.as_bytes()),
    ) {
        Ok(keys) => {
            let keys: Vec<String> = keys
                .into_iter()
                .filter_map(|k| String::from_utf8(k).ok())
                .collect();
            ok_id(id, serde_json::json!({ "keys": keys }))
        }
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_history(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let key = match require_string_arg(&req.args, "key") {
        Some(k) if !k.is_empty() && k.len() <= 2048 => k,
        _ => return unavailable_id(id),
    };
    if req.args.keys().any(|k| k != "key") {
        return unavailable_id(id);
    }
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }
    let hist = match ctx
        .store
        .history_collection(coll.as_bytes(), key.as_bytes())
    {
        Ok(h) => h,
        Err(_) => return unavailable_id(id),
    };
    let mut versions = Vec::new();
    for ev in hist.events {
        let kind = match ev.kind {
            residiuum_store::EventKind::Put => "put",
            residiuum_store::EventKind::Delete => "delete",
        };
        let mut obj = serde_json::Map::new();
        obj.insert("kind".into(), Value::String(kind.into()));
        obj.insert("event_id".into(), Value::String(hex16(&ev.event_id)));
        obj.insert("item_id".into(), Value::String(hex16(&ev.item_id)));
        obj.insert("segment_id".into(), Value::String(hex16(&ev.segment_id)));
        obj.insert(
            "known_gap_before".into(),
            Value::Bool(ev.known_gap_before),
        );
        if ev.kind == residiuum_store::EventKind::Put && ev.body.first() == Some(&0x01) {
            if let Ok(json) = serde_json::from_slice::<Value>(&ev.body[1..]) {
                obj.insert("json".into(), json);
            }
        }
        versions.push(Value::Object(obj));
    }
    ok_id(
        id,
        serde_json::json!({
            "key": key,
            "has_known_holes": hist.has_known_holes,
            "versions": versions,
        }),
    )
}

fn dispatch_find(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let filter_json = match req.args.get("filter") {
        Some(v) if v.is_object() => v,
        _ => return unavailable_id(id),
    };
    let filter = match Filter::from_json(filter_json) {
        Ok(f) => f,
        Err(_) => return unavailable_id(id),
    };
    let limit = match limit_arg(&req.args) {
        Some(l) => l,
        None => return unavailable_id(id), // invalid limit value
    };
    // Only filter + limit allowed in args.
    for k in req.args.keys() {
        if k != "filter" && k != "limit" {
            return unavailable_id(id);
        }
    }
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }

    // Prefer secondary-index candidates for equality / AND-of-equalities.
    let eqs = equality_fields(&filter);
    if !eqs.is_empty() {
        match ctx.store.lookup_index_keys(coll.as_bytes(), &eqs) {
            Ok(Some(keys)) => {
                let mut out = Vec::new();
                for key in keys {
                    if out.len() >= limit {
                        break;
                    }
                    let Ok(key_s) = String::from_utf8(key.clone()) else {
                        continue;
                    };
                    let body = match ctx.store.get_collection(coll.as_bytes(), &key) {
                        Ok(Some(b)) => b,
                        Ok(None) => continue,
                        Err(_) => return unavailable_id(id),
                    };
                    if body.first() != Some(&0x01) {
                        continue;
                    }
                    let Ok(json) = serde_json::from_slice::<Value>(&body[1..]) else {
                        continue;
                    };
                    if !filter.matches(&json) {
                        continue;
                    }
                    out.push(serde_json::json!({ "key": key_s, "json": json }));
                }
                return ok_id(id, serde_json::json!({ "rows": out }));
            }
            Ok(None) => {} // fall through to scan
            Err(_) => return unavailable_id(id),
        }
    }

    // Scan fallback (non-equality filters, or no usable index).
    let scan_cap = limit.saturating_mul(8).clamp(limit, 4096);
    let scanned = match ctx.store.scan_collection(coll.as_bytes(), scan_cap, None) {
        Ok(rows) => rows,
        Err(_) => return unavailable_id(id),
    };
    let mut out = Vec::new();
    for (key, body) in scanned {
        if out.len() >= limit {
            break;
        }
        let Ok(key_s) = String::from_utf8(key) else {
            continue;
        };
        if body.first() != Some(&0x01) {
            continue;
        }
        let Ok(json) = serde_json::from_slice::<Value>(&body[1..]) else {
            continue;
        };
        if !filter.matches(&json) {
            continue;
        }
        out.push(serde_json::json!({ "key": key_s, "json": json }));
    }
    ok_id(id, serde_json::json!({ "rows": out }))
}

/// Shallow equality constraints for index acceleration (Eq + AND of Eq only).
fn equality_fields(filter: &Filter) -> Vec<(String, Value)> {
    match filter {
        Filter::Field {
            path,
            pred: Pred::Eq(v),
        } => vec![(path.clone(), v.clone())],
        Filter::And(parts) => {
            let mut out = Vec::new();
            for p in parts {
                out.extend(equality_fields(p));
            }
            out
        }
        _ => Vec::new(),
    }
}

fn dispatch_scan_json(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let limit = match limit_arg(&req.args) {
        Some(l) => l,
        None => return unavailable_id(id),
    };
    let after = match req.args.get("after_key") {
        None => None,
        Some(Value::String(s)) if s.len() <= 2048 => Some(s.as_str()),
        _ => return unavailable_id(id),
    };
    for k in req.args.keys() {
        if k != "limit" && k != "after_key" {
            return unavailable_id(id);
        }
    }
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }
    match ctx
        .store
        .scan_collection(coll.as_bytes(), limit, after.map(|s| s.as_bytes()))
    {
        Ok(rows) => {
            let mut out = Vec::new();
            for (key, body) in rows {
                let Ok(key_s) = String::from_utf8(key) else {
                    continue;
                };
                // Typed JSON only (tag 0x01).
                if body.first() != Some(&0x01) {
                    continue;
                }
                let Ok(json) = serde_json::from_slice::<Value>(&body[1..]) else {
                    continue;
                };
                out.push(serde_json::json!({ "key": key_s, "json": json }));
            }
            ok_id(id, serde_json::json!({ "rows": out }))
        }
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_get(
    id: u64,
    req: &HeapRpcRequest,
    ctx: HeapDataCtx<'_>,
    bytes_mode: bool,
) -> HeapDispatchResult {
    let key = match require_string_arg(&req.args, "key") {
        Some(k) if !k.is_empty() && k.len() <= 2048 => k,
        _ => return unavailable_id(id),
    };
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    // Verify collection is known for this heap (catalog or chain).
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }
    match ctx.store.get_collection(coll.as_bytes(), key.as_bytes()) {
        Ok(None) => ok_id(id, serde_json::json!({ "found": false })),
        Ok(Some(body)) => {
            if bytes_mode {
                // Typed bytes body: tag 0x02 + payload.
                if body.first() == Some(&0x02) {
                    let b64 = residiuum_client::b64u_encode(&body[1..]);
                    ok_id(
                        id,
                        serde_json::json!({ "found": true, "bytes_b64": b64 }),
                    )
                } else {
                    unavailable_id(id)
                }
            } else {
                // Typed JSON: tag 0x01 + JSON.
                if body.first() == Some(&0x01) {
                    match serde_json::from_slice::<Value>(&body[1..]) {
                        Ok(json) => ok_id(id, serde_json::json!({ "found": true, "json": json })),
                        Err(_) => unavailable_id(id),
                    }
                } else {
                    unavailable_id(id)
                }
            }
        }
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_put(
    id: u64,
    req: &HeapRpcRequest,
    ctx: HeapDataCtx<'_>,
    bytes_mode: bool,
) -> HeapDispatchResult {
    let key = match require_string_arg(&req.args, "key") {
        Some(k) if !k.is_empty() && k.len() <= 2048 => k,
        _ => return unavailable_id(id),
    };
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }
    let body = if bytes_mode {
        let b64 = match require_string_arg(&req.args, "bytes_b64") {
            Some(s) if !s.is_empty() => s,
            _ => return unavailable_id(id),
        };
        let raw = match b64u_decode(&b64) {
            Ok(b) => b,
            Err(_) => return unavailable_id(id),
        };
        let mut body = Vec::with_capacity(1 + raw.len());
        body.push(0x02);
        body.extend_from_slice(&raw);
        body
    } else {
        let json = match req.args.get("json") {
            Some(v) => v,
            None => return unavailable_id(id),
        };
        let mut body = Vec::new();
        body.push(0x01);
        if serde_json::to_writer(&mut body, json).is_err() {
            return unavailable_id(id);
        }
        body
    };
    match ctx.store.put_collection(coll.as_bytes(), key.as_bytes(), &body) {
        Ok(receipt) => ok_id(id, receipt_result(&receipt)),
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_delete(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let key = match require_string_arg(&req.args, "key") {
        Some(k) if !k.is_empty() && k.len() <= 2048 => k,
        _ => return unavailable_id(id),
    };
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return unavailable_id(id);
    }
    let existed = ctx
        .store
        .get_collection(coll.as_bytes(), key.as_bytes())
        .ok()
        .flatten()
        .is_some();
    match ctx.store.delete_collection(coll.as_bytes(), key.as_bytes()) {
        Ok(receipt) => {
            let mut result = receipt_result(&receipt);
            if let Some(obj) = result.as_object_mut() {
                obj.insert("removed".into(), Value::Bool(existed));
            }
            ok_id(id, result)
        }
        Err(_) => unavailable_id(id),
    }
}

fn index_meta_json(idx: &residiuum_store::SecondaryIndex) -> Value {
    serde_json::json!({
        "name": idx.meta.name,
        "fields": idx.meta.fields,
        "state": idx.meta.state.as_str(),
        "entry_count": idx.meta.entry_count,
        "complete_coverage": idx.meta.complete_coverage,
    })
}

fn require_known_collection<'a>(
    req: &'a HeapRpcRequest,
    ctx: &HeapDataCtx<'_>,
) -> Option<CollectionId> {
    let cid_s = req.collection_id.as_deref()?;
    let coll = parse_collection_id(cid_s)?;
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(ctx.layout, &heap_id, ObjectKind::Collection, coll.as_bytes())
        .ok()
        .flatten()
        .is_none()
    {
        return None;
    }
    Some(coll)
}

fn dispatch_index_list(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    if !req.args.is_empty() {
        return unavailable_id(id);
    }
    let coll = match require_known_collection(req, &ctx) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    match ctx.store.list_indexes(coll.as_bytes()) {
        Ok(indexes) => {
            let indexes: Vec<Value> = indexes.iter().map(index_meta_json).collect();
            ok_id(id, serde_json::json!({ "indexes": indexes }))
        }
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_index_create(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let name = match require_string_arg(&req.args, "name") {
        Some(n) if !n.is_empty() && n.len() <= 256 => n,
        _ => return unavailable_id(id),
    };
    let fields: Vec<String> = match req.args.get("fields") {
        Some(Value::Array(arr)) if (1..=16).contains(&arr.len()) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                let Some(s) = v.as_str() else {
                    return unavailable_id(id);
                };
                if s.is_empty() || s.len() > 256 {
                    return unavailable_id(id);
                }
                out.push(s.to_string());
            }
            out
        }
        _ => return unavailable_id(id),
    };
    for k in req.args.keys() {
        if k != "name" && k != "fields" {
            return unavailable_id(id);
        }
    }
    let coll = match require_known_collection(req, &ctx) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let field_refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
    match ctx
        .store
        .create_index(coll.as_bytes(), &name, &field_refs)
    {
        Ok(idx) => ok_id(id, index_meta_json(&idx)),
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_index_drop(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let name = match require_string_arg(&req.args, "name") {
        Some(n) if !n.is_empty() && n.len() <= 256 => n,
        _ => return unavailable_id(id),
    };
    if req.args.keys().any(|k| k != "name") {
        return unavailable_id(id);
    }
    let coll = match require_known_collection(req, &ctx) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    match ctx.store.drop_index(coll.as_bytes(), &name) {
        Ok(()) => ok_id(id, serde_json::json!({ "dropped": true })),
        Err(_) => unavailable_id(id),
    }
}

fn dispatch_index_rebuild(
    id: u64,
    req: &HeapRpcRequest,
    ctx: HeapDataCtx<'_>,
) -> HeapDispatchResult {
    let name = match require_string_arg(&req.args, "name") {
        Some(n) if !n.is_empty() && n.len() <= 256 => n,
        _ => return unavailable_id(id),
    };
    if req.args.keys().any(|k| k != "name") {
        return unavailable_id(id);
    }
    let coll = match require_known_collection(req, &ctx) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    match ctx.store.rebuild_index(coll.as_bytes(), &name) {
        Ok(idx) => ok_id(id, index_meta_json(&idx)),
        Err(_) => unavailable_id(id),
    }
}

/// True when the request registry lists `op_id` as active.
pub fn request_registry_allows(op_id: u16) -> bool {
    active_operation_ids().contains(&op_id)
        && matches!(Operation::status(op_id), Ok(OperationStatus::Active))
}

// --- APP-7 T6: op 118 rql_query -------------------------------------------------

/// Store-backed [`DocScan`] for Application Core execution on the server.
struct HeapStoreDocScan<'a> {
    store: &'a HeapStore,
    collection_id: [u8; 16],
}

impl DocScan for HeapStoreDocScan<'_> {
    fn list_keys(
        &mut self,
        limit: Option<usize>,
        after_key: Option<&str>,
    ) -> Result<Vec<String>, SdkError> {
        let lim = limit.unwrap_or(256).clamp(1, 4096);
        let after = after_key.map(|s| s.as_bytes());
        let keys = self
            .store
            .list_collection_keys(&self.collection_id, lim, after)
            .map_err(|e| SdkError::Internal(format!("list_collection_keys: {e}")))?;
        let mut out = Vec::with_capacity(keys.len());
        for k in keys {
            let s = String::from_utf8(k)
                .map_err(|_| SdkError::Internal("non-UTF-8 collection key".into()))?;
            out.push(s);
        }
        Ok(out)
    }

    fn get_json(&mut self, key: &str) -> Result<Option<Value>, SdkError> {
        let body = self
            .store
            .get_collection(&self.collection_id, key.as_bytes())
            .map_err(|e| SdkError::Internal(format!("get_collection: {e}")))?;
        let Some(body) = body else {
            return Ok(None);
        };
        if body.first() != Some(&0x01) {
            return Ok(None);
        }
        let json = serde_json::from_slice(&body[1..])
            .map_err(|e| SdkError::Internal(format!("json decode: {e}")))?;
        Ok(Some(json))
    }

    fn try_equality_index_keys(
        &mut self,
        equalities: &[(String, Value)],
    ) -> Result<Option<Vec<String>>, SdkError> {
        let found = self
            .store
            .lookup_index_keys(&self.collection_id, equalities)
            .map_err(|e| SdkError::Internal(format!("lookup_index_keys: {e}")))?;
        match found {
            None => Ok(None),
            Some(keys) => {
                let mut out = Vec::with_capacity(keys.len());
                for k in keys {
                    out.push(
                        String::from_utf8(k)
                            .map_err(|_| SdkError::Internal("non-UTF-8 index key".into()))?,
                    );
                }
                Ok(Some(out))
            }
        }
    }
}

fn collection_name_for_id(ctx: &HeapDataCtx<'_>, coll: &[u8; 16]) -> Option<String> {
    let heap_id = *ctx.store.capability().heap_id().as_bytes();
    let entries = try_load_collections_catalog(ctx.layout, &heap_id).ok().flatten()?;
    for entry in entries {
        if &entry.object_id == coll {
            return Some(entry.name);
        }
    }
    None
}

fn dispatch_rql_query(id: u64, req: &HeapRpcRequest, ctx: HeapDataCtx<'_>) -> HeapDispatchResult {
    let cid_s = match req.collection_id.as_deref() {
        Some(s) => s,
        None => return unavailable_id(id),
    };
    let coll = match parse_collection_id(cid_s) {
        Some(c) => c,
        None => return unavailable_id(id),
    };
    let heap_id_bytes = *ctx.store.capability().heap_id().as_bytes();
    if rebuild_object_entry_from_chain(
        ctx.layout,
        &heap_id_bytes,
        ObjectKind::Collection,
        coll.as_bytes(),
    )
    .ok()
    .flatten()
    .is_none()
    {
        return unavailable_id(id);
    }
    let collection_name = match collection_name_for_id(&ctx, coll.as_bytes()) {
        Some(n) => n,
        None => return unavailable_id(id),
    };
    let heap_id = match HeapId::from_bytes_unchecked_nonzero(heap_id_bytes) {
        Ok(h) => h,
        Err(_) => return unavailable_id(id),
    };

    let explain = req.args.get("explain").and_then(|v| v.as_bool()).unwrap_or(false);
    let source = req.args.get("source").and_then(|v| v.as_str());

    // Allowed arg keys for first cut (APP-7 T6).
    const ALLOWED: &[&str] = &[
        "source",
        "plan",
        "parameters",
        "explain",
        "page_size",
        "coverage",
        "consistency",
        "continuation",
        "budget",
    ];
    for k in req.args.keys() {
        if !ALLOWED.contains(&k.as_str()) {
            return unavailable_id(id);
        }
    }

    if explain {
        let Some(src) = source else {
            return err_code(id, "validation_failed");
        };
        match explain_rql_source(src, coll, &collection_name) {
            Ok(ex) => {
                return ok_id(
                    id,
                    serde_json::json!({
                        "query_id": "0".repeat(32),
                        "plan_hash": hex32(&ex.plan_hash),
                        "heap_id": heap_id.to_string(),
                        "collection_id": coll.to_string(),
                        "rows": [],
                        "exhausted": true,
                        "coverage": { "complete": true, "mode": "complete" },
                        "consistency": { "mode": "available" },
                        "explain": {
                            "plan_profile": ex.plan_profile,
                            "plan_hash": hex32(&ex.plan_hash),
                            "tree": ex.tree,
                        }
                    }),
                );
            }
            Err(_) => return unavailable_id(id),
        }
    }

    let Some(src) = source else {
        // plan-only / continuation-only residual: require source for T6 first cut.
        return err_code(id, "validation_failed");
    };

    let mut params = Parameters::default();
    if let Some(Value::Object(m)) = req.args.get("parameters") {
        for (k, v) in m {
            params.values.insert(k.clone(), v.clone());
        }
    }

    let mut options = QueryRunOptions::default();
    if let Some(n) = req.args.get("page_size").and_then(|v| v.as_u64()) {
        if (1..=4096).contains(&n) {
            options.page_size = Some(n as u32);
        } else {
            return err_code(id, "validation_failed");
        }
    }
    if let Some(s) = req.args.get("coverage").and_then(|v| v.as_str()) {
        options.coverage = match s {
            "complete" => CoveragePolicy::Complete,
            "incomplete_allowed" => CoveragePolicy::IncompleteAllowed,
            _ => return err_code(id, "validation_failed"),
        };
    }
    if let Some(s) = req.args.get("consistency").and_then(|v| v.as_str()) {
        options.consistency = match s {
            "available" => ConsistencyMode::Available,
            "current" => ConsistencyMode::Current,
            _ => return err_code(id, "validation_failed"),
        };
    }
    if let Some(s) = req.args.get("continuation").and_then(|v| v.as_str()) {
        // Cursor tokens are JSON bytes; accept UTF-8 or base64url.
        let token = if s.starts_with('{') {
            s.as_bytes().to_vec()
        } else {
            match b64u_decode(s) {
                Ok(t) => t,
                Err(_) => return err_code(id, "validation_failed"),
            }
        };
        options.after = Some(Continuation { token });
    }
    if let Some(Value::Object(b)) = req.args.get("budget") {
        options.budget = Some(AppQueryBudget {
            max_documents: b.get("max_documents").and_then(|v| v.as_u64()),
            max_bytes: b.get("max_bytes").and_then(|v| v.as_u64()),
            max_result_bytes: b.get("max_result_bytes").and_then(|v| v.as_u64()),
        });
    }

    let mut scan = HeapStoreDocScan {
        store: ctx.store,
        collection_id: *coll.as_bytes(),
    };
    let page = match execute_rql(
        &mut scan,
        src,
        &params,
        &options,
        heap_id,
        coll,
        &collection_name,
    ) {
        Ok(p) => p,
        Err(e) => {
            // Map public app errors to registry codes where possible.
            let code = match e {
                SdkError::CoverageIncomplete(_) => "coverage_incomplete",
                SdkError::ResourceLimit(_) => "resource_limit",
                SdkError::DeadlineExceeded(_) => "deadline_exceeded",
                SdkError::QueryInvalid(_) => "validation_failed",
                _ => return unavailable_id(id),
            };
            return err_code(id, code);
        }
    };

    let rows: Vec<Value> = page
        .rows
        .iter()
        .map(|r| serde_json::json!({ "key": r.key, "value": r.value }))
        .collect();
    let next = page.next.as_ref().map(|c| {
        // Prefer UTF-8 cursor JSON; fall back to base64url.
        match std::str::from_utf8(&c.token) {
            Ok(s) => s.to_string(),
            Err(_) => residiuum_client::b64u_encode(&c.token),
        }
    });
    let cov_mode = match page.coverage.mode {
        CoveragePolicy::Complete => "complete",
        CoveragePolicy::IncompleteAllowed => "incomplete_allowed",
    };
    let cons_mode = match page.consistency.mode {
        ConsistencyMode::Available => "available",
        ConsistencyMode::Current => "current",
    };
    let query_id_hex: String = page.query_id.0.iter().map(|b| format!("{b:02x}")).collect();
    let result = serde_json::json!({
        "query_id": query_id_hex,
        "plan_hash": hex32(&page.plan_hash),
        "heap_id": heap_id.to_string(),
        "collection_id": coll.to_string(),
        "rows": rows,
        "next": next,
        "exhausted": page.exhausted,
        "coverage": {
            "complete": page.coverage.complete,
            "mode": cov_mode,
        },
        "consistency": { "mode": cons_mode },
        "remaining_limit": page.remaining_limit,
        "known_holes": page.known_holes.iter().map(|h| {
            serde_json::json!({ "code": h.code, "key": h.key })
        }).collect::<Vec<_>>(),
    });
    ok_id(id, result)
}

/// Build a [`HeapMetaLayout`] for a store data root.
pub fn layout_for_root(root: &Path) -> HeapMetaLayout {
    HeapMetaLayout::new(root)
}