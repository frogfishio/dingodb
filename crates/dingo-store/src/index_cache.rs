//! Optional on-disk primary index cache (OVERVIEW §10 / Stage 3c).
//!
//! Derived only. Missing, corrupt, or fingerprint-mismatched caches MUST NOT
//! prevent recovery: the store rebuilds from segments.

use crate::error::StoreError;
use crate::index::{IndexEntry, LiveValue, PrimaryIndex};
use blake3::Hasher;
use std::fs;
use std::path::{Path, PathBuf};

/// Filename under `indexes/` for the primary current-state cache.
pub const PRIMARY_CACHE_FILE: &str = "primary.idx";

const MAGIC: &[u8; 8] = b"DIDX0001";
const VERSION: u32 = 1;

const KIND_LIVE: u8 = 1;
const KIND_DELETED: u8 = 2;

/// Absolute path of the primary index cache file.
pub fn primary_cache_path(indexes_dir: &Path) -> PathBuf {
    indexes_dir.join(PRIMARY_CACHE_FILE)
}

/// Fingerprint of authoritative segment files used to invalidate a cache.
///
/// Covers every `*.dingo` under `segments/` plus `active/active.dingo` when
/// present: relative name + byte length, sorted, then BLAKE3-256.
pub fn segment_fingerprint(segment_paths: &[PathBuf]) -> Result<[u8; 32], StoreError> {
    let mut pairs: Vec<(String, u64)> = Vec::with_capacity(segment_paths.len());
    for path in segment_paths {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        // Include parent role so active vs sealed of same name cannot collide.
        let parent = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        pairs.push((format!("{parent}/{name}"), len));
    }
    pairs.sort();
    let mut hasher = Hasher::new();
    for (name, len) in pairs {
        hasher.update(name.as_bytes());
        hasher.update(&[0]);
        hasher.update(&len.to_le_bytes());
    }
    Ok(*hasher.finalize().as_bytes())
}

/// Load a primary index cache if magic, version, store_id, and fingerprint match.
///
/// Returns `Ok(None)` when the file is absent or not usable (never an error for
/// ordinary salvage; callers fall back to rebuild).
pub fn try_load_primary_index(
    path: &Path,
    store_id: [u8; 16],
    expected_fp: [u8; 32],
) -> Result<Option<PrimaryIndex>, StoreError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    match decode_cache(&bytes, store_id, expected_fp) {
        Some(index) => Ok(Some(index)),
        None => Ok(None),
    }
}

/// Persist the primary index to `path` (creates parent dirs as needed).
pub fn write_primary_index(
    path: &Path,
    store_id: [u8; 16],
    fingerprint: [u8; 32],
    index: &PrimaryIndex,
) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = encode_cache(store_id, fingerprint, index);
    let tmp = path.with_extension("idx.tmp");
    crate::failpoint::hit("store.index_cache.before_write")?;
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)?;
    crate::failpoint::hit("store.index_cache.after_write")?;
    Ok(())
}

fn encode_cache(store_id: [u8; 16], fingerprint: [u8; 32], index: &PrimaryIndex) -> Vec<u8> {
    let entries: Vec<_> = index.iter_all().collect();
    let mut out = Vec::with_capacity(8 + 4 + 16 + 32 + 8 + entries.len() * 64);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&store_id);
    out.extend_from_slice(&fingerprint);
    out.extend_from_slice(&(entries.len() as u64).to_le_bytes());
    for (subject, entry) in entries {
        let subject_len = u16::try_from(subject.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&subject_len.to_le_bytes());
        out.extend_from_slice(&subject[..subject_len as usize]);
        match entry {
            IndexEntry::Live(lv) => {
                out.push(KIND_LIVE);
                out.extend_from_slice(&lv.item_id);
                out.extend_from_slice(&lv.event_id);
                out.extend_from_slice(&lv.segment_id);
                out.extend_from_slice(&lv.writer_sequence.to_le_bytes());
                let body_len = u32::try_from(lv.body.len()).unwrap_or(u32::MAX);
                out.extend_from_slice(&body_len.to_le_bytes());
                out.extend_from_slice(&lv.body[..body_len as usize]);
            }
            IndexEntry::Deleted {
                item_id,
                event_id,
                segment_id,
                writer_sequence,
            } => {
                out.push(KIND_DELETED);
                out.extend_from_slice(item_id);
                out.extend_from_slice(event_id);
                out.extend_from_slice(segment_id);
                out.extend_from_slice(&writer_sequence.to_le_bytes());
                out.extend_from_slice(&0u32.to_le_bytes());
            }
        }
    }
    out
}

fn decode_cache(bytes: &[u8], store_id: [u8; 16], expected_fp: [u8; 32]) -> Option<PrimaryIndex> {
    let mut off = 0usize;
    if bytes.len() < 8 + 4 + 16 + 32 + 8 {
        return None;
    }
    if &bytes[off..off + 8] != MAGIC.as_slice() {
        return None;
    }
    off += 8;
    let version = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?);
    off += 4;
    if version != VERSION {
        return None;
    }
    let file_store: [u8; 16] = bytes[off..off + 16].try_into().ok()?;
    off += 16;
    if file_store != store_id {
        return None;
    }
    let fp: [u8; 32] = bytes[off..off + 32].try_into().ok()?;
    off += 32;
    if fp != expected_fp {
        return None;
    }
    let entry_count = u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?) as usize;
    off += 8;

    let mut index = PrimaryIndex::new();
    for _ in 0..entry_count {
        if off + 2 > bytes.len() {
            return None;
        }
        let subject_len = u16::from_le_bytes(bytes[off..off + 2].try_into().ok()?) as usize;
        off += 2;
        if off + subject_len + 1 + 16 * 3 + 8 + 4 > bytes.len() {
            return None;
        }
        let subject = bytes[off..off + subject_len].to_vec();
        off += subject_len;
        let kind = bytes[off];
        off += 1;
        let item_id: [u8; 16] = bytes[off..off + 16].try_into().ok()?;
        off += 16;
        let event_id: [u8; 16] = bytes[off..off + 16].try_into().ok()?;
        off += 16;
        let segment_id: [u8; 16] = bytes[off..off + 16].try_into().ok()?;
        off += 16;
        let writer_sequence = u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?);
        off += 8;
        let body_len = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?) as usize;
        off += 4;
        match kind {
            KIND_LIVE => {
                if off + body_len > bytes.len() {
                    return None;
                }
                let body = bytes[off..off + body_len].to_vec();
                off += body_len;
                index.insert_entry(
                    subject,
                    IndexEntry::Live(LiveValue {
                        body,
                        item_id,
                        event_id,
                        segment_id,
                        writer_sequence,
                    }),
                );
            }
            KIND_DELETED => {
                if body_len != 0 {
                    return None;
                }
                index.insert_entry(
                    subject,
                    IndexEntry::Deleted {
                        item_id,
                        event_id,
                        segment_id,
                        writer_sequence,
                    },
                );
            }
            _ => return None,
        }
    }
    if off != bytes.len() {
        return None;
    }
    Some(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::EventKind;

    #[test]
    fn cache_roundtrip() {
        let mut index = PrimaryIndex::new();
        index.apply_event(
            b"a".to_vec(),
            EventKind::Put,
            b"val".to_vec(),
            [1u8; 16],
            [2u8; 16],
            [3u8; 16],
            7,
        );
        index.apply_event(
            b"b".to_vec(),
            EventKind::Delete,
            vec![],
            [4u8; 16],
            [5u8; 16],
            [6u8; 16],
            8,
        );
        let store_id = [9u8; 16];
        let fp = [0xabu8; 32];
        let bytes = encode_cache(store_id, fp, &index);
        let loaded = decode_cache(&bytes, store_id, fp).unwrap();
        assert_eq!(loaded.get_live(b"a"), Some(b"val".as_slice()));
        assert!(loaded.get_live(b"b").is_none());
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn fingerprint_mismatch_rejects() {
        let mut index = PrimaryIndex::new();
        index.apply_event(
            b"a".to_vec(),
            EventKind::Put,
            b"v".to_vec(),
            [1u8; 16],
            [2u8; 16],
            [3u8; 16],
            0,
        );
        let bytes = encode_cache([0u8; 16], [1u8; 32], &index);
        assert!(decode_cache(&bytes, [0u8; 16], [2u8; 32]).is_none());
    }
}
