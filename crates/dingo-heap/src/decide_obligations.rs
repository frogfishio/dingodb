//! Executable §39 Verus proof obligations (HP-010 / Gate H6 connected evidence).
//!
//! These property tests are the CI-connected stand-in until Verus proves the same
//! predicates over [`crate::decide`]. They do **not** flip `qualified=true`.

use crate::certificate::VerifiedCertificate;
use crate::constraints::{Constraint, Constraints};
use crate::decide::{
    authority_binding_holds, mint_capability, refresh_capability_or_terminate,
};
use crate::ids::{
    AuthorityEpoch, AuthorityGeneration, CertificateId, CollectionId, DeploymentId, HeapId,
    SecurityRevision,
};
use crate::rights::Rights;
use crate::security_time::TrustedInstant;
use crate::snapshot::{HeapAdministrativeState, HeapSecuritySnapshot, HeapSlot};
use std::sync::Arc;

fn uuidish(seed: u8) -> [u8; 16] {
    let mut id = [seed; 16];
    id[6] = (id[6] & 0x0f) | 0x40;
    id[8] = (id[8] & 0x3f) | 0x80;
    id
}

fn snap(
    heap: HeapId,
    dep: DeploymentId,
    master: [u8; 32],
    state: HeapAdministrativeState,
) -> HeapSecuritySnapshot {
    HeapSecuritySnapshot {
        deployment_id: dep,
        heap_id: heap,
        authority_epoch: AuthorityEpoch::new(1).unwrap(),
        authority_generation: AuthorityGeneration::new(1).unwrap(),
        previous_generation: None,
        grace_deadline_unix_s: None,
        master_public_key: master,
        previous_master_public_key: None,
        security_revision: SecurityRevision::new(1).unwrap(),
        authority_chain_head_hash: [9u8; 32],
        administrative_state: state,
        blacklist: vec![],
        policy_rights_ceiling: None,
    }
}

fn cert_for(snap: &HeapSecuritySnapshot, heap: HeapId) -> VerifiedCertificate {
    use crate::capability::sha256;
    VerifiedCertificate {
        cose_bytes: vec![0x01],
        fingerprint: [3u8; 32],
        deployment_id: snap.deployment_id,
        heap_id: heap,
        authority_epoch: snap.authority_epoch,
        authority_generation: snap.authority_generation,
        certificate_id: CertificateId::new_random().unwrap(),
        holder_public_key: [4u8; 32],
        rights: Rights::from_bits_certificate(0x5).unwrap(),
        constraints: Constraints::empty(),
        not_before: 1,
        expires_at: 4_000_000_000,
        issuer_master_key_id: sha256(&snap.master_public_key),
    }
}

/// Obligation runners used by HP-010 Accept.
pub mod obligations {
    use super::*;

    /// O1: allow for non-public ops requires authority_binding_holds.
    pub fn allow_implies_authority_binding() -> bool {
        let dep = DeploymentId::from_bytes(uuidish(0x01)).unwrap();
        let heap_a = HeapId::from_bytes(uuidish(0xf0)).unwrap();
        let heap_b = HeapId::from_bytes(uuidish(0xf1)).unwrap();
        let master = [0x11u8; 32];
        let snapshot = snap(heap_a, dep, master, HeapAdministrativeState::Active);
        let ok = cert_for(&snapshot, heap_a);
        let bad = cert_for(&snapshot, heap_b);
        authority_binding_holds(&snapshot, &ok) && !authority_binding_holds(&snapshot, &bad)
    }

    /// O2/O3: critical collection allowlists cannot admit foreign collections.
    pub fn unknown_constraints_cannot_allow_foreign() -> bool {
        let allowed = CollectionId::from_bytes(uuidish(0xf3)).unwrap();
        let foreign = CollectionId::from_bytes(uuidish(0xf4)).unwrap();
        let constraints = Constraints::from_sorted(vec![Constraint::CollectionAllowlist(vec![
            allowed,
        ])])
        .unwrap();
        constraints.allows_collection(allowed) && !constraints.allows_collection(foreign)
    }

    /// O4: terminal / non-serving states refuse capability refresh.
    pub fn terminal_or_non_serving_refuses_refresh() -> bool {
        let dep = DeploymentId::from_bytes(uuidish(0x01)).unwrap();
        let heap = HeapId::from_bytes(uuidish(0xf5)).unwrap();
        let master = [0x22u8; 32];
        for state in [
            HeapAdministrativeState::Purged,
            HeapAdministrativeState::Retired,
            HeapAdministrativeState::Suspended,
            HeapAdministrativeState::Purging,
        ] {
            let slot = Arc::new(HeapSlot::new(snap(heap, dep, master, state)));
            let cert = cert_for(&slot.load(), heap);
            let cap =
                mint_capability(Arc::clone(&slot), &cert, TrustedInstant { unix_s: 1_700_000_000 })
                    .unwrap();
            if refresh_capability_or_terminate(&cap).is_ok() {
                return false;
            }
        }
        // Active still refreshes.
        let slot = Arc::new(HeapSlot::new(snap(
            heap,
            dep,
            master,
            HeapAdministrativeState::Active,
        )));
        let cert = cert_for(&slot.load(), heap);
        let cap = mint_capability(slot, &cert, TrustedInstant { unix_s: 1_700_000_000 }).unwrap();
        refresh_capability_or_terminate(&cap).is_ok()
    }

    /// O5: epoch mismatch fails authority_binding_holds.
    pub fn epoch_mismatch_fails_binding() -> bool {
        let dep = DeploymentId::from_bytes(uuidish(0x01)).unwrap();
        let heap = HeapId::from_bytes(uuidish(0xf6)).unwrap();
        let master = [0x33u8; 32];
        let mut snapshot = snap(heap, dep, master, HeapAdministrativeState::Active);
        let mut cert = cert_for(&snapshot, heap);
        assert!(authority_binding_holds(&snapshot, &cert));
        cert.authority_epoch = AuthorityEpoch::new(2).unwrap();
        !authority_binding_holds(&snapshot, &cert)
            && {
                snapshot.authority_epoch = AuthorityEpoch::new(2).unwrap();
                authority_binding_holds(&snapshot, &cert)
            }
    }
}

#[cfg(test)]
mod tests {
    use super::obligations;

    #[test]
    fn verus_connected_obligations_hold() {
        assert!(obligations::allow_implies_authority_binding());
        assert!(obligations::unknown_constraints_cannot_allow_foreign());
        assert!(obligations::terminal_or_non_serving_refuses_refresh());
        assert!(obligations::epoch_mismatch_fails_binding());
    }
}
