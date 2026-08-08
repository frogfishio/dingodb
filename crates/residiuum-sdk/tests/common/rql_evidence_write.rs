//! Evidence report write policy for RQL-Q3 integration tests (F8).
//!
//! Default: write under `target/rql-q3/` only (never dirty checked-in `spec/`).
//! Opt-in publish: set env `RESIDIUUM_WRITE_SPEC_EVIDENCE=1` to also write
//! the checked-in snapshot under `spec/rql/qualification/corpus-v1/`.
//!
//! Include with:
//! ```ignore
//! #[path = "common/rql_evidence_write.rs"]
//! mod rql_evidence_write;
//! ```

use std::fs;
use std::path::{Path, PathBuf};

/// Env flag: when truthy, also write checked-in `spec/` snapshots.
pub fn write_spec_evidence_enabled() -> bool {
    match std::env::var("RESIDIUUM_WRITE_SPEC_EVIDENCE") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}


/// Always write `body` under `target/rql-q3/{file_name}`.
/// When [`write_spec_evidence_enabled`], also write
/// `spec/rql/qualification/corpus-v1/{file_name}`.
///
/// Returns the **target** path (the default durable-for-verify location).
pub fn write_q3_report(root: &Path, file_name: &str, body: &str) -> PathBuf {
    let target_dir = root.join("target/rql-q3");
    let _ = fs::create_dir_all(&target_dir);
    let target = target_dir.join(file_name);
    fs::write(&target, body).unwrap_or_else(|e| panic!("write {}: {e}", target.display()));

    if write_spec_evidence_enabled() {
        let spec = root
            .join("spec/rql/qualification/corpus-v1")
            .join(file_name);
        if let Some(p) = spec.parent() {
            let _ = fs::create_dir_all(p);
        }
        fs::write(&spec, body).unwrap_or_else(|e| panic!("write {}: {e}", spec.display()));
    }
    target
}
