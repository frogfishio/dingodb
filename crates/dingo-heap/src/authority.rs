//! Authority event fragments used by the kernel (`HEAP_SPEC` §31.5).

/// Blacklist entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlacklistKind {
    /// Certificate SHA-256.
    CertificateHash = 1,
    /// Holder public-key SHA-256.
    HolderPublicKeyHash = 2,
}

/// One blacklist entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlacklistEntry {
    /// Kind.
    pub kind: BlacklistKind,
    /// Generation the entry applies to.
    pub generation: u64,
    /// Fingerprint.
    pub fingerprint: [u8; 32],
}

/// Master-signed mutation kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AuthorityMutationKind {
    /// Add blacklist entry.
    AddBlacklist = 1,
    /// Remove blacklist entry.
    RemoveBlacklist = 2,
    /// End grace.
    EndGrace = 3,
    /// Fail committed creation permanently.
    FailCreation = 4,
}
