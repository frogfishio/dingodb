//! AWO-1: short-write / uncertain tail poisons the writer; mutations refuse until reopen.

use residiuum_store::{
    arm_failpoint_once, clear_failpoints, enable_failpoint_hit_proof, require_failpoint_visited,
    DurabilityMode, FailpointAction, Store, StoreError,
};

#[test]
fn short_write_poisons_and_blocks_put_many() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    clear_failpoints();
    enable_failpoint_hit_proof();
    arm_failpoint_once(
        "store.active.write_tail.short_write",
        FailpointAction::ShortWrite,
    );

    let err = store
        .put_many(
            &[("awo/sw/1", b"payload-one"), ("awo/sw/2", b"payload-two")],
            DurabilityMode::Buffered,
        )
        .expect_err("short write must fail batch");
    assert!(
        matches!(err, StoreError::Io(_)),
        "expected Io short-write, got {err:?}"
    );
    require_failpoint_visited("store.active.write_tail.short_write");
    assert!(
        store.is_awo_writer_poisoned(),
        "short write must poison adaptive writer"
    );

    // Index must not claim successful publish of the batch.
    assert!(store.get("awo/sw/1").unwrap().is_none());
    assert!(store.get("awo/sw/2").unwrap().is_none());

    // Further mutations refuse until reopen.
    clear_failpoints();
    let blocked = store.put("awo/sw/3", b"x", DurabilityMode::Buffered);
    assert!(
        matches!(blocked, Err(StoreError::AdaptiveWriterPoisoned)),
        "got {blocked:?}"
    );
    let blocked_many = store.put_many(&[("awo/sw/4", b"y")], DurabilityMode::Buffered);
    assert!(matches!(
        blocked_many,
        Err(StoreError::AdaptiveWriterPoisoned)
    ));

    // Ordinary close/reopen clears poison (new handle).
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    assert!(!store.is_awo_writer_poisoned());
    store
        .put("awo/sw/reopen", b"ok", DurabilityMode::Buffered)
        .expect("reopen recovers mutation path");
    assert!(store.get("awo/sw/reopen").unwrap().is_some());
}
