//! Secondary index state and on-disk derived index files (Stage 6).
//!
//! Secondary indexes are **derived** projections. They never hold the sole copy
//! of authoritative data. States follow DX_SPEC §8.2.

use crate::error::StoreError;
use crate::layout::StorePaths;
use blake3::Hasher;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Lifecycle state of a secondary index (DX_SPEC §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexState {
    /// Build in progress; may be incomplete.
    Building,
    /// Ready for use; coverage matches declared fingerprint.
    Ready,
    /// Contents may lag authoritative data.
    Stale,
    /// Only a subset of keys are indexed (declared partial coverage).
    Partial,
    /// Last build failed; query must not trust this index.
    Failed,
    /// Online rebuild in progress.
    Rebuilding,
}

impl IndexState {
    /// Stable snake_case name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Building => "building",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Rebuilding => "rebuilding",
        }
    }

    fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Building),
            2 => Some(Self::Ready),
            3 => Some(Self::Stale),
            4 => Some(Self::Partial),
            5 => Some(Self::Failed),
            6 => Some(Self::Rebuilding),
            _ => None,
        }
    }

    fn as_u8(self) -> u8 {
        match self {
            Self::Building => 1,
            Self::Ready => 2,
            Self::Stale => 3,
            Self::Partial => 4,
            Self::Failed => 5,
            Self::Rebuilding => 6,
        }
    }

    /// Whether a query may use this index as an acceleration path.
    pub fn usable(self) -> bool {
        matches!(self, Self::Ready | Self::Partial)
    }

    /// Parse the snake_case name produced by [`Self::as_str`].
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "building" => Some(Self::Building),
            "ready" => Some(Self::Ready),
            "stale" => Some(Self::Stale),
            "partial" => Some(Self::Partial),
            "failed" => Some(Self::Failed),
            "rebuilding" => Some(Self::Rebuilding),
            _ => None,
        }
    }
}

const MAGIC: &[u8; 8] = b"DSIX0001";
const VERSION: u32 = 1;

/// Metadata for one secondary index definition + build state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondaryIndexMeta {
    /// Index name (unique within a collection).
    pub name: String,
    /// Collection the index applies to.
    pub collection: String,
    /// Ordered field paths (JSON dotted paths for Stage 6 field indexes).
    pub fields: Vec<String>,
    /// Lifecycle state.
    pub state: IndexState,
    /// Number of entries in the index body.
    pub entry_count: u64,
    /// Segment fingerprint the index was built against (empty if never ready).
    pub built_fingerprint: [u8; 32],
    /// Whether the index claims complete coverage of live collection keys.
    pub complete_coverage: bool,
}

/// In-memory secondary index: serialized field key → list of subject keys.
#[derive(Debug, Clone)]
pub struct SecondaryIndex {
    /// Index metadata.
    pub meta: SecondaryIndexMeta,
    /// Map from index key bytes → application subject keys (as subject bytes).
    pub entries: BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
}

impl SecondaryIndex {
    /// Create a new empty index in `Building` state.
    pub fn new_building(collection: &str, name: &str, fields: Vec<String>) -> Self {
        Self {
            meta: SecondaryIndexMeta {
                name: name.to_string(),
                collection: collection.to_string(),
                fields,
                state: IndexState::Building,
                entry_count: 0,
                built_fingerprint: [0u8; 32],
                complete_coverage: false,
            },
            entries: BTreeMap::new(),
        }
    }

    /// Insert a mapping (index key → subject).
    pub fn insert(&mut self, index_key: Vec<u8>, subject: Vec<u8>) {
        let list = self.entries.entry(index_key).or_default();
        if !list.iter().any(|s| s == &subject) {
            list.push(subject);
            self.meta.entry_count = self.meta.entry_count.saturating_add(1);
        }
    }

    /// Remove all entries for a subject.
    pub fn remove_subject(&mut self, subject: &[u8]) {
        let mut empty_keys = Vec::new();
        for (k, list) in self.entries.iter_mut() {
            let before = list.len();
            list.retain(|s| s.as_slice() != subject);
            let removed = before - list.len();
            self.meta.entry_count = self.meta.entry_count.saturating_sub(removed as u64);
            if list.is_empty() {
                empty_keys.push(k.clone());
            }
        }
        for k in empty_keys {
            self.entries.remove(&k);
        }
    }

    /// Lookup subjects for an exact index key.
    pub fn lookup(&self, index_key: &[u8]) -> &[Vec<u8>] {
        self.entries
            .get(index_key)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Mark ready after a full build.
    pub fn mark_ready(&mut self, fingerprint: [u8; 32]) {
        self.meta.state = IndexState::Ready;
        self.meta.built_fingerprint = fingerprint;
        self.meta.complete_coverage = true;
    }

    /// Mark stale (writes happened after build).
    pub fn mark_stale(&mut self) {
        if self.meta.state == IndexState::Ready || self.meta.state == IndexState::Partial {
            self.meta.state = IndexState::Stale;
            self.meta.complete_coverage = false;
        }
    }
}

/// Directory for secondary indexes of one collection.
pub fn secondary_index_dir(paths: &StorePaths, collection: &str) -> PathBuf {
    paths
        .indexes_dir()
        .join("sec")
        .join(sanitize_name(collection))
}

/// Path for one secondary index file.
pub fn secondary_index_path(paths: &StorePaths, collection: &str, name: &str) -> PathBuf {
    secondary_index_dir(paths, collection).join(format!("{}.six", sanitize_name(name)))
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Persist a secondary index (atomic replace).
pub fn write_secondary_index(
    path: &Path,
    store_id: [u8; 16],
    index: &SecondaryIndex,
) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = encode_secondary(store_id, index);
    let tmp = path.with_extension("six.tmp");
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Load a secondary index if magic/version/store_id match.
pub fn try_load_secondary_index(
    path: &Path,
    store_id: [u8; 16],
) -> Result<Option<SecondaryIndex>, StoreError> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(_) => return Ok(None),
    };
    Ok(decode_secondary(&bytes, store_id))
}

/// List secondary index files under a collection's sec directory.
pub fn list_secondary_index_paths(
    paths: &StorePaths,
    collection: &str,
) -> Result<Vec<PathBuf>, StoreError> {
    let dir = secondary_index_dir(paths, collection);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("six") && path.is_file() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Delete one secondary index file (never touches authoritative segments).
pub fn delete_secondary_index(path: &Path) -> Result<(), StoreError> {
    if path.is_file() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn encode_secondary(store_id: [u8; 16], index: &SecondaryIndex) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&store_id);
    write_str(&mut out, &index.meta.name);
    write_str(&mut out, &index.meta.collection);
    out.push(index.meta.state.as_u8());
    out.extend_from_slice(&index.meta.entry_count.to_le_bytes());
    out.extend_from_slice(&index.meta.built_fingerprint);
    out.push(if index.meta.complete_coverage { 1 } else { 0 });
    out.extend_from_slice(&(index.meta.fields.len() as u32).to_le_bytes());
    for f in &index.meta.fields {
        write_str(&mut out, f);
    }
    out.extend_from_slice(&(index.entries.len() as u32).to_le_bytes());
    for (k, subjects) in &index.entries {
        write_bytes(&mut out, k);
        out.extend_from_slice(&(subjects.len() as u32).to_le_bytes());
        for s in subjects {
            write_bytes(&mut out, s);
        }
    }
    let mut hasher = Hasher::new();
    hasher.update(&out);
    out.extend_from_slice(hasher.finalize().as_bytes());
    out
}

fn decode_secondary(bytes: &[u8], store_id: [u8; 16]) -> Option<SecondaryIndex> {
    if bytes.len() < 8 + 4 + 16 + 32 {
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
    let mut cursor = 28usize;
    let name = read_str(bytes, &mut cursor)?;
    let collection = read_str(bytes, &mut cursor)?;
    if cursor >= bytes.len() {
        return None;
    }
    let state = IndexState::from_u8(bytes[cursor])?;
    cursor += 1;
    if cursor + 8 + 32 + 1 > bytes.len() {
        return None;
    }
    let entry_count = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().ok()?);
    cursor += 8;
    let built_fingerprint: [u8; 32] = bytes[cursor..cursor + 32].try_into().ok()?;
    cursor += 32;
    let complete_coverage = bytes[cursor] != 0;
    cursor += 1;
    let n_fields = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().ok()?) as usize;
    cursor += 4;
    let mut fields = Vec::with_capacity(n_fields);
    for _ in 0..n_fields {
        fields.push(read_str(bytes, &mut cursor)?);
    }
    let n_entries = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().ok()?) as usize;
    cursor += 4;
    let mut entries = BTreeMap::new();
    for _ in 0..n_entries {
        let key = read_bytes(bytes, &mut cursor)?;
        let n_subj = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().ok()?) as usize;
        cursor += 4;
        let mut subjects = Vec::with_capacity(n_subj);
        for _ in 0..n_subj {
            subjects.push(read_bytes(bytes, &mut cursor)?);
        }
        entries.insert(key, subjects);
    }
    if cursor + 32 != bytes.len() {
        return None;
    }
    let mut hasher = Hasher::new();
    hasher.update(&bytes[..cursor]);
    if hasher.finalize().as_bytes() != &bytes[cursor..cursor + 32] {
        return None;
    }
    Some(SecondaryIndex {
        meta: SecondaryIndexMeta {
            name,
            collection,
            fields,
            state,
            entry_count,
            built_fingerprint,
            complete_coverage,
        },
        entries,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secondary_roundtrip() {
        let mut idx = SecondaryIndex::new_building("users", "by-email", vec!["email".into()]);
        idx.insert(b"a@x.com".to_vec(), b"subj1".to_vec());
        idx.insert(b"b@x.com".to_vec(), b"subj2".to_vec());
        idx.mark_ready([9u8; 32]);
        let store_id = [3u8; 16];
        let enc = encode_secondary(store_id, &idx);
        let dec = decode_secondary(&enc, store_id).unwrap();
        assert_eq!(dec.meta.name, "by-email");
        assert_eq!(dec.meta.state, IndexState::Ready);
        assert_eq!(dec.lookup(b"a@x.com"), &[b"subj1".to_vec()]);
    }
}
