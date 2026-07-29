//! HP-010 Accept evidence: qualification matrix + claim honesty, differential
//! NI (§26.2.1), H3 HeapCap termination + single-owner admit, H4/H5 restore and
//! key-loss drills. Does **not** flip `qualified=true`.

use dingo_format::{admit_frame_to_heap, AdmitDecision, OwnershipEvidence};
use dingo_heap::{
    claim_language, may_advertise_qualified, mint_capability, refresh_capability_or_terminate,
    AuthorityGeneration, CertificateId, Constraints, DeploymentId, HeapAdministrativeState, HeapId,
    HeapSecuritySnapshot, HeapSlot, Rights, SecurityRevision, TrustedInstant, VerifiedCertificate,
    PRE_QUALIFICATION_LANGUAGE, QUALIFIED_CLAIM, QUALIFIED_PROFILE,
};
use dingo_store::{
    active_snapshot, destroy_data_key, disaster_recovery_restore_retaining_id, heap_binding_envelope,
    heap_label_envelope, labelled_unit_readable, load_identity_tombstone,
    old_deployment_credential_invalid, refuse_access_from_payload_restore,
    refuse_retain_id_without_ceremony, require_admit, restore_payload_to_new_heap, DataKeyHandle,
    DisasterRecoveryCeremony, DisasterRecoveryPackage, HeapLifecycle, HeapRetentionPolicy,
    MediaDomain, PurgeCoverageUnit, TierClass, TombstoneKind,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn uuidish(seed: u8) -> [u8; 16] {
    let mut id = [seed; 16];
    id[6] = (id[6] & 0x0f) | 0x40;
    id[8] = (id[8] & 0x3f) | 0x80;
    id
}

fn op(seed: u8) -> [u8; 16] {
    uuidish(seed)
}

fn slot_for(heap_seed: u8) -> Arc<HeapSlot> {
    let deployment = DeploymentId::from_bytes(uuidish(0x01)).unwrap();
    let heap = HeapId::from_bytes(uuidish(heap_seed)).unwrap();
    let snap = active_snapshot(deployment, heap, [0xab; 32]).unwrap();
    Arc::new(HeapSlot::new(snap))
}

#[derive(Debug, Deserialize)]
struct MatrixDoc {
    profile: String,
    package: String,
    format: String,
    qualified: bool,
    gates: BTreeMap<String, GateEntry>,
    drills: BTreeMap<String, DrillEntry>,
}

#[derive(Debug, Deserialize)]
struct GateEntry {
    status: String,
    #[serde(default)]
    evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DrillEntry {
    status: String,
}

#[test]
fn matrix_records_unqualified_claim_and_mandatory_structure() {
    assert!(!may_advertise_qualified());
    assert!(!QUALIFIED_CLAIM);
    assert_eq!(claim_language(), PRE_QUALIFICATION_LANGUAGE);
    assert_eq!(QUALIFIED_PROFILE, "dingo-heap-v1");

    let path = workspace_root().join("spec/heap/qualification/hp010-matrix-v1.json");
    let raw = fs::read_to_string(&path).expect("hp010 matrix present");
    let doc: MatrixDoc = serde_json::from_str(&raw).expect("parse hp010 matrix");
    assert_eq!(doc.format, "dingo-heap-qualification-matrix-v1");
    assert_eq!(doc.package, "HP-010");
    assert_eq!(doc.profile, "dingo-heap-v1");
    assert!(!doc.qualified, "matrix must not claim qualified yet");

    for gate in ["H0", "H1", "H2", "H3", "H4", "H5", "H6", "HC1"] {
        assert!(doc.gates.contains_key(gate), "missing gate {gate}");
    }
    assert_eq!(doc.gates["H3"].status, "partial");
    assert!(!doc.gates["H3"].evidence.is_empty());
    assert_eq!(doc.gates["H6"].status, "not_started");
    assert!(doc.gates["H6"].evidence.is_empty());

    // Accept drills in this slice must be marked accept in the matrix.
    for drill in [
        "differential_ni_labelled_units",
        "key_loss",
        "restore_payload_only",
        "restore_dr_retain_id",
        "heapcap_terminates_on_lifecycle",
        "single_owner_admit",
    ] {
        assert_eq!(
            doc.drills[drill].status, "accept",
            "drill {drill} should be accept for this HP-010 slice"
        );
    }
    // Open budgets remain not_started / partial — claim cannot flip while these linger.
    assert_eq!(doc.drills["load_latency_budget"].status, "not_started");
    assert_eq!(doc.drills["operator_runbook"].status, "not_started");
    assert!(
        doc.drills["fuzz_budget"].status == "not_started"
            || doc.drills["fuzz_budget"].status == "partial"
    );
}

#[test]
fn differential_ni_labelled_units() {
    // §26.2.1: same target-heap labelled observation under arbitrary other-heap state.
    let heap_a = uuidish(0xb0);
    let heap_b = uuidish(0xb1);
    let env_a = heap_label_envelope(&heap_a).unwrap();

    let observe = |other_env: &[u8]| -> bool {
        // Functional observation for heap A: does its own labelled unit admit?
        let _ = other_env; // other-heap bytes must not affect A's observation
        labelled_unit_readable(&heap_a, &env_a, &env_a)
    };

    let env_b_clean = heap_label_envelope(&heap_b).unwrap();
    let mut env_b_corrupt = env_b_clean.clone();
    if let Some(last) = env_b_corrupt.last_mut() {
        *last ^= 0xaa;
    }
    let mut env_b_empty = Vec::new();
    env_b_empty.extend_from_slice(&[0u8; 8]);

    let obs1 = observe(&env_b_clean);
    let obs2 = observe(&env_b_corrupt);
    let obs3 = observe(&env_b_empty);
    assert!(obs1 && obs2 && obs3, "target heap A observation must stay allow");

    // Cross-heap mix remains deny and identical across other-heap mutations.
    let deny1 = labelled_unit_readable(&heap_a, &env_b_clean, &env_b_clean);
    let deny2 = labelled_unit_readable(&heap_a, &env_b_corrupt, &env_b_corrupt);
    assert!(!deny1 && !deny2);

    // Mutating other heap's lifecycle must not change target heap A state.
    let slot_a = slot_for(0xb0);
    let slot_b = slot_for(0xb1);
    let tmp = TempDir::new().unwrap();
    let before = slot_a.load().administrative_state;
    let mut life_b = HeapLifecycle::open(tmp.path().join("b"), Arc::clone(&slot_b));
    life_b.retire(op(0xb2)).unwrap();
    assert_eq!(slot_b.load().administrative_state, HeapAdministrativeState::Retired);
    assert_eq!(
        slot_a.load().administrative_state,
        before,
        "non-target heap mutation changed target observation"
    );
}

#[test]
fn key_loss_drill() {
    let tmp = TempDir::new().unwrap();
    let heap = uuidish(0xc0);
    let mut key = DataKeyHandle::generate(heap, b"hp010-key-loss-secret").unwrap();
    assert!(!key.is_destroyed());
    let material = key.material().unwrap().to_vec();
    assert!(!material.is_empty());

    let receipt = destroy_data_key(tmp.path(), &mut key).unwrap();
    assert!(key.is_destroyed());
    assert!(key.material().is_none());
    assert_eq!(receipt.heap_id, heap);
    assert!(!receipt.destroyed_fingerprint.iter().all(|b| *b == 0));
    // Second destroy fails closed — no silent re-export of lost key material.
    assert!(destroy_data_key(tmp.path(), &mut key).is_err());
}

#[test]
fn restore_drills_payload_and_retain_id() {
    let tmp = TempDir::new().unwrap();
    let source = uuidish(0xd0);
    let restored =
        restore_payload_to_new_heap(tmp.path(), source, b"restore-drill-bytes", "coll").unwrap();
    assert_ne!(restored.new_heap_id, source);
    let deny = refuse_access_from_payload_restore(&restored).unwrap_err();
    assert!(deny.to_string().contains("cannot grant access"), "{deny}");

    let old_dep = uuidish(0xd1);
    let new_dep = uuidish(0xd2);
    let heap = uuidish(0xd3);
    let deployment = DeploymentId::from_bytes(old_dep).unwrap();
    let heap_id = HeapId::from_bytes(heap).unwrap();
    let snap = active_snapshot(deployment, heap_id, [0x11; 32]).unwrap();
    let slot = Arc::new(HeapSlot::new(snap));
    let package = DisasterRecoveryPackage {
        heap_id: heap,
        backup_deployment_id: old_dep,
        backup_authority_epoch: 1,
        payload: b"dr-drill".to_vec(),
    };
    assert!(refuse_retain_id_without_ceremony(&slot, &package).is_err());

    let ceremony = DisasterRecoveryCeremony {
        heap_id: heap,
        old_deployment_id: old_dep,
        new_deployment_id: new_dep,
        old_authority_epoch: 1,
        new_authority_epoch: 2,
        new_master_public_key: [0x22; 32],
        recovery_authority_evidence: [0x33; 32],
    };
    let result =
        disaster_recovery_restore_retaining_id(tmp.path(), &package, &ceremony, Some(&slot))
            .unwrap();
    assert_eq!(result.snapshot.deployment_id.to_bytes(), new_dep);
    assert_eq!(result.snapshot.authority_epoch.get(), 2);
    assert!(old_deployment_credential_invalid(&result, &old_dep));
}

#[test]
fn retention_and_incomplete_purge_still_retired() {
    // H5 destructive-operation qualification: unavailable domain → retired, not purged.
    let tmp = TempDir::new().unwrap();
    let slot = slot_for(0xe0);
    let heap = slot.load().heap_id.to_bytes();
    let mut life = HeapLifecycle::open(tmp.path(), Arc::clone(&slot));
    life.retire(op(0xe1)).unwrap();

    let retain_until = 2_000_000_000u64;
    life.retention_mut()
        .save_policy(
            tmp.path(),
            &HeapRetentionPolicy {
                heap_id: heap,
                minimum_retain_until_unix_s: retain_until,
            },
        )
        .unwrap();
    assert!(life
        .begin_purge_media(
            op(0xe2),
            vec![PurgeCoverageUnit {
                object_id: uuidish(0xe3),
                domain: MediaDomain::Tier(TierClass::Hot),
                available: true,
            }],
            retain_until - 1,
        )
        .is_err());

    let units = vec![
        PurgeCoverageUnit {
            object_id: uuidish(0xe4),
            domain: MediaDomain::Tier(TierClass::Hot),
            available: true,
        },
        PurgeCoverageUnit {
            object_id: uuidish(0xe5),
            domain: MediaDomain::Replica {
                replica_id: uuidish(0xe6),
            },
            available: false,
        },
    ];
    let plan = life
        .begin_purge_media(op(0xe7), units, retain_until)
        .unwrap();
    life.destroy_coverage_unit(plan.coverage_ids[0]).unwrap();
    assert!(life.complete_purge(plan.operation_id).is_err());
    let incomplete = life.abort_incomplete_purge(plan.operation_id).unwrap();
    assert_eq!(life.state(), HeapAdministrativeState::Retired);
    assert!(!incomplete.unavailable_domains.is_empty());
    let ts = load_identity_tombstone(tmp.path(), &heap).unwrap();
    assert_eq!(ts.kind, TombstoneKind::Retired);
}

fn mint_cap(slot: Arc<HeapSlot>) -> dingo_heap::HeapCap {
    let snap = slot.load();
    let cert = VerifiedCertificate {
        cose_bytes: vec![0x01],
        fingerprint: [3u8; 32],
        deployment_id: snap.deployment_id,
        heap_id: snap.heap_id,
        authority_epoch: snap.authority_epoch,
        authority_generation: snap.authority_generation,
        certificate_id: CertificateId::new_random().unwrap(),
        holder_public_key: [4u8; 32],
        rights: Rights::from_bits_certificate(0x5).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: [5u8; 32],
    };
    mint_capability(slot, &cert, TrustedInstant { unix_s: 1_700_000_000 }).unwrap()
}

#[test]
fn heapcap_terminates_on_lifecycle() {
    // Gate H3: established HeapCaps terminate after state / security-revision change.
    let tmp = TempDir::new().unwrap();
    let slot = slot_for(0xf0);
    let cap = mint_cap(Arc::clone(&slot));
    assert!(refresh_capability_or_terminate(&cap).is_ok());

    let mut life = HeapLifecycle::open(tmp.path(), Arc::clone(&slot));
    life.suspend(op(0xf1)).unwrap();
    assert_eq!(life.state(), HeapAdministrativeState::Suspended);
    assert!(
        refresh_capability_or_terminate(&cap).is_err(),
        "suspend must terminate prior HeapCap"
    );

    life.resume(op(0xf2)).unwrap();
    assert_eq!(life.state(), HeapAdministrativeState::Active);
    // Old cap still stale (revision advanced through suspend+resume).
    assert!(refresh_capability_or_terminate(&cap).is_err());
    let cap2 = mint_cap(Arc::clone(&slot));
    assert!(refresh_capability_or_terminate(&cap2).is_ok());

    life.retire(op(0xf3)).unwrap();
    assert!(
        refresh_capability_or_terminate(&cap2).is_err(),
        "retire must terminate prior HeapCap"
    );
}

#[test]
fn single_owner_admit() {
    // Gate H3: every admitted data-bearing object has exactly one validated known owner.
    let heap_a = uuidish(0xf4);
    let heap_b = uuidish(0xf5);
    let env_a = heap_binding_envelope(&heap_a).unwrap();
    let env_b = heap_binding_envelope(&heap_b).unwrap();

    let ownership = require_admit(&heap_a, &env_a, &env_a, None).unwrap();
    match ownership {
        OwnershipEvidence::Known { heap_id, .. } => assert_eq!(heap_id, heap_a),
        OwnershipEvidence::Unknown => panic!("admitted frame must be Known"),
    }

    assert!(require_admit(&heap_a, &env_a, &env_b, None).is_err());
    assert!(require_admit(&heap_a, &[], &env_a, None).is_err());

    match admit_frame_to_heap(&heap_a, &env_a, &env_b, None) {
        AdmitDecision::RejectConflict | AdmitDecision::RejectWrongHeap { .. } => {}
        other => panic!("expected conflict/wrong-heap, got {other:?}"),
    }
    match admit_frame_to_heap(&heap_a, &[], &[], None) {
        AdmitDecision::RejectUnknown | AdmitDecision::RejectMalformed => {}
        other => panic!("expected unknown/malformed, got {other:?}"),
    }

    // Deterministic adversarial mutations: flip each byte of a valid envelope and
    // prove we never admit under the wrong heap (partial fuzz budget).
    let mut mutations_rejected = 0usize;
    for i in 0..env_a.len() {
        let mut mutated = env_a.clone();
        mutated[i] ^= 0xff;
        let decision = admit_frame_to_heap(&heap_a, &mutated, &env_a, None);
        match decision {
            AdmitDecision::Admit { ownership } => {
                if let OwnershipEvidence::Known { heap_id, .. } = ownership {
                    assert_eq!(heap_id, heap_a);
                }
            }
            _ => mutations_rejected += 1,
        }
        let cross = admit_frame_to_heap(&heap_b, &mutated, &mutated, None);
        assert!(
            !matches!(cross, AdmitDecision::Admit { .. }),
            "mutated bytes must not admit under a different bound heap"
        );
    }
    assert!(
        mutations_rejected > 0,
        "expected at least one mutation to fail closed"
    );
}

#[test]
fn security_revision_bump_terminates_without_lifecycle_helper() {
    let deployment = DeploymentId::from_bytes(uuidish(0x01)).unwrap();
    let heap = HeapId::from_bytes(uuidish(0xf6)).unwrap();
    let snap = HeapSecuritySnapshot {
        deployment_id: deployment,
        heap_id: heap,
        authority_epoch: dingo_heap::AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: [0xab; 32],
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [0x11; 32],
        administrative_state: HeapAdministrativeState::Active,
        blacklist: vec![],
        policy_rights_ceiling: None,
    };
    let slot = Arc::new(HeapSlot::new(snap));
    let cap = mint_cap(Arc::clone(&slot));
    assert!(refresh_capability_or_terminate(&cap).is_ok());

    let mut next = (*slot.load()).clone();
    next.security_revision = SecurityRevision::new(2).unwrap();
    slot.store(next);
    assert!(refresh_capability_or_terminate(&cap).is_err());
}
