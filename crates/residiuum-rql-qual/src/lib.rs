//! Residiuum RQL-Q4 cross-engine qualification harness (`residiuum-rql-qual`).
//!
//! **Q4.1 scope:** architecture skeleton — lanes, fixture handles, engine adapter
//! trait, result canonicalisation digests, §7.4 metric types, evidence bundle
//! writer aligned with Q0 env fingerprint.
//!
//! **Non-claims:** not Gate-1; not competitive; Mongo/CBL adapters are stubs;
//! Residiuum product execute path is optional (`residiuum-embedded` feature).

#![forbid(unsafe_code)]

pub mod canonicalize;
pub mod cells;
pub mod engine;
pub mod evidence;
pub mod fixture;
pub mod lane;
pub mod metrics;

/// Profile stamp for harness artefacts (distinct from product QVM profiles).
pub const HARNESS_PROFILE: &str = "residiuum-rql-qual-harness-v1";

/// Evidence bundle schema id (machine contracts under `spec/rql/qualification/harness-v1/`).
pub const EVIDENCE_BUNDLE_SCHEMA: &str = "residiuum-rql-qual-evidence-bundle-v1";

/// Env fingerprint schema id.
pub const ENV_FINGERPRINT_SCHEMA: &str = "residiuum-rql-qual-env-fingerprint-v1";

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn profile_stamps_stable() {
        assert!(HARNESS_PROFILE.contains("rql-qual"));
        assert!(EVIDENCE_BUNDLE_SCHEMA.contains("evidence-bundle"));
        assert!(ENV_FINGERPRINT_SCHEMA.contains("env-fingerprint"));
    }
}
