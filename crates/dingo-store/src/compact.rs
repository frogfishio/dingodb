//! Compaction and derived checkpoints (OVERVIEW §13, Stage 6).
//!
//! Compaction creates new immutable segments from verified live state while
//! preserving item identities. It MUST NOT convert an uncertain history into an
//! apparently complete snapshot: source segments remain until explicitly
//! reclaimed, and reports declare coverage.

use crate::envelope::{encode_item_envelope, EventKind, ItemEnvelope};
use crate::error::StoreError;
use crate::index::{IndexEntry, PrimaryIndex};
use crate::layout::{hex16, StorePaths};
use dingo_format::{ActiveSegment, FrameKind, SafetyLimits, SegmentId};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Report from a live-state compaction pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactReport {
    /// New segment that received live puts.
    pub segment_id: [u8; 16],
    /// Number of live subjects rewritten.
    pub live_subjects_written: usize,
    /// Source segment files that contributed to the pre-compaction index
    /// (paths relative to store root when possible).
    pub source_segments: Vec<String>,
    /// Whether source segments were left in place (true for Stage 6 default).
    pub sources_retained: bool,
    /// Declared coverage: `"live-projection"` means only current live values
    /// were rewritten; full event history remains in source segments.
    pub coverage: &'static str,
}

/// Derived checkpoint metadata (OVERVIEW §13.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointMeta {
    /// Opaque checkpoint id (16 bytes).
    pub checkpoint_id: [u8; 16],
    /// Live subject count at capture.
    pub live_subjects: usize,
    /// Segment fingerprint at capture.
    pub segment_fingerprint: [u8; 32],
    /// Coverage declaration.
    pub coverage: String,
    /// Projection rule tag.
    pub projection: String,
    /// Nanoseconds timestamp when written.
    pub created_ns: u64,
}

const CHECKPOINT_MAGIC: &[u8; 8] = b"DCHKPT01";

/// Write live puts for every live index entry into a new sealed segment.
///
/// Does **not** delete source segments (hole honesty / history retention).
/// New events receive fresh `event_id`s; `item_id` and bodies are preserved.
pub fn compact_live_to_new_segment(
    paths: &StorePaths,
    store_id: [u8; 16],
    limits: SafetyLimits,
    index: &PrimaryIndex,
    source_segment_names: &[String],
    next_segment_id: [u8; 16],
    mint_event_id: &mut dyn FnMut() -> [u8; 16],
    created_ns: u64,
) -> Result<CompactReport, StoreError> {
    let ids = SegmentId::new(store_id, next_segment_id);
    let mut seg = ActiveSegment::create(ids, limits, created_ns)?;

    let mut written = 0usize;
    for (subject, entry) in index.iter_all() {
        let IndexEntry::Live(lv) = entry else {
            continue;
        };
        let event_id = mint_event_id();
        let env = ItemEnvelope {
            store_id,
            segment_id: next_segment_id,
            item_id: lv.item_id,
            event_kind: EventKind::Put,
            created_ns,
            subject: subject.clone(),
        };
        let envelope = encode_item_envelope(&env).map_err(StoreError::BadEnvelope)?;
        seg.append(FrameKind::ItemEvent, &envelope, &lv.body, event_id)?;
        written += 1;
    }

    let sealed = seg.seal()?;
    let dest = paths.sealed_segment(&next_segment_id);
    {
        let mut out = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&dest)?;
        out.write_all(sealed.as_bytes())?;
        out.sync_all()?;
    }
    sync_dir_best_effort(&paths.segments_dir());

    Ok(CompactReport {
        segment_id: next_segment_id,
        live_subjects_written: written,
        source_segments: source_segment_names.to_vec(),
        sources_retained: true,
        coverage: "live-projection",
    })
}

/// Write a derived checkpoint under `snapshots/` with declared coverage.
pub fn write_checkpoint(
    paths: &StorePaths,
    store_id: [u8; 16],
    meta: &CheckpointMeta,
    live_pairs: &[(&[u8], &[u8])],
) -> Result<PathBuf, StoreError> {
    let dir = paths.snapshots_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.ckpt", hex16(&meta.checkpoint_id)));
    let mut out = Vec::new();
    out.extend_from_slice(CHECKPOINT_MAGIC);
    out.extend_from_slice(&store_id);
    out.extend_from_slice(&meta.checkpoint_id);
    out.extend_from_slice(&meta.created_ns.to_le_bytes());
    out.extend_from_slice(&(meta.live_subjects as u64).to_le_bytes());
    out.extend_from_slice(&meta.segment_fingerprint);
    write_str(&mut out, &meta.coverage);
    write_str(&mut out, &meta.projection);
    out.extend_from_slice(&(live_pairs.len() as u32).to_le_bytes());
    for (subj, body) in live_pairs {
        write_bytes(&mut out, subj);
        write_bytes(&mut out, body);
    }
    let tmp = path.with_extension("ckpt.tmp");
    fs::write(&tmp, &out)?;
    fs::rename(&tmp, &path)?;
    Ok(path)
}

/// Load checkpoint metadata + live pairs if the file verifies as a draft checkpoint.
pub fn try_load_checkpoint(
    path: &std::path::Path,
    store_id: [u8; 16],
) -> Result<Option<(CheckpointMeta, Vec<(Vec<u8>, Vec<u8>)>)>, StoreError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    if bytes.len() < 8 + 16 + 16 + 8 + 8 + 32 {
        return Ok(None);
    }
    if &bytes[0..8] != CHECKPOINT_MAGIC.as_slice() {
        return Ok(None);
    }
    let sid: [u8; 16] = bytes[8..24].try_into().unwrap_or([0; 16]);
    if sid != store_id {
        return Ok(None);
    }
    let checkpoint_id: [u8; 16] = bytes[24..40].try_into().unwrap_or([0; 16]);
    let created_ns = u64::from_le_bytes(bytes[40..48].try_into().unwrap_or([0; 8]));
    let live_subjects = u64::from_le_bytes(bytes[48..56].try_into().unwrap_or([0; 8])) as usize;
    let segment_fingerprint: [u8; 32] = bytes[56..88].try_into().unwrap_or([0; 32]);
    let mut cursor = 88usize;
    let Some(coverage) = read_str(&bytes, &mut cursor) else {
        return Ok(None);
    };
    let Some(projection) = read_str(&bytes, &mut cursor) else {
        return Ok(None);
    };
    if cursor + 4 > bytes.len() {
        return Ok(None);
    }
    let n = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0; 4])) as usize;
    cursor += 4;
    let mut pairs = Vec::with_capacity(n);
    for _ in 0..n {
        let Some(subj) = read_bytes(&bytes, &mut cursor) else {
            return Ok(None);
        };
        let Some(body) = read_bytes(&bytes, &mut cursor) else {
            return Ok(None);
        };
        pairs.push((subj, body));
    }
    if cursor != bytes.len() {
        return Ok(None);
    }
    Ok(Some((
        CheckpointMeta {
            checkpoint_id,
            live_subjects,
            segment_fingerprint,
            coverage,
            projection,
            created_ns,
        },
        pairs,
    )))
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    write_bytes(out, s.as_bytes());
}

fn write_bytes(out: &mut Vec<u8>, b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_le_bytes());
    out.extend_from_slice(b);
}

fn read_str(bytes: &[u8], cursor: &mut usize) -> Option<String> {
    let b = read_bytes(bytes, cursor)?;
    String::from_utf8(b).ok()
}

fn read_bytes(bytes: &[u8], cursor: &mut usize) -> Option<Vec<u8>> {
    if *cursor + 4 > bytes.len() {
        return None;
    }
    let len = u32::from_le_bytes(bytes[*cursor..*cursor + 4].try_into().ok()?) as usize;
    *cursor += 4;
    if *cursor + len > bytes.len() {
        return None;
    }
    let v = bytes[*cursor..*cursor + len].to_vec();
    *cursor += len;
    Some(v)
}

fn sync_dir_best_effort(path: &std::path::Path) {
    #[cfg(unix)]
    {
        if let Ok(dir) = File::open(path) {
            let _ = dir.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

/// Wall-clock nanoseconds helper for checkpoint timestamps.
pub fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}
