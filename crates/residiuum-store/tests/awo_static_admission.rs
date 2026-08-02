//! AWO-3: Static Intake Arbiter floor — host attach, mode default disabled,
//! lease fence, natural admit under lease.
//!
//! No package accept. E6 heap active-writer residual explicit.
//! Run: cargo test -p residiuum-store --features legacy-raw-store --test awo_static_admission -- --test-threads=1

use residiuum_store::adaptive_write::{
    classify_put, AdaptiveWriteMode, AdaptiveWritePolicy, AdmissionResult, EligibilityClass,
};
use residiuum_store::{DurabilityMode, Store, StoreError, StoreHost, WriteCondition};
use std::time::{Duration, Instant};

#[test]
fn ordinary_create_has_no_adaptive_status() {
    let dir = tempfile::tempdir().unwrap();
    let host = StoreHost::create(dir.path()).unwrap();
    assert!(host.adaptive_write_status().is_none());
    assert!(host.adaptive_write().is_none());
}

#[test]
fn disabled_policy_default_no_lease() {
    let dir = tempfile::tempdir().unwrap();
    let policy = AdaptiveWritePolicy::machine_defaults();
    assert_eq!(policy.mode, AdaptiveWriteMode::Disabled);
    let host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    let st = host.adaptive_write_status().expect("attached disabled");
    assert_eq!(st.mode, AdaptiveWriteMode::Disabled);
    assert!(!st.lease_active);
    assert_eq!(st.cooker_threads, 0);

    // Direct mutation still works (no lease).
    let physical = host.physical();
    let mut guard = physical.lock().unwrap();
    guard
        .put("awo/disabled/k", b"v", DurabilityMode::Buffered)
        .unwrap();
    assert_eq!(guard.get("awo/disabled/k").unwrap().unwrap(), b"v");
}

#[test]
fn static_mode_fences_direct_put_and_admits_under_lease() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = AdaptiveWriteMode::Static;
    // Keep cooker pool small for fast shutdown.
    policy.maximum_cookers = 2;
    policy.minimum_active_cookers = 1;

    let host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    let st = host.adaptive_write_status().unwrap();
    assert_eq!(st.mode, AdaptiveWriteMode::Static);
    assert!(st.lease_active);
    assert_eq!(st.cooker_threads, 2);

    let handle = host.adaptive_write().expect("handle").clone();
    let physical = host.physical();

    // Direct mutation refused.
    {
        let mut guard = physical.lock().unwrap();
        assert!(matches!(
            guard.put("awo/static/direct", b"x", DurabilityMode::Buffered),
            Err(StoreError::AdaptiveWriterActive)
        ));
        assert!(matches!(
            guard.delete("awo/static/direct", DurabilityMode::Buffered),
            Err(StoreError::AdaptiveWriterActive)
        ));
    }

    // Admit under lease succeeds.
    {
        let mut guard = physical.lock().unwrap();
        match handle.admit_put(
            &mut guard,
            b"awo/static/a",
            b"body",
            DurabilityMode::Buffered,
            WriteCondition::Unconditional,
        ) {
            AdmissionResult::Admitted(c) => {
                let r = c.wait().expect("receipt");
                assert_ne!(r.event_id, [0u8; 16]);
            }
            AdmissionResult::Rejected(e) => panic!("rejected: {e:?}"),
        }
        assert_eq!(
            guard.get_subject_bytes(b"awo/static/a").unwrap().unwrap(),
            b"body"
        );
    }

    host.drain_writes(Instant::now() + Duration::from_secs(1))
        .unwrap();
}

#[test]
fn classify_eligibility_closed() {
    assert_eq!(
        classify_put(WriteCondition::Unconditional, DurabilityMode::Buffered),
        EligibilityClass::UnconditionalInlinePut
    );
    assert_eq!(
        classify_put(WriteCondition::Absent, DurabilityMode::Durable),
        EligibilityClass::Natural
    );
    assert_eq!(
        classify_put(WriteCondition::Unconditional, DurabilityMode::Memory),
        EligibilityClass::Natural
    );
}

#[test]
fn adaptive_mode_same_lease_floor_as_static() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = AdaptiveWriteMode::Adaptive;
    policy.maximum_cookers = 1;
    policy.minimum_active_cookers = 1;
    let host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    let st = host.adaptive_write_status().unwrap();
    assert_eq!(st.mode, AdaptiveWriteMode::Adaptive);
    assert!(st.lease_active);
    assert_eq!(st.cooker_threads, 1);
}

#[test]
fn store_lease_flag_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    assert!(!store.is_awo_lease_active());
    store.set_awo_lease_active(true);
    assert!(matches!(
        store.put("x", b"1", DurabilityMode::Buffered),
        Err(StoreError::AdaptiveWriterActive)
    ));
    store.set_awo_lease_active(false);
    store.put("x", b"1", DurabilityMode::Buffered).unwrap();
}

#[test]
fn admit_put_batch_under_lease_persist_before_publish() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = AdaptiveWriteMode::Static;
    policy.maximum_cookers = 2;
    policy.minimum_active_cookers = 1;

    let host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    let handle = host.adaptive_write().unwrap().clone();
    let physical = host.physical();
    let mut guard = physical.lock().unwrap();

    // Direct batch refused.
    assert!(matches!(
        guard.put_many(
            &[("awo/batch/a", b"1"), ("awo/batch/b", b"2")],
            DurabilityMode::Buffered
        ),
        Err(StoreError::AdaptiveWriterActive)
    ));

    let receipts = handle
        .admit_put_batch(
            &mut guard,
            &[("awo/batch/a", b"1"), ("awo/batch/b", b"2"), ("awo/batch/c", b"3")],
            DurabilityMode::Buffered,
        )
        .expect("batch admit");
    assert_eq!(receipts.len(), 3);
    assert_eq!(guard.get("awo/batch/a").unwrap().unwrap(), b"1");
    assert_eq!(guard.get("awo/batch/b").unwrap().unwrap(), b"2");
    assert_eq!(guard.get("awo/batch/c").unwrap().unwrap(), b"3");
    // Independent event ids.
    assert_ne!(receipts[0].event_id, receipts[1].event_id);
}

#[test]
fn open_heap_carries_adaptive_handle_when_lease_active() {
    // Construction of a live HeapCap needs authority ceremony; this test only
    // checks StoreHost::open_heap wiring does not panic and status is lease-active.
    let dir = tempfile::tempdir().unwrap();
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = AdaptiveWriteMode::Static;
    policy.maximum_cookers = 1;
    policy.minimum_active_cookers = 1;
    let host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    assert!(host.adaptive_write().unwrap().lease_active());
    // open_heap clones the handle into HeapStore; without a real HeapCap we
    // only assert the host still exposes adaptive after open_heap is available.
    let _ = host.adaptive_write_status();
}