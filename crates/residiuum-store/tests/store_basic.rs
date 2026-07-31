//! Stage 3a: put/get/delete, durability receipts, seal, reopen.

use residiuum_store::{DurabilityMode, EventKind, Store};
use tempfile::tempdir;

#[test]
fn put_get_delete_roundtrip() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();

    let put = store
        .put("user-42", br#"{"name":"Alice"}"#, DurabilityMode::Durable)
        .unwrap();
    assert_eq!(put.event_kind, EventKind::Put);
    assert_eq!(put.durability, DurabilityMode::Durable);
    assert_eq!(put.store_id, store.store_id());

    assert_eq!(
        store.get("user-42").unwrap().as_deref(),
        Some(br#"{"name":"Alice"}"#.as_slice())
    );

    let del = store.delete("user-42", DurabilityMode::Buffered).unwrap();
    assert_eq!(del.event_kind, EventKind::Delete);
    assert_eq!(del.durability, DurabilityMode::Buffered);
    assert!(store.get("user-42").unwrap().is_none());
}

#[test]
fn overwrite_put_updates_current_state() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    let r1 = store.put("k", b"v1", DurabilityMode::Durable).unwrap();
    let r2 = store.put("k", b"v2", DurabilityMode::Durable).unwrap();
    assert_eq!(store.get("k").unwrap().as_deref(), Some(b"v2".as_slice()));
    // Same item lineage for subject; distinct event ids.
    assert_eq!(r2.item_id, r1.item_id);
    assert_ne!(r2.event_id, r1.event_id);
}

#[test]
fn reopen_after_durable_writes() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store.put("a", b"1", DurabilityMode::Durable).unwrap();
        store.put("b", b"2", DurabilityMode::Durable).unwrap();
        store.delete("a", DurabilityMode::Durable).unwrap();
    }
    let store = Store::open(&path).unwrap();
    assert!(store.get("a").unwrap().is_none());
    assert_eq!(store.get("b").unwrap().as_deref(), Some(b"2".as_slice()));
    assert_eq!(store.live_count(), 1);
}

#[test]
fn memory_mode_not_visible_after_reopen() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store.put("disk", b"yes", DurabilityMode::Durable).unwrap();
        store.put("mem", b"nope", DurabilityMode::Memory).unwrap();
        // memory publish is visible in-process
        assert_eq!(
            store.get("mem").unwrap().as_deref(),
            Some(b"nope".as_slice())
        );
    }
    let store = Store::open(&path).unwrap();
    assert_eq!(
        store.get("disk").unwrap().as_deref(),
        Some(b"yes".as_slice())
    );
    assert!(
        store.get("mem").unwrap().is_none(),
        "memory-mode writes must not survive process restart"
    );
}

#[test]
fn seal_moves_active_to_segments() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    store.put("x", b"1", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    store.put("y", b"2", DurabilityMode::Durable).unwrap();

    let segments = dir.path().join("segments");
    let sealed: Vec<_> = std::fs::read_dir(&segments)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("residiuum"))
        .collect();
    assert!(
        !sealed.is_empty(),
        "seal should place at least one file under segments/"
    );

    // State remains readable.
    assert_eq!(store.get("x").unwrap().as_deref(), Some(b"1".as_slice()));
    assert_eq!(store.get("y").unwrap().as_deref(), Some(b"2".as_slice()));

    // Reopen still sees both (sealed + active).
    drop(store);
    let store = Store::open(dir.path()).unwrap();
    assert_eq!(store.get("x").unwrap().as_deref(), Some(b"1".as_slice()));
    assert_eq!(store.get("y").unwrap().as_deref(), Some(b"2".as_slice()));
}

#[test]
fn open_is_create_or_open() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nested/store");
    let mut store = Store::open(&path).unwrap();
    store.put("k", b"v", DurabilityMode::Durable).unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(store.get("k").unwrap().as_deref(), Some(b"v".as_slice()));
}
