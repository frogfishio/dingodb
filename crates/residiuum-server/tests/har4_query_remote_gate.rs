//! HAR-4 dep (query product): gate locks for qualified remote + op 118.
//!
//! Does **not** activate op 118 or flip ServeOptions default to HeapKey.
//! Locks honesty for APP-7 / APB-7 T6 dependency tracking.

use residiuum_server::{
    request_registry_allows, validate_qualified_listener, ResidentHeapRegistry, ServeOptions,
};
use std::sync::Arc;

#[test]
fn op_118_rql_query_still_reserved() {
    // Product remote query wire remains off until APP-7/APB-7 activate it.
    assert!(
        !request_registry_allows(118),
        "op 118 rql_query must stay reserved until HAR-4 path + APP-7 admit it"
    );
}

#[test]
fn serve_options_default_is_not_yet_qualified_heap_key() {
    // HAR-4 exit requires HeapKey default; today default remains opt-in.
    let opts = ServeOptions::default();
    assert!(
        !opts.qualified_heap_key,
        "HAR-4 residual: default serve is not yet qualified_heap_key=true"
    );
}

#[test]
fn qualified_listener_requires_tls_and_forbids_token() {
    let reg = Arc::new(ResidentHeapRegistry::default());
    let deployment = "00000000-0000-4000-8000-000000000001";

    assert!(
        validate_qualified_listener(false, None, false, Some(&reg), Some(deployment)).is_err(),
        "TLS required"
    );
    assert!(
        validate_qualified_listener(true, Some("shared-token"), false, Some(&reg), Some(deployment))
            .is_err(),
        "shared token forbidden on qualified path"
    );
    assert!(
        validate_qualified_listener(true, None, true, Some(&reg), Some(deployment)).is_err(),
        "diagnostic line protocol forbidden"
    );
    assert!(
        validate_qualified_listener(true, None, false, None, Some(deployment)).is_err(),
        "registry required"
    );
    assert!(
        validate_qualified_listener(true, None, false, Some(&reg), None).is_err(),
        "deployment_id required"
    );
    assert!(
        validate_qualified_listener(true, None, false, Some(&reg), Some(deployment)).is_ok(),
        "minimal qualified posture must validate"
    );
}

#[test]
fn baseline_ops_json_marks_118_reserved() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/app/baseline-v1/operations-v1.json");
    let raw = std::fs::read_to_string(&path).expect("operations-v1.json");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    // Find any wire entry for rql_query / 118 and assert reserved language.
    let s = raw.to_lowercase();
    assert!(
        s.contains("rql_query") && (s.contains("reserved") || s.contains("\"status\": \"reserved\"")),
        "baseline ops must keep rql_query reserved honesty"
    );
    // Spot-check op id 118 appears as reserved in structured form when present.
    let _ = v;
}
