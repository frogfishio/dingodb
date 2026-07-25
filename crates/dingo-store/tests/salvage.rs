//! Stage 3: catalog-independent recovery and incomplete-tail safety.
//!
//! OVERVIEW §4.5 / §6.1 / §8.5 / §16 case 7 (destroy catalogs/indexes).

use dingo_store::{DurabilityMode, Store};
use std::fs::{self, OpenOptions};
use std::io::Write;
use tempfile::tempdir;

#[test]
fn destroy_catalogs_and_indexes_still_recovers() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store.put("alpha", b"A", DurabilityMode::Durable).unwrap();
        store.put("beta", b"B", DurabilityMode::Durable).unwrap();
        store.delete("alpha", DurabilityMode::Durable).unwrap();
        // Optional derived dirs may be empty; write junk so deletion is meaningful.
        for d in store.derived_dirs() {
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join("junk.idx"), b"not-authoritative").unwrap();
        }
    }

    // Wipe every derived directory (OVERVIEW §16.7).
    for name in ["catalogs", "indexes", "snapshots"] {
        let p = path.join(name);
        if p.exists() {
            fs::remove_dir_all(&p).unwrap();
        }
    }

    let store = Store::open(&path).unwrap();
    assert!(store.get("alpha").unwrap().is_none());
    assert_eq!(store.get("beta").unwrap().as_deref(), Some(b"B".as_slice()));

    let report = store.salvage().unwrap();
    assert!(report.files_scanned >= 1);
    assert!(report.item_events >= 3);
    assert_eq!(report.live_subjects, 1);
}

#[test]
fn incomplete_tail_does_not_poison_earlier_frames() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store
            .put("keep", b"alive", DurabilityMode::Durable)
            .unwrap();
    }

    // Corrupt the active segment with a truncated / garbage tail.
    let active = path.join("active").join("active.dingo");
    let mut f = OpenOptions::new().append(true).open(&active).unwrap();
    // Partial magic + noise (not a complete frame).
    f.write_all(b"DINGOFRM").unwrap();
    f.write_all(&[0u8; 40]).unwrap();
    f.sync_all().unwrap();

    let store = Store::open(&path).unwrap();
    assert_eq!(
        store.get("keep").unwrap().as_deref(),
        Some(b"alive".as_slice()),
        "earlier complete frames must remain readable after incomplete tail"
    );

    // Salvage should still count the surviving item event.
    let report = store.salvage().unwrap();
    assert!(report.item_events >= 1);
    assert_eq!(report.live_subjects, 1);
}

#[test]
fn salvage_to_new_path_is_non_destructive() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src");
    let dst = dir.path().join("dst");
    {
        let mut store = Store::create(&src).unwrap();
        store
            .put("keep", b"alive", DurabilityMode::Durable)
            .unwrap();
        store.put("gone", b"x", DurabilityMode::Durable).unwrap();
        store.delete("gone", DurabilityMode::Durable).unwrap();
    }
    let active_meta = fs::metadata(src.join("active").join("active.dingo"))
        .unwrap()
        .modified()
        .unwrap();

    let src_store = Store::open_inspect(&src).unwrap();
    let report = src_store.salvage_to(&dst).unwrap();
    assert_eq!(report.subjects_copied, 1);
    assert_eq!(report.source.live_subjects, 1);
    assert_eq!(report.mode, dingo_store::SalvageMode::Evidence);
    assert!(report.frames_copied >= 3, "put/put/delete item frames");
    assert!(report.manifest_path.is_some());

    // Source active segment mtime unchanged.
    let after = fs::metadata(src.join("active").join("active.dingo"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(active_meta, after);

    let recovered = Store::open(&dst).unwrap();
    assert_eq!(
        recovered.get("keep").unwrap().as_deref(),
        Some(b"alive".as_slice())
    );
    assert!(recovered.get("gone").unwrap().is_none());
}

#[test]
fn rebuild_index_matches_get() {
    let dir = tempdir().unwrap();
    let mut store = Store::create(dir.path()).unwrap();
    store.put("s1", b"one", DurabilityMode::Durable).unwrap();
    store.put("s2", b"two", DurabilityMode::Durable).unwrap();
    store.seal_active().unwrap();
    store.put("s3", b"three", DurabilityMode::Durable).unwrap();

    store.rebuild_index().unwrap();
    assert_eq!(store.get("s1").unwrap().as_deref(), Some(b"one".as_slice()));
    assert_eq!(store.get("s2").unwrap().as_deref(), Some(b"two".as_slice()));
    assert_eq!(
        store.get("s3").unwrap().as_deref(),
        Some(b"three".as_slice())
    );
    assert_eq!(store.live_count(), 3);
}

#[test]
fn middle_byte_corruption_still_finds_later_items_via_salvage() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_path_buf();
    {
        let mut store = Store::create(&path).unwrap();
        store.put("early", b"1", DurabilityMode::Durable).unwrap();
        store.seal_active().unwrap();
        store.put("late", b"2", DurabilityMode::Durable).unwrap();
    }

    // Corrupt a sealed segment's middle bytes (OVERVIEW §16.2 style).
    let segments = path.join("segments");
    let seg_file = fs::read_dir(&segments)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|x| x.to_str()) == Some("dingo"))
        .expect("sealed segment");
    let mut bytes = fs::read(&seg_file).unwrap();
    if bytes.len() > 80 {
        // Flip a body-ish region without destroying every magic.
        let i = bytes.len() / 2;
        bytes[i] ^= 0xff;
        fs::write(&seg_file, &bytes).unwrap();
    }

    let store = Store::open(&path).unwrap();
    // Late item is on a different (active) segment and must survive.
    assert_eq!(store.get("late").unwrap().as_deref(), Some(b"2".as_slice()));

    let report = store.salvage().unwrap();
    assert!(
        report.verified_frames >= 1,
        "salvage must still find verified islands"
    );
}
