//! Stage 9 archive-path performance class skeleton (OVERVIEW §12).
//!
//! Separate from the hot-path Stage 6 bench. Numbers are diagnostic only —
//! **not** subject to hot-path latency SLOs. Do not publish archive timings
//! as Redis-class claims (OVERVIEW §12.2, USP §6 / §8).

use residuum_store::{DurabilityMode, Store, TierClass, TierMoveMode};
use std::time::Instant;
use tempfile::tempdir;

#[test]
fn archive_path_bench_skeleton() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path().join("archive-bench")).unwrap();
    store.set_seal_threshold(4 * 1024);

    let n = 100usize;
    let payload = b"{\"archive\":true,\"payload\":\"xxxxxxxxxxxxxxxx\"}";

    // Hot ingest (for setup only — not the archive claim).
    for i in 0..n {
        store
            .put(&format!("arch/{i}"), payload, DurabilityMode::Durable)
            .unwrap();
    }
    store.seal_active().unwrap();

    let segs = store.list_segment_ids();
    assert!(!segs.is_empty());

    // --- archive-path class: tier transfer ---
    let t0 = Instant::now();
    for &seg in &segs {
        store
            .transfer_segment_to_tier(seg, TierClass::Cold, TierMoveMode::Copy)
            .unwrap();
    }
    let tier_transfer = t0.elapsed();

    // --- archive-path class: cold segment catalog rebuild ---
    let t0 = Instant::now();
    store.rebuild_segment_catalog().unwrap();
    let catalog_rebuild = t0.elapsed();

    // --- archive-path class: cold summary listing (hierarchical prune) ---
    let t0 = Instant::now();
    let summaries = store.list_segment_summaries();
    let cold_list = t0.elapsed();
    assert!(!summaries.is_empty());

    // --- archive-path class: get after tier move (may hit cold media) ---
    let t0 = Instant::now();
    for i in 0..n {
        let v = store.get(&format!("arch/{i}")).unwrap();
        assert!(v.is_some());
    }
    let cold_point_read = t0.elapsed();

    let cov = store.tier_coverage();
    assert!(
        cov.notes
            .iter()
            .any(|n| n.contains("hot-path") || n.contains("archive")),
        "archive-path coverage must disclose non-hot SLO"
    );

    eprintln!(
        "stage9_archive_bench n={n} segs={}\n  \
         tier_transfer={tier_transfer:?}\n  \
         catalog_rebuild={catalog_rebuild:?}\n  \
         cold_list={cold_list:?} summaries={}\n  \
         cold_point_read={cold_point_read:?}\n  \
         DISCLOSURE: archive/cold path — not comparable to hot-path Stage 6 benches",
        segs.len(),
        summaries.len()
    );

    // CI safety only — not a performance target.
    assert!(tier_transfer.as_secs() < 60);
    assert!(cold_point_read.as_secs() < 30);
}
