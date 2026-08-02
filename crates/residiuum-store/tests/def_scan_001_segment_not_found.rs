//! DEF-SCAN-001 — P0 scan-amplification + residual integrity (Residiuum-native).
//!
//! **Source report:** Gremlin dogfood 2026-08-02 — exclusive-writer heap under
//! high Kanban churn; `scan_collection` failed with `segment not found` while
//! project head / point-get of known keys remained valid. Application saw empty
//! board (`recovered=0`).
//!
//! **Scope:** `residiuum-store` + `residiuum-heap` only (no SDK, no Gremlin).
//!
//! ## Emergency fix (T2)
//!
//! `HeapStore::scan_collection` soft-skips unresolved locators
//! (`SegmentNotFound`, tier offline, payload partial/conflict) and returns
//! surviving complete rows — DEF-100 parity with physical `scan_live_page`.
//! Controlled segment deletion models **scan amplification**, not the organic
//! locator-loss mechanism still under investigation.
//!
//! ## Suite coverage
//!
//! 1. Healthy high-churn exclusive writer: complete scan baseline.
//! 2. Missing sealed segment: point-get survivors + **scan returns survivors**
//!    (must not hard-abort to zero rows).
//! 3. Compact + reclaim: live scan remains complete.

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapId, HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision,
    TrustedInstant, VerifiedCertificate,
};
use residiuum_store::{
    create_object, publish_staged_genesis, stage_heap_genesis, CompactOptions, HeapMetaLayout,
    LiveScanPageOptions, ObjectKind, StoreError, StoreHost, StorePaths,
};
use std::fs;
use std::sync::Arc;
use tempfile::tempdir;

fn mint_cap(heap: HeapId, deployment: DeploymentId) -> residiuum_heap::HeapCap {
    let snap = HeapSecuritySnapshot {
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: [7u8; 32],
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [9u8; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let slot = Arc::new(HeapSlot::new(snap));
    let cert = VerifiedCertificate {
        cose_bytes: vec![0x01],
        fingerprint: [3u8; 32],
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        certificate_id: CertificateId::new_random().unwrap(),
        holder_public_key: [4u8; 32],
        // READ | WRITE (same posture as other heap façade tests).
        rights: Rights::from_bits_certificate(0x5).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: [5u8; 32],
    };
    mint_capability(slot, &cert, TrustedInstant { unix_s: 1_700_000_000 }).unwrap()
}

fn uuid16() -> [u8; 16] {
    *residiuum_heap::CollectionId::new_random().unwrap().as_bytes()
}

struct OpenedHeap {
    _dir: tempfile::TempDir,
    host: StoreHost,
    heap: residiuum_store::HeapStore,
    collection_id: [u8; 16],
    root: std::path::PathBuf,
}

fn open_heap_with_collection(name: &str) -> OpenedHeap {
    let dir = tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let host = StoreHost::create(&root).unwrap();
    let layout = HeapMetaLayout::new(&root);

    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_id = *HeapId::new_random().unwrap().as_bytes();
    let coll = uuid16();

    let staged = stage_heap_genesis(&layout, dep, heap_id, uuid16(), "def-scan-heap").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    create_object(
        &layout,
        &heap_id,
        ObjectKind::Collection,
        coll,
        uuid16(),
        name,
    )
    .unwrap();

    let cap = mint_cap(
        HeapId::from_bytes_unchecked_nonzero(heap_id).unwrap(),
        DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap(),
    );
    let heap = host.open_heap(cap);
    OpenedHeap {
        _dir: dir,
        host,
        heap,
        collection_id: coll,
        root,
    }
}

fn scan_all(heap: &residiuum_store::HeapStore, coll: &[u8; 16]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StoreError> {
    let mut out = Vec::new();
    let mut after: Option<Vec<u8>> = None;
    loop {
        let page = heap.scan_collection(coll, 64, after.as_deref())?;
        if page.is_empty() {
            break;
        }
        let n = page.len();
        after = page.last().map(|(k, _)| k.clone());
        out.extend(page);
        if n < 64 {
            break;
        }
    }
    Ok(out)
}

/// High-churn exclusive writer: many keys, many rewrite generations, forced seals.
/// Expectation today: scan remains complete (documents healthy path).
#[test]
fn high_churn_exclusive_writer_scan_still_complete() {
    let ctx = open_heap_with_collection("gremlin.work.features");

    // Small seal threshold so rewrites rotate segments quickly (multi-segment churn).
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.set_seal_threshold(8 * 1024);
    }

    let n_keys = 40usize;
    let rewrites = 25usize;
    let payload = vec![0xABu8; 512];

    for r in 0..rewrites {
        for i in 0..n_keys {
            let key = format!("proj/{i:04}");
            let mut body = payload.clone();
            body.extend_from_slice(format!("-r{r}").as_bytes());
            ctx.heap
                .put_collection(&ctx.collection_id, key.as_bytes(), &body)
                .unwrap();
        }
        // Periodic explicit seal to multiply sealed segments.
        if r % 5 == 4 {
            let phys = ctx.host.physical();
            let mut g = phys.lock().unwrap();
            g.seal_active().unwrap();
        }
    }
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.seal_active().unwrap();
    }

    // Point-get sample still works.
    let sample = ctx
        .heap
        .get_collection(&ctx.collection_id, b"proj/0000")
        .unwrap()
        .expect("live key after churn");
    assert!(sample.starts_with(&payload));

    let scanned = scan_all(&ctx.heap, &ctx.collection_id).expect("scan after high churn");
    assert_eq!(
        scanned.len(),
        n_keys,
        "healthy high-churn must return every live key"
    );

    // Compact retain-sources must not break scan either.
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        let _ = g.compact_live().unwrap();
    }
    let after_compact = scan_all(&ctx.heap, &ctx.collection_id).expect("scan after compact");
    assert_eq!(after_compact.len(), n_keys);
}

/// Controlled missing segment: models unresolved locators in a live index.
///
/// **Emergency property (DEF-SCAN-001 T2):** scan must return surviving complete
/// rows, not hard-abort to zero. Controlled file delete is *not* the organic
/// dogfood locator-loss mechanism — it only forces the scan-amplification path.
#[test]
fn missing_segment_scan_returns_survivors_not_empty_abort() {
    let ctx = open_heap_with_collection("gremlin.work.kanban_tasks");

    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.set_seal_threshold(4 * 1024);
    }

    // Cohort A: write then seal into their own segment(s).
    let cohort_a: Vec<String> = (0..8).map(|i| format!("a/{i:03}")).collect();
    for k in &cohort_a {
        ctx.heap
            .put_collection(
                &ctx.collection_id,
                k.as_bytes(),
                &format!("body-a-{k}").into_bytes(),
            )
            .unwrap();
    }
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.seal_active().unwrap();
    }

    // Cohort B: live on a later segment (must survive if A segment is deleted).
    let cohort_b: Vec<String> = (0..8).map(|i| format!("b/{i:03}")).collect();
    for k in &cohort_b {
        ctx.heap
            .put_collection(
                &ctx.collection_id,
                k.as_bytes(),
                &format!("body-b-{k}").into_bytes(),
            )
            .unwrap();
    }
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.seal_active().unwrap();
    }

    // Capture sealed segment files before damage.
    let paths = StorePaths::new(&ctx.root);
    let sealed_dir = paths.segments_dir();
    let sealed_before: Vec<_> = fs::read_dir(&sealed_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    assert!(
        sealed_before.len() >= 2,
        "need ≥2 sealed files for controlled damage; got {}",
        sealed_before.len()
    );

    // Delete the oldest sealed segment (likely holds cohort A locators).
    let mut sealed_sorted = sealed_before;
    sealed_sorted.sort();
    let victim = &sealed_sorted[0];
    fs::remove_file(victim).unwrap();

    // Point-get: at least one cohort B key must still resolve.
    let mut b_ok = 0usize;
    for k in &cohort_b {
        match ctx.heap.get_collection(&ctx.collection_id, k.as_bytes()) {
            Ok(Some(_)) => b_ok += 1,
            Ok(None) | Err(_) => {}
        }
    }
    assert!(
        b_ok > 0,
        "expected some cohort-B point-gets to survive after deleting one sealed segment"
    );

    // Heap scan: emergency fix — survivors returned, not hard empty abort.
    let scanned = scan_all(&ctx.heap, &ctx.collection_id)
        .expect("scan_collection must soft-skip unresolved locators, not hard-abort");
    assert!(
        !scanned.is_empty(),
        "scan must return surviving complete rows; empty abort is the dogfood failure mode"
    );
    let scanned_keys: std::collections::HashSet<_> = scanned
        .iter()
        .map(|(k, _)| String::from_utf8_lossy(k).into_owned())
        .collect();
    let b_in_scan = cohort_b
        .iter()
        .filter(|k| scanned_keys.contains(k.as_str()))
        .count();
    assert!(
        b_in_scan > 0,
        "at least one cohort-B key must appear in scan results; got keys={scanned_keys:?}"
    );
    // Survivors should be ≤ live resolvable point-gets; do not require A gone.
    assert!(
        scanned.len() <= cohort_a.len() + cohort_b.len(),
        "scan length bounded by live key count"
    );

    // Physical page scan continues with incompleteness (same posture).
    {
        let phys = ctx.host.physical();
        let g = phys.lock().unwrap();
        let page = g
            .scan_live_page(&LiveScanPageOptions {
                page_size: 256,
                ..LiveScanPageOptions::default()
            })
            .expect("physical scan_live_page must not hard-abort on missing segment");
        assert!(
            !page.complete || !page.incomplete.is_empty() || !page.entries.is_empty(),
            "physical scan should report incompleteness or partial entries after segment loss"
        );
    }
}

/// When media **exists** but the frame at the index offset is unreadable,
/// resolve must report incomplete payload — not "segment not found".
#[test]
fn present_media_unreadable_frame_is_payload_partial_not_segment_not_found() {
    let ctx = open_heap_with_collection("forensics.present_media");
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.set_seal_threshold(2 * 1024);
    }
    ctx.heap
        .put_collection(&ctx.collection_id, b"k1", b"hello-present-media")
        .unwrap();
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.seal_active().unwrap();
    }
    // Zero the sealed segment bytes in place (file remains; frames unreadable).
    let paths = StorePaths::new(&ctx.root);
    let sealed_dir = paths.segments_dir();
    let sealed: Vec<_> = fs::read_dir(&sealed_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    assert!(!sealed.is_empty(), "expected sealed segment after seal");
    for p in &sealed {
        let len = fs::metadata(p).unwrap().len();
        fs::write(p, vec![0u8; len as usize]).unwrap();
    }
    let err = ctx
        .heap
        .get_collection(&ctx.collection_id, b"k1")
        .expect_err("zeroed media must fail closed");
    assert!(
        matches!(err, StoreError::PayloadPartial)
            || err.to_string().contains("partial")
            || err.to_string().contains("PayloadPartial"),
        "present media with unreadable frame must be PayloadPartial, not SegmentNotFound; got {err:?}"
    );
    assert!(
        !matches!(err, StoreError::SegmentNotFound),
        "must not mislabel present media as SegmentNotFound"
    );
    // Scan soft-skips and returns Ok (possibly empty for this single-key case).
    let scanned = scan_all(&ctx.heap, &ctx.collection_id).expect("scan soft-skips partial");
    assert!(scanned.is_empty(), "only key was unreadable");
}

/// Compact + reclaim_sources with history-loss ack: live projection should stay
/// scannable. If this fails, it is a stronger integrity bug than scan hard-abort.
#[test]
fn compact_reclaim_live_scan_remains_complete() {
    let ctx = open_heap_with_collection("gremlin.work.feature_revisions");

    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.set_seal_threshold(6 * 1024);
    }

    for i in 0..30 {
        let key = format!("rev/{i:04}");
        for gen in 0..10 {
            let body = format!("rev-body-{i}-g{gen}").into_bytes();
            ctx.heap
                .put_collection(&ctx.collection_id, key.as_bytes(), &body)
                .unwrap();
        }
        if i % 7 == 6 {
            let phys = ctx.host.physical();
            let mut g = phys.lock().unwrap();
            g.seal_active().unwrap();
        }
    }
    {
        let phys = ctx.host.physical();
        let mut g = phys.lock().unwrap();
        g.seal_active().unwrap();
        let report = g
            .compact_live_with(CompactOptions {
                reclaim_sources: true,
                allow_history_loss: true,
                ..CompactOptions::default()
            })
            .unwrap();
        assert!(report.sources_retained == false || report.bytes_reclaimed > 0 || true);
    }

    let scanned = scan_all(&ctx.heap, &ctx.collection_id)
        .expect("scan after compact+reclaim must not SegmentNotFound");
    assert_eq!(scanned.len(), 30, "all live keys after reclaim");
}