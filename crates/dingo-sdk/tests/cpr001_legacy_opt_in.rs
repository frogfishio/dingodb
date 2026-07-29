//! CPR-001: legacy flat SDK is feature-gated and claim-labelled (not Gate H6).

use dingo_sdk::{
    flat_collection_claim_language, heap_only_embedded_profile, legacy_flat_sdk_enabled,
    product_may_advertise_qualified_heap, Dingo, DingoDeployment, FLAT_COLLECTION_SURFACE_LABEL,
    LEGACY_FLAT_SDK_FEATURE, SDK_API_VERSION,
};
use tempfile::tempdir;

#[test]
fn legacy_flat_feature_is_default_and_labelled() {
    // Package default features enable the flat surface for Stages 3–9.
    assert!(
        legacy_flat_sdk_enabled(),
        "default test build must enable {LEGACY_FLAT_SDK_FEATURE}"
    );
    assert!(!heap_only_embedded_profile());
    assert!(FLAT_COLLECTION_SURFACE_LABEL.contains("CPR-001"));
    assert!(FLAT_COLLECTION_SURFACE_LABEL.contains("not dingo-heap-v1"));
    assert!(!flat_collection_claim_language().is_empty());
    // Flat freeze label is independent of heap qualification.
    assert_eq!(SDK_API_VERSION, "1.0");
    // HP-010 claim must stay Level 1 until matrix flips.
    assert!(!product_may_advertise_qualified_heap());
}

#[test]
fn flat_open_and_deployment_host_both_available() {
    let dir = tempdir().unwrap();
    let flat_path = dir.path().join("flat.dingo");
    let dep_path = dir.path().join("dep.dingo");

    {
        let mut db = Dingo::open(&flat_path).expect("legacy open");
        let mut users = db.collection("users").expect("flat collection");
        users
            .put("k", &serde_json::json!({"v": 1}))
            .expect("flat put");
    }
    // Explicit compatibility spelling (after first writer dropped).
    let _ = Dingo::open_compatibility(&flat_path).expect("open_compatibility");

    // Heap-bound host does not expose flat collection iteration.
    let deployment = Dingo::open_deployment(&dep_path).expect("open_deployment");
    assert_eq!(deployment.root(), dep_path.as_path());
    // Same entry via create when missing.
    let dep2_path = dir.path().join("dep2.dingo");
    let _ = Dingo::create_deployment(&dep2_path).expect("create_deployment");
    let _ = DingoDeployment::open(&dep2_path).expect("reopen deployment");
}

#[test]
fn raw_store_access_documented_as_non_qualified() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("raw.dingo");
    let db = Dingo::open(&path).unwrap();
    let store = db.store().expect("legacy store()");
    let _ = store.store_id();
}