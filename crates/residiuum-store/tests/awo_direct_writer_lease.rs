//! AWO-1/3: writer poison gate (lease precursor).
//!
//! Full AdaptiveWriteLease that fences direct put while AWO runtime owns the
//! writer is AWO-3. This test locks the AWO-1 poison refusal surface used until
//! then: poisoned writers return AdaptiveWriterPoisoned on put/delete.

use residiuum_store::{
    arm_failpoint_once, clear_failpoints, DurabilityMode, FailpointAction, Store, StoreError,
};

#[test]
fn poison_refuses_put_and_delete_until_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    clear_failpoints();
    arm_failpoint_once(
        "store.active.write_tail.short_write",
        FailpointAction::ShortWrite,
    );
    let _ = store
        .put_many(&[("awo/lease/k", b"v")], DurabilityMode::Buffered)
        .expect_err("short write");
    assert!(store.is_awo_writer_poisoned());

    clear_failpoints();
    assert!(matches!(
        store.put("awo/lease/p", b"1", DurabilityMode::Buffered),
        Err(StoreError::AdaptiveWriterPoisoned)
    ));
    assert!(matches!(
        store.delete("awo/lease/k", DurabilityMode::Buffered),
        Err(StoreError::AdaptiveWriterPoisoned)
    ));

    drop(store);
    let store = Store::open(dir.path()).unwrap();
    assert!(!store.is_awo_writer_poisoned());
}

#[test]
fn adaptive_writer_active_error_exists_for_awo3() {
    // Compile-time / API surface: variant reserved for AdaptiveWriteLease (AWO-3).
    let e = StoreError::AdaptiveWriterActive;
    assert!(e.to_string().contains("adaptive writer active"));
}
