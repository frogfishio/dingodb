//! Pre-mutation authoritative media inventory + immutable publish helpers (P0).
//!
//! Before pending recovery, index rebuild, or any filesystem mutation that can
//! overwrite segment media, Residiuum inventories every authoritative physical
//! source, maps `segment_id → paths`, and **fails closed** on collisions.
//!
//! Recovery Shadows / Hydra / Chimera may contribute to allocator high-water
//! discovery elsewhere; they are **not** authoritative segment ownership here.

use crate::error::StoreError;
use crate::layout::{list_residiuum_files, segment_id_from_filename, StorePaths};
use crate::seal_pipeline::list_pending_paths;
use residiuum_format::{decode_descriptor_body, scan_forward, FrameKind, SafetyLimits};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// One authoritative physical owner of a segment id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoritativeOwner {
    /// Absolute or store-relative path.
    pub path: PathBuf,
    /// Role label for diagnostics.
    pub role: &'static str,
}

/// Inventory: `segment_id → one or more authoritative owners`.
#[derive(Debug, Default, Clone)]
pub struct MediaInventory {
    /// Map of segment id to owners (collision when `owners.len() > 1`).
    pub by_id: BTreeMap<[u8; 16], Vec<AuthoritativeOwner>>,
}

impl MediaInventory {
    /// First collision if any owner list has length > 1.
    pub fn first_collision(&self) -> Option<([u8; 16], Vec<PathBuf>)> {
        for (id, owners) in &self.by_id {
            if owners.len() > 1 {
                let paths = owners.iter().map(|o| o.path.clone()).collect();
                return Some((*id, paths));
            }
        }
        None
    }

    fn record(&mut self, id: [u8; 16], path: PathBuf, role: &'static str) {
        self.by_id
            .entry(id)
            .or_default()
            .push(AuthoritativeOwner { path, role });
    }
}

fn decode_segment_id_from_bytes(
    bytes: &[u8],
    store_id: [u8; 16],
    limits: SafetyLimits,
) -> Result<Option<[u8; 16]>, StoreError> {
    if bytes.is_empty() {
        return Ok(None);
    }
    let report = scan_forward(bytes, limits);
    for region in &report.regions {
        if let residiuum_format::ScanRegion::VerifiedFrame { frame, .. } = region {
            if frame.header.known_kind() == Some(FrameKind::SegmentDescriptor) {
                if let Some((ids, _, _)) = decode_descriptor_body(&frame.body) {
                    if ids.store_id == store_id {
                        return Ok(Some(ids.segment_id));
                    }
                }
            }
        }
    }
    Ok(None)
}

fn validate_filename_id(path: &Path, descriptor_id: [u8; 16]) -> Result<(), StoreError> {
    if let Some(name_id) = segment_id_from_filename(path) {
        if name_id != descriptor_id {
            return Err(StoreError::CorruptMeta(
                "filename segment id does not match descriptor segment id",
            ));
        }
    }
    Ok(())
}

fn inventory_residiuum_file(
    inv: &mut MediaInventory,
    path: PathBuf,
    store_id: [u8; 16],
    limits: SafetyLimits,
    role: &'static str,
) -> Result<(), StoreError> {
    let bytes = fs::read(&path)?;
    let Some(id) = decode_segment_id_from_bytes(&bytes, store_id, limits)? else {
        if !bytes.is_empty() {
            return Err(StoreError::CorruptMeta(
                "authoritative segment media without recoverable store-matching descriptor",
            ));
        }
        return Ok(());
    };
    validate_filename_id(&path, id)?;
    inv.record(id, path, role);
    Ok(())
}

/// Build authoritative inventory (active / pending / sealed / tier copies).
///
/// Does **not** classify Recovery Shadow, Hydra, or Chimera as authoritative
/// segment ownership.
pub fn build_authoritative_inventory(
    paths: &StorePaths,
    store_id: [u8; 16],
    writer_shards: usize,
    limits: SafetyLimits,
) -> Result<MediaInventory, StoreError> {
    let mut inv = MediaInventory::default();

    for path in list_residiuum_files(&paths.segments_dir())? {
        inventory_residiuum_file(&mut inv, path, store_id, limits, "sealed")?;
    }
    for path in list_pending_paths(paths)? {
        inventory_residiuum_file(&mut inv, path, store_id, limits, "pending")?;
    }

    // Tier placement copies (stable segment identity on other mount roots).
    let tier_root = paths.root.join("tiers");
    if tier_root.is_dir() {
        for ent in walkdir_residiuum(&tier_root)? {
            inventory_residiuum_file(&mut inv, ent, store_id, limits, "tier")?;
        }
    }

    // Compaction outputs under recovery (if any residual `.residiuum`).
    let compact_dir = paths.recovery_dir().join("compaction");
    if compact_dir.is_dir() {
        for ent in walkdir_residiuum(&compact_dir)? {
            inventory_residiuum_file(&mut inv, ent, store_id, limits, "compaction")?;
        }
    }

    for path in paths.list_active_segment_paths(writer_shards.max(1)) {
        if !path.is_file() {
            continue;
        }
        inventory_residiuum_file(&mut inv, path, store_id, limits, "active")?;
    }

    Ok(inv)
}

fn walkdir_residiuum(dir: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), StoreError> {
        for ent in fs::read_dir(dir)? {
            let p = ent?.path();
            if p.is_dir() {
                walk(&p, out)?;
            } else if p.extension().and_then(|e| e.to_str()) == Some("residiuum") {
                out.push(p);
            }
        }
        Ok(())
    }
    walk(dir, &mut out)?;
    Ok(out)
}

/// Fail closed if any segment id has multiple authoritative owners.
///
/// Call **before** pending recovery, index rebuild, or media mutation.
pub fn refuse_authoritative_collisions(
    paths: &StorePaths,
    store_id: [u8; 16],
    writer_shards: usize,
    limits: SafetyLimits,
) -> Result<MediaInventory, StoreError> {
    let inv = build_authoritative_inventory(paths, store_id, writer_shards, limits)?;
    if let Some((segment_id, collision_paths)) = inv.first_collision() {
        return Err(StoreError::SegmentIdCollision {
            segment_id,
            paths: collision_paths,
        });
    }
    Ok(inv)
}

fn files_byte_identical(a: &Path, b: &Path) -> Result<bool, StoreError> {
    let ma = fs::metadata(a)?;
    let mb = fs::metadata(b)?;
    if ma.len() != mb.len() {
        return Ok(false);
    }
    let mut fa = fs::File::open(a)?;
    let mut fb = fs::File::open(b)?;
    let mut ba = [0u8; 64 * 1024];
    let mut bb = [0u8; 64 * 1024];
    loop {
        let na = fa.read(&mut ba)?;
        let nb = fb.read(&mut bb)?;
        if na != nb || ba[..na] != bb[..nb] {
            return Ok(false);
        }
        if na == 0 {
            return Ok(true);
        }
    }
}

/// Publish `src` to `dest` with **atomic exclusive** semantics (P0).
///
/// Does **not** use check-then-`rename` (TOCTOU replace on Unix). Protocol:
/// 1. If `dest` exists and bytes match `src` → idempotent: unlink `src` only.
/// 2. If `dest` exists and bytes differ → [`StoreError::SegmentIdCollision`].
/// 3. Else `hard_link(src, dest)` — fails atomically if `dest` appears/races.
/// 4. On success, unlink `src` (dest remains the sole name).
/// 5. Cross-device (`EXDEV`): fall back to `create_new` + copy + unlink `src`
///    (`create_new` is atomic exclusive on the destination).
pub fn rename_exclusive(
    src: &Path,
    dest: &Path,
    segment_id: [u8; 16],
) -> Result<(), StoreError> {
    if dest.exists() {
        if files_byte_identical(src, dest)? {
            let _ = fs::remove_file(src);
            return Ok(());
        }
        return Err(StoreError::SegmentIdCollision {
            segment_id,
            paths: vec![src.to_path_buf(), dest.to_path_buf()],
        });
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::hard_link(src, dest) {
        Ok(()) => {
            let _ = fs::remove_file(src);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lost the race: dest appeared between our exists() check and link.
            if files_byte_identical(src, dest).unwrap_or(false) {
                let _ = fs::remove_file(src);
                return Ok(());
            }
            Err(StoreError::SegmentIdCollision {
                segment_id,
                paths: vec![src.to_path_buf(), dest.to_path_buf()],
            })
        }
        Err(e)
            if e.raw_os_error() == Some(18) /* EXDEV */
                || e.kind() == std::io::ErrorKind::Unsupported =>
        {
            // Different mount: exclusive create + copy, never replace.
            let mut out = create_new_exclusive(dest, segment_id)?;
            {
                let mut input = fs::File::open(src)?;
                std::io::copy(&mut input, &mut out)?;
                out.sync_all()?;
            }
            let _ = fs::remove_file(src);
            Ok(())
        }
        Err(e) => Err(StoreError::Io(e)),
    }
}

/// Create `dest` exclusively (no truncate-overwrite of an existing file).
pub fn create_new_exclusive(dest: &Path, segment_id: [u8; 16]) -> Result<fs::File, StoreError> {
    if dest.exists() {
        return Err(StoreError::SegmentIdCollision {
            segment_id,
            paths: vec![dest.to_path_buf()],
        });
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(dest)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                StoreError::SegmentIdCollision {
                    segment_id,
                    paths: vec![dest.to_path_buf()],
                }
            } else {
                StoreError::Io(e)
            }
        })
}

/// Read descriptor id from an active file (empty → None).
#[allow(dead_code)]
pub fn active_descriptor_id(
    path: &Path,
    store_id: [u8; 16],
    limits: SafetyLimits,
) -> Result<Option<[u8; 16]>, StoreError> {
    let bytes = fs::read(path)?;
    decode_segment_id_from_bytes(&bytes, store_id, limits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rename_exclusive_refuses_different_dest() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.residiuum");
        let dest = dir.path().join("b.residiuum");
        fs::write(&src, b"src-bytes").unwrap();
        fs::write(&dest, b"dest-bytes").unwrap();
        let id = [1u8; 16];
        let err = rename_exclusive(&src, &dest, id).unwrap_err();
        match err {
            StoreError::SegmentIdCollision { paths, .. } => {
                assert_eq!(paths.len(), 2);
                assert!(src.is_file());
                assert!(dest.is_file());
                assert_eq!(fs::read(&dest).unwrap(), b"dest-bytes");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn rename_exclusive_idempotent_same_bytes() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.residiuum");
        let dest = dir.path().join("b.residiuum");
        fs::write(&src, b"same").unwrap();
        fs::write(&dest, b"same").unwrap();
        rename_exclusive(&src, &dest, [2u8; 16]).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read(&dest).unwrap(), b"same");
    }

    #[test]
    fn rename_exclusive_uses_hard_link_not_replace() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("a.residiuum");
        let dest = dir.path().join("b.residiuum");
        fs::write(&src, b"payload-bytes").unwrap();
        rename_exclusive(&src, &dest, [3u8; 16]).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read(&dest).unwrap(), b"payload-bytes");
        // Dest already present → collision, not replace.
        fs::write(&src, b"other").unwrap();
        let err = rename_exclusive(&src, &dest, [3u8; 16]).unwrap_err();
        assert!(matches!(err, StoreError::SegmentIdCollision { .. }));
        assert_eq!(fs::read(&dest).unwrap(), b"payload-bytes");
        assert_eq!(fs::read(&src).unwrap(), b"other");
    }
}
