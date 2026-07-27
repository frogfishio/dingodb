//! Stage 9 — tiering, archive path, long retention (OVERVIEW §9).
//!
//! Exit criteria covered here:
//! - Segment move/copy preserves segment identity
//! - Hierarchical segment catalog rebuild after catalog loss
//! - Offline tier → explicit coverage hole (not empty success)
//! - Multi-generation format classification preserves bytes

use dingo_store::{
    classify_segment_bytes, DurabilityMode, FormatClassification, Store, TierClass, TierMoveMode,
};
use std::fs;

#[test]
fn move_segment_to_cold_preserves_identity_and_data() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    store.set_seal_threshold(64 * 1024 * 1024);
    store
        .put("keep/me", b"fifteen-years", DurabilityMode::Durable)
        .unwrap();
    store.seal_active().unwrap();

    let seg = store
        .list_segment_summaries()
        .into_iter()
        .find(|s| s.item_events > 0)
        .expect("data-bearing sealed segment")
        .segment_id;

    let evidence = store
        .transfer_segment_to_tier(seg, TierClass::Cold, TierMoveMode::Move)
        .unwrap();
    assert_eq!(evidence.segment_id, seg);
    assert_eq!(evidence.from_tier, TierClass::Hot);
    assert_eq!(evidence.to_tier, TierClass::Cold);
    assert_eq!(evidence.source_hash, evidence.dest_hash);

    // Hot path gone; cold media holds the bytes.
    let hot = store.paths().sealed_segment(&seg);
    assert!(!hot.is_file());
    let cold = store
        .paths()
        .tiers_dir()
        .join("cold")
        .join(format!("{}.dingo", dingo_store::hex16(&seg)));
    assert!(cold.is_file());

    // Placement primary is cold; data still readable.
    assert_eq!(
        store.tier_placement().get(&seg).unwrap().tier,
        TierClass::Cold
    );
    assert_eq!(
        store.get("keep/me").unwrap().as_deref(),
        Some(b"fifteen-years".as_slice())
    );

    // Reopen discovers cold placement.
    drop(store);
    let store = Store::open(dir.path().join("s")).unwrap();
    assert_eq!(
        store.get("keep/me").unwrap().as_deref(),
        Some(b"fifteen-years".as_slice())
    );
    assert_eq!(
        store.tier_placement().get(&seg).unwrap().tier,
        TierClass::Cold
    );
}

#[test]
fn copy_then_offline_archive_is_coverage_hole_not_empty_success() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    // High threshold so the put and seal share one data-bearing segment
    // (low thresholds seal the descriptor-only active segment first).
    store.set_seal_threshold(64 * 1024 * 1024);
    store
        .put("only/on/archive", b"cold-bytes", DurabilityMode::Durable)
        .unwrap();
    store.seal_active().unwrap();

    // Move the segment that actually holds item events.
    let seg = store
        .list_segment_summaries()
        .into_iter()
        .find(|s| s.item_events > 0)
        .expect("data-bearing sealed segment")
        .segment_id;

    // Move (not copy) so the only copy lives on archive.
    store
        .transfer_segment_to_tier(seg, TierClass::Archive, TierMoveMode::Move)
        .unwrap();
    assert!(store.tier_coverage().is_complete());
    let hot = store.paths().sealed_segment(&seg);
    let arch = store
        .paths()
        .tiers_dir()
        .join("archive")
        .join(format!("{}.dingo", dingo_store::hex16(&seg)));
    assert!(!hot.is_file(), "move must remove hot copy");
    assert!(arch.is_file(), "archive must hold the segment");

    store.set_tier_available(TierClass::Archive, false).unwrap();

    let cov = store.tier_coverage();
    assert!(cov.is_incomplete());
    assert!(cov.offline.contains(&TierClass::Archive));
    assert!(cov.unavailable_segments.contains(&seg));
    assert!(cov
        .notes
        .iter()
        .any(|n| n.contains("offline") || n.contains("incomplete")));

    let got = store.get_with_tier_coverage("only/on/archive").unwrap();
    // Index rebuilt without archive → value gone from hot projection.
    assert!(
        got.value.is_none(),
        "expected missing value when sole copy is on offline archive; live_count={} placement={:?}",
        store.live_count(),
        store.tier_placement().get(&seg)
    );
    // Must NOT claim proven absence.
    assert!(!got.absence_proven);
    assert!(got.coverage.is_incomplete());
}

#[test]
fn hierarchical_catalog_rebuilds_after_loss() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    store.set_seal_threshold(64 * 1024 * 1024);
    store.put("a", b"1", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    store.put("b", b"2", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();

    let before = store.list_segment_summaries();
    assert!(!before.is_empty());
    let n = before.len();

    // Wipe derived catalogs (including segment + tier placement catalogs).
    let catalogs = store.paths().catalogs_dir();
    let _ = fs::remove_dir_all(&catalogs);
    fs::create_dir_all(&catalogs).unwrap();

    store.rebuild_segment_catalog().unwrap();
    let after = store.list_segment_summaries();
    assert!(
        after.len() >= n,
        "rebuild should rediscover sealed segments from media"
    );
    assert!(after.iter().any(|s| s.available && s.item_events > 0));
}

#[test]
fn cold_search_summaries_list_archive_tier() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    store.set_seal_threshold(64 * 1024 * 1024);
    for i in 0..5 {
        store
            .put(&format!("k{i}"), b"x", DurabilityMode::Durable)
            .unwrap();
    }
    store.seal_active().unwrap();
    let seg = store
        .list_segment_summaries()
        .into_iter()
        .find(|s| s.item_events > 0)
        .expect("data segment")
        .segment_id;
    store
        .transfer_segment_to_tier(seg, TierClass::Warm, TierMoveMode::Copy)
        .unwrap();

    let summaries = store.list_segment_summaries();
    let s = summaries.iter().find(|s| s.segment_id == seg).unwrap();
    assert_eq!(s.tier, TierClass::Warm);
    assert!(s.size > 0);
    assert!(s.verified_frames > 0);

    // Cold-search filter helper.
    let big = store.segment_catalog().filter_min_size(1);
    assert!(!big.is_empty());
}

#[test]
fn format_classification_preserves_unsupported_identity() {
    // Garbage bytes: unreadable but classified without rewrite.
    let junk = b"not a dingo frame at all................";
    let c = classify_segment_bytes(junk);
    match c {
        FormatClassification::Unreadable {
            byte_len,
            content_hash,
        } => {
            assert_eq!(byte_len, junk.len() as u64);
            assert_ne!(content_hash, [0u8; 32]);
        }
        other => panic!("expected Unreadable, got {other:?}"),
    }

    // Real sealed segment should classify as supported.
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    store.put("x", b"y", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    let seg = store.list_segment_ids()[0];
    match store.classify_segment(&seg).unwrap() {
        FormatClassification::Supported { wire_major, .. } => {
            assert_eq!(wire_major, Some(1));
        }
        other => panic!("expected Supported, got {other:?}"),
    }
}

#[test]
fn migration_evidence_written() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    store.set_seal_threshold(64 * 1024 * 1024);
    store.put("m", b"e", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    let seg = store
        .list_segment_summaries()
        .into_iter()
        .find(|s| s.item_events > 0)
        .expect("data segment")
        .segment_id;
    store
        .transfer_segment_to_tier(seg, TierClass::Cold, TierMoveMode::Copy)
        .unwrap();

    let mig = store.paths().recovery_dir().join("migrations");
    assert!(mig.is_dir());
    let entries: Vec<_> = fs::read_dir(&mig).unwrap().collect();
    assert!(!entries.is_empty());
    let text = fs::read_to_string(entries[0].as_ref().unwrap().path()).unwrap();
    assert!(text.contains("dingo-migration-v1"));
    assert!(text.contains("tool_version=dingo-store-9"));
    assert!(text.contains(&dingo_store::hex16(&seg)));
}

#[test]
fn archive_path_note_not_hot_slo() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path().join("s")).unwrap();
    store.set_seal_threshold(64 * 1024 * 1024);
    store.put("z", b"1", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    let seg = store
        .list_segment_summaries()
        .into_iter()
        .find(|s| s.item_events > 0)
        .expect("data segment")
        .segment_id;
    store
        .transfer_segment_to_tier(seg, TierClass::Cold, TierMoveMode::Move)
        .unwrap();
    let cov = store.tier_coverage();
    assert!(cov
        .notes
        .iter()
        .any(|n| n.contains("not subject to hot-path")));
}
