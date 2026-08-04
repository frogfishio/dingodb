//! Filesystem layout helpers (OVERVIEW §6.1).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Directory names under a store root. Names are conventional, not normative.
pub const STORE_INFO: &str = "store-info";
pub const ACTIVE: &str = "active";
pub const SEGMENTS: &str = "segments";
pub const CHUNKS: &str = "chunks";
pub const CATALOGS: &str = "catalogs";
pub const INDEXES: &str = "indexes";
pub const SNAPSHOTS: &str = "snapshots";
pub const RECOVERY: &str = "recovery";
/// Tier media roots and operator tier config (Stage 9).
pub const TIERS: &str = "tiers";

/// Meta file under `store-info/`.
pub const STORE_ID_FILE: &str = "store_id";
pub const META_FILE: &str = "meta";
/// Single-frame store descriptor under `store-info/` (Stage 3c).
pub const STORE_DESCRIPTOR_FILE: &str = "descriptor.residiuum";

/// Active segment filename (single active writer in Stage 3 / Axis B shard-0 legacy).
pub const ACTIVE_SEGMENT_FILE: &str = "active.residiuum";

/// Subdirectory under `active/` for rotated segments awaiting background seal
/// finalize (DEF-096 Axis A dual-slot / async lifecycle). Shared across shards
/// (segment ids are globally unique).
pub const PENDING_SEAL_DIR: &str = "pending";

/// Filename under `store-info/` recording writer shard count (DEF-096 Axis B).
/// Absent → 1 shard (legacy single-active layout).
pub const WRITER_SHARDS_FILE: &str = "writer_shards";

/// Directory name prefix for multi-shard actives: `active/shard-00/`, …
pub const SHARD_DIR_PREFIX: &str = "shard-";

/// Paths derived from a store root.
#[derive(Debug, Clone)]
pub struct StorePaths {
    /// Store root directory.
    pub root: PathBuf,
}

impl StorePaths {
    /// Construct from a root path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `store-info/`
    pub fn store_info(&self) -> PathBuf {
        self.root.join(STORE_INFO)
    }

    /// `store-info/store_id`
    pub fn store_id_file(&self) -> PathBuf {
        self.store_info().join(STORE_ID_FILE)
    }

    /// `store-info/meta`
    pub fn meta_file(&self) -> PathBuf {
        self.store_info().join(META_FILE)
    }

    /// `store-info/descriptor.residiuum` — framed store descriptor (Stage 3c).
    pub fn store_descriptor_file(&self) -> PathBuf {
        self.store_info().join(STORE_DESCRIPTOR_FILE)
    }

    /// `active/`
    pub fn active_dir(&self) -> PathBuf {
        self.root.join(ACTIVE)
    }

    /// `active/active.residiuum` — legacy single-writer path (writer_shards == 1).
    pub fn active_segment(&self) -> PathBuf {
        self.active_dir().join(ACTIVE_SEGMENT_FILE)
    }

    /// Directory for writer shard `shard` under `active/`.
    ///
    /// Shard 0 with `writer_shards == 1` uses the legacy `active/` root (no
    /// `shard-00/` subdirectory) so existing stores stay on-disk compatible.
    pub fn active_shard_dir(&self, shard: usize, writer_shards: usize) -> PathBuf {
        if writer_shards <= 1 {
            self.active_dir()
        } else {
            self.active_dir()
                .join(format!("{SHARD_DIR_PREFIX}{shard:02}"))
        }
    }

    /// Active segment file for a writer shard (DEF-096 Axis B).
    pub fn active_segment_for_shard(&self, shard: usize, writer_shards: usize) -> PathBuf {
        self.active_shard_dir(shard, writer_shards)
            .join(ACTIVE_SEGMENT_FILE)
    }

    /// `active/pending/` — rotated segments waiting for seal finalize (DEF-096).
    pub fn pending_seal_dir(&self) -> PathBuf {
        self.active_dir().join(PENDING_SEAL_DIR)
    }

    /// Path for a pending (rotated, not yet finalized) segment by id.
    pub fn pending_segment(&self, segment_id: &[u8; 16]) -> PathBuf {
        self.pending_seal_dir()
            .join(format!("{}.residiuum", hex16(segment_id)))
    }

    /// `store-info/writer_shards` — Axis B shard count (ASCII decimal + newline).
    pub fn writer_shards_file(&self) -> PathBuf {
        self.store_info().join(WRITER_SHARDS_FILE)
    }

    /// List on-disk active segment paths for `writer_shards` (existing files only).
    pub fn list_active_segment_paths(&self, writer_shards: usize) -> Vec<PathBuf> {
        let n = writer_shards.max(1);
        let mut out = Vec::with_capacity(n);
        for shard in 0..n {
            let p = self.active_segment_for_shard(shard, n);
            if p.is_file() {
                out.push(p);
            }
        }
        // Legacy: multi-shard config but only root active.residiuum exists (upgrade).
        if out.is_empty() {
            let legacy = self.active_segment();
            if legacy.is_file() {
                out.push(legacy);
            }
        }
        out
    }

    /// `segments/`
    pub fn segments_dir(&self) -> PathBuf {
        self.root.join(SEGMENTS)
    }

    /// Path for a sealed segment file by hex segment id.
    pub fn sealed_segment(&self, segment_id: &[u8; 16]) -> PathBuf {
        self.segments_dir()
            .join(format!("{}.residiuum", hex16(segment_id)))
    }

    /// `chunks/`
    pub fn chunks_dir(&self) -> PathBuf {
        self.root.join(CHUNKS)
    }

    /// `catalogs/`
    pub fn catalogs_dir(&self) -> PathBuf {
        self.root.join(CATALOGS)
    }

    /// `indexes/`
    pub fn indexes_dir(&self) -> PathBuf {
        self.root.join(INDEXES)
    }

    /// `snapshots/`
    pub fn snapshots_dir(&self) -> PathBuf {
        self.root.join(SNAPSHOTS)
    }

    /// `recovery/`
    pub fn recovery_dir(&self) -> PathBuf {
        self.root.join(RECOVERY)
    }

    /// `tiers/` — warm/cold/archive media roots + roots.txt (Stage 9).
    pub fn tiers_dir(&self) -> PathBuf {
        self.root.join(TIERS)
    }

    /// Create the full directory tree for a new store.
    pub fn create_dirs(&self) -> io::Result<()> {
        for dir in [
            self.store_info(),
            self.active_dir(),
            self.pending_seal_dir(),
            self.segments_dir(),
            self.chunks_dir(),
            self.catalogs_dir(),
            self.indexes_dir(),
            self.snapshots_dir(),
            self.recovery_dir(),
            self.recovery_dir().join("shadow"),
            self.tiers_dir(),
            self.tiers_dir().join("warm"),
            self.tiers_dir().join("cold"),
            self.tiers_dir().join("archive"),
        ] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// Whether this root looks like an existing store.
    pub fn looks_like_store(&self) -> bool {
        self.store_id_file().is_file()
    }
}

/// Hex-encode 16 bytes (lowercase, no separators).
pub fn hex16(id: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in id {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Decode 32 hex chars into 16 bytes.
pub fn unhex16(s: &str) -> Option<[u8; 16]> {
    if s.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for i in 0..16 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
        out[i] = byte;
    }
    Some(out)
}

/// Parse a sealed segment filename `{hex32}.residiuum` into a segment id.
pub fn segment_id_from_filename(path: &Path) -> Option<[u8; 16]> {
    let stem = path.file_stem()?.to_str()?;
    unhex16(stem)
}

/// List `*.residiuum` files under a directory (sorted by name for determinism).
pub fn list_residiuum_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("residiuum") && path.is_file() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let id = [0xabu8, 0xcd, 0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff];
        let h = hex16(&id);
        assert_eq!(h.len(), 32);
        assert_eq!(unhex16(&h), Some(id));
        let path = PathBuf::from(format!("{h}.residiuum"));
        assert_eq!(segment_id_from_filename(&path), Some(id));
    }
}
