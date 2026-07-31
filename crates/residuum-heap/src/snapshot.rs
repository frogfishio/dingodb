//! Heap security snapshot and atomic slot (`HEAP_SPEC` §8.12 / §30).

use crate::authority::BlacklistEntry;
use crate::ids::{AuthorityEpoch, AuthorityGeneration, DeploymentId, HeapId, SecurityRevision};
use crate::rights::Rights;
use arc_swap::ArcSwap;
use std::sync::Arc;

/// Administrative state of a heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeapAdministrativeState {
    /// Ordinary service.
    Active = 1,
    /// Reads only.
    ReadOnly = 2,
    /// Suspended.
    Suspended = 3,
    /// Retired.
    Retired = 4,
    /// Purge in progress.
    Purging = 5,
    /// Terminal purged.
    Purged = 6,
}

impl HeapAdministrativeState {
    /// Wire / descriptor value.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Parse descriptor state byte.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Active),
            2 => Some(Self::ReadOnly),
            3 => Some(Self::Suspended),
            4 => Some(Self::Retired),
            5 => Some(Self::Purging),
            6 => Some(Self::Purged),
            _ => None,
        }
    }

    /// State name used by the admission matrix.
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::ReadOnly => "read_only",
            Self::Suspended => "suspended",
            Self::Retired => "retired",
            Self::Purging => "purging",
            Self::Purged => "purged",
        }
    }

    /// Whether the state is terminal for ordinary data service.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Purged)
    }

    /// Whether ordinary data service is available (`HeapAuthority` `Serving`).
    ///
    /// Active and ReadOnly are serving; Suspended/Retired/Purging/Purged are not.
    #[must_use]
    pub fn is_serving(self) -> bool {
        matches!(self, Self::Active | Self::ReadOnly)
    }
}

/// Resident heap security snapshot used by the hot path.
#[derive(Debug, Clone)]
pub struct HeapSecuritySnapshot {
    /// Deployment.
    pub deployment_id: DeploymentId,
    /// Heap.
    pub heap_id: HeapId,
    /// Authority epoch.
    pub authority_epoch: AuthorityEpoch,
    /// Current generation.
    pub authority_generation: AuthorityGeneration,
    /// Previous generation during grace (if any).
    pub previous_generation: Option<AuthorityGeneration>,
    /// Grace deadline unix seconds.
    pub grace_deadline_unix_s: Option<u64>,
    /// Current master public key.
    pub master_public_key: [u8; 32],
    /// Previous master public key during grace.
    pub previous_master_public_key: Option<[u8; 32]>,
    /// Security revision.
    pub security_revision: SecurityRevision,
    /// Authority chain head hash.
    pub authority_chain_head_hash: [u8; 32],
    /// Administrative state.
    pub administrative_state: HeapAdministrativeState,
    /// Blacklist (bounded).
    pub blacklist: Vec<BlacklistEntry>,
    /// Optional rights ceiling from access policy (None = no extra ceiling).
    pub policy_rights_ceiling: Option<Rights>,
}

/// Atomic replaceable snapshot slot.
pub struct HeapSlot {
    inner: ArcSwap<HeapSecuritySnapshot>,
}

impl HeapSlot {
    /// Create a slot with an initial snapshot.
    pub fn new(snapshot: HeapSecuritySnapshot) -> Self {
        Self {
            inner: ArcSwap::from_pointee(snapshot),
        }
    }

    /// Load current snapshot.
    pub fn load(&self) -> Arc<HeapSecuritySnapshot> {
        self.inner.load_full()
    }

    /// Replace snapshot (security revision must advance).
    pub fn store(&self, snapshot: HeapSecuritySnapshot) {
        self.inner.store(Arc::new(snapshot));
    }
}
