//! AWO-7 productisation floor: inspect/drain/reset; default mode disabled.
//!
//! Default-on is principal-only (G12). This package never flips the default.
//!
//! Run:
//!   cargo test -p residiuum-store --features legacy-raw-store --test awo_productisation -- --test-threads=1

use residiuum_store::adaptive_write::{
    AdaptiveWriteMode, AdaptiveWritePolicy, SUPPORT_MATRIX, UPGRADE_ROLLBACK_NOTE,
};
use residiuum_store::{DurabilityMode, StoreHost};
use std::time::{Duration, Instant};

#[test]
fn ordinary_create_inspect_reports_disabled() {
    let dir = tempfile::tempdir().unwrap();
    let host = StoreHost::create(dir.path()).unwrap();
    let rep = host.adaptive_write_inspect();
    assert_eq!(rep.mode, AdaptiveWriteMode::Disabled);
    assert!(rep.default_mode_is_disabled);
    assert!(!rep.lease_active);
    assert!(rep.support_matrix.contains(&SUPPORT_MATRIX[0]));
    assert!(rep.upgrade_rollback_note.contains("opt-in"));
    assert!(!UPGRADE_ROLLBACK_NOTE.is_empty());
}

#[test]
fn disabled_attach_inspect_and_natural_put() {
    let dir = tempfile::tempdir().unwrap();
    let policy = AdaptiveWritePolicy::machine_defaults();
    assert_eq!(policy.mode, AdaptiveWriteMode::Disabled);
    let host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    let rep = host.adaptive_write_inspect();
    assert_eq!(rep.mode, AdaptiveWriteMode::Disabled);
    assert!(!rep.lease_active);
    assert_eq!(rep.cooker_threads, 0);

    let physical = host.physical();
    let mut g = physical.lock().unwrap();
    g.put("awo/prod/k", b"v", DurabilityMode::Buffered).unwrap();
    assert_eq!(g.get("awo/prod/k").unwrap().unwrap(), b"v");
}

#[test]
fn static_attach_drain_and_reset() {
    let dir = tempfile::tempdir().unwrap();
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = AdaptiveWriteMode::Static;
    policy.maximum_cookers = 1;
    policy.minimum_active_cookers = 1;
    let mut host = StoreHost::create_with_adaptive_write(dir.path(), policy).unwrap();
    let rep = host.adaptive_write_inspect();
    assert_eq!(rep.mode, AdaptiveWriteMode::Static);
    assert!(rep.lease_active);
    assert_eq!(rep.cooker_threads, 1);
    assert!(rep.benchmark_disclosure.contains("AWO-G8"));

    host.drain_writes(Instant::now() + Duration::from_secs(1))
        .unwrap();
    host.reset_adaptive_write().unwrap();
    assert!(host.adaptive_write().is_none());
    let rep2 = host.adaptive_write_inspect();
    assert_eq!(rep2.mode, AdaptiveWriteMode::Disabled);
    assert!(!rep2.lease_active);

    // Natural put works after reset.
    let physical = host.physical();
    let mut g = physical.lock().unwrap();
    g.put("awo/prod/after-reset", b"1", DurabilityMode::Buffered)
        .unwrap();
}

#[test]
fn machine_defaults_never_enable_by_accident() {
    let p = AdaptiveWritePolicy::machine_defaults();
    assert_eq!(p.mode.as_str(), "disabled");
}
