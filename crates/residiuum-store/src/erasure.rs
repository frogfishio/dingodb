//! Erasure-coded archive layout (product follow-on scaffold).
//!
//! Normative intent (OVERVIEW retention): large archive media MAY stripe a
//! sealed segment across *k* data + *m* parity shards so *m* shard losses are
//! recoverable. This module records the **identity and placement contract**
//! only; encoding/decoding codecs are intentionally out of tree until a
//! production codec is chosen (Reed–Solomon / XOR parity / etc.).
//!
//! ## Contract
//!
//! - Logical segment id is unchanged (stable identity across tiers).
//! - Each shard is an opaque object key under archive media:
//!   `{segment_hex}.s{index:02}.residiuum` with a parity manifest
//!   `{segment_hex}.erasure.json`.
//! - Reconstruction requires any *k* of *k+m* shards; missing shards are
//!   coverage holes, never silent zeros.

use crate::error::StoreError;
use std::path::Path;

/// Default data-shard count for archive erasure sets (placeholder profile).
pub const DEFAULT_DATA_SHARDS: u8 = 4;
/// Default parity-shard count (placeholder profile).
pub const DEFAULT_PARITY_SHARDS: u8 = 2;

/// Declared erasure coding profile for a segment set.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErasureProfile {
    /// Stable name (`rs-4-2`, `xor-1`, …).
    pub name: String,
    /// Data shard count *k*.
    pub data_shards: u8,
    /// Parity shard count *m*.
    pub parity_shards: u8,
}

impl ErasureProfile {
    /// Placeholder Reed–Solomon-style 4+2 profile (codec not shipped).
    pub fn rs_4_2() -> Self {
        Self {
            name: "rs-4-2".into(),
            data_shards: DEFAULT_DATA_SHARDS,
            parity_shards: DEFAULT_PARITY_SHARDS,
        }
    }

    /// Total shards *k + m*.
    pub fn total_shards(&self) -> u8 {
        self.data_shards.saturating_add(self.parity_shards)
    }

    /// Minimum shards required to reconstruct.
    pub fn reconstruct_threshold(&self) -> u8 {
        self.data_shards
    }
}

/// Manifest listing shard object keys for one logical segment.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErasureManifest {
    /// Format tag.
    pub format: String,
    /// Hex segment id (stable identity).
    pub segment_id_hex: String,
    /// Coding profile.
    pub profile: ErasureProfile,
    /// Shard object keys in index order (length = total_shards).
    pub shard_keys: Vec<String>,
}

impl ErasureManifest {
    /// Build a manifest with conventional shard key names.
    pub fn for_segment(segment_id_hex: &str, profile: ErasureProfile) -> Self {
        let n = profile.total_shards() as usize;
        let mut shard_keys = Vec::with_capacity(n);
        for i in 0..n {
            shard_keys.push(format!("{segment_id_hex}.s{i:02}.residiuum"));
        }
        Self {
            format: "residiuum-erasure-1".into(),
            segment_id_hex: segment_id_hex.to_string(),
            profile,
            shard_keys,
        }
    }

    /// Object key for the JSON manifest next to shards.
    pub fn manifest_key(segment_id_hex: &str) -> String {
        format!("{segment_id_hex}.erasure.json")
    }
}

/// Encode segment bytes into shards — **not implemented** in this build.
///
/// Returns [`StoreError::MediaUnsupported`] so callers fail loudly rather than
/// inventing a non-interoperable codec.
pub fn encode_shards(
    _profile: &ErasureProfile,
    _segment_bytes: &[u8],
) -> Result<Vec<Vec<u8>>, StoreError> {
    Err(StoreError::MediaUnsupported(
        "erasure encode codec not shipped; archive still uses whole-segment objects".into(),
    ))
}

/// Reconstruct segment bytes from any *k* shards — **not implemented**.
pub fn decode_shards(
    _profile: &ErasureProfile,
    _shards: &[Option<Vec<u8>>],
) -> Result<Vec<u8>, StoreError> {
    Err(StoreError::MediaUnsupported(
        "erasure decode codec not shipped".into(),
    ))
}

/// Whether a path looks like an erasure shard object (naming only).
pub fn is_shard_key(name: &str) -> bool {
    // e.g. abcd.s00.residiuum
    let Some(stem) = name.strip_suffix(".residiuum") else {
        return false;
    };
    let Some((left, idx)) = stem.rsplit_once(".s") else {
        return false;
    };
    !left.is_empty() && idx.len() == 2 && idx.chars().all(|c| c.is_ascii_digit())
}

/// Placeholder path helper for future on-disk shard trees.
pub fn shard_layout_note(_media_root: &Path) -> &'static str {
    "erasure shards use {segment}.sNN.residiuum keys under archive media; codec follow-on"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_shard_keys() {
        let m = ErasureManifest::for_segment("deadbeef", ErasureProfile::rs_4_2());
        assert_eq!(m.shard_keys.len(), 6);
        assert_eq!(m.shard_keys[0], "deadbeef.s00.residiuum");
        assert_eq!(m.shard_keys[5], "deadbeef.s05.residiuum");
        assert_eq!(
            ErasureManifest::manifest_key("deadbeef"),
            "deadbeef.erasure.json"
        );
    }

    #[test]
    fn encode_refuses() {
        let err = encode_shards(&ErasureProfile::rs_4_2(), b"x").unwrap_err();
        assert!(matches!(err, StoreError::MediaUnsupported(_)));
    }

    #[test]
    fn shard_key_naming() {
        assert!(is_shard_key("abcd.s00.residiuum"));
        assert!(!is_shard_key("abcd.residiuum"));
        assert!(!is_shard_key("abcd.s0.residiuum"));
    }
}
