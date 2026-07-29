//! Qualified heap request dispatch (`HEAP_SPEC` §33.6 / HP-008 / §32.4 data cut).
//!
//! Under `heap-key-v1`, heap identity comes solely from the channel [`HeapCap`].
//! Token/RBAC fields are rejected. Active ops: process 1–3 plus collection data
//! 105 / 111 / 112 / 120 / 121 / 122.

use dingo_client::b64u_decode;
use dingo_heap::{
    active_operation_ids, refresh_capability_or_terminate, CollectionId, HeapCap, Operation,
    OperationStatus, Rights,
};
use dingo_sdk::Filter;
use dingo_store::{
    hex16, rebuild_object_entry_from_chain, try_load_collections_catalog, HeapMetaLayout, HeapStore,
    ObjectKind, WriteReceipt,
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
    let required_rights = match req.op_id {
        1 | 2 | 3 => Rights::EMPTY,
        105 | 110 | 111 | 112 | 114 | 115 | 116 | 117 => Rights::READ,
        120 | 121 | 122 => Rights::WRITE,
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

fn require_string_arg(args: &Map<String, Value>, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn parse_collection_id(s: &str) -> Option<CollectionId> {
    s.parse().ok()
}

fn receipt_result(r: &WriteReceipt) -> Value {
    serde_json::json!({
        "event_id": hex16(&r.event_id),
        "version": hex16(&r.item_id),
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
            dingo_store::EventKind::Put => "put",
            dingo_store::EventKind::Delete => "delete",
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
        if ev.kind == dingo_store::EventKind::Put && ev.body.first() == Some(&0x01) {
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
    // Scan more rows than limit so filters that hit sparsely still return results.
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
                    let b64 = dingo_client::b64u_encode(&body[1..]);
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

/// True when the request registry lists `op_id` as active.
pub fn request_registry_allows(op_id: u16) -> bool {
    active_operation_ids().contains(&op_id)
        && matches!(Operation::status(op_id), Ok(OperationStatus::Active))
}

/// Build a [`HeapMetaLayout`] for a store data root.
pub fn layout_for_root(root: &Path) -> HeapMetaLayout {
    HeapMetaLayout::new(root)
}