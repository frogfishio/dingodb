//! Unforgeable heap capabilities (`HEAP_SPEC` §30.7).

use crate::constraints::Constraints;
use crate::ids::{
    AuthorityEpoch, AuthorityGeneration, CapabilityId, CertificateId, DeploymentId, HeapId,
    SecurityRevision,
};
use crate::rights::Rights;
use crate::snapshot::HeapSlot;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Private capability payload.
#[allow(dead_code)] // Fields are part of the capability contract; read as gates expand.
pub struct CapInner {
    /// Capability instance id.
    pub(crate) capability_id: CapabilityId,
    /// Bound slot.
    pub(crate) slot: Arc<HeapSlot>,
    /// Deployment.
    pub(crate) deployment_id: DeploymentId,
    /// Bound heap (cached from slot at mint time).
    pub(crate) heap_id: HeapId,
    /// Certificate id.
    pub(crate) certificate_id: CertificateId,
    /// Certificate fingerprint.
    pub(crate) certificate_fingerprint: [u8; 32],
    /// Holder key fingerprint.
    pub(crate) holder_fingerprint: [u8; 32],
    /// Epoch at mint.
    pub(crate) authority_epoch: AuthorityEpoch,
    /// Generation at mint.
    pub(crate) authority_generation: AuthorityGeneration,
    /// Security revision at mint.
    pub(crate) validated_security_revision: SecurityRevision,
    /// Chain head at mint.
    pub(crate) validated_authority_chain_head_hash: [u8; 32],
    /// Effective rights.
    pub(crate) effective_rights: Rights,
    /// Effective constraints.
    pub(crate) effective_constraints: Constraints,
    /// Validity deadline.
    pub(crate) validity_deadline_unix_s: u64,
}

/// Heap-bound capability. Not serializable; no public constructor.
#[derive(Clone)]
pub struct HeapCap {
    pub(crate) inner: Arc<CapInner>,
}

impl HeapCap {
    /// Mint from kernel-private fields (crate-local only).
    pub(crate) fn mint(inner: CapInner) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Pointer equality for composition.
    pub fn same_instance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// Bound heap id.
    pub fn heap_id(&self) -> HeapId {
        self.inner.heap_id
    }

    /// Deployment id.
    pub fn deployment_id(&self) -> DeploymentId {
        self.inner.deployment_id
    }

    /// Effective rights.
    pub fn rights(&self) -> Rights {
        self.inner.effective_rights
    }

    /// Effective constraints.
    pub fn constraints(&self) -> &Constraints {
        &self.inner.effective_constraints
    }

    /// Capability id.
    pub fn capability_id(&self) -> CapabilityId {
        self.inner.capability_id
    }

    /// Validated security revision.
    pub fn security_revision(&self) -> SecurityRevision {
        self.inner.validated_security_revision
    }

    /// Bound slot (for revision checks).
    pub fn slot(&self) -> &Arc<HeapSlot> {
        &self.inner.slot
    }
}

impl std::fmt::Debug for HeapCap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeapCap")
            .field("heap_id", &self.inner.heap_id)
            .field("generation", &self.inner.authority_generation)
            .field("revision", &self.inner.validated_security_revision)
            .field(
                "certificate_fingerprint",
                &redact(&self.inner.certificate_fingerprint),
            )
            .finish()
    }
}

fn redact(fp: &[u8; 32]) -> String {
    format!("{:02x}{:02x}…", fp[0], fp[1])
}

/// SHA-256 of bytes.
pub(crate) fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&Sha256::digest(bytes));
    out
}

/// Non-serializable local maintenance capability.
#[derive(Clone)]
#[allow(dead_code)]
pub struct HeapMaintenanceCap {
    pub(crate) inner: Arc<CapInner>,
}

impl std::fmt::Debug for HeapMaintenanceCap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HeapMaintenanceCap(..)")
    }
}

/// Replica capability (cluster).
#[derive(Clone)]
#[allow(dead_code)]
pub struct ReplicaCap {
    pub(crate) _inner: Arc<CapInner>,
}

impl std::fmt::Debug for ReplicaCap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReplicaCap(..)")
    }
}

/// Recovery capability (local recovery plane).
#[derive(Clone)]
#[allow(dead_code)]
pub struct RecoveryCap {
    pub(crate) _inner: Arc<CapInner>,
}

impl std::fmt::Debug for RecoveryCap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecoveryCap(..)")
    }
}
