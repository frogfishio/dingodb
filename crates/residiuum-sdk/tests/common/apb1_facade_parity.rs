//! APB-1 G6 dual-backend façade parity scenarios.
//!
//! Shared behavioral checks for [`HeapClient`] / [`CollectionClient`] that both
//! the **embedded** suite (`apb1_heap_client_embedded`) and the **remote** suite
//! (`residiuum-server` `apb1_heap_client_from_remote_*`) must run.
//!
//! Normative: MUST_ADD §5 (one app source differs only by constructor);
//! inventory G6. **Not** package accept — suite expands over time.
//!
//! Include from another integration test crate with:
//! ```ignore
//! #[path = "../../residiuum-sdk/tests/apb1_facade_parity.rs"]
//! mod apb1_facade_parity;
//! ```

use residiuum_sdk::{CollectionClient, HeapClient};

/// Scenario ids currently covered by this module (checklist for G6).
pub const SCENARIO_IDS: &[&str] = &[
    "create_open_list",
    "put_get_delete",
    "history_versions",
    "index_lifecycle",
];

/// Create → list → open by name; asserts bound ids and list contains the collection.
pub fn scenario_create_open_list(client: &mut HeapClient, name: &str) -> CollectionClient {
    assert!(
        client.is_bound(),
        "parity: HeapClient must be bound (From<Heap|RemoteHeap>)"
    );
    let created = client
        .create_collection(name)
        .unwrap_or_else(|e| panic!("parity create_collection({name}): {e:?}"));
    assert!(created.collection.is_bound());
    assert_eq!(created.collection.name(), name);
    assert_eq!(created.collection.heap_id(), client.id());
    assert_eq!(created.receipt.heap_id, client.id());
    assert_eq!(created.receipt.collection_id, created.collection.id());

    let listed = client
        .list_collections()
        .unwrap_or_else(|e| panic!("parity list_collections: {e:?}"));
    assert!(
        listed.iter().any(|e| e.name == name && e.collection_id == created.collection.id()),
        "parity list must include {name}: {listed:?}"
    );

    let opened = client
        .open_collection(name)
        .unwrap_or_else(|e| panic!("parity open_collection({name}): {e:?}"));
    assert_eq!(opened.id(), created.collection.id());
    assert_eq!(opened.name(), name);
    assert!(opened.is_bound());
    opened
}

/// Put JSON + bytes, get, delete; returns the collection for chaining.
pub fn scenario_put_get_delete(col: &mut CollectionClient) {
    col.put("k1", &serde_json::json!({"n": 1}))
        .unwrap_or_else(|e| panic!("parity put k1: {e:?}"));
    let got = col
        .get("k1")
        .unwrap_or_else(|e| panic!("parity get k1: {e:?}"));
    assert_eq!(got, Some(serde_json::json!({"n": 1})));

    col.put_bytes("blob-1", b"\x00\xff")
        .unwrap_or_else(|e| panic!("parity put_bytes: {e:?}"));
    let bytes = col
        .get_bytes("blob-1")
        .unwrap_or_else(|e| panic!("parity get_bytes: {e:?}"))
        .expect("blob present");
    assert_eq!(bytes, b"\x00\xff");

    let del = col
        .delete("k1")
        .unwrap_or_else(|e| panic!("parity delete k1: {e:?}"));
    assert!(del.removed, "parity delete should report removed=true");
    assert!(
        col.get("k1")
            .unwrap_or_else(|e| panic!("parity get after delete: {e:?}"))
            .is_none()
    );
}

/// Multi-version history after put/put/delete/put.
pub fn scenario_history_versions(col: &mut CollectionClient) {
    col.put("hist", &serde_json::json!({"v": 1}))
        .unwrap_or_else(|e| panic!("parity hist put1: {e:?}"));
    col.put("hist", &serde_json::json!({"v": 2}))
        .unwrap_or_else(|e| panic!("parity hist put2: {e:?}"));
    col.delete("hist")
        .unwrap_or_else(|e| panic!("parity hist delete: {e:?}"));
    col.put("hist", &serde_json::json!({"v": 3}))
        .unwrap_or_else(|e| panic!("parity hist put3: {e:?}"));

    let hist = col
        .history("hist")
        .unwrap_or_else(|e| panic!("parity history: {e:?}"));
    assert_eq!(hist.key, "hist");
    assert!(
        hist.versions.len() >= 4,
        "parity history expects ≥4 versions, got {}",
        hist.versions.len()
    );
    let kinds: Vec<&str> = hist.versions.iter().map(|v| v.kind).collect();
    assert!(
        kinds.contains(&"put") && kinds.contains(&"delete"),
        "parity history kinds={kinds:?}"
    );
    assert_eq!(hist.versions[0].kind, "put");
    if let Some(j) = hist.versions[0].json.as_ref() {
        assert_eq!(j["v"], 1);
    }
}

/// Index list → create → list → rebuild → drop (requires IndexAdmin on the session).
pub fn scenario_index_lifecycle(col: &mut CollectionClient) {
    col.put("ia", &serde_json::json!({"status": "active", "n": 1}))
        .unwrap_or_else(|e| panic!("parity idx put ia: {e:?}"));
    col.put("ib", &serde_json::json!({"status": "paused", "n": 2}))
        .unwrap_or_else(|e| panic!("parity idx put ib: {e:?}"));

    let mut im = col.indexes();
    assert!(
        im.list()
            .unwrap_or_else(|e| panic!("parity index list empty: {e:?}"))
            .is_empty()
    );
    let created = im
        .create("by-status", &["status"])
        .unwrap_or_else(|e| panic!("parity index create: {e:?}"));
    assert_eq!(created.name, "by-status");
    assert_eq!(created.fields, vec!["status".to_string()]);
    assert!(
        created.entry_count >= 2,
        "parity index entry_count={}",
        created.entry_count
    );

    let listed = im
        .list()
        .unwrap_or_else(|e| panic!("parity index list: {e:?}"));
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "by-status");

    let rebuilt = im
        .rebuild("by-status")
        .unwrap_or_else(|e| panic!("parity index rebuild: {e:?}"));
    assert_eq!(rebuilt.name, "by-status");

    im.drop("by-status")
        .unwrap_or_else(|e| panic!("parity index drop: {e:?}"));
    assert!(
        im.list()
            .unwrap_or_else(|e| panic!("parity index list after drop: {e:?}"))
            .is_empty()
    );
}

/// Collection data-plane scenarios shared by embedded and remote (no admin create).
///
/// Covers: put/get/delete, history, index lifecycle. Requires Read + Write +
/// IndexAdmin on the bound session.
pub fn run_collection_plane_parity(col: &mut CollectionClient) {
    scenario_put_get_delete(col);
    scenario_history_versions(col);
    scenario_index_lifecycle(col);
}

/// Full pack including create/list/open (needs create rights on the session).
///
/// Embedded mint caps can grant create freely. Remote HP-007 vector certs are
/// typically Read|Write|IndexAdmin only — use [`run_collection_plane_parity`]
/// after open of a pre-provisioned collection, plus list/open assertions.
pub fn run_full_facade_parity(client: &mut HeapClient, collection_name: &str) {
    let mut col = scenario_create_open_list(client, collection_name);
    run_collection_plane_parity(&mut col);
}

/// Open/list residual for remote sessions without HeapAdmin create.
pub fn scenario_list_and_open(client: &mut HeapClient, name: &str) -> CollectionClient {
    assert!(client.is_bound());
    let listed = client
        .list_collections()
        .unwrap_or_else(|e| panic!("parity list_collections: {e:?}"));
    assert!(
        listed.iter().any(|e| e.name == name),
        "parity list must include {name}: {listed:?}"
    );
    let opened = client
        .open_collection(name)
        .unwrap_or_else(|e| panic!("parity open_collection({name}): {e:?}"));
    assert_eq!(opened.name(), name);
    assert!(opened.is_bound());
    opened
}