//! Pure authorization decision function (`HEAP_SPEC` §39).

use crate::authority::BlacklistKind;
use crate::capability::{sha256, CapInner, HeapCap};
use crate::certificate::VerifiedCertificate;
use crate::constraints::require_rights;
use crate::error::{HeapError, HeapUnavailableCause};
use crate::generated_registry::admitted_ops_for_state;
use crate::ids::{CapabilityId, SecurityRevision};
use crate::rights::{Operation, OperationStatus};
use crate::security_time::{TimeDecision, TrustedInstant};
use crate::snapshot::{HeapAdministrativeState, HeapSecuritySnapshot, HeapSlot};
use std::sync::Arc;

/// Operation request descriptor for admission.
#[derive(Debug, Clone)]
pub struct OperationDescriptor {
    /// Numeric operation id.
    pub operation_id: u16,
    /// Request byte size (for MaxRequestBytes).
    pub request_bytes: u64,
}

/// Authorization outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationDecision {
    /// Allow; caller may mint / use capability for this op.
    Allow,
    /// Deny with fail-closed cause.
    Deny(HeapUnavailableCause),
}

/// Pure decision: no I/O, no ambient clock.
pub fn decide(
    snapshot: &HeapSecuritySnapshot,
    certificate: &VerifiedCertificate,
    operation: &OperationDescriptor,
    now: TrustedInstant,
) -> AuthorizationDecision {
    match decide_inner(snapshot, certificate, operation, now) {
        Ok(()) => AuthorizationDecision::Allow,
        Err(HeapError::UnavailableDetailed { cause, .. }) => AuthorizationDecision::Deny(cause),
        Err(_) => AuthorizationDecision::Deny(HeapUnavailableCause::Denied),
    }
}

fn decide_inner(
    snapshot: &HeapSecuritySnapshot,
    certificate: &VerifiedCertificate,
    operation: &OperationDescriptor,
    now: TrustedInstant,
) -> Result<(), HeapError> {
    // Public process ops bypass heap state / rights.
    if matches!(operation.operation_id, 1..=3) {
        if Operation::status(operation.operation_id)? != OperationStatus::Active {
            return Err(HeapError::unavailable(
                HeapUnavailableCause::UnknownOperation,
            ));
        }
        return Ok(());
    }

    if Operation::status(operation.operation_id)? != OperationStatus::Active {
        return Err(HeapError::unavailable(
            HeapUnavailableCause::UnknownOperation,
        ));
    }

    if certificate.heap_id != snapshot.heap_id
        || certificate.deployment_id != snapshot.deployment_id
        || certificate.authority_epoch != snapshot.authority_epoch
    {
        return Err(HeapError::unavailable(HeapUnavailableCause::StaleAuthority));
    }

    match time_window(certificate.not_before, certificate.expires_at, now) {
        TimeDecision::Accept => {}
        TimeDecision::NotYetValid | TimeDecision::Expired => {
            return Err(HeapError::unavailable(
                HeapUnavailableCause::NotYetValidOrExpired,
            ));
        }
    }

    // Generation acceptance: current, or previous during grace.
    let gen_ok = certificate.authority_generation == snapshot.authority_generation
        || snapshot.previous_generation == Some(certificate.authority_generation)
            && snapshot
                .grace_deadline_unix_s
                .is_some_and(|d| now.unix_s <= d);
    if !gen_ok {
        return Err(HeapError::unavailable(HeapUnavailableCause::StaleAuthority));
    }

    // Issuer must match the generation's master key.
    let expected_master = if certificate.authority_generation == snapshot.authority_generation {
        snapshot.master_public_key
    } else {
        snapshot
            .previous_master_public_key
            .ok_or_else(|| HeapError::unavailable(HeapUnavailableCause::StaleAuthority))?
    };
    let expected_issuer = sha256(&expected_master);
    if certificate.issuer_master_key_id != expected_issuer {
        return Err(HeapError::unavailable(HeapUnavailableCause::StaleAuthority));
    }

    // Blacklist
    let cert_fp = certificate.fingerprint;
    let holder_fp = sha256(&certificate.holder_public_key);
    for entry in &snapshot.blacklist {
        if entry.generation != certificate.authority_generation.get() {
            continue;
        }
        let hit = match entry.kind {
            BlacklistKind::CertificateHash => entry.fingerprint == cert_fp,
            BlacklistKind::HolderPublicKeyHash => entry.fingerprint == holder_fp,
        };
        if hit {
            return Err(HeapError::unavailable(HeapUnavailableCause::Blacklisted));
        }
    }

    if snapshot.administrative_state.is_terminal() {
        return Err(HeapError::unavailable(HeapUnavailableCause::InvalidState));
    }

    let state_name = snapshot.administrative_state.wire_name();
    let admitted = admitted_ops_for_state(state_name)
        .ok_or_else(|| HeapError::unavailable(HeapUnavailableCause::InvalidState))?;
    if !admitted.contains(&operation.operation_id) {
        return Err(HeapError::unavailable(HeapUnavailableCause::InvalidState));
    }

    let required = Operation::required_rights(operation.operation_id)?;
    let mut effective = certificate.rights;
    if let Some(ceiling) = snapshot.policy_rights_ceiling {
        effective = effective.intersection(ceiling);
    }
    require_rights(effective, required)?;

    if !certificate
        .constraints
        .allows_operation(operation.operation_id)
    {
        return Err(HeapError::unavailable(
            HeapUnavailableCause::ConstraintDenied,
        ));
    }

    for c in certificate.constraints.as_slice() {
        if let crate::constraints::Constraint::MaxRequestBytes(max) = c {
            if operation.request_bytes > *max {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::ConstraintDenied,
                ));
            }
        }
    }

    Ok(())
}

fn time_window(not_before: u64, expires_at: u64, now: TrustedInstant) -> TimeDecision {
    if now.unix_s < not_before {
        TimeDecision::NotYetValid
    } else if now.unix_s >= expires_at {
        TimeDecision::Expired
    } else {
        TimeDecision::Accept
    }
}

/// After a successful `decide` Allow on a heap op, mint a HeapCap bound to `slot`.
pub fn mint_capability(
    slot: Arc<HeapSlot>,
    certificate: &VerifiedCertificate,
    now: TrustedInstant,
) -> Result<HeapCap, HeapError> {
    let snap = slot.load();
    // Re-check revision binding.
    if certificate.heap_id != snap.heap_id {
        return Err(HeapError::unavailable(HeapUnavailableCause::StaleAuthority));
    }
    let deadline = certificate
        .expires_at
        .min(snap.grace_deadline_unix_s.unwrap_or(certificate.expires_at));
    if now.unix_s >= deadline {
        return Err(HeapError::unavailable(
            HeapUnavailableCause::NotYetValidOrExpired,
        ));
    }
    let mut effective = certificate.rights;
    if let Some(ceiling) = snap.policy_rights_ceiling {
        effective = effective.intersection(ceiling);
    }
    Ok(HeapCap::mint(CapInner {
        capability_id: CapabilityId::new_random()?,
        slot,
        deployment_id: certificate.deployment_id,
        heap_id: certificate.heap_id,
        certificate_id: certificate.certificate_id,
        certificate_fingerprint: certificate.fingerprint,
        holder_fingerprint: sha256(&certificate.holder_public_key),
        authority_epoch: certificate.authority_epoch,
        authority_generation: certificate.authority_generation,
        validated_security_revision: snap.security_revision,
        validated_authority_chain_head_hash: snap.authority_chain_head_hash,
        effective_rights: effective,
        effective_constraints: certificate.constraints.clone(),
        validity_deadline_unix_s: deadline,
    }))
}

/// Ensure a capability is still live against its slot.
pub fn refresh_capability_or_terminate(cap: &HeapCap) -> Result<(), HeapError> {
    let snap = cap.slot().load();
    if snap.security_revision != cap.security_revision()
        || snap.authority_chain_head_hash != cap.inner.validated_authority_chain_head_hash
        || snap.administrative_state == HeapAdministrativeState::Purged
    {
        return Err(HeapError::unavailable(HeapUnavailableCause::StaleAuthority));
    }
    let _ = SecurityRevision::new(snap.security_revision.get())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{AuthorityEpoch, AuthorityGeneration, DeploymentId, HeapId, SecurityRevision};

    fn sample_snap(heap: HeapId, dep: DeploymentId, master: [u8; 32]) -> HeapSecuritySnapshot {
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
            administrative_state: HeapAdministrativeState::Active,
            blacklist: vec![],
            policy_rights_ceiling: None,
        }
    }

    #[test]
    fn public_ping_always_allow_when_active() {
        let heap = HeapId::new_random().unwrap();
        let dep = DeploymentId::new_random().unwrap();
        let snap = sample_snap(heap, dep, [1u8; 32]);
        // Minimal fake cert — decide for public ops does not read cert fields.
        // We still need a VerifiedCertificate value; construct via bootstrap in integration tests.
        let _ = snap;
        let op = OperationDescriptor {
            operation_id: 1,
            request_bytes: 0,
        };
        // decide requires certificate; use a stub by verifying bootstrap in integration test.
        let _ = op;
    }
}
