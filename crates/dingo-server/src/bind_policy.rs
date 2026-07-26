//! Bind policy for `dingo serve` / `serve-cluster` (DEF-002 + DEF-032).
//!
//! Defaults stay on loopback. Non-loopback **plaintext** binds are refused
//! unless the operator opts in with `--allow-insecure-bind`. Non-loopback
//! binds **with TLS** (DEF-032) are allowed. Plaintext remains a loopback-only
//! development profile.

use dingo_sdk::Error;
use std::net::IpAddr;

/// True when the host portion of a bind address is loopback-only.
///
/// Recognizes `127.0.0.0/8`, `::1`, and the name `localhost` (any case).
/// Unresolvable hostnames and unspecified addresses (`0.0.0.0`, `::`) are
/// **not** loopback.
pub fn host_is_loopback(host: &str) -> bool {
    let h = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if h.is_empty() {
        return false;
    }
    if h.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match h.parse::<IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
    }
}

/// Extract the host portion of a `host:port` or `[ipv6]:port` bind string.
///
/// Returns a validation error when the string is empty or has no host.
pub fn bind_host(bind: &str) -> Result<&str, Error> {
    let bind = bind.trim();
    if bind.is_empty() {
        return Err(Error::ValidationMsg("bind address must not be empty".into()));
    }
    if let Some(rest) = bind.strip_prefix('[') {
        let end = rest
            .find(']')
            .ok_or_else(|| Error::ValidationMsg(format!("invalid IPv6 bind address: {bind:?}")))?;
        let host = &rest[..end];
        if host.is_empty() {
            return Err(Error::ValidationMsg(format!(
                "invalid IPv6 bind address (empty host): {bind:?}"
            )));
        }
        return Ok(host);
    }
    // host:port — split on last ':' so bare IPv6 without brackets still fails
    // cleanly (we require brackets for IPv6).
    if let Some((host, _port)) = bind.rsplit_once(':') {
        if host.is_empty() {
            return Err(Error::ValidationMsg(format!(
                "invalid bind address (empty host): {bind:?}"
            )));
        }
        // Reject unbracketed IPv6 (multiple colons without []).
        if host.contains(':') {
            return Err(Error::ValidationMsg(format!(
                "IPv6 bind addresses must be written as [addr]:port, got {bind:?}"
            )));
        }
        return Ok(host);
    }
    Err(Error::ValidationMsg(format!(
        "bind address must be host:port, got {bind:?}"
    )))
}

/// Refuse non-loopback plaintext binds unless `allow_insecure_bind` is set.
///
/// Prefer [`validate_bind`] when TLS may be enabled (DEF-032).
pub fn validate_plaintext_bind(bind: &str, allow_insecure_bind: bool) -> Result<(), Error> {
    validate_bind(bind, allow_insecure_bind, false)
}

/// Validate a serve bind address under the DEF-002 / DEF-032 policy.
///
/// - Loopback: always allowed (plaintext or TLS).
/// - Non-loopback + TLS: allowed (production path).
/// - Non-loopback + plaintext: requires `allow_insecure_bind`.
pub fn validate_bind(
    bind: &str,
    allow_insecure_bind: bool,
    tls_enabled: bool,
) -> Result<(), Error> {
    let host = bind_host(bind)?;
    if host_is_loopback(host) {
        return Ok(());
    }
    if tls_enabled {
        return Ok(());
    }
    if allow_insecure_bind {
        return Ok(());
    }
    Err(Error::ValidationMsg(format!(
        "refusing non-loopback plaintext bind {bind:?}: \
         enable TLS (--tls-cert / --tls-key) for public binds, or pass \
         allow_insecure_bind / --allow-insecure-bind for development-only \
         plaintext (DEF-002, DEF-032). Prefer 127.0.0.1 or localhost."
    )))
}

/// Structured startup status for serve / serve-cluster (DEF-002 / DEF-032).
///
/// Printed to stderr so operators cannot miss transport and durability limits.
#[derive(Debug, Clone)]
pub struct ServeStartupReport {
    /// `serve` or `serve-cluster`.
    pub mode: &'static str,
    /// Store or cluster path shown to the operator.
    pub path: String,
    /// Bind address.
    pub bind: String,
    /// Whether a shared auth token is required.
    pub auth_enabled: bool,
    /// Whether TLS protects the listener (DEF-032).
    pub tls_enabled: bool,
    /// Whether mTLS (client certificates) is required.
    pub mtls_required: bool,
    /// Durability story for this process.
    pub durability: &'static str,
    /// Replication story for this process.
    pub replication: &'static str,
    /// Store-lock status (exclusive ownership not yet enforced — DEF-020).
    pub store_lock: &'static str,
    /// Optional dense node index for cluster serve.
    pub node_index: Option<u32>,
    /// Whether the operator opted into a non-loopback plaintext bind.
    pub allow_insecure_bind: bool,
}

impl ServeStartupReport {
    /// Single-node `dingo serve` defaults.
    pub fn single_node(
        path: impl Into<String>,
        bind: impl Into<String>,
        auth_enabled: bool,
        allow_insecure_bind: bool,
    ) -> Self {
        Self::single_node_tls(path, bind, auth_enabled, allow_insecure_bind, false, false)
    }

    /// Single-node `dingo serve` with explicit TLS flags.
    pub fn single_node_tls(
        path: impl Into<String>,
        bind: impl Into<String>,
        auth_enabled: bool,
        allow_insecure_bind: bool,
        tls_enabled: bool,
        mtls_required: bool,
    ) -> Self {
        Self {
            mode: "serve",
            path: path.into(),
            bind: bind.into(),
            auth_enabled,
            tls_enabled,
            mtls_required,
            durability: "local-store-only (no network quorum)",
            replication: "none (single process)",
            store_lock: "exclusive-writer (OS advisory + in-process; DEF-020)",
            node_index: None,
            allow_insecure_bind,
        }
    }

    /// Network `dingo serve-cluster` defaults (routing prototype, not quorum).
    pub fn cluster_node(
        path: impl Into<String>,
        bind: impl Into<String>,
        auth_enabled: bool,
        allow_insecure_bind: bool,
        node_index: u32,
    ) -> Self {
        Self::cluster_node_tls(
            path,
            bind,
            auth_enabled,
            allow_insecure_bind,
            node_index,
            false,
            false,
        )
    }

    /// Network `dingo serve-cluster` with explicit TLS flags.
    pub fn cluster_node_tls(
        path: impl Into<String>,
        bind: impl Into<String>,
        auth_enabled: bool,
        allow_insecure_bind: bool,
        node_index: u32,
        tls_enabled: bool,
        mtls_required: bool,
    ) -> Self {
        Self {
            mode: "serve-cluster",
            path: path.into(),
            bind: bind.into(),
            auth_enabled,
            tls_enabled,
            mtls_required,
            durability: "this-node-store-only (routing advertise; not quorum commit)",
            replication: "none over network (in-process quorum only via Dingo::open_cluster)",
            store_lock: "exclusive-writer per node store (DEF-020)",
            node_index: Some(node_index),
            allow_insecure_bind,
        }
    }

    /// Human-readable multi-line report for stderr.
    pub fn format_lines(&self) -> String {
        let auth = if self.auth_enabled {
            "shared-token + authz (DEF-033)"
        } else {
            "none (open; DEF-033 anonymous superuser)"
        };
        let tls = if self.tls_enabled {
            if self.mtls_required {
                "on (mTLS)"
            } else {
                "on"
            }
        } else {
            "off"
        };
        let insecure = if self.allow_insecure_bind {
            "yes (development override)"
        } else {
            "no"
        };
        let mut lines = vec![
            format!("dingo {}: startup status", self.mode),
            format!("  path: {}", self.path),
            format!("  bind: {}", self.bind),
            format!("  transport_security: tls={tls}"),
            format!("  authentication: {auth}"),
            format!("  durability: {}", self.durability),
            format!("  replication: {}", self.replication),
            format!("  store_lock: {}", self.store_lock),
            format!("  allow_insecure_bind: {insecure}"),
        ];
        if let Some(n) = self.node_index {
            lines.push(format!("  node_index: {n}"));
        }
        if self.mode == "serve-cluster" {
            lines.push(
                "  warning: serve-cluster is experimental routing/advertise only; \
                 writes apply to this node alone. Quorum replication exists only \
                 in the in-process harness (Dingo::open_cluster)."
                    .into(),
            );
        }
        if !self.tls_enabled {
            lines.push(
                "  warning: plaintext transport; loopback-only unless \
                 --allow-insecure-bind. Prefer --tls-cert/--tls-key for non-loopback."
                    .into(),
            );
        }
        lines.join("\n")
    }

    /// Emit the report to stderr.
    pub fn emit_stderr(&self) {
        eprintln!("{}", self.format_lines());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts() {
        assert!(host_is_loopback("127.0.0.1"));
        assert!(host_is_loopback("127.0.0.2"));
        assert!(host_is_loopback("::1"));
        assert!(host_is_loopback("localhost"));
        assert!(host_is_loopback("LOCALHOST"));
        assert!(!host_is_loopback("0.0.0.0"));
        assert!(!host_is_loopback("::"));
        assert!(!host_is_loopback("192.168.1.1"));
        assert!(!host_is_loopback("example.com"));
    }

    #[test]
    fn bind_host_parsing() {
        assert_eq!(bind_host("127.0.0.1:7434").unwrap(), "127.0.0.1");
        assert_eq!(bind_host("0.0.0.0:7434").unwrap(), "0.0.0.0");
        assert_eq!(bind_host("[::1]:7434").unwrap(), "::1");
        assert_eq!(bind_host("[::]:7434").unwrap(), "::");
        assert!(bind_host("::1:7434").is_err());
        assert!(bind_host("").is_err());
    }

    #[test]
    fn plaintext_bind_policy() {
        validate_plaintext_bind("127.0.0.1:7434", false).unwrap();
        validate_plaintext_bind("localhost:7434", false).unwrap();
        assert!(validate_plaintext_bind("0.0.0.0:7434", false).is_err());
        assert!(validate_plaintext_bind("[::]:80", false).is_err());
        validate_plaintext_bind("0.0.0.0:7434", true).unwrap();
    }

    #[test]
    fn tls_allows_public_bind() {
        validate_bind("0.0.0.0:7434", false, true).unwrap();
        validate_bind("192.168.1.10:7434", false, true).unwrap();
        assert!(validate_bind("0.0.0.0:7434", false, false).is_err());
    }

    #[test]
    fn startup_report_does_not_claim_replication() {
        let r = ServeStartupReport::cluster_node("./c", "127.0.0.1:1", false, false, 0);
        let s = r.format_lines();
        assert!(s.contains("experimental"));
        assert!(s.contains("this-node-store-only"));
        assert!(!s.to_lowercase().contains("replicated durability"));
        assert!(s.contains("replication: none over network"));
    }
}
