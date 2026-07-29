//! Operational-surface confinement for Gate H3 (§9.5 / §13.2 / §26.4).
//!
//! Metrics, logs, audit, and export observations are part of `Obs` and cannot
//! escape non-interference by being called diagnostics. This module encodes the
//! closed base declassification registry and heap-scoped filtering helpers.

use crate::capability::HeapCap;
use crate::decide::refresh_capability_or_terminate;
use crate::error::{HeapError, HeapUnavailableCause};
use crate::ids::HeapId;

/// Closed unauthenticated declassification registry (`HEAP_SPEC` §13.2).
pub const UNAUTHENTICATED_DECLASSIFIED_FIELDS: &[&str] =
    &["protocol_versions", "live", "ready", "build_id"];

/// Whether an unauthenticated caller may observe `field` under `heap-data-isolated`.
#[must_use]
pub fn unauthenticated_field_allowed(field: &str) -> bool {
    UNAUTHENTICATED_DECLASSIFIED_FIELDS
        .iter()
        .any(|f| *f == field)
}

/// One operational log/metric/audit event offered to a capability-bound observer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalEvent {
    /// Heap the event pertains to, when heap-local. `None` = deployment-wide.
    pub heap_id: Option<HeapId>,
    /// Stable field / metric name (not a free-form label).
    pub field: String,
    /// Redacted value (never credentials or payloads).
    pub value: String,
}

/// Confined operational observation after filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfinedOperationalObservation {
    /// Bound heap of the observing capability.
    pub heap_id: HeapId,
    /// Events that survived confinement.
    pub events: Vec<OperationalEvent>,
}

/// Confine metrics/logs/audit events to the live capability (§9.5).
///
/// Rules:
/// - capability must refresh;
/// - unauthenticated-class fields may pass without a heap tag;
/// - heap-tagged events must equal the capability heap;
/// - foreign-heap events are dropped (not disclosed);
/// - fields outside the closed registry that claim to be deployment-wide
///   (`heap_id == None` and not in the base table) are denied.
pub fn confine_operational_observation(
    cap: &HeapCap,
    events: &[OperationalEvent],
) -> Result<ConfinedOperationalObservation, HeapError> {
    refresh_capability_or_terminate(cap)?;
    let bound = cap.heap_id();
    let mut out = Vec::with_capacity(events.len());
    for ev in events {
        match ev.heap_id {
            Some(h) if h == bound => out.push(ev.clone()),
            Some(_) => {
                // Foreign heap — drop silently (no existence leak via error shape).
            }
            None => {
                if unauthenticated_field_allowed(&ev.field) {
                    out.push(ev.clone());
                } else {
                    return Err(HeapError::unavailable(
                        HeapUnavailableCause::ConstraintDenied,
                    ));
                }
            }
        }
    }
    Ok(ConfinedOperationalObservation {
        heap_id: bound,
        events: out,
    })
}

/// Confine an ordinary export/backup request to the capability heap (§9.6 / H3).
///
/// An ordinary `HeapCap` may only export its own heap. Requesting any other heap
/// (or an empty multi-heap bag that would imply deployment inventory) fails closed.
pub fn confine_export_heaps(
    cap: &HeapCap,
    requested: &[HeapId],
) -> Result<Vec<HeapId>, HeapError> {
    refresh_capability_or_terminate(cap)?;
    let bound = cap.heap_id();
    if requested.is_empty() {
        return Ok(vec![bound]);
    }
    for h in requested {
        if *h != bound {
            return Err(HeapError::unavailable(HeapUnavailableCause::StaleAuthority));
        }
    }
    Ok(vec![bound])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::certificate::VerifiedCertificate;
    use crate::constraints::Constraints;
    use crate::decide::mint_capability;
    use crate::ids::{
        AuthorityEpoch, AuthorityGeneration, CertificateId, DeploymentId, SecurityRevision,
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

    fn mint() -> HeapCap {
        let deployment = DeploymentId::from_bytes(uuidish(0x01)).unwrap();
        let heap = HeapId::from_bytes(uuidish(0xc0)).unwrap();
        let snap = HeapSecuritySnapshot {
            deployment_id: deployment,
            heap_id: heap,
            authority_epoch: AuthorityEpoch::new(1).unwrap(),
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
        let cert = VerifiedCertificate {
            cose_bytes: vec![0x01],
            fingerprint: [3u8; 32],
            deployment_id: deployment,
            heap_id: heap,
            authority_epoch: AuthorityEpoch::new(1).unwrap(),
            authority_generation: AuthorityGeneration::new(1).unwrap(),
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
    fn drops_foreign_heap_metrics_and_denies_undeclared_global() {
        let cap = mint();
        let foreign = HeapId::from_bytes(uuidish(0xc1)).unwrap();
        let events = vec![
            OperationalEvent {
                heap_id: None,
                field: "live".into(),
                value: "1".into(),
            },
            OperationalEvent {
                heap_id: Some(cap.heap_id()),
                field: "usage_bytes".into(),
                value: "42".into(),
            },
            OperationalEvent {
                heap_id: Some(foreign),
                field: "usage_bytes".into(),
                value: "99".into(),
            },
        ];
        let obs = confine_operational_observation(&cap, &events).unwrap();
        assert_eq!(obs.events.len(), 2);
        assert!(obs.events.iter().all(|e| e.heap_id != Some(foreign)));

        assert!(confine_operational_observation(
            &cap,
            &[OperationalEvent {
                heap_id: None,
                field: "heap_count".into(),
                value: "2".into(),
            }]
        )
        .is_err());
    }

    #[test]
    fn export_confined_to_bound_heap() {
        let cap = mint();
        let foreign = HeapId::from_bytes(uuidish(0xc2)).unwrap();
        assert_eq!(
            confine_export_heaps(&cap, &[]).unwrap(),
            vec![cap.heap_id()]
        );
        assert_eq!(
            confine_export_heaps(&cap, &[cap.heap_id()]).unwrap(),
            vec![cap.heap_id()]
        );
        assert!(confine_export_heaps(&cap, &[foreign]).is_err());
        assert!(confine_export_heaps(&cap, &[cap.heap_id(), foreign]).is_err());
    }
}
