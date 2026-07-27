//! Hydra adaptive per-segment indexes: seal-time sidecar + multithreaded rebuild.

use dingo_store::{
    hydra_index_path, segment_id_from_filename, DurabilityMode, HydraBuildOptions, IndexKind,
    Store, StorePaths,
};

#[test]
fn seal_writes_hydra_sidecar() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    for i in 0..20u64 {
        let k = format!("k{i:02}");
        store
            .put(&k, format!("v{i}").as_bytes(), DurabilityMode::Buffered)
            .unwrap();
    }
    store.seal_active().unwrap();

    let seg_dir = dir.path().join("segments");
    let segs: Vec<_> = std::fs::read_dir(&seg_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("dingo"))
        .collect();
    assert_eq!(segs.len(), 1);
    let seg_id = segment_id_from_filename(&segs[0]).expect("segment id");

    let idx = store
        .load_hydra_index(seg_id)
        .unwrap()
        .expect("hydra sidecar after seal");
    assert_eq!(idx.kind(), IndexKind::Eytzinger);
    assert!(idx.get(b"k05").is_some());
    assert!(idx.get(b"missing").is_none());

    let path = hydra_index_path(&StorePaths::new(dir.path()), &seg_id);
    assert!(path.is_file());
}

#[test]
fn rebuild_hydra_multithread() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    for wave in 0..2u64 {
        for i in 0..30u64 {
            let k = format!("w{wave}-k{i:03}");
            store.put(&k, b"x", DurabilityMode::Buffered).unwrap();
        }
        store.seal_active().unwrap();
    }
    let seg_idx = dir.path().join("indexes").join("seg");
    if seg_idx.is_dir() {
        let _ = std::fs::remove_dir_all(&seg_idx);
    }
    let opts = HydraBuildOptions {
        threads: 2,
        ..Default::default()
    };
    let n = store.rebuild_hydra_indexes(&opts).unwrap();
    assert_eq!(n, 2);

    // Spot-check one rebuilt index.
    let segs: Vec<_> = std::fs::read_dir(dir.path().join("segments"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    let mut found = 0;
    for path in segs {
        let Some(id) = segment_id_from_filename(&path) else {
            continue;
        };
        if let Some(idx) = store.load_hydra_index(id).unwrap() {
            found += 1;
            assert!(!idx.is_empty());
        }
    }
    assert_eq!(found, 2);
}
