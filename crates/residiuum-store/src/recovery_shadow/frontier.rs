//! `protected_frontier` — which sealed generations currently have durable Shadow.

use super::wire::shadow_dir;
use crate::atomic_file;
use crate::error::StoreError;
use crate::layout::StorePaths;
use blake3::Hasher;
use std::fs;
use std::path::PathBuf;

/// Frontier control-document filename under `recovery/shadow/`.
pub const FRONTIER_FILE: &str = "protected_frontier";

const MAGIC: &[u8; 8] = b"RSHPF001";

/// Durable protection frontier for Recovery Shadows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectedFrontier {
    /// Owning store.
    pub store_id: [u8; 16],
    /// Highest seal generation with complete durable Shadow coverage.
    pub protected_frontier: u64,
    /// Highest seal generation observed (may lead protected).
    pub sealed_frontier: u64,
}

/// Observable lag between sealed and shadow-protected frontiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectionLag {
    /// Sealed frontier.
    pub sealed_frontier: u64,
    /// Protected frontier (complete Shadows only).
    pub protected_frontier: u64,
    /// `sealed_frontier.saturating_sub(protected_frontier)`.
    pub lag: u64,
}

/// Path: `recovery/shadow/protected_frontier`.
pub fn frontier_path(paths: &StorePaths) -> PathBuf {
    shadow_dir(paths).join(FRONTIER_FILE)
}

/// Compute protection lag telemetry.
pub fn protection_lag(frontier: &ProtectedFrontier) -> ProtectionLag {
    ProtectionLag {
        sealed_frontier: frontier.sealed_frontier,
        protected_frontier: frontier.protected_frontier,
        lag: frontier
            .sealed_frontier
            .saturating_sub(frontier.protected_frontier),
    }
}

/// Load frontier; missing → zeros (no P★ claimed).
pub fn load_protected_frontier(
    paths: &StorePaths,
    expect_store: [u8; 16],
) -> Result<ProtectedFrontier, StoreError> {
    let path = frontier_path(paths);
    if !path.is_file() {
        return Ok(ProtectedFrontier {
            store_id: expect_store,
            protected_frontier: 0,
            sealed_frontier: 0,
        });
    }
    let bytes = fs::read(&path)?;
    decode_frontier(&bytes, expect_store).ok_or(StoreError::CorruptControl {
        path: path.display().to_string(),
        detail: "protected_frontier decode failed".into(),
        recovery: "rebuild frontier from verified .rsh files; do not invent coverage".into(),
    })
}

/// Atomically publish frontier. **Never** advances `protected_frontier` past
/// incomplete coverage — callers must only pass verified complete generations.
pub fn publish_protected_frontier(
    paths: &StorePaths,
    frontier: &ProtectedFrontier,
) -> Result<(), StoreError> {
    if frontier.protected_frontier > frontier.sealed_frontier {
        return Err(StoreError::CorruptMeta(
            "protected_frontier must not exceed sealed_frontier",
        ));
    }
    fs::create_dir_all(shadow_dir(paths))?;
    let bytes = encode_frontier(frontier);
    atomic_file::write_atomic(&frontier_path(paths), &bytes)?;
    Ok(())
}

fn encode_frontier(f: &ProtectedFrontier) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 16 + 8 + 8 + 32);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&f.store_id);
    out.extend_from_slice(&f.protected_frontier.to_le_bytes());
    out.extend_from_slice(&f.sealed_frontier.to_le_bytes());
    let mut h = Hasher::new();
    h.update(&out);
    out.extend_from_slice(h.finalize().as_bytes());
    out
}

fn decode_frontier(bytes: &[u8], expect_store: [u8; 16]) -> Option<ProtectedFrontier> {
    if bytes.len() != 8 + 16 + 8 + 8 + 32 {
        return None;
    }
    if &bytes[0..8] != MAGIC.as_slice() {
        return None;
    }
    let store_id: [u8; 16] = bytes[8..24].try_into().ok()?;
    if store_id != expect_store {
        return None;
    }
    let protected_frontier = u64::from_le_bytes(bytes[24..32].try_into().ok()?);
    let sealed_frontier = u64::from_le_bytes(bytes[32..40].try_into().ok()?);
    if protected_frontier > sealed_frontier {
        return None;
    }
    let mut h = Hasher::new();
    h.update(&bytes[..40]);
    let want = *h.finalize().as_bytes();
    let got: [u8; 32] = bytes[40..72].try_into().ok()?;
    if got != want {
        return None;
    }
    Some(ProtectedFrontier {
        store_id,
        protected_frontier,
        sealed_frontier,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sid() -> [u8; 16] {
        [7u8; 16]
    }

    #[test]
    fn frontier_roundtrip_and_lag() {
        let dir = tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        paths.create_dirs().unwrap();
        let f = ProtectedFrontier {
            store_id: sid(),
            protected_frontier: 3,
            sealed_frontier: 5,
        };
        publish_protected_frontier(&paths, &f).unwrap();
        let loaded = load_protected_frontier(&paths, sid()).unwrap();
        assert_eq!(loaded, f);
        let lag = protection_lag(&loaded);
        assert_eq!(lag.lag, 2);
    }

    #[test]
    fn refuses_protected_ahead_of_sealed() {
        let dir = tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        paths.create_dirs().unwrap();
        let f = ProtectedFrontier {
            store_id: sid(),
            protected_frontier: 9,
            sealed_frontier: 3,
        };
        assert!(publish_protected_frontier(&paths, &f).is_err());
    }
}
