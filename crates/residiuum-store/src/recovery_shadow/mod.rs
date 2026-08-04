//! Recovery Shadow (`.rsh`) — P★ salvage artifact (CSE-3 Hybrid Stage 2).
//!
//! **Not** derived / disposable Chimera. Loss of Shadow for segment \(S\)
//! withdraws P★ for that coverage until rebuilt. Product sealing remains on
//! Materialized Chimera until Stage 2 delivery step 8.
//!
//! ## Stage 2a invariants (foundation accept)
//!
//! 1. **Atomic publication:** tmp write → file `sync_all` → rename → parent
//!    directory sync ([`crate::atomic_file`]) before protection is claimed.
//! 2. **Self-verifying:** each `.rsh` binds store/segment identity, magic
//!    version, record boundaries/count, per-record hashes, whole-artifact hash.
//! 3. **Gap-aware frontier:** protected prefix is downward closed over sealed
//!    order — completing seq 12 cannot conceal missing seq 11.
//! 4. **Multi-shard:** per-shard coverage; aggregate claim is **min** prefix,
//!    never a single max scalar that overstates protection.
//!
//! Normative: `CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`,
//! `CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md`.

mod frontier;
mod integrate;
mod wire;

pub use frontier::{
    load_protected_coverage, load_protected_frontier, protection_lag, protection_lag_from_coverage,
    publish_protected_coverage, publish_protected_frontier, ProtectedCoverage, ProtectedFrontier,
    ProtectionLag, FRONTIER_FILE,
};
pub use integrate::{
    build_and_publish_shadow, current_protection_lag, delete_shadow, is_recovery_shadow_path,
    note_segment_sealed, publish_shadow_claiming_protection, rebuild_coverage_from_shadows,
    retire_shadows_after_replacement, secure_erase_shadow, snapshot_telemetry, ShadowTelemetry,
};
pub use wire::{
    decode_shadow, encode_shadow, project_live, publish_shadow, shadow_dir, shadow_path,
    try_load_shadow, DecodedShadow, LiveState, ShadowLoad, ShadowRecord, ShadowWriter, TAG_PUT,
    TAG_TOMBSTONE, RSH_MAGIC,
};

use crate::layout::StorePaths;
use std::path::PathBuf;

/// Ensure `recovery/shadow/` exists under the store root.
pub fn ensure_shadow_dirs(paths: &StorePaths) -> std::io::Result<()> {
    std::fs::create_dir_all(shadow_dir(paths))
}

/// Absolute path of the protected-frontier control document.
pub fn protected_frontier_path(paths: &StorePaths) -> PathBuf {
    frontier::frontier_path(paths)
}
