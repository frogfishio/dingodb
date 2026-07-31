//! Stage 6 benchmark harness skeleton (no marketing claims).
//!
//! Times point read, append by durability mode, and salvage scan throughput.
//! Assertions only guard against absurd failures (timeouts), not performance
//! targets. OVERVIEW §12.2 disclosures apply before any public comparison.

use residiuum_store::{DurabilityMode, Store};
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn bench_skeleton_point_read_append_salvage() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path().join("bench")).unwrap();

    // --- append by durability mode ---
    let n = 200usize;
    let payload = b"{\"bench\":true,\"payload\":\"xxxxxxxx\"}";

    let t0 = Instant::now();
    for i in 0..n {
        store
            .put(&format!("k{i}"), payload, DurabilityMode::Memory)
            .unwrap();
    }
    let mem_append = t0.elapsed();

    let t0 = Instant::now();
    for i in 0..n {
        store
            .put(&format!("b{i}"), payload, DurabilityMode::Buffered)
            .unwrap();
    }
    let buffered_append = t0.elapsed();

    let t0 = Instant::now();
    for i in 0..n {
        store
            .put(&format!("d{i}"), payload, DurabilityMode::Durable)
            .unwrap();
    }
    let durable_append = t0.elapsed();

    // --- point read ---
    let t0 = Instant::now();
    for i in 0..n {
        let v = store.get(&format!("d{i}")).unwrap();
        assert!(v.is_some());
    }
    let point_read = t0.elapsed();

    // --- salvage scan ---
    let t0 = Instant::now();
    let report = store.salvage().unwrap();
    let salvage = t0.elapsed();
    assert!(report.files_scanned >= 1);
    assert!(report.live_subjects >= n);

    // Skeleton only: record numbers for humans; no performance gate.
    eprintln!(
        "stage6_bench_skeleton n={n}\n  append memory={mem_append:?}\n  append buffered={buffered_append:?}\n  append durable={durable_append:?}\n  point_read={point_read:?}\n  salvage={salvage:?} frames={} live={}",
        report.verified_frames, report.live_subjects
    );

    // Sanity: operations finished in under a generous bound (CI safety).
    assert!(point_read.as_secs() < 30);
    assert!(salvage.as_secs() < 30);
    assert!(durable_append.as_secs() < 60);
}
