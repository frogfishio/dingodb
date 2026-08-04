//! Recovery Shadow (`.rsh`) — P★ salvage artifact (CSE-3 Hybrid Stage 2).
//!
//! **Not** derived / disposable Chimera. Loss of Shadow for segment \(S\)
//! withdraws P★ for that coverage until rebuilt. Product sealing remains on
//! Materialized Chimera until Stage 2 delivery step 8.
//!
//! Normative: `CSE3_STAGE1_HYBRID_RECOVERY_SHADOW.md`,
//! `CSE3_STAGE2_RECOVERY_SHADOW_IMPLEMENT.md`.

mod frontier;
mod wire;

pub use frontier::{
    load_protected_frontier, protection_lag, publish_protected_frontier, ProtectedFrontier,
    ProtectionLag, FRONTIER_FILE,
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
