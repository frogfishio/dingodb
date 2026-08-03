//! AWO-Q1.3 — Concurrent façade admit + clean reopen correctness.
//!
//! Proves under genuine concurrent `HeapStore` callers (Static + Adaptive):
//! - every acknowledged seq binds to key + body hash + length;
//! - clean close + normal product reopen preserves every ack exactly once;
//! - pre-close value digest matches reopened-state digest;
//! - multi-seed / concurrency / outstanding matrix + optional segment rotation;
//! - records file_sync / logical_ack evidence for claim table.
//!
//! Crash-boundary semantics are **out of scope** (later crash matrix).
//!
//! ```text
//! cargo test -p residiuum-store --features legacy-raw-store --test awo_q13_reopen_correctness -- --test-threads=1
//! ```

use residiuum_heap::{
    mint_capability, AuthorityEpoch, AuthorityGeneration, CertificateId, Constraints, DeploymentId,
    HeapAdministrativeState, HeapCap, HeapId, HeapSecuritySnapshot, HeapSlot, Rights,
    SecurityRevision, TrustedInstant, VerifiedCertificate,
};
use residiuum_store::adaptive_write::{AdaptiveWriteMode, AdaptiveWritePolicy};
use residiuum_store::{
    create_object, publish_staged_genesis, stage_heap_genesis, BoundaryKind, HeapMetaLayout,
    ObjectKind, StoreHost,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::time::{Duration, Instant};

/// Expected binding for one logical put (authoritative oracle).
#[derive(Clone, Debug)]
struct ExpectedOp {
    seq: u64,
    key: Vec<u8>,
    body: Vec<u8>,
    body_hash: [u8; 32],
}

fn fill_body(seed: u64, seq: u64, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len.max(1)];
    let mut state = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(seq.wrapping_mul(0xBF58_476D_1CE4_E5B9));
    for b in &mut out {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        *b = (state >> 33) as u8;
    }
    out
}

fn body_hash(body: &[u8]) -> [u8; 32] {
    *blake3::hash(body).as_bytes()
}

/// Chain digest over (seq, key_bytes, body_hash, len) — independent of store.
fn chain_digest(ops: &[(u64, &[u8], [u8; 32], u64)]) -> String {
    let mut prev: Option<[u8; 32]> = None;
    for (seq, key, hash, len) in ops {
        let mut h = blake3::Hasher::new();
        if let Some(p) = prev {
            h.update(&p);
        }
        h.update(&seq.to_le_bytes());
        h.update(key);
        h.update(hash);
        h.update(&len.to_le_bytes());
        prev = Some(*h.finalize().as_bytes());
    }
    prev.map(|p| hex_encode(&p)).unwrap_or_else(|| "empty".into())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

/// Global in-flight credit (outstanding is shared, not per-worker).
struct GlobalOutstanding {
    available: Mutex<usize>,
    capacity: usize,
}

impl GlobalOutstanding {
    fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            available: Mutex::new(capacity),
            capacity,
        }
    }

    fn acquire(&self) {
        loop {
            {
                let mut g = self.available.lock().unwrap();
                if *g > 0 {
                    *g -= 1;
                    return;
                }
            }
            std::thread::yield_now();
        }
    }

    fn release(&self) {
        let mut g = self.available.lock().unwrap();
        *g = (*g).saturating_add(1).min(self.capacity);
    }
}

struct GenesisIds {
    dep: [u8; 16],
    heap_id: [u8; 16],
    coll: [u8; 16],
    cap: HeapCap,
}

fn mint_cap(heap: HeapId, deployment: DeploymentId) -> HeapCap {
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
        rights: Rights::from_bits_certificate(Rights::READ.bits() | Rights::WRITE.bits()).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: [5u8; 32],
    };
    mint_capability(slot, &cert, TrustedInstant { unix_s: 1_700_000_000 }).unwrap()
}

fn provision(root: &Path) -> GenesisIds {
    let layout = HeapMetaLayout::new(root);
    let dep = *DeploymentId::new_random().unwrap().as_bytes();
    let heap_id = *HeapId::new_random().unwrap().as_bytes();
    let coll = *residiuum_heap::CollectionId::new_random()
        .unwrap()
        .as_bytes();
    let staged = stage_heap_genesis(&layout, dep, heap_id, [1u8; 16], "q13-heap").unwrap();
    publish_staged_genesis(&layout, &staged.staging_id, &staged.descriptor_hash).unwrap();
    create_object(
        &layout,
        &heap_id,
        ObjectKind::Collection,
        coll,
        [2u8; 16],
        "q13.collection",
    )
    .unwrap();
    let cap = mint_cap(
        HeapId::from_bytes_unchecked_nonzero(heap_id).unwrap(),
        DeploymentId::from_bytes_unchecked_nonzero(dep).unwrap(),
    );
    GenesisIds {
        dep,
        heap_id,
        coll,
        cap,
    }
}

fn policy_for(mode: AdaptiveWriteMode) -> AdaptiveWritePolicy {
    let mut policy = AdaptiveWritePolicy::machine_defaults();
    policy.mode = mode;
    policy.maximum_cookers = 2;
    policy.minimum_active_cookers = 1;
    policy.maximum_collection_delay = Duration::from_millis(8);
    policy
}

#[derive(Debug)]
struct CellReport {
    mode: &'static str,
    seed: u64,
    workers: usize,
    outstanding: usize,
    issued: u64,
    acked: u64,
    pre_digest: String,
    reopen_digest: String,
    file_sync: u64,
    appends: u64,
    segment_rotate: u64,
    sync_per_logical_ack: f64,
}

/// Run one correctness cell: concurrent façade puts → drain → close → reopen → verify.
fn run_cell(
    mode: AdaptiveWriteMode,
    seed: u64,
    workers: usize,
    outstanding: usize,
    total_ops: u64,
    force_rotate: bool,
    payload_len: usize,
) -> CellReport {
    let dir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().to_path_buf();
    let policy = policy_for(mode);
    let host = StoreHost::create_with_adaptive_write(&root, policy.clone()).unwrap();
    assert!(
        host.adaptive_write().unwrap().lease_active(),
        "AWO lease must be active for concurrent collection"
    );
    let ids = provision(&root);
    let coll = ids.coll;
    let heap = Arc::new(host.open_heap(ids.cap.clone()));

    {
        let physical = host.physical();
        let mut g = physical.lock().unwrap();
        g.enable_boundary_probe();
        if force_rotate {
            // Small seal threshold so Durable volume triggers SegmentRotate mid-run.
            g.set_seal_threshold(16 * 1024);
        }
    }

    // Authoritative expected table for every issued seq.
    let expected: Arc<Vec<ExpectedOp>> = Arc::new(
        (0..total_ops)
            .map(|seq| {
                let key = format!("q13-s{seed:04x}-{seq:08x}").into_bytes();
                let body = fill_body(seed, seq, payload_len);
                let hash = body_hash(&body);
                ExpectedOp {
                    seq,
                    key,
                    body,
                    body_hash: hash,
                }
            })
            .collect(),
    );

    let next = Arc::new(AtomicU64::new(0));
    let credits = Arc::new(GlobalOutstanding::new(outstanding));
    // seq → terminal ack recorded once (detect multi-terminal).
    let acked: Arc<Mutex<BTreeMap<u64, ()>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let barrier = Arc::new(Barrier::new(workers));

    let mut joins = Vec::new();
    for _ in 0..workers {
        let heap = Arc::clone(&heap);
        let expected = Arc::clone(&expected);
        let next = Arc::clone(&next);
        let credits = Arc::clone(&credits);
        let acked = Arc::clone(&acked);
        let barrier = Arc::clone(&barrier);
        joins.push(std::thread::spawn(move || {
            barrier.wait();
            loop {
                let seq = next.fetch_add(1, Ordering::Relaxed);
                if seq >= total_ops {
                    break;
                }
                credits.acquire();
                let op = &expected[seq as usize];
                heap.put_collection(&coll, &op.key, &op.body)
                    .unwrap_or_else(|e| panic!("put_collection seq={seq}: {e}"));
                {
                    let mut g = acked.lock().unwrap();
                    assert!(
                        g.insert(seq, ()).is_none(),
                        "double-ack for seq={seq}"
                    );
                }
                credits.release();
            }
        }));
    }
    for j in joins {
        j.join().expect("worker panic invalidates evidence");
    }

    let issued = next.load(Ordering::Relaxed).min(total_ops);
    assert_eq!(issued, total_ops, "all sequences must be issued");
    {
        let g = acked.lock().unwrap();
        assert_eq!(g.len() as u64, total_ops, "every issued seq must ack exactly once");
        for s in 0..total_ops {
            assert!(g.contains_key(&s), "missing ack for seq={s}");
        }
    }

    // Pre-close: verify live façade + build digest from store-read values.
    let mut pre_chain: Vec<(u64, Vec<u8>, [u8; 32], u64)> = Vec::with_capacity(total_ops as usize);
    for op in expected.iter() {
        let got = heap
            .get_collection(&coll, &op.key)
            .expect("get")
            .unwrap_or_else(|| panic!("missing key before close seq={}", op.seq));
        assert_eq!(
            got, op.body,
            "pre-close body mismatch seq={}",
            op.seq
        );
        assert_eq!(body_hash(&got), op.body_hash);
        assert_eq!(got.len() as u64, op.body.len() as u64);
        pre_chain.push((op.seq, op.key.clone(), op.body_hash, op.body.len() as u64));
    }
    let pre_refs: Vec<(u64, &[u8], [u8; 32], u64)> = pre_chain
        .iter()
        .map(|(s, k, h, l)| (*s, k.as_slice(), *h, *l))
        .collect();
    let pre_digest = chain_digest(&pre_refs);

    let (file_sync, appends, segment_rotate) = {
        let physical = host.physical();
        let g = physical.lock().unwrap();
        let snap = g.boundary_snapshot();
        (
            snap.counters.count(BoundaryKind::FileSync),
            snap.counters.count(BoundaryKind::AppendEncodedFrame),
            snap.counters.count(BoundaryKind::SegmentRotate),
        )
    };

    host.drain_writes(Instant::now() + Duration::from_secs(3))
        .expect("drain");
    drop(heap);
    drop(host);

    // Normal product reopen path (not crash salvage).
    let host2 = StoreHost::open_with_adaptive_write(&root, policy).unwrap();
    let heap2 = host2.open_heap(ids.cap);
    let mut reopen_chain: Vec<(u64, Vec<u8>, [u8; 32], u64)> =
        Vec::with_capacity(total_ops as usize);
    for op in expected.iter() {
        let got = heap2
            .get_collection(&coll, &op.key)
            .expect("reopen get")
            .unwrap_or_else(|| panic!("missing after reopen seq={}", op.seq));
        assert_eq!(
            got, op.body,
            "reopen body mismatch seq={}",
            op.seq
        );
        let h = body_hash(&got);
        assert_eq!(h, op.body_hash, "reopen hash mismatch seq={}", op.seq);
        reopen_chain.push((op.seq, op.key.clone(), h, got.len() as u64));
    }
    // No unexpected keys: collection scan not required if we only wrote known keys
    // and every known key matches; extra keys would not affect digest equality.
    let reopen_refs: Vec<(u64, &[u8], [u8; 32], u64)> = reopen_chain
        .iter()
        .map(|(s, k, h, l)| (*s, k.as_slice(), *h, *l))
        .collect();
    let reopen_digest = chain_digest(&reopen_refs);
    assert_eq!(
        pre_digest, reopen_digest,
        "pre-close digest must equal reopen digest (mode={mode:?} seed={seed})"
    );

    let logical_ack = total_ops;
    let sync_per = if logical_ack > 0 {
        file_sync as f64 / logical_ack as f64
    } else {
        0.0
    };
    // Concurrent Durable collection should amortize syncs when outstanding/workers > 1.
    if workers > 1 && outstanding > 1 && total_ops >= 16 {
        assert!(
            file_sync > 0 && file_sync < logical_ack,
            "expected multi-write barrier amortization: file_sync={file_sync} acks={logical_ack}"
        );
    }
    assert!(appends >= logical_ack, "append count {appends} < acks {logical_ack}");
    if force_rotate {
        assert!(
            segment_rotate > 0,
            "force_rotate cell expected SegmentRotate>0"
        );
    }

    host2
        .drain_writes(Instant::now() + Duration::from_secs(2))
        .ok();

    CellReport {
        mode: match mode {
            AdaptiveWriteMode::Static => "static",
            AdaptiveWriteMode::Adaptive => "adaptive",
            AdaptiveWriteMode::Disabled => "disabled",
        },
        seed,
        workers,
        outstanding,
        issued: total_ops,
        acked: logical_ack,
        pre_digest,
        reopen_digest,
        file_sync,
        appends,
        segment_rotate,
        sync_per_logical_ack: sync_per,
    }
}

#[test]
fn q13_static_concurrent_reopen_matrix() {
    let cells = [
        // seed, workers, outstanding, ops, rotate, payload
        (1u64, 4usize, 2usize, 24u64, false, 128usize),
        (42, 4, 4, 32, false, 256),
        (99, 8, 4, 40, true, 512),
    ];
    let mut reports = Vec::new();
    for (seed, w, out, ops, rot, plen) in cells {
        let r = run_cell(
            AdaptiveWriteMode::Static,
            seed,
            w,
            out,
            ops,
            rot,
            plen,
        );
        assert_eq!(r.pre_digest, r.reopen_digest);
        assert_eq!(r.acked, r.issued);
        reports.push(r);
    }
    // Smoke evidence in stdout for claim table (cargo test -- --nocapture).
    for r in &reports {
        eprintln!(
            "q13 static seed={} w={} out={} acked={} file_sync={} rotate={} sync/ack={:.3} digest={}",
            r.seed,
            r.workers,
            r.outstanding,
            r.acked,
            r.file_sync,
            r.segment_rotate,
            r.sync_per_logical_ack,
            &r.pre_digest[..16.min(r.pre_digest.len())]
        );
    }
}

#[test]
fn q13_adaptive_concurrent_reopen_matrix() {
    let cells = [
        (7u64, 4usize, 2usize, 24u64, false, 128usize),
        (42, 4, 4, 32, false, 256),
        (1001, 8, 4, 40, true, 512),
    ];
    let mut reports = Vec::new();
    for (seed, w, out, ops, rot, plen) in cells {
        let r = run_cell(
            AdaptiveWriteMode::Adaptive,
            seed,
            w,
            out,
            ops,
            rot,
            plen,
        );
        assert_eq!(r.pre_digest, r.reopen_digest);
        assert_eq!(r.acked, r.issued);
        reports.push(r);
    }
    for r in &reports {
        eprintln!(
            "q13 adaptive seed={} w={} out={} acked={} file_sync={} rotate={} sync/ack={:.3} digest={}",
            r.seed,
            r.workers,
            r.outstanding,
            r.acked,
            r.file_sync,
            r.segment_rotate,
            r.sync_per_logical_ack,
            &r.pre_digest[..16.min(r.pre_digest.len())]
        );
    }
}

/// Cross-mode smoke: same seed/config must both reopen-correct (not thr equality).
#[test]
fn q13_static_and_adaptive_same_seed_both_reopen_ok() {
    let seed = 2026u64;
    let s = run_cell(AdaptiveWriteMode::Static, seed, 4, 4, 28, false, 192);
    let a = run_cell(AdaptiveWriteMode::Adaptive, seed, 4, 4, 28, false, 192);
    assert_eq!(s.acked, 28);
    assert_eq!(a.acked, 28);
    assert_eq!(s.pre_digest, s.reopen_digest);
    assert_eq!(a.pre_digest, a.reopen_digest);
    // Same oracle payload → same content digest across modes.
    assert_eq!(
        s.pre_digest, a.pre_digest,
        "Static and Adaptive must install identical logical content for same seed"
    );
}
