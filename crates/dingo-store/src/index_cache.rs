//! Optional on-disk primary index cache (OVERVIEW §10 / Stage 3c, DEF-023).
//!
//! Derived only. Missing, corrupt, or frontier-mismatched caches MUST NOT
//! prevent recovery: the store rebuilds from segments.
//!
//! ## Frontier checkpoint (DEF-023 / DEF-095)
//!
//! Version **3** caches record a durable frontier plus per-entry **frame offsets**
//! and optional slim bodies (chunk manifests / legacy inline only). Full payload
//! bodies are not written. Loading a legacy v1/v2 fat cache that still embeds
//! full dataset bodies is refused so open cannot re-inflate O(dataset) RSS;
//! the store falls back to a slim segment rebuild instead.
//!
//! Open validation is O(number of segments) for metadata, then O(active tail)
//! to apply frames beyond `active_covered_len`. Full segment rescans remain the
//! recovery path when the sealed set changes or the cache is absent.

use crate::error::StoreError;
use crate::index::{IndexEntry, LiveValue, PrimaryIndex};
use blake3::Hasher;
use std::fs;
use std::path::{Path, PathBuf};

/// Filename under `indexes/` for the primary current-state cache.
pub const PRIMARY_CACHE_FILE: &str = "primary.idx";

const MAGIC_V1: &[u8; 8] = b"DIDX0001";
const MAGIC_V2: &[u8; 8] = b"DIDX0002";
const MAGIC_V3: &[u8; 8] = b"DIDX0003";
const VERSION_V1: u32 = 1;
const VERSION_V2: u32 = 2;
const VERSION_V3: u32 = 3;

const KIND_LIVE: u8 = 1;
const KIND_DELETED: u8 = 2;

/// Refuse to load a legacy fat cache whose resident bodies exceed this budget.
///
/// Forces a slim rebuild instead of pinning multi-GB payload copies in RSS.
const FAT_CACHE_BODY_BUDGET: u64 = 64 * 1024 * 1024;

/// Durable frontier recorded with a v2/v3 primary index checkpoint (DEF-023).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexFrontier {
    /// Fingerprint of **sealed** segment files only (name + length metadata).
    pub sealed_fingerprint: [u8; 32],
    /// Active segment id covered by the checkpoint (`[0;16]` if none).
    pub active_segment_id: [u8; 16],
    /// Exclusive end offset in the active segment included in the checkpoint.
    pub active_covered_len: u64,
}

/// Absolute path of the primary index cache file.
pub fn primary_cache_path(indexes_dir: &Path) -> PathBuf {
    indexes_dir.join(PRIMARY_CACHE_FILE)
}

/// Fingerprint of authoritative segment files used to invalidate a cache.
///
/// Covers every path in `segment_paths`: relative name + byte length, sorted,
/// then BLAKE3-256. Callers pass sealed-only or sealed+active as needed.
/// Cost is O(number of paths) metadata, not O(total segment bytes).
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
/// Supports legacy v1 only when resident bodies stay under [`FAT_CACHE_BODY_BUDGET`].
/// Prefer [`try_load_primary_index_frontier`] for DEF-023 open paths.
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
    // v2/v3 files are not loadable via the v1 full-fingerprint API.
    if bytes.len() >= 8 && (&bytes[..8] == MAGIC_V2.as_slice() || &bytes[..8] == MAGIC_V3.as_slice())
    {
        return Ok(None);
    }
    match decode_cache_v1(&bytes, store_id, expected_fp) {
        Some(index) if index.resident_body_bytes() <= FAT_CACHE_BODY_BUDGET => Ok(Some(index)),
        _ => Ok(None),
    }
}

/// Load a v3 (or small legacy v2) frontier checkpoint when store_id matches.
///
/// Returns `Ok(None)` for absent/corrupt/fat-legacy files.
pub fn try_load_primary_index_frontier(
    path: &Path,
    store_id: [u8; 16],
) -> Result<Option<(PrimaryIndex, IndexFrontier)>, StoreError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    if let Some((index, frontier)) = decode_cache_v3(&bytes, store_id) {
        return Ok(Some((index, frontier)));
    }
    // Legacy v2: accept only when bodies are small (dev fixtures / tiny stores).
    if let Some((index, frontier)) = decode_cache_v2(&bytes, store_id) {
        if index.resident_body_bytes() <= FAT_CACHE_BODY_BUDGET {
            return Ok(Some((index, frontier)));
        }
        // Fat v2 would re-inflate multi-GB RSS — force slim rebuild.
        return Ok(None);
    }
    Ok(None)
}

/// Persist the primary index as a v1 full-fingerprint cache (tests / compat).
#[allow(dead_code)] // retained for v1 fixtures and external tooling
pub fn write_primary_index(
    path: &Path,
    store_id: [u8; 16],
    fingerprint: [u8; 32],
    index: &PrimaryIndex,
) -> Result<(), StoreError> {
    let bytes = encode_cache_v1(store_id, fingerprint, index);
    crate::failpoint::hit("store.index_cache.before_write")?;
    crate::atomic_file::write_atomic(path, &bytes)?;
    crate::failpoint::hit("store.index_cache.after_write")?;
    Ok(())
}

/// Persist a v3 frontier checkpoint (DEF-023 + DEF-095). Atomic durable replace.
pub fn write_primary_index_frontier(
    path: &Path,
    store_id: [u8; 16],
    frontier: &IndexFrontier,
    index: &PrimaryIndex,
) -> Result<(), StoreError> {
    let bytes = encode_cache_v3(store_id, frontier, index);
    crate::failpoint::hit("store.index_cache.before_write")?;
    crate::atomic_file::write_atomic(path, &bytes)?;
    crate::failpoint::hit("store.index_cache.after_write")?;
    Ok(())
}

fn encode_entries_v2(out: &mut Vec<u8>, index: &PrimaryIndex) {
    // Legacy layout without frame_offset (body embedded).
    out.extend_from_slice(&(index.len() as u64).to_le_bytes());
    for (subject, entry) in index.iter_all() {
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
                ..
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
}

fn encode_entries_v3(out: &mut Vec<u8>, index: &PrimaryIndex) {
    out.extend_from_slice(&(index.len() as u64).to_le_bytes());
    for (subject, entry) in index.iter_all() {
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
                out.extend_from_slice(&lv.frame_offset.to_le_bytes());
                let body_len = u32::try_from(lv.body.len()).unwrap_or(u32::MAX);
                out.extend_from_slice(&body_len.to_le_bytes());
                out.extend_from_slice(&lv.body[..body_len as usize]);
            }
            IndexEntry::Deleted {
                item_id,
                event_id,
                segment_id,
                writer_sequence,
                frame_offset,
            } => {
                out.push(KIND_DELETED);
                out.extend_from_slice(item_id);
                out.extend_from_slice(event_id);
                out.extend_from_slice(segment_id);
                out.extend_from_slice(&writer_sequence.to_le_bytes());
                out.extend_from_slice(&frame_offset.to_le_bytes());
                out.extend_from_slice(&0u32.to_le_bytes());
            }
        }
    }
}

fn decode_entries_v2(bytes: &[u8], mut off: usize) -> Option<(PrimaryIndex, usize)> {
    if off + 8 > bytes.len() {
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
                        frame_offset: 0,
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
                        frame_offset: 0,
                    },
                );
            }
            _ => return None,
        }
    }
    Some((index, off))
}

fn decode_entries_v3(bytes: &[u8], mut off: usize) -> Option<(PrimaryIndex, usize)> {
    if off + 8 > bytes.len() {
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
        // subject + kind + 3 ids + writer_seq + frame_offset + body_len
        if off + subject_len + 1 + 16 * 3 + 8 + 8 + 4 > bytes.len() {
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
        let frame_offset = u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?);
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
                        frame_offset,
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
                        frame_offset,
                    },
                );
            }
            _ => return None,
        }
    }
    Some((index, off))
}

fn encode_cache_v1(store_id: [u8; 16], fingerprint: [u8; 32], index: &PrimaryIndex) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 4 + 16 + 32 + 8 + index.len() * 64);
    out.extend_from_slice(MAGIC_V1);
    out.extend_from_slice(&VERSION_V1.to_le_bytes());
    out.extend_from_slice(&store_id);
    out.extend_from_slice(&fingerprint);
    encode_entries_v2(&mut out, index);
    out
}

fn decode_cache_v1(bytes: &[u8], store_id: [u8; 16], expected_fp: [u8; 32]) -> Option<PrimaryIndex> {
    if bytes.len() < 8 + 4 + 16 + 32 + 8 {
        return None;
    }
    if &bytes[..8] != MAGIC_V1.as_slice() {
        return None;
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    if version != VERSION_V1 {
        return None;
    }
    let file_store: [u8; 16] = bytes[12..28].try_into().ok()?;
    if file_store != store_id {
        return None;
    }
    let fp: [u8; 32] = bytes[28..60].try_into().ok()?;
    if fp != expected_fp {
        return None;
    }
    let (index, off) = decode_entries_v2(bytes, 60)?;
    if off != bytes.len() {
        return None;
    }
    Some(index)
}

#[cfg(test)]
fn encode_cache_v2(store_id: [u8; 16], frontier: &IndexFrontier, index: &PrimaryIndex) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 4 + 16 + 32 + 16 + 8 + 8 + index.len() * 64);
    out.extend_from_slice(MAGIC_V2);
    out.extend_from_slice(&VERSION_V2.to_le_bytes());
    out.extend_from_slice(&store_id);
    out.extend_from_slice(&frontier.sealed_fingerprint);
    out.extend_from_slice(&frontier.active_segment_id);
    out.extend_from_slice(&frontier.active_covered_len.to_le_bytes());
    encode_entries_v2(&mut out, index);
    let mut hasher = Hasher::new();
    hasher.update(&out);
    out.extend_from_slice(hasher.finalize().as_bytes());
    out
}

fn decode_cache_v2(bytes: &[u8], store_id: [u8; 16]) -> Option<(PrimaryIndex, IndexFrontier)> {
    if bytes.len() < 8 + 4 + 16 + 32 + 16 + 8 + 8 + 32 {
        return None;
    }
    if &bytes[..8] != MAGIC_V2.as_slice() {
        return None;
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    if version != VERSION_V2 {
        return None;
    }
    let file_store: [u8; 16] = bytes[12..28].try_into().ok()?;
    if file_store != store_id {
        return None;
    }
    let sealed_fingerprint: [u8; 32] = bytes[28..60].try_into().ok()?;
    let active_segment_id: [u8; 16] = bytes[60..76].try_into().ok()?;
    let active_covered_len = u64::from_le_bytes(bytes[76..84].try_into().ok()?);
    let (index, off) = decode_entries_v2(bytes, 84)?;
    if off + 32 != bytes.len() {
        return None;
    }
    let mut hasher = Hasher::new();
    hasher.update(&bytes[..off]);
    let expect = hasher.finalize();
    if expect.as_bytes() != &bytes[off..off + 32] {
        return None;
    }
    Some((
        index,
        IndexFrontier {
            sealed_fingerprint,
            active_segment_id,
            active_covered_len,
        },
    ))
}

fn encode_cache_v3(store_id: [u8; 16], frontier: &IndexFrontier, index: &PrimaryIndex) -> Vec<u8> {
    // Metadata-sized capacity: keys + locators, not payload bodies.
    let mut out = Vec::with_capacity(8 + 4 + 16 + 32 + 16 + 8 + 8 + index.len() * 96);
    out.extend_from_slice(MAGIC_V3);
    out.extend_from_slice(&VERSION_V3.to_le_bytes());
    out.extend_from_slice(&store_id);
    out.extend_from_slice(&frontier.sealed_fingerprint);
    out.extend_from_slice(&frontier.active_segment_id);
    out.extend_from_slice(&frontier.active_covered_len.to_le_bytes());
    encode_entries_v3(&mut out, index);
    let mut hasher = Hasher::new();
    hasher.update(&out);
    out.extend_from_slice(hasher.finalize().as_bytes());
    out
}

fn decode_cache_v3(bytes: &[u8], store_id: [u8; 16]) -> Option<(PrimaryIndex, IndexFrontier)> {
    if bytes.len() < 8 + 4 + 16 + 32 + 16 + 8 + 8 + 32 {
        return None;
    }
    if &bytes[..8] != MAGIC_V3.as_slice() {
        return None;
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    if version != VERSION_V3 {
        return None;
    }
    let file_store: [u8; 16] = bytes[12..28].try_into().ok()?;
    if file_store != store_id {
        return None;
    }
    let sealed_fingerprint: [u8; 32] = bytes[28..60].try_into().ok()?;
    let active_segment_id: [u8; 16] = bytes[60..76].try_into().ok()?;
    let active_covered_len = u64::from_le_bytes(bytes[76..84].try_into().ok()?);
    let (index, off) = decode_entries_v3(bytes, 84)?;
    if off + 32 != bytes.len() {
        return None;
    }
    let mut hasher = Hasher::new();
    hasher.update(&bytes[..off]);
    let expect = hasher.finalize();
    if expect.as_bytes() != &bytes[off..off + 32] {
        return None;
    }
    Some((
        index,
        IndexFrontier {
            sealed_fingerprint,
            active_segment_id,
            active_covered_len,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::EventKind;

    fn sample_index() -> PrimaryIndex {
        let mut index = PrimaryIndex::new();
        index.apply_event(
            b"a".to_vec(),
            EventKind::Put,
            b"val".to_vec(),
            [1u8; 16],
            [2u8; 16],
            [3u8; 16],
            7,
            100,
        );
        index.apply_event(
            b"b".to_vec(),
            EventKind::Delete,
            vec![],
            [4u8; 16],
            [5u8; 16],
            [6u8; 16],
            8,
            200,
        );
        index
    }

    #[test]
    fn cache_roundtrip_v1() {
        let index = sample_index();
        let store_id = [9u8; 16];
        let fp = [0xabu8; 32];
        let bytes = encode_cache_v1(store_id, fp, &index);
        let loaded = decode_cache_v1(&bytes, store_id, fp).unwrap();
        assert_eq!(loaded.get_live(b"a"), Some(b"val".as_slice()));
        assert!(loaded.get_live(b"b").is_none());
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn fingerprint_mismatch_rejects_v1() {
        let index = sample_index();
        let bytes = encode_cache_v1([0u8; 16], [1u8; 32], &index);
        assert!(decode_cache_v1(&bytes, [0u8; 16], [2u8; 32]).is_none());
    }

    #[test]
    fn frontier_cache_roundtrip_v2() {
        let index = sample_index();
        let store_id = [9u8; 16];
        let frontier = IndexFrontier {
            sealed_fingerprint: [0x11u8; 32],
            active_segment_id: [0x22u8; 16],
            active_covered_len: 4096,
        };
        let bytes = encode_cache_v2(store_id, &frontier, &index);
        let (loaded, fr) = decode_cache_v2(&bytes, store_id).unwrap();
        assert_eq!(fr, frontier);
        assert_eq!(loaded.get_live(b"a"), Some(b"val".as_slice()));
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn frontier_cache_roundtrip_v3_preserves_offset() {
        let index = sample_index();
        let store_id = [9u8; 16];
        let frontier = IndexFrontier {
            sealed_fingerprint: [0x11u8; 32],
            active_segment_id: [0x22u8; 16],
            active_covered_len: 4096,
        };
        let bytes = encode_cache_v3(store_id, &frontier, &index);
        let (loaded, fr) = decode_cache_v3(&bytes, store_id).unwrap();
        assert_eq!(fr, frontier);
        assert_eq!(loaded.get_live(b"a"), Some(b"val".as_slice()));
        let lv = match loaded.get(b"a") {
            Some(IndexEntry::Live(v)) => v,
            _ => panic!("expected live"),
        };
        assert_eq!(lv.frame_offset, 100);
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn frontier_cache_corruption_rejects() {
        let index = sample_index();
        let mut bytes = encode_cache_v3(
            [0u8; 16],
            &IndexFrontier {
                sealed_fingerprint: [1u8; 32],
                active_segment_id: [0u8; 16],
                active_covered_len: 0,
            },
            &index,
        );
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        assert!(decode_cache_v3(&bytes, [0u8; 16]).is_none());
    }
}
