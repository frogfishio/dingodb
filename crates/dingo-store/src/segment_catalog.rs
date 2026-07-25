//! Hierarchical segment summary catalog (OVERVIEW §9.3, Stage 9).
//!
//! Higher catalog layers accelerate cold discovery but are not authoritative.
//! Loss of this file increases recovery cost; it MUST NOT erase segment bytes.

use crate::error::StoreError;
use crate::layout::StorePaths;
use crate::tier::{available_sealed_paths, resolve_placement_path, TierClass, TierPlacement};
use blake3::Hasher;
use dingo_format::{scan_forward, FrameKind, SafetyLimits};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Filename under `catalogs/` for the segment summary hierarchy.
pub const SEGMENT_CATALOG_FILE: &str = "segments.cat";

const MAGIC: &[u8; 8] = b"DSEGC001";
const VERSION: u32 = 1;

/// Summary of one sealed segment for cold search / hierarchy pruning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentSummary {
    /// Stable segment identity.
    pub segment_id: [u8; 16],
    /// Tier where the segment primarily resides.
    pub tier: TierClass,
    /// File size in bytes.
    pub size: u64,
    /// BLAKE3-256 of segment file bytes.
    pub content_hash: [u8; 32],
    /// Count of structurally verified frames.
    pub verified_frames: u64,
    /// Count of verified item-event frames.
    pub item_events: u64,
    /// Explicit salvage holes observed in the last scan.
    pub holes: u64,
    /// Whether media was available when this summary was built.
    pub available: bool,
}

/// Derived hierarchical catalog of segment summaries (replaceable).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SegmentCatalog {
    /// segment_id → summary
    summaries: BTreeMap<[u8; 16], SegmentSummary>,
}

impl SegmentCatalog {
    /// Empty catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// All summaries in segment-id order.
    pub fn summaries(&self) -> impl Iterator<Item = &SegmentSummary> {
        self.summaries.values()
    }

    /// Lookup one segment.
    pub fn get(&self, segment_id: &[u8; 16]) -> Option<&SegmentSummary> {
        self.summaries.get(segment_id)
    }

    /// Number of summarized segments.
    pub fn len(&self) -> usize {
        self.summaries.len()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.summaries.is_empty()
    }

    /// Insert or replace.
    pub fn upsert(&mut self, summary: SegmentSummary) {
        self.summaries.insert(summary.segment_id, summary);
    }

    /// Segments on a given tier.
    pub fn on_tier(&self, tier: TierClass) -> impl Iterator<Item = &SegmentSummary> {
        self.summaries.values().filter(move |s| s.tier == tier)
    }

    /// Filter summaries by minimum size (cold-search pruning helper).
    pub fn filter_min_size(&self, min_size: u64) -> Vec<&SegmentSummary> {
        self.summaries
            .values()
            .filter(|s| s.size >= min_size)
            .collect()
    }

    /// Merge: keep offline summaries from `prior` when rebuild could not scan them.
    pub fn merge_offline_from(&mut self, prior: &SegmentCatalog, placement: &TierPlacement) {
        for (id, prior_sum) in &prior.summaries {
            if self.summaries.contains_key(id) {
                continue;
            }
            // Retain last-known summary when segment is offline / unavailable.
            if !placement.is_tier_available(prior_sum.tier)
                || placement.get(id).map(|p| !p.available).unwrap_or(false)
            {
                let mut s = prior_sum.clone();
                s.available = false;
                self.summaries.insert(*id, s);
            }
        }
    }
}

/// Path of the segment catalog file.
pub fn segment_catalog_path(catalogs_dir: &Path) -> PathBuf {
    catalogs_dir.join(SEGMENT_CATALOG_FILE)
}

/// Build a summary by scanning one segment file.
pub fn summarize_segment_file(
    segment_id: [u8; 16],
    tier: TierClass,
    path: &Path,
    content_hash: [u8; 32],
    size: u64,
    limits: SafetyLimits,
) -> Result<SegmentSummary, StoreError> {
    let bytes = fs::read(path)?;
    let report = scan_forward(&bytes, limits);
    let mut item_events = 0u64;
    for (_off, frame) in report.verified_frames() {
        if frame.header.known_kind() == Some(FrameKind::ItemEvent) {
            item_events += 1;
        }
    }
    Ok(SegmentSummary {
        segment_id,
        tier,
        size,
        content_hash,
        verified_frames: report.verified_count() as u64,
        item_events,
        holes: report.holes().count() as u64,
        available: true,
    })
}

/// Rebuild segment catalog from available media + placement; merge offline priors.
pub fn rebuild_segment_catalog(
    paths: &StorePaths,
    placement: &TierPlacement,
    prior: Option<&SegmentCatalog>,
    limits: SafetyLimits,
) -> Result<SegmentCatalog, StoreError> {
    let mut cat = SegmentCatalog::new();

    // Summarize from placement entries that are available.
    for p in placement.entries() {
        if !p.available || !placement.is_tier_available(p.tier) {
            // Placeholder offline summary from placement metadata.
            cat.upsert(SegmentSummary {
                segment_id: p.segment_id,
                tier: p.tier,
                size: p.size,
                content_hash: p.content_hash,
                verified_frames: 0,
                item_events: 0,
                holes: 0,
                available: false,
            });
            continue;
        }
        let path = resolve_placement_path(paths, p)?;
        if !path.is_file() {
            cat.upsert(SegmentSummary {
                segment_id: p.segment_id,
                tier: p.tier,
                size: p.size,
                content_hash: p.content_hash,
                verified_frames: 0,
                item_events: 0,
                holes: 0,
                available: false,
            });
            continue;
        }
        let summary =
            summarize_segment_file(p.segment_id, p.tier, &path, p.content_hash, p.size, limits)?;
        cat.upsert(summary);
    }

    // Also summarize any available sealed files not yet in placement.
    for path in available_sealed_paths(paths, placement)? {
        let Some(id) = crate::layout::segment_id_from_filename(&path) else {
            continue;
        };
        if cat.get(&id).map(|s| s.available).unwrap_or(false) {
            continue;
        }
        let (hash, size) = crate::tier::hash_file(&path)?;
        let tier = placement.get(&id).map(|p| p.tier).unwrap_or(TierClass::Hot);
        let summary = summarize_segment_file(id, tier, &path, hash, size, limits)?;
        cat.upsert(summary);
    }

    if let Some(prior) = prior {
        cat.merge_offline_from(prior, placement);
    }

    Ok(cat)
}

/// Persist segment catalog (atomic durable replace, DEF-021).
pub fn write_segment_catalog(
    path: &Path,
    store_id: [u8; 16],
    catalog: &SegmentCatalog,
) -> Result<(), StoreError> {
    let bytes = encode_catalog(store_id, catalog);
    crate::atomic_file::write_atomic(path, &bytes)?;
    Ok(())
}

/// Load when magic/version/store_id/hash match.
pub fn try_load_segment_catalog(
    path: &Path,
    store_id: [u8; 16],
) -> Result<Option<SegmentCatalog>, StoreError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    Ok(decode_catalog(&bytes, store_id))
}

fn encode_catalog(store_id: [u8; 16], catalog: &SegmentCatalog) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&store_id);
    let entries: Vec<_> = catalog.summaries().cloned().collect();
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    for s in &entries {
        out.extend_from_slice(&s.segment_id);
        out.push(s.tier as u8);
        out.push(u8::from(s.available));
        out.extend_from_slice(&s.size.to_le_bytes());
        out.extend_from_slice(&s.content_hash);
        out.extend_from_slice(&s.verified_frames.to_le_bytes());
        out.extend_from_slice(&s.item_events.to_le_bytes());
        out.extend_from_slice(&s.holes.to_le_bytes());
    }
    let mut hasher = Hasher::new();
    hasher.update(&out);
    out.extend_from_slice(hasher.finalize().as_bytes());
    out
}

fn decode_catalog(bytes: &[u8], store_id: [u8; 16]) -> Option<SegmentCatalog> {
    if bytes.len() < 8 + 4 + 16 + 4 + 32 {
        return None;
    }
    if &bytes[0..8] != MAGIC.as_slice() {
        return None;
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    if version != VERSION {
        return None;
    }
    let sid: [u8; 16] = bytes[12..28].try_into().ok()?;
    if sid != store_id {
        return None;
    }
    let n = u32::from_le_bytes(bytes[28..32].try_into().ok()?) as usize;
    let mut cursor = 32usize;
    let mut cat = SegmentCatalog::new();
    let entry_len = 16 + 1 + 1 + 8 + 32 + 8 + 8 + 8;
    for _ in 0..n {
        if cursor + entry_len > bytes.len().saturating_sub(32) {
            return None;
        }
        let segment_id: [u8; 16] = bytes[cursor..cursor + 16].try_into().ok()?;
        cursor += 16;
        let tier_b = bytes[cursor];
        cursor += 1;
        let available = bytes[cursor] != 0;
        cursor += 1;
        let size = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
        cursor += 8;
        let content_hash: [u8; 32] = bytes[cursor..cursor + 32].try_into().ok()?;
        cursor += 32;
        let verified_frames = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
        cursor += 8;
        let item_events = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
        cursor += 8;
        let holes = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
        cursor += 8;
        let tier = match tier_b {
            0 => TierClass::Hot,
            1 => TierClass::Warm,
            2 => TierClass::Cold,
            3 => TierClass::Archive,
            _ => return None,
        };
        cat.upsert(SegmentSummary {
            segment_id,
            tier,
            size,
            content_hash,
            verified_frames,
            item_events,
            holes,
            available,
        });
    }
    if cursor + 32 != bytes.len() {
        return None;
    }
    let mut hasher = Hasher::new();
    hasher.update(&bytes[..cursor]);
    if hasher.finalize().as_bytes() != &bytes[cursor..cursor + 32] {
        return None;
    }
    Some(cat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_catalog_roundtrip() {
        let mut cat = SegmentCatalog::new();
        cat.upsert(SegmentSummary {
            segment_id: [5u8; 16],
            tier: TierClass::Warm,
            size: 100,
            content_hash: [1u8; 32],
            verified_frames: 3,
            item_events: 2,
            holes: 0,
            available: true,
        });
        let bytes = encode_catalog([9u8; 16], &cat);
        let decoded = decode_catalog(&bytes, [9u8; 16]).unwrap();
        assert_eq!(decoded.len(), 1);
        let s = decoded.get(&[5u8; 16]).unwrap();
        assert_eq!(s.tier, TierClass::Warm);
        assert_eq!(s.item_events, 2);
    }
}
