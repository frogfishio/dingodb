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

/// Meta file under `store-info/`.
pub const STORE_ID_FILE: &str = "store_id";
pub const META_FILE: &str = "meta";
/// Single-frame store descriptor under `store-info/` (Stage 3c).
pub const STORE_DESCRIPTOR_FILE: &str = "descriptor.dingo";

/// Active segment filename (single active writer in Stage 3).
pub const ACTIVE_SEGMENT_FILE: &str = "active.dingo";

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

    /// `store-info/descriptor.dingo` — framed store descriptor (Stage 3c).
    pub fn store_descriptor_file(&self) -> PathBuf {
        self.store_info().join(STORE_DESCRIPTOR_FILE)
    }

    /// `active/`
    pub fn active_dir(&self) -> PathBuf {
        self.root.join(ACTIVE)
    }

    /// `active/active.dingo`
    pub fn active_segment(&self) -> PathBuf {
        self.active_dir().join(ACTIVE_SEGMENT_FILE)
    }

    /// `segments/`
    pub fn segments_dir(&self) -> PathBuf {
        self.root.join(SEGMENTS)
    }

    /// Path for a sealed segment file by hex segment id.
    pub fn sealed_segment(&self, segment_id: &[u8; 16]) -> PathBuf {
        self.segments_dir()
            .join(format!("{}.dingo", hex16(segment_id)))
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

    /// Create the full directory tree for a new store.
    pub fn create_dirs(&self) -> io::Result<()> {
        for dir in [
            self.store_info(),
            self.active_dir(),
            self.segments_dir(),
            self.chunks_dir(),
            self.catalogs_dir(),
            self.indexes_dir(),
            self.snapshots_dir(),
            self.recovery_dir(),
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

/// Parse a sealed segment filename `{hex32}.dingo` into a segment id.
pub fn segment_id_from_filename(path: &Path) -> Option<[u8; 16]> {
    let stem = path.file_stem()?.to_str()?;
    unhex16(stem)
}

/// List `*.dingo` files under a directory (sorted by name for determinism).
pub fn list_dingo_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("dingo") && path.is_file() {
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
        let path = PathBuf::from(format!("{h}.dingo"));
        assert_eq!(segment_id_from_filename(&path), Some(id));
    }
}
