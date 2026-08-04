//! CSE-3 Stage 2 step 6 — Recovery Shadow F0–F5 + lifecycle/security matrix.
//!
//! Proves Shadow reproduces Materialized P★ through damage, deletion, and
//! lifecycle transitions. **No performance work. No product flip.**
//!
//! Charter: `doc/todo/performance-qualification/CSE3_STAGE2_STEP6_CSE_MATRIX.md`.

use residiuum_store::{
    build_and_publish_shadow, build_materialized_layout, chimera_layout_path, clear_failpoints,
    contains_plaintext, decode_shadow, delete_shadow, envelope_open, envelope_seal,
    load_protected_coverage, mint_sortable_segment_id, project_live, publish_shadow,
    publish_shadow_claiming_protection, reset_shadow_reclaim_policy_for_tests,
    retire_shadows_after_replacement_with_policy, secure_erase_shadow, set_shadow_reclaim_policy,
    shadow_dir, shadow_path, try_load_shadow, write_chimera_layout, ClassifyOptions, LiveState,
    ProtectedCoverage, ShadowLoad, ShadowReclaimPolicy, ShadowRecord, ShadowWriter, StorePaths,
};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

const KEYS: [&str; 3] = ["t", "m", "l"];

fn body(k: &str) -> Vec<u8> {
    match k {
        "t" => b"tiny-cse3-s2".to_vec(),
        "m" => vec![0x3cu8; 200],
        "l" => vec![0x5au8; 4096],
        _ => panic!("{k}"),
    }
}

fn store_id() -> [u8; 16] {
    [0x11; 16]
}

fn seg(seq: u64) -> [u8; 16] {
    mint_sortable_segment_id(seq, &store_id())
}

fn live_map() -> BTreeMap<Vec<u8>, (Option<Vec<u8>>, u64)> {
    let mut m = BTreeMap::new();
    for (i, k) in KEYS.iter().enumerate() {
        m.insert(k.as_bytes().to_vec(), (Some(body(k)), (i as u64) + 1));
    }
    m
}

fn vs_from_shadow(records: &[ShadowRecord]) -> BTreeMap<Vec<u8>, Vec<u8>> {
    project_live(records)
        .into_iter()
        .filter_map(|(k, st)| match st {
            LiveState::Put { value, .. } => Some((k, value)),
            LiveState::Tombstone { .. } => None,
        })
        .collect()
}

fn vs_from_materialized(pairs: &[(Vec<u8>, Vec<u8>)]) -> BTreeMap<Vec<u8>, Vec<u8>> {
    pairs.iter().cloned().collect()
}

/// F0 — Materialized and Shadow reconstruct identical \(V_S\).
#[test]
fn f0_healthy_materialized_eq_shadow() {
    let pairs: Vec<(Vec<u8>, Vec<u8>)> = KEYS
        .iter()
        .map(|k| (k.as_bytes().to_vec(), body(k)))
        .collect();
    let layout = build_materialized_layout(&pairs, 1, &ClassifyOptions::default());
    let mut mat = BTreeMap::new();
    for k in KEYS {
        mat.insert(
            k.as_bytes().to_vec(),
            layout.get(k.as_bytes()).unwrap().unwrap(),
        );
    }

    let mut w = ShadowWriter::new(store_id(), seg(1), 1);
    for (k, v) in &pairs {
        w.push_put(k.clone(), 1, v.clone());
    }
    let bytes = w.finish();
    let ShadowLoad::Ok(d) = decode_shadow(&bytes, Some(store_id())) else {
        panic!("shadow decode");
    };
    let sh = vs_from_shadow(&d.records);
    assert_eq!(mat, sh);
    assert_eq!(mat, vs_from_materialized(&pairs));
}

/// F1 — deleting Compact/Chimera does not affect Shadow recovery.
#[test]
fn f1_index_loss_shadow_intact() {
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(1);
    build_and_publish_shadow(&paths, store_id(), segment, 1, 0, &live_map()).unwrap();

    let pairs: Vec<(Vec<u8>, Vec<u8>)> = KEYS
        .iter()
        .map(|k| (k.as_bytes().to_vec(), body(k)))
        .collect();
    let layout = build_materialized_layout(&pairs, 1, &ClassifyOptions::default());
    let cmr = chimera_layout_path(&paths, &segment);
    write_chimera_layout(&cmr, store_id(), segment, &layout).unwrap();
    fs::remove_file(&cmr).unwrap();
    assert!(!cmr.is_file());

    let ShadowLoad::Ok(d) =
        try_load_shadow(&shadow_path(&paths, &segment), Some(store_id())).unwrap()
    else {
        panic!("shadow must survive chimera wipe");
    };
    assert_eq!(vs_from_shadow(&d.records).len(), 3);
}

/// F2 — Shadow damage fails closed; does not invent values.
#[test]
fn f2_shadow_damage_fail_closed() {
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(1);
    build_and_publish_shadow(&paths, store_id(), segment, 1, 0, &live_map()).unwrap();
    let path = shadow_path(&paths, &segment);
    let mut bytes = fs::read(&path).unwrap();
    bytes.truncate(bytes.len() - 10);
    fs::write(&path, &bytes).unwrap();
    assert!(matches!(
        try_load_shadow(&path, Some(store_id())).unwrap(),
        ShadowLoad::Incomplete | ShadowLoad::Corrupt { .. }
    ));
    let mut good = ShadowWriter::new(store_id(), segment, 1);
    good.push_put(b"t".to_vec(), 1, body("t"));
    let mut b = good.finish();
    let last = b.len() - 1;
    b[last] ^= 0xff;
    assert!(matches!(
        decode_shadow(&b, Some(store_id())),
        ShadowLoad::Corrupt { .. }
    ));
}

/// F3 — authoritative segment loss: exact values from Shadow.
#[test]
fn f3_authoritative_loss_shadow_reconstructs() {
    clear_failpoints();
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(2);
    build_and_publish_shadow(&paths, store_id(), segment, 1, 0, &live_map()).unwrap();
    let seg_file = paths.sealed_segment(&segment);
    let _ = fs::remove_file(&seg_file);
    let ShadowLoad::Ok(d) =
        try_load_shadow(&shadow_path(&paths, &segment), Some(store_id())).unwrap()
    else {
        panic!("shadow recovery");
    };
    let vs = vs_from_shadow(&d.records);
    for k in KEYS {
        assert_eq!(
            vs.get(k.as_bytes()).map(|v| v.as_slice()),
            Some(body(k).as_slice())
        );
    }
}

/// F4 — overwrites + tombstones: latest generation, no resurrection.
#[test]
fn f4_generation_tombstone_no_resurrection() {
    let records = vec![
        ShadowRecord::Put {
            key: b"t".to_vec(),
            gen: 1,
            value: b"old".to_vec(),
        },
        ShadowRecord::Put {
            key: b"t".to_vec(),
            gen: 2,
            value: b"new".to_vec(),
        },
        ShadowRecord::Tombstone {
            key: b"t".to_vec(),
            gen: 3,
        },
        ShadowRecord::Put {
            key: b"m".to_vec(),
            gen: 1,
            value: body("m"),
        },
    ];
    let live = project_live(&records);
    assert!(matches!(
        live.get(&b"t"[..]),
        Some(LiveState::Tombstone { gen: 3 })
    ));
    assert_eq!(
        live.get(&b"m"[..]),
        Some(&LiveState::Put {
            value: body("m"),
            gen: 1
        })
    );
    assert!(!matches!(live.get(&b"t"[..]), Some(LiveState::Put { .. })));
}

/// F5 — interrupted / partial Shadow never qualifies for P★ (lifecycle crash class).
#[test]
fn f5_lifecycle_partial_shadow_never_p_star() {
    clear_failpoints();
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(3);
    let mut w = ShadowWriter::new(store_id(), segment, 1);
    w.push_put(b"t".to_vec(), 1, body("t"));
    let bytes = w.finish();
    let torn = &bytes[..bytes.len().saturating_sub(20)];
    assert!(publish_shadow(&paths, &segment, torn).is_err());
    let path = shadow_path(&paths, &segment);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, torn).unwrap();
    assert!(matches!(
        try_load_shadow(&path, Some(store_id())).unwrap(),
        ShadowLoad::Incomplete | ShadowLoad::Corrupt { .. }
    ));
    let cov = load_protected_coverage(&paths, store_id()).unwrap();
    assert_eq!(cov.protected_prefix(0), 0);
}

/// Gaps / out-of-order multi-shard completion.
#[test]
fn variant_multi_shard_gaps() {
    let mut cov = ProtectedCoverage::empty(store_id());
    for seq in [1u64, 2, 4] {
        cov.note_sealed(0, seq);
        cov.note_durable(0, seq);
    }
    cov.note_sealed(0, 3);
    assert_eq!(cov.protected_prefix(0), 2);

    for seq in 1..=5u64 {
        cov.note_sealed(1, seq);
    }
    for seq in [1u64, 2, 5] {
        cov.note_durable(1, seq);
    }
    assert_eq!(cov.protected_prefix(1), 2);
    assert_eq!(cov.aggregate_protected_prefix(), 2);
}

/// Wrong-store / wrong-segment substitution fails closed.
#[test]
fn variant_wrong_store_segment_substitution() {
    let mut w = ShadowWriter::new(store_id(), seg(1), 1);
    w.push_put(b"t".to_vec(), 1, body("t"));
    let bytes = w.finish();
    let other_store = [0x22; 16];
    assert!(matches!(
        decode_shadow(&bytes, Some(other_store)),
        ShadowLoad::Corrupt { .. }
    ));
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    assert!(publish_shadow(&paths, &seg(99), &bytes).is_err());
}

/// Backup → restore → total auth loss still recovers from Shadow bytes.
#[test]
fn variant_backup_restore_auth_loss() {
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(5);
    build_and_publish_shadow(&paths, store_id(), segment, 1, 0, &live_map()).unwrap();
    let shadow_bytes = fs::read(shadow_path(&paths, &segment)).unwrap();

    let backup = dir.path().join("backup.rsh");
    fs::write(&backup, &shadow_bytes).unwrap();
    fs::remove_dir_all(paths.recovery_dir()).unwrap();
    paths.create_dirs().unwrap();
    let dest = shadow_path(&paths, &segment);
    fs::create_dir_all(shadow_dir(&paths)).unwrap();
    fs::write(&dest, fs::read(&backup).unwrap()).unwrap();

    let ShadowLoad::Ok(d) = try_load_shadow(&dest, Some(store_id())).unwrap() else {
        panic!("restored shadow");
    };
    assert_eq!(vs_from_shadow(&d.records).len(), 3);
}

/// Encrypted Shadow contains no plaintext payload.
#[test]
fn variant_encrypted_no_plaintext() {
    let key = *blake3::hash(b"cse3-shadow-key").as_bytes();
    let plain = b"must-not-appear-in-ciphertext-body";
    let sealed = envelope_seal(&key, 42, plain);
    assert!(!contains_plaintext(&sealed, plain));

    let mut w = ShadowWriter::new(store_id(), seg(6), 1);
    w.push_put(b"secret".to_vec(), 1, sealed.clone());
    let file = w.finish();
    assert!(!contains_plaintext(&file, plain));
    let ShadowLoad::Ok(d) = decode_shadow(&file, Some(store_id())) else {
        panic!("decode");
    };
    let LiveState::Put { value, .. } = project_live(&d.records)
        .remove(b"secret".as_slice())
        .unwrap()
    else {
        panic!("put");
    };
    let (_kid, opened) = envelope_open(&key, &value).unwrap();
    assert_eq!(opened, plain);
}

/// Key rotation preserves recovery (reseal under new key).
#[test]
fn variant_key_rotation_preserves_recovery() {
    let k1 = *blake3::hash(b"key-v1").as_bytes();
    let k2 = *blake3::hash(b"key-v2").as_bytes();
    let plain = body("l");
    let e1 = envelope_seal(&k1, 1, &plain);
    let (_id, p) = envelope_open(&k1, &e1).unwrap();
    let e2 = envelope_seal(&k2, 2, &p);
    assert!(!contains_plaintext(&e2, &plain));
    let (_id2, p2) = envelope_open(&k2, &e2).unwrap();
    assert_eq!(p2, plain);
    assert!(envelope_open(&k1, &e2).is_err());
}

/// Cryptographic erase makes retired Shadow unrecoverable.
#[test]
fn variant_crypto_erase_unrecoverable() {
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(7);
    build_and_publish_shadow(&paths, store_id(), segment, 1, 0, &live_map()).unwrap();
    let path = shadow_path(&paths, &segment);
    secure_erase_shadow(&paths, store_id(), &segment, 0).unwrap();
    assert!(!path.is_file());
    assert!(matches!(
        try_load_shadow(&path, Some(store_id())).unwrap(),
        ShadowLoad::Missing
    ));
}

/// Compaction cannot retire last valid recovery source (post-flip policy).
#[test]
fn variant_compaction_cannot_retire_last_source() {
    reset_shadow_reclaim_policy_for_tests();
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let old = seg(8);
    let repl = seg(9);
    build_and_publish_shadow(&paths, store_id(), old, 1, 0, &live_map()).unwrap();
    set_shadow_reclaim_policy(ShadowReclaimPolicy::RequireReplacementShadow);
    let err = retire_shadows_after_replacement_with_policy(
        &paths,
        store_id(),
        &repl,
        &[old],
        0,
        ShadowReclaimPolicy::RequireReplacementShadow,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("replacement") || msg.contains("recovery"));
    assert!(shadow_path(&paths, &old).is_file());
    reset_shadow_reclaim_policy_for_tests();
}

/// Dual-run clarification: Materialized authority allows retire without replacement Shadow.
#[test]
fn clarification_dual_run_allows_materialized_authority() {
    reset_shadow_reclaim_policy_for_tests();
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let old = seg(10);
    let repl = seg(11);
    build_and_publish_shadow(&paths, store_id(), old, 1, 0, &live_map()).unwrap();
    retire_shadows_after_replacement_with_policy(
        &paths,
        store_id(),
        &repl,
        &[old],
        0,
        ShadowReclaimPolicy::DualRunMaterializedAuthority,
    )
    .unwrap();
    assert!(!shadow_path(&paths, &old).is_file());
}

/// Claiming protection after successful publish advances gap-aware frontier.
#[test]
fn f5_frontier_update_after_publish() {
    let dir = tempdir().unwrap();
    let paths = StorePaths::new(dir.path());
    paths.create_dirs().unwrap();
    let segment = seg(1);
    let mut w = ShadowWriter::new(store_id(), segment, 1);
    w.push_put(b"t".to_vec(), 1, body("t"));
    let bytes = w.finish();
    publish_shadow_claiming_protection(&paths, store_id(), &segment, 0, &bytes).unwrap();
    let cov = load_protected_coverage(&paths, store_id()).unwrap();
    assert_eq!(cov.protected_prefix(0), 1);
    delete_shadow(&paths, store_id(), &segment, 0).unwrap();
    let cov2 = load_protected_coverage(&paths, store_id()).unwrap();
    assert_eq!(cov2.protected_prefix(0), 0);
}
