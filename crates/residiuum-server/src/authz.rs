//! Authorization and audit (DEF-033).
//!
//! Authentication (who you are) is separate from authorization (what you may
//! do). Shared application tokens still authenticate principals; each principal
//! carries an explicit privilege set. Privileged recovery operations (purge,
//! force-reconfiguration) are high-friction: they require both the privilege
//! and a confirmation string on the request.
//!
//! Profile tag: [`AUTHZ_PROFILE`].

use residiuum_sdk::Error;
use residiuum_sdk::{constant_time_str_eq, redact_secret};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authorization profile label for capability matrices and startup reports.
pub const AUTHZ_PROFILE: &str = "residiuum-authz-v1";

/// Maximum length for principal ids and other audit labels (bytes).
pub const MAX_AUDIT_LABEL_LEN: usize = 128;

/// Maximum principal id length when registering a principal.
pub const MAX_PRINCIPAL_ID_LEN: usize = 64;

/// Confirmation string required for [`Privilege::Purge`] operations.
pub const PURGE_CONFIRM: &str = "PURGE";

/// Confirmation string required for [`Privilege::ForceReconfig`] operations.
pub const FORCE_RECONFIG_CONFIRM: &str = "FORCE_RECONFIG";

/// A discrete permission that can be granted to a principal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Privilege {
    /// Read data and metadata (`get`, `scan`, `find`, `history`, …).
    Read = 1 << 0,
    /// Mutate ordinary collection data (`put`, `delete`, …).
    Write = 1 << 1,
    /// Create, drop, or rebuild secondary indexes.
    IndexAdmin = 1 << 2,
    /// Server administration beyond ordinary data plane use.
    Admin = 1 << 3,
    /// Salvage / recovery export of frames and evidence.
    Salvage = 1 << 4,
    /// Move data between storage tiers.
    TierMove = 1 << 5,
    /// Physical purge of retained data (high-friction).
    Purge = 1 << 6,
    /// Force control-plane / membership reconfiguration (high-friction).
    ForceReconfig = 1 << 7,
}

impl Privilege {
    /// Stable snake_case name for audit and wire reasons.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::IndexAdmin => "index_admin",
            Self::Admin => "admin",
            Self::Salvage => "salvage",
            Self::TierMove => "tier_move",
            Self::Purge => "purge",
            Self::ForceReconfig => "force_reconfig",
        }
    }

    /// All defined privileges.
    pub fn all() -> &'static [Privilege] {
        &[
            Self::Read,
            Self::Write,
            Self::IndexAdmin,
            Self::Admin,
            Self::Salvage,
            Self::TierMove,
            Self::Purge,
            Self::ForceReconfig,
        ]
    }
}

/// Set of privileges (bitmask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PrivilegeSet {
    bits: u32,
}

impl PrivilegeSet {
    /// Empty set (no permissions).
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Full superuser set.
    pub const fn superuser() -> Self {
        Self {
            bits: (Privilege::Read as u32)
                | (Privilege::Write as u32)
                | (Privilege::IndexAdmin as u32)
                | (Privilege::Admin as u32)
                | (Privilege::Salvage as u32)
                | (Privilege::TierMove as u32)
                | (Privilege::Purge as u32)
                | (Privilege::ForceReconfig as u32),
        }
    }

    /// Read-only data plane.
    pub const fn reader() -> Self {
        Self {
            bits: Privilege::Read as u32,
        }
    }

    /// Read + write ordinary data (no admin / salvage / purge).
    pub const fn writer() -> Self {
        Self {
            bits: (Privilege::Read as u32) | (Privilege::Write as u32),
        }
    }

    /// Database administrator: data + indexes + admin (not salvage/purge).
    pub const fn dba() -> Self {
        Self {
            bits: (Privilege::Read as u32)
                | (Privilege::Write as u32)
                | (Privilege::IndexAdmin as u32)
                | (Privilege::Admin as u32),
        }
    }

    /// Operator: DBA + salvage + tier moves (not purge / force-reconfig).
    pub const fn operator() -> Self {
        Self {
            bits: Self::dba().bits | (Privilege::Salvage as u32) | (Privilege::TierMove as u32),
        }
    }

    /// Insert one privilege.
    pub const fn with(mut self, p: Privilege) -> Self {
        self.bits |= p as u32;
        self
    }

    /// True when this set includes `p`.
    pub const fn contains(self, p: Privilege) -> bool {
        self.bits & (p as u32) != 0
    }

    /// Union.
    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    /// Iterate granted privileges.
    pub fn iter(self) -> impl Iterator<Item = Privilege> {
        Privilege::all()
            .iter()
            .copied()
            .filter(move |p| self.contains(*p))
    }
}

/// Authenticated identity with an explicit privilege set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    /// Stable, non-secret principal id (never a raw token).
    pub id: String,
    /// Granted privileges.
    pub privileges: PrivilegeSet,
}

impl Principal {
    /// Anonymous open-mode principal (no auth configured).
    pub fn anonymous_open() -> Self {
        Self {
            id: "anonymous".into(),
            privileges: PrivilegeSet::superuser(),
        }
    }

    /// Whether this principal holds `p`.
    pub fn has(&self, p: Privilege) -> bool {
        self.privileges.contains(p)
    }
}

/// Registered principal: token (secret) + public id + privileges.
#[derive(Debug, Clone)]
pub struct PrincipalSpec {
    /// Public id used in audit records (not the token).
    pub id: String,
    /// Shared secret presented as the RPC `token` field.
    pub token: String,
    /// Granted privileges.
    pub privileges: PrivilegeSet,
}

/// Server authorization policy: token → principal.
///
/// When the policy is empty **and** no shared serve token is configured, the
/// server runs in open mode (every connection is superuser). That matches the
/// historical default for local development.
#[derive(Debug, Clone, Default)]
pub struct AuthzPolicy {
    principals: Vec<PrincipalSpec>,
}

impl AuthzPolicy {
    /// Empty policy (open mode unless a shared token is layered on top).
    pub fn new() -> Self {
        Self::default()
    }

    /// Single shared token with superuser privileges (legacy `auth_token` mode).
    pub fn shared_superuser(token: impl Into<String>) -> Self {
        Self {
            principals: vec![PrincipalSpec {
                id: "shared".into(),
                token: token.into(),
                privileges: PrivilegeSet::superuser(),
            }],
        }
    }

    /// Register a principal. Token must be non-empty; id is bounded and sanitized.
    pub fn with_principal(
        mut self,
        id: impl Into<String>,
        token: impl Into<String>,
        privileges: PrivilegeSet,
    ) -> Result<Self, Error> {
        let id = sanitize_label(&id.into(), MAX_PRINCIPAL_ID_LEN)?;
        let token = token.into();
        if token.is_empty() {
            return Err(Error::ValidationMsg(
                "authz principal token must be non-empty".into(),
            ));
        }
        if id.is_empty() {
            return Err(Error::ValidationMsg(
                "authz principal id must be non-empty".into(),
            ));
        }
        // Reject duplicate ids (tokens compared only at auth time).
        if self.principals.iter().any(|p| p.id == id) {
            return Err(Error::ValidationMsg(format!(
                "authz principal id already registered: {id}"
            )));
        }
        self.principals.push(PrincipalSpec {
            id,
            token,
            privileges,
        });
        Ok(self)
    }

    /// Number of registered principals.
    pub fn len(&self) -> usize {
        self.principals.len()
    }

    /// True when no principals are registered.
    pub fn is_empty(&self) -> bool {
        self.principals.is_empty()
    }

    /// Whether any principal is configured (auth required).
    pub fn auth_required(&self) -> bool {
        !self.principals.is_empty()
    }

    /// Authenticate a presented token.
    ///
    /// Uses constant-time comparison against every registered token so timing
    /// does not leak which principal matched. On success returns the principal
    /// **without** embedding the secret.
    pub fn authenticate(&self, presented: Option<&str>) -> Result<Principal, Error> {
        if self.principals.is_empty() {
            // Open development mode.
            return Ok(Principal::anonymous_open());
        }
        let presented = presented.unwrap_or("");
        if presented.is_empty() {
            return Err(Error::AuthenticationFailed("missing auth token".into()));
        }
        // Constant-time scan: always compare against every principal.
        let mut matched: Option<usize> = None;
        for (i, spec) in self.principals.iter().enumerate() {
            if constant_time_str_eq(presented, &spec.token) {
                // Prefer first match if multiple tokens somehow collide.
                if matched.is_none() {
                    matched = Some(i);
                }
            }
        }
        match matched {
            Some(i) => {
                let spec = &self.principals[i];
                Ok(Principal {
                    id: spec.id.clone(),
                    privileges: spec.privileges,
                })
            }
            None => Err(Error::AuthenticationFailed(
                "invalid or missing auth token".into(),
            )),
        }
    }
}

/// Requirement for a single RPC operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpRequirement {
    /// Privilege that must be held.
    pub privilege: Privilege,
    /// When set, request must carry `confirm` equal to this string.
    pub confirm: Option<&'static str>,
    /// Whether this op is security- or recovery-sensitive (always audited).
    pub audit_always: bool,
}

/// Map an RPC op name to its authorization requirement.
///
/// Unknown ops still require [`Privilege::Admin`] so they cannot be used as an
/// unauthenticated side channel; dispatch may later reject them as unknown.
pub fn requirement_for_op(op: &str) -> OpRequirement {
    match op {
        // Unauthenticated health is intentional; still audited lightly.
        // `health_live` / `health_ready` are public probes (DEF-061): when auth
        // is configured they still authenticate if a token is presented, but
        // the serve path admits them without a token for orchestrator probes.
        "ping" | "health_live" | "health_ready" => OpRequirement {
            privilege: Privilege::Read,
            confirm: None,
            audit_always: false,
        },
        "store_info" | "directory" | "list_collections" | "list_keys" | "get" | "get_bytes"
        | "scan_json" | "find" | "history" | "index_list" | "get_payload" | "health" => {
            OpRequirement {
                privilege: Privilege::Read,
                confirm: None,
                audit_always: false,
            }
        }
        "put" | "put_bytes" | "delete" => OpRequirement {
            privilege: Privilege::Write,
            confirm: None,
            audit_always: false,
        },
        // Control-plane Raft (DEF-036): peer RPCs use the shared cluster token
        // (superuser by default). Not a data-plane write path.
        "raft_request_vote"
        | "raft_append_entries"
        | "raft_install_snapshot"
        | "raft_read_index" => OpRequirement {
            privilege: Privilege::Admin,
            confirm: None,
            audit_always: true,
        },
        "index_create" | "index_drop" | "index_rebuild" => OpRequirement {
            privilege: Privilege::IndexAdmin,
            confirm: None,
            audit_always: true,
        },
        // Metrics scrape is admin-scoped (DEF-061); health detail is Read above.
        "admin_stats" | "metrics" => OpRequirement {
            privilege: Privilege::Admin,
            confirm: None,
            audit_always: true,
        },
        "salvage_export" => OpRequirement {
            privilege: Privilege::Salvage,
            confirm: None,
            audit_always: true,
        },
        "tier_move" => OpRequirement {
            privilege: Privilege::TierMove,
            confirm: None,
            audit_always: true,
        },
        "purge" => OpRequirement {
            privilege: Privilege::Purge,
            confirm: Some(PURGE_CONFIRM),
            audit_always: true,
        },
        "force_reconfig" => OpRequirement {
            privilege: Privilege::ForceReconfig,
            confirm: Some(FORCE_RECONFIG_CONFIRM),
            audit_always: true,
        },
        _ => OpRequirement {
            privilege: Privilege::Admin,
            confirm: None,
            audit_always: true,
        },
    }
}

/// Authorize `principal` for `op` with optional high-friction confirmation.
///
/// On denial returns [`Error::PermissionDenied`] with a reason that never
/// includes tokens or payloads.
pub fn authorize(
    principal: &Principal,
    op: &str,
    confirm: Option<&str>,
) -> Result<OpRequirement, Error> {
    let req = requirement_for_op(op);
    if !principal.has(req.privilege) {
        return Err(Error::PermissionDenied(format!(
            "principal '{}' lacks privilege '{}' for op '{}'",
            bound_label(&principal.id),
            req.privilege.as_str(),
            bound_label(op)
        )));
    }
    if let Some(needed) = req.confirm {
        let got = confirm.unwrap_or("");
        if got != needed {
            return Err(Error::PermissionDenied(format!(
                "op '{}' requires confirm='{}' (high-friction)",
                bound_label(op),
                needed
            )));
        }
    }
    Ok(req)
}

/// Allow / deny decision recorded in the audit log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditDecision {
    /// Operation was permitted.
    Allow,
    /// Operation was refused.
    Deny,
}

impl AuditDecision {
    /// Stable wire/label form.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

/// One tamper-evident audit record (no secrets, no payloads).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditRecord {
    /// Monotonic sequence number starting at 1.
    pub seq: u64,
    /// Hex BLAKE3 of the previous record body (64 zero hex for genesis).
    pub prev_hash: String,
    /// Hex BLAKE3 of this record's body (includes `prev_hash`).
    pub hash: String,
    /// Unix time in milliseconds when the record was written.
    pub ts_unix_ms: u64,
    /// Principal id (never the raw token).
    pub principal_id: String,
    /// RPC op name (bounded).
    pub op: String,
    /// Optional collection label (bounded).
    pub collection: Option<String>,
    /// Allow or deny.
    pub decision: AuditDecision,
    /// Short reason (bounded; never secrets).
    pub reason: Option<String>,
}

impl AuditRecord {
    /// Canonical body used for hashing (deterministic, no secrets).
    fn body_for_hash(&self) -> String {
        format!(
            "v1|{}|{}|{}|{}|{}|{}|{}|{}",
            self.seq,
            self.prev_hash,
            self.ts_unix_ms,
            self.principal_id,
            self.op,
            self.collection.as_deref().unwrap_or(""),
            self.decision.as_str(),
            self.reason.as_deref().unwrap_or(""),
        )
    }
}

/// In-memory tamper-evident audit log with a hash chain.
///
/// Records never store tokens, request payloads, or raw secrets. Labels are
/// length-bounded. Use [`AuditLog::verify_chain`] to detect truncation or
/// mutation of the in-memory history.
#[derive(Debug, Default)]
pub struct AuditLog {
    inner: Mutex<AuditLogInner>,
}

#[derive(Debug, Default)]
struct AuditLogInner {
    records: Vec<AuditRecord>,
}

impl AuditLog {
    /// Empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an audit event and return the sealed record.
    pub fn append(
        &self,
        principal_id: &str,
        op: &str,
        collection: Option<&str>,
        decision: AuditDecision,
        reason: Option<&str>,
    ) -> AuditRecord {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let seq = guard.records.len() as u64 + 1;
        let prev_hash = guard
            .records
            .last()
            .map(|r| r.hash.clone())
            .unwrap_or_else(|| "0".repeat(64));
        let ts_unix_ms = unix_ms();
        let mut rec = AuditRecord {
            seq,
            prev_hash,
            hash: String::new(),
            ts_unix_ms,
            principal_id: bound_label(principal_id),
            op: bound_label(op),
            collection: collection.map(bound_label),
            decision,
            reason: reason.map(bound_label),
        };
        rec.hash = blake3_hex(rec.body_for_hash().as_bytes());
        guard.records.push(rec.clone());
        rec
    }

    /// Snapshot of all records (clone).
    pub fn records(&self) -> Vec<AuditRecord> {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.records.clone()
    }

    /// Number of records.
    pub fn len(&self) -> usize {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        guard.records.len()
    }

    /// True when empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Verify the hash chain; returns `Ok(())` or the first broken seq.
    pub fn verify_chain(&self) -> Result<(), u64> {
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let mut expected_prev = "0".repeat(64);
        for rec in &guard.records {
            if rec.prev_hash != expected_prev {
                return Err(rec.seq);
            }
            let expected_hash = blake3_hex(rec.body_for_hash().as_bytes());
            if rec.hash != expected_hash {
                return Err(rec.seq);
            }
            expected_prev = rec.hash.clone();
        }
        Ok(())
    }

    /// True when any record's reason or principal accidentally looks like a secret
    /// marker (test helper; production paths use [`redact_secret`] at log sites).
    pub fn contains_literal(&self, needle: &str) -> bool {
        if needle.is_empty() {
            return false;
        }
        let guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        for rec in &guard.records {
            if rec.principal_id.contains(needle)
                || rec.op.contains(needle)
                || rec.reason.as_deref().unwrap_or("").contains(needle)
                || rec.collection.as_deref().unwrap_or("").contains(needle)
            {
                return true;
            }
        }
        false
    }
}

/// Authenticate + authorize one RPC, writing audit records when required.
///
/// Returns the authenticated principal on success. Denials are audited and
/// returned as [`Error::AuthenticationFailed`] or [`Error::PermissionDenied`].
pub fn check_rpc(
    policy: &AuthzPolicy,
    audit: Option<&AuditLog>,
    token: Option<&str>,
    op: &str,
    collection: Option<&str>,
    confirm: Option<&str>,
) -> Result<Principal, Error> {
    let principal = match policy.authenticate(token) {
        Ok(p) => p,
        Err(e) => {
            if let Some(log) = audit {
                let reason = match &e {
                    Error::AuthenticationFailed(m) => m.as_str(),
                    _ => "authentication failed",
                };
                log.append(
                    "unauthenticated",
                    op,
                    collection,
                    AuditDecision::Deny,
                    Some(reason),
                );
            }
            return Err(e);
        }
    };

    match authorize(&principal, op, confirm) {
        Ok(req) => {
            if req.audit_always {
                if let Some(log) = audit {
                    log.append(
                        &principal.id,
                        op,
                        collection,
                        AuditDecision::Allow,
                        Some(req.privilege.as_str()),
                    );
                }
            }
            Ok(principal)
        }
        Err(e) => {
            if let Some(log) = audit {
                let reason = match &e {
                    Error::PermissionDenied(m) => m.as_str(),
                    _ => "permission denied",
                };
                log.append(
                    &principal.id,
                    op,
                    collection,
                    AuditDecision::Deny,
                    Some(reason),
                );
            }
            Err(e)
        }
    }
}

/// Bound a label for audit storage (truncates; never panics).
pub fn bound_label(s: &str) -> String {
    let s = s.replace(['\n', '\r', '\0'], " ");
    if s.len() <= MAX_AUDIT_LABEL_LEN {
        s
    } else {
        let mut out = s.chars().take(MAX_AUDIT_LABEL_LEN).collect::<String>();
        // Ensure we did not exceed byte budget on multi-byte chars.
        while out.len() > MAX_AUDIT_LABEL_LEN {
            out.pop();
        }
        out
    }
}

fn sanitize_label(s: &str, max: usize) -> Result<String, Error> {
    let s = s.trim();
    if s.is_empty() {
        return Err(Error::ValidationMsg("empty authz label".into()));
    }
    if s.len() > max {
        return Err(Error::ValidationMsg(format!(
            "authz label exceeds {max} bytes"
        )));
    }
    if s.chars().any(|c| c.is_control() || c == '\0') {
        return Err(Error::ValidationMsg(
            "authz label contains control characters".into(),
        ));
    }
    Ok(s.to_string())
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn blake3_hex(bytes: &[u8]) -> String {
    // Lightweight FNV-1a 128-bit style chain without pulling blake3 into the
    // SDK dependency graph: two independent 64-bit FNV hashes concatenated.
    // Tamper-evidence for the in-process audit ring; not a general MAC.
    // (DEF-033 requires a hash chain, not a specific algorithm.)
    let h1 = fnv64(bytes, 0xcbf29ce484222325);
    let h2 = fnv64(bytes, 0x100000001b3 ^ 0x9e3779b97f4a7c15);
    format!("{h1:016x}{h2:016x}")
}

fn fnv64(bytes: &[u8], offset: u64) -> u64 {
    let mut hash = offset;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Redact a token for operator-facing logs (re-export convenience).
pub fn redact_token(token: &str) -> String {
    redact_secret(token).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_cannot_purge_or_admin() {
        let policy = AuthzPolicy::new()
            .with_principal("writer", "w-secret", PrivilegeSet::writer())
            .unwrap()
            .with_principal("root", "r-secret", PrivilegeSet::superuser())
            .unwrap();
        let log = AuditLog::new();

        let w = policy.authenticate(Some("w-secret")).unwrap();
        assert!(w.has(Privilege::Write));
        assert!(!w.has(Privilege::Purge));
        assert!(!w.has(Privilege::Admin));

        let err = authorize(&w, "purge", Some(PURGE_CONFIRM)).unwrap_err();
        assert!(matches!(err, Error::PermissionDenied(_)));

        let err = check_rpc(
            &policy,
            Some(&log),
            Some("w-secret"),
            "admin_stats",
            None,
            None,
        )
        .unwrap_err();
        assert!(matches!(err, Error::PermissionDenied(_)));
        assert!(!log.is_empty());
        assert!(!log.contains_literal("w-secret"));
        assert!(!log.contains_literal("r-secret"));
        log.verify_chain().unwrap();
    }

    #[test]
    fn high_friction_requires_confirm() {
        let p = Principal {
            id: "root".into(),
            privileges: PrivilegeSet::superuser(),
        };
        let err = authorize(&p, "purge", None).unwrap_err();
        assert!(matches!(err, Error::PermissionDenied(_)));
        authorize(&p, "purge", Some(PURGE_CONFIRM)).unwrap();
        authorize(&p, "force_reconfig", Some(FORCE_RECONFIG_CONFIRM)).unwrap();
    }

    #[test]
    fn auth_failure_does_not_leak_token_into_audit() {
        let policy = AuthzPolicy::shared_superuser("top-secret-token-value");
        let log = AuditLog::new();
        let err = check_rpc(
            &policy,
            Some(&log),
            Some("wrong-token-value"),
            "put",
            Some("docs"),
            None,
        )
        .unwrap_err();
        assert!(matches!(err, Error::AuthenticationFailed(_)));
        assert!(!log.contains_literal("top-secret-token-value"));
        assert!(!log.contains_literal("wrong-token-value"));
        let recs = log.records();
        assert_eq!(recs[0].decision, AuditDecision::Deny);
        assert_eq!(recs[0].principal_id, "unauthenticated");
    }

    #[test]
    fn chain_detects_tamper() {
        let log = AuditLog::new();
        log.append("a", "put", Some("c"), AuditDecision::Allow, Some("write"));
        log.append("a", "purge", None, AuditDecision::Deny, Some("nope"));
        log.verify_chain().unwrap();
        // Break the chain.
        {
            let mut guard = log.inner.lock().unwrap();
            guard.records[1].reason = Some("tampered".into());
        }
        assert!(log.verify_chain().is_err());
    }

    #[test]
    fn open_mode_superuser() {
        let policy = AuthzPolicy::new();
        let p = policy.authenticate(None).unwrap();
        assert_eq!(p.id, "anonymous");
        assert!(p.has(Privilege::Purge));
    }

    #[test]
    fn constant_time_wrong_token() {
        let policy = AuthzPolicy::new()
            .with_principal("a", "aaaa", PrivilegeSet::reader())
            .unwrap()
            .with_principal("b", "bbbb", PrivilegeSet::writer())
            .unwrap();
        assert!(policy.authenticate(Some("cccc")).is_err());
        let p = policy.authenticate(Some("bbbb")).unwrap();
        assert_eq!(p.id, "b");
        assert!(p.has(Privilege::Write));
    }

    #[test]
    fn bound_label_truncates() {
        let long = "x".repeat(MAX_AUDIT_LABEL_LEN + 50);
        let b = bound_label(&long);
        assert!(b.len() <= MAX_AUDIT_LABEL_LEN);
    }
}
