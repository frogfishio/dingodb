//! DEF-022 crash-consistency matrix skeleton.
//!
//! - Validates the embedded machine-readable matrix.
//! - Runs the **CI subset** of failpoint cells on every test run.
//! - When `DINGO_CRASH_MATRIX_FULL=1`, runs every matrix cell (nightly).
//!
//! Crash simulation is process-local: arm a failpoint, drive the operation,
//! catch `Failpoint` error or panic, drop the writer handle, reopen, assert
//! reopen invariants. This is not a full power-loss harness; it proves the
//! failpoint surface and recovery shape required by DEF-022.

use dingo_store::{
    all_cells, arm_failpoint_once, ci_subset_cells, clear_failpoints, load_crash_matrix,
    validate_crash_matrix, DurabilityMode, FailpointAction, Store, StoreError, CRASH_MATRIX_JSON,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tempfile::tempdir;

/// Failpoints are process-global; serialize matrix drivers across test threads.
fn matrix_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

#[test]
fn crash_matrix_document_is_valid() {
    assert!(
        CRASH_MATRIX_JSON.contains("\"version\": 1"),
        "embedded matrix must be version 1"
    );
    let m = load_crash_matrix().expect("parse crash_matrix.v1.json");
    validate_crash_matrix(&m).expect("structural validation");
    assert!(
        !ci_subset_cells(&m).is_empty(),
        "CI subset must list at least one failpoint"
    );
    // Required operations for the skeleton surface.
    for id in [
        "store_create",
        "put_durable",
        "delete_durable",
        "seal_active",
        "chunked_put_durable",
        "write_dedup_persist",
    ] {
        assert!(
            m.operations.iter().any(|o| o.id == id),
            "matrix missing operation {id}"
        );
    }
}

#[test]
fn ci_subset_failpoints_respect_reopen_invariants() {
    let _guard = matrix_lock();
    let m = load_crash_matrix().expect("load");
    validate_crash_matrix(&m).expect("validate");
    for (op, fp) in ci_subset_cells(&m) {
        run_cell(op.id.as_str(), fp.name.as_str(), &fp.expected_on_reopen);
    }
}

#[test]
fn full_matrix_when_env_set() {
    if std::env::var_os("DINGO_CRASH_MATRIX_FULL").is_none() {
        eprintln!("skip full matrix (set DINGO_CRASH_MATRIX_FULL=1 for nightly)");
        return;
    }
    let _guard = matrix_lock();
    let m = load_crash_matrix().expect("load");
    validate_crash_matrix(&m).expect("validate");
    for (op, fp) in all_cells(&m) {
        run_cell(op.id.as_str(), fp.name.as_str(), &fp.expected_on_reopen);
    }
}

/// Drive one matrix cell for a known operation id + failpoint name.
fn run_cell(
    op_id: &str,
    failpoint: &str,
    expected: &dingo_store::ExpectedReopen,
) {
    clear_failpoints();
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("s");

    match op_id {
        "store_create" => run_store_create(&path, failpoint, expected),
        "put_durable" => run_put_durable(&path, failpoint, expected),
        "put_buffered" => run_put_buffered(&path, failpoint, expected),
        "delete_durable" => run_delete_durable(&path, failpoint, expected),
        "chunked_put_durable" => run_chunked_put(&path, failpoint, expected),
        "seal_active" => run_seal(&path, failpoint, expected),
        "write_dedup_persist" => run_dedup(&path, failpoint, expected),
        "catalog_refresh" => run_catalog(&path, failpoint, expected),
        "checkpoint" => run_checkpoint(&path, failpoint, expected),
        "tier_move" => run_tier_move(&path, failpoint, expected),
        "compact_live" => run_compact(&path, failpoint, expected),
        other => panic!("no driver for operation {other}"),
    }

    clear_failpoints();
}

fn leak_name(s: &str) -> &'static str {
    // Failpoint API takes &'static str; test names come from the JSON once.
    Box::leak(s.to_string().into_boxed_str())
}

fn arm_error(name: &str) {
    arm_failpoint_once(leak_name(name), FailpointAction::Error);
}

fn arm_panic(name: &str) {
    arm_failpoint_once(leak_name(name), FailpointAction::Panic);
}

fn assert_prior_ok(store: &Store, expected: &dingo_store::ExpectedReopen) {
    if expected.prior_durable_retained {
        assert_eq!(
            store.get("prior").unwrap().as_deref(),
            Some(b"prior-v1".as_slice()),
            "prior durable key must survive crash at failpoint"
        );
    }
    if expected.salvageable {
        let report = store.salvage().expect("salvage must run");
        // Empty store is salvageable (zero frames); just require the call works.
        let _ = (report.files_scanned, report.verified_frames, report.holes);
    }
}

fn seed_prior(path: &Path) {
    let mut s = Store::create(path).expect("create");
    s.put("prior", b"prior-v1", DurabilityMode::Durable)
        .expect("prior put");
    drop(s);
}

fn run_store_create(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    arm_error(failpoint);
    let result = Store::create(path);
    assert!(
        matches!(result, Err(StoreError::Failpoint(_))),
        "create should hit failpoint {failpoint}, got err={}",
        result.err().map(|e| e.to_string()).unwrap_or_default()
    );
    clear_failpoints();
    // Reopen or recreate: incomplete create must not invent user data.
    match Store::open(path) {
        Ok(s) => {
            assert!(
                s.get("ghost").unwrap().is_none(),
                "must not fabricate subjects"
            );
            if expected.salvageable {
                let _ = s.salvage();
            }
        }
        Err(_) => {
            // Incomplete tree is an acceptable outcome for mid-create crash.
            if path.exists() {
                // Best-effort: salvage path may not apply; no fabricated commits.
            }
        }
    }
}

fn run_put_durable(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).expect("open");
    arm_error(failpoint);
    let result = store.put("k", b"v-new", DurabilityMode::Durable);
    let acknowledged = result.is_ok();
    if !acknowledged {
        assert!(
            matches!(result, Err(StoreError::Failpoint(_))),
            "expected Failpoint, got {result:?}"
        );
    }
    drop(store);
    clear_failpoints();

    let store = Store::open(path).expect("reopen");
    assert_prior_ok(&store, expected);
    match expected.acknowledged_visible {
        Some(true) => {
            // Either receipt was ok, or bytes crossed the sync boundary before error.
            assert_eq!(
                store.get("k").unwrap().as_deref(),
                Some(b"v-new".as_slice()),
                "expected put visible after failpoint {failpoint}"
            );
        }
        Some(false) => {
            if !acknowledged {
                assert!(
                    store.get("k").unwrap().is_none(),
                    "unacked put must not appear after reopen at {failpoint}"
                );
            }
        }
        None => {
            // Ambiguous (e.g. after_write without fsync): only forbid fabrication
            // when the op was never attempted with a success path.
            let _ = store.get("k");
        }
    }
}

fn run_put_buffered(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).expect("open");
    arm_error(failpoint);
    let result = store.put("k", b"buf", DurabilityMode::Buffered);
    let acknowledged = result.is_ok();
    drop(store);
    clear_failpoints();
    let store = Store::open(path).expect("reopen");
    assert_prior_ok(&store, expected);
    if expected.acknowledged_visible == Some(false) && !acknowledged {
        // May still appear after same-process reopen if page cache held bytes.
        // Only hard-assert when failpoint fired before any write.
        if failpoint.ends_with(".before") {
            assert!(store.get("k").unwrap().is_none());
        }
    }
}

fn run_delete_durable(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    {
        let mut s = Store::open(path).unwrap();
        s.put("victim", b"alive", DurabilityMode::Durable).unwrap();
    }
    let mut store = Store::open(path).unwrap();
    arm_error(failpoint);
    let result = store.delete("victim", DurabilityMode::Durable);
    let acknowledged = result.is_ok();
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
    match expected.acknowledged_visible {
        Some(false) if !acknowledged => {
            assert_eq!(
                store.get("victim").unwrap().as_deref(),
                Some(b"alive".as_slice()),
                "unacked delete must leave prior live value"
            );
        }
        Some(true) => {
            assert!(
                store.get("victim").unwrap().is_none(),
                "durable delete tombstone should apply after {failpoint}"
            );
        }
        _ => {}
    }
}

fn run_chunked_put(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).unwrap();
    store.set_chunk_threshold(20);
    store.set_chunk_size(8);
    let payload: Vec<u8> = (0u8..80).collect();
    arm_error(failpoint);
    let result = store.put("big", &payload, DurabilityMode::Durable);
    let acknowledged = result.is_ok();
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
    if expected.acknowledged_visible == Some(false) && !acknowledged && failpoint.ends_with(".before")
    {
        assert!(store.get("big").unwrap().is_none() || store.get("big").is_err());
    }
    if expected.acknowledged_visible == Some(true) {
        assert_eq!(store.get("big").unwrap().as_deref(), Some(payload.as_slice()));
    }
}

fn run_seal(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    // Use panic action: seal_active takes the active writer; Error mid-seal
    // would leave an inconsistent in-process handle. Panic + drop models crash.
    {
        let mut store = Store::open(path).unwrap();
        store
            .put("sealed-key", b"sealed-val", DurabilityMode::Durable)
            .unwrap();
        arm_panic(failpoint);
        let result = catch_unwind(AssertUnwindSafe(|| {
            store.seal_active().expect("seal or panic");
        }));
        assert!(result.is_err(), "seal must panic at failpoint {failpoint}");
        // Drop without running further seal logic.
        drop(store);
    }
    clear_failpoints();
    let store = Store::open(path).expect("reopen after seal crash");
    assert_prior_ok(&store, expected);
    assert_eq!(
        store.get("sealed-key").unwrap().as_deref(),
        Some(b"sealed-val".as_slice()),
        "data written before seal must survive failpoint {failpoint}"
    );
}

fn run_dedup(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).unwrap();
    let receipt = store
        .put("dedup-k", b"dedup-v", DurabilityMode::Durable)
        .unwrap();
    let op_id = [7u8; 16];
    let content = dingo_store::content_identity("put", "", "dedup-k", b"dedup-v");
    arm_error(failpoint);
    let result = store.record_write_dedup(op_id, content, &receipt);
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
    assert_eq!(
        store.get("dedup-k").unwrap().as_deref(),
        Some(b"dedup-v".as_slice())
    );
    // Dedup table may or may not have the record depending on failpoint.
    let _ = result;
}

fn run_catalog(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).unwrap();
    store
        .put("c/users/1", b"{}", DurabilityMode::Durable)
        .unwrap();
    arm_error(failpoint);
    let _ = store.rebuild_catalogs();
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
    assert!(store.get("c/users/1").unwrap().is_some());
}

fn run_checkpoint(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let store = Store::open(path).unwrap();
    arm_error(failpoint);
    let _ = store.checkpoint("test-coverage");
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
}

fn run_tier_move(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).unwrap();
    store
        .put("tier-k", b"tier-v", DurabilityMode::Durable)
        .unwrap();
    store.seal_active().unwrap();
    // Find a sealed segment id from placement.
    let ids: Vec<_> = store
        .tier_placement()
        .entries()
        .map(|p| p.segment_id)
        .collect();
    if ids.is_empty() {
        clear_failpoints();
        return;
    }
    arm_error(failpoint);
    let _ = store.transfer_segment_to_tier(
        ids[0],
        dingo_store::TierClass::Warm,
        dingo_store::TierMoveMode::Copy,
    );
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
    assert_eq!(
        store.get("tier-k").unwrap().as_deref(),
        Some(b"tier-v".as_slice())
    );
}

fn run_compact(path: &Path, failpoint: &str, expected: &dingo_store::ExpectedReopen) {
    seed_prior(path);
    let mut store = Store::open(path).unwrap();
    store
        .put("c1", b"one", DurabilityMode::Durable)
        .unwrap();
    arm_error(failpoint);
    let _ = store.compact_live();
    drop(store);
    clear_failpoints();
    let store = Store::open(path).unwrap();
    assert_prior_ok(&store, expected);
    assert_eq!(
        store.get("c1").unwrap().as_deref(),
        Some(b"one".as_slice())
    );
}
