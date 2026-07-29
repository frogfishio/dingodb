//! DingoDB heap identity, capability, and authority kernel (`dingo-heap-v1`).
//!
//! Normative: `HEAP_SPEC.md` §§30–32. Contract artifacts: `spec/heap/`.

#![deny(missing_docs)]

mod authority;
mod capability;
mod certificate;
mod constraints;
mod decide;
mod decide_obligations;
mod error;
mod holder_proof;
mod ids;
mod isolation;
mod isolation_model;
mod operational;
mod qualification;
mod rights;
mod security_time;
mod snapshot;
mod wire;

pub use authority::{AuthorityMutationKind, BlacklistEntry, BlacklistKind};
pub use capability::{HeapCap, HeapMaintenanceCap, RecoveryCap, ReplicaCap};
pub use certificate::{sig_structure_for, verify_certificate, VerifiedCertificate};
pub use constraints::{Constraint, Constraints, SourceNetwork};
pub use decide::{
    authority_binding_holds, decide, mint_capability, refresh_capability_or_terminate,
    AuthorizationDecision, OperationDescriptor,
};
pub use decide_obligations::obligations as h6_decide_obligations;
pub use error::{HeapError, HeapUnavailableCause};
pub use holder_proof::{verify_holder_proof, VerifiedHolderProof};
pub use ids::{
    AuthorityEpoch, AuthorityGeneration, CapabilityId, CertificateId, CollectionId, DeploymentId,
    HeapId, SecurityRevision, StreamId,
};
pub use isolation::{
    confine_query_observation, ConfinedObservation, QueryObservationRequest,
    H6_PUBLISHED_LIMITATIONS,
};
pub use isolation_model::{connected_model_smoke, IsolationModel, ModelUnit};
pub use operational::{
    confine_export_heaps, confine_health_detail, confine_operational_observation,
    confine_support_bundle, unauthenticated_field_allowed, ConfinedHealthDetail,
    ConfinedOperationalObservation, ConfinedSupportBundle, HealthDetailInput, OperationalEvent,
    SupportBundleEntry, UNAUTHENTICATED_DECLASSIFIED_FIELDS,
};
pub use qualification::{
    claim_language, may_advertise_qualified, HP010_MATRIX_REL, PRE_QUALIFICATION_LANGUAGE,
    QUALIFIED_CLAIM, QUALIFIED_PROFILE,
};
pub use rights::{Operation, OperationStatus, Rights};
pub use security_time::{SecurityTimeFloor, TimeDecision, TrustedInstant};
pub use snapshot::{HeapAdministrativeState, HeapSecuritySnapshot, HeapSlot};
pub use wire::{
    AUDIENCE_DATA_V1, CONTENT_TYPE_CERTIFICATE, CONTENT_TYPE_HOLDER_PROOF,
    EXTERNAL_AAD_CERTIFICATE, EXTERNAL_AAD_HOLDER_PROOF, PROFILE_VERSION,
};

#[allow(missing_docs)]
mod generated_registry {
    include!(concat!(env!("OUT_DIR"), "/generated_registry.rs"));
}

pub use generated_registry::{
    active_operation_ids, admitted_ops_for_state, allowed_states_for_op, generated_op,
    generated_rights, GeneratedOp, GENERATED_OPS,
};

/// Profile label for the frozen heap isolation contract.
pub const HEAP_PROFILE: &str = "dingo-heap-v1";

/// Spec artifact lengths recorded at build time (drift detection).
pub mod artifacts {
    /// Byte length of `operations-v1.json` at build.
    pub const OPS_LEN: usize = parse_usize(env!("HEAP_OPS_LEN"));
    /// Byte length of `cbor-v1.json` at build.
    pub const CBOR_LEN: usize = parse_usize(env!("HEAP_CBOR_LEN"));
    /// Byte length of `vectors-v1.json` at build.
    pub const VECTORS_LEN: usize = parse_usize(env!("HEAP_VECTORS_LEN"));

    const fn parse_usize(s: &str) -> usize {
        let bytes = s.as_bytes();
        let mut n = 0usize;
        let mut i = 0;
        while i < bytes.len() {
            n = n * 10 + (bytes[i] - b'0') as usize;
            i += 1;
        }
        n
    }
}
