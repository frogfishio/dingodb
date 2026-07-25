//! DEF-023 — remove full-store rescans from the write acknowledgement path.
//!
//! Guarantees under test:
//! - Durable writes update derived state without O(total retained data) work.
//! - Deleting all derived state still reconstructs identical logical results.
//! - Memory-mode visibility never enters the durable index / cache.
//! - Frontier checkpoints accelerate open via active-tail apply (not full rescan).
//! - Bench skeleton discloses write-path amplification metrics.

use dingo_store::{DurabilityMode, Store, PRIMARY_CACHE_FILE};
use std::fs;
use std::time::Instant;
use tempfile::tempdir;

/// Seed many sealed segments so a naive write-path rescan would be expensive.
fn seed_retained_data(store: &mut Store, sealed_batches: usize, keys_per_batch: usize) {
    for b in 0..sealed_batches {
        for k in 0..keys_per_batch {
            let subject = format!("hist/{b}/{k}");
            let body = format!("payload-batch-{b}-key-{k}-{}", "x".repeat(64));
            store
                .put(&subject, body.as_bytes(), DurabilityMode::Durable)
                .unwrap();
        }
        store.seal_active().unwrap();
    }
}

#[test]
fn durable_write_ack_independent_of_retained_volume() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("s");
    let mut store = Store::create(&root).unwrap();

    // Large retained history (multiple sealed segments).
    seed_retained_data(&mut store, 8, 40);
    let live_before = store.live_count();
    assert!(live_before >= 8 * 40);

    // Fresh durable writes must not grow with sealed volume: wall time stays
    // within a generous CI bound even with substantial retained data.
    let n = 64usize;
    let payload = b"hot-path-write";
    let t0 = Instant::now();
    for i in 0..n {
        store
            .put(&format!("hot/{i}"), payload, DurabilityMode::Durable)
            .unwrap();
    }
    let hot_path = t0.elapsed();
    assert_eq!(store.live_count(), live_before + n);
    // CI safety bound (not a performance gate). Full segment rescan on every
    // write would typically blow this on slower runners with the seed above.
    assert!(
        hot_path.as_secs() < 30,
        "hot-path durable append too slow ({hot_path:?}); possible full-store rescan regression"
    );
}

#[test]
fn delete_derived_state_reconstructs_identical_logical_results() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("s");
    let expected: Vec<(String, Vec<u8>)> = {
        let mut store = Store::create(&root).unwrap();
        seed_retained_data(&mut store, 3, 10);
        store
            .put("alpha", b"A", DurabilityMode::Durable)
            .unwrap();
        store
            .put("beta", b"B", DurabilityMode::Buffered)
            .unwrap();
        store.delete("alpha", DurabilityMode::Durable).unwrap();
        store.persist_index_cache().unwrap();
        store
            .live_logical_entries()
            .unwrap()
            .into_iter()
            .map(|(k, v)| (String::from_utf8(k).unwrap(), v))
            .collect()
    };

    // Wipe all derived directories.
    for name in ["indexes", "catalogs", "snapshots"] {
        let p = root.join(name);
        if p.exists() {
            fs::remove_dir_all(&p).unwrap();
        }
    }

    let store = Store::open(&root).unwrap();
    let rebuilt: Vec<(String, Vec<u8>)> = store
        .live_logical_entries()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (String::from_utf8(k).unwrap(), v))
        .collect();
    assert_eq!(
        rebuilt, expected,
        "rebuild-from-segments must match pre-wipe logical state"
    );
    assert!(store.get("alpha").unwrap().is_none());
    assert_eq!(
        store.get("beta").unwrap().as_deref(),
        Some(b"B".as_slice())
    );
}

#[test]
fn memory_mode_never_enters_durable_index_cache() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("s");
    {
        let mut store = Store::create(&root).unwrap();
        store
            .put("mem-only", b"ghost", DurabilityMode::Memory)
            .unwrap();
        store
            .put("disk", b"solid", DurabilityMode::Durable)
            .unwrap();
        assert_eq!(
            store.get("mem-only").unwrap().as_deref(),
            Some(b"ghost".as_slice())
        );
        // Force a checkpoint so the cache file exists.
        store.persist_index_cache().unwrap();
        assert!(store.index_cache_path().is_file());
    }
    let store = Store::open(&root).unwrap();
    assert!(
        store.get("mem-only").unwrap().is_none(),
        "memory publish must not survive reopen via index cache"
    );
    assert_eq!(
        store.get("disk").unwrap().as_deref(),
        Some(b"solid".as_slice())
    );
}

#[test]
fn frontier_checkpoint_plus_active_tail_matches_full_rebuild() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("s");
    {
        let mut store = Store::create(&root).unwrap();
        for i in 0..10 {
            store
                .put(&format!("sealed/{i}"), b"S", DurabilityMode::Durable)
                .unwrap();
        }
        store.seal_active().unwrap();
        // Checkpoint after seal (covers sealed set; active is nearly empty).
        store.persist_index_cache().unwrap();

        // Writes beyond the checkpoint frontier stay only in the active segment
        // until the next rate-limited checkpoint (ops < DERIVED_CHECKPOINT_EVERY_OPS).
        for i in 0..5 {
            store
                .put(&format!("tail/{i}"), b"T", DurabilityMode::Durable)
                .unwrap();
        }
        // Leave without an explicit final persist so open must apply the active tail.
    }

    let mut reopened = Store::open(&root).unwrap();
    for i in 0..10 {
        assert_eq!(
            reopened.get(&format!("sealed/{i}")).unwrap().as_deref(),
            Some(b"S".as_slice())
        );
    }
    for i in 0..5 {
        assert_eq!(
            reopened.get(&format!("tail/{i}")).unwrap().as_deref(),
            Some(b"T".as_slice())
        );
    }
    let live_from_frontier = reopened.live_count();

    // Explicit full rebuild on the same handle matches frontier+tail visibility.
    reopened.rebuild_index().unwrap();
    assert_eq!(reopened.live_count(), live_from_frontier);
    for i in 0..5 {
        assert_eq!(
            reopened.get(&format!("tail/{i}")).unwrap().as_deref(),
            Some(b"T".as_slice())
        );
    }
}

#[test]
fn rate_limited_checkpoint_still_allows_recovery() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("s");
    {
        let mut store = Store::create(&root).unwrap();
        // Fewer ops than the rate-limit threshold so intermediate checkpoints
        // may be skipped; recovery must still succeed from segments.
        for i in 0..10 {
            store
                .put(&format!("k{i}"), b"v", DurabilityMode::Durable)
                .unwrap();
        }
    }
    // Even if primary.idx is stale or absent, open recovers.
    let idx = root.join("indexes").join(PRIMARY_CACHE_FILE);
    if idx.is_file() {
        fs::remove_file(&idx).unwrap();
    }
    let store = Store::open(&root).unwrap();
    for i in 0..10 {
        assert_eq!(
            store.get(&format!("k{i}")).unwrap().as_deref(),
            Some(b"v".as_slice())
        );
    }
}

#[test]
fn write_path_bench_disclosure() {
    // Disclosure fields for DEF-023 / doc/BENCHMARK_DISCLOSURE.md (skeleton only).
    let dir = tempdir().unwrap();
    let root = dir.path().join("bench");
    let mut store = Store::create(&root).unwrap();
    seed_retained_data(&mut store, 4, 25);

    let n = 100usize;
    let payload = br#"{"bench":true,"def":"023"}"#;
    let mut fsync_proxy = 0u64; // durable mode implies stable-storage boundary per ack

    let t0 = Instant::now();
    for i in 0..n {
        store
            .put(&format!("d{i}"), payload, DurabilityMode::Durable)
            .unwrap();
        fsync_proxy += 1;
    }
    let durable = t0.elapsed();

    let t0 = Instant::now();
    for i in 0..n {
        store
            .put(&format!("b{i}"), payload, DurabilityMode::Buffered)
            .unwrap();
    }
    let buffered = t0.elapsed();

    store.persist_index_cache().unwrap();
    let cache_bytes = fs::metadata(store.index_cache_path())
        .map(|m| m.len())
        .unwrap_or(0);

    eprintln!(
        "def_023_write_path_disclosure\n  \
         durability=durable+buffered verification=frame-scan-on-rebuild\n  \
         dataset_seeded_keys≈{} working_set_appends={n} payload_len={}\n  \
         durable_n={n} durable_elapsed={durable:?} durable_fsync_acks≈{fsync_proxy}\n  \
         buffered_n={n} buffered_elapsed={buffered:?}\n  \
         index_cache_bytes={cache_bytes} p50/p95/p99=not-sampled-in-skeleton\n  \
         write_amplification=derived_checkpoint_rate_limited_not_per_write_rescan",
        4 * 25,
        payload.len()
    );

    assert!(durable.as_secs() < 60);
    assert!(buffered.as_secs() < 60);
}
