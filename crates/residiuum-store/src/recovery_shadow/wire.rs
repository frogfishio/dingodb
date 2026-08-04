//! Versioned Recovery Shadow wire (`RSHD0001`) + streaming writer + salvage.

use crate::atomic_file;
use crate::error::StoreError;
use crate::layout::{hex16, StorePaths};
use blake3::Hasher;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// On-disk magic / version tag for Recovery Shadow files.
pub const RSH_MAGIC: &[u8; 8] = b"RSHD0001";

/// Put record tag.
pub const TAG_PUT: u8 = 1;
/// Tombstone record tag (delete; no payload).
pub const TAG_TOMBSTONE: u8 = 2;

const HASH_LEN: usize = 32;
const HEADER_LEN: usize = 8 + 16 + 16 + 8 + 4;

/// One Shadow record before encode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowRecord {
    /// Live put at `gen`.
    Put {
        /// Subject key bytes.
        key: Vec<u8>,
        /// Generation binding.
        gen: u64,
        /// Payload.
        value: Vec<u8>,
    },
    /// Tombstone at `gen` (suppresses older puts under recovery).
    Tombstone {
        /// Subject key bytes.
        key: Vec<u8>,
        /// Generation binding.
        gen: u64,
    },
}

impl ShadowRecord {
    fn key(&self) -> &[u8] {
        match self {
            Self::Put { key, .. } | Self::Tombstone { key, .. } => key,
        }
    }

    fn gen(&self) -> u64 {
        match self {
            Self::Put { gen, .. } | Self::Tombstone { gen, .. } => *gen,
        }
    }

    fn sort_key(&self) -> (&[u8], u64) {
        (self.key(), self.gen())
    }
}

/// Decoded complete Shadow (integrity verified).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedShadow {
    /// Owning store id.
    pub store_id: [u8; 16],
    /// Segment this Shadow covers.
    pub segment_id: [u8; 16],
    /// Seal / layout generation stamped on the file.
    pub generation: u64,
    /// Records in wire order (sorted).
    pub records: Vec<ShadowRecord>,
}

/// Outcome of attempting to load a Shadow file (fail-closed; never invents).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShadowLoad {
    /// Path missing.
    Missing,
    /// Truncated / partial / interrupted file — **never** P★.
    Incomplete,
    /// Magic/hash/length/store-segment mismatch.
    Corrupt {
        /// Human-readable reason.
        detail: String,
    },
    /// Complete verified Shadow.
    Ok(DecodedShadow),
}

/// Latest state for one subject after Shadow projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveState {
    /// Live value at generation.
    Put {
        /// Payload.
        value: Vec<u8>,
        /// Winning generation.
        gen: u64,
    },
    /// Deleted at generation (must not resurrect older puts).
    Tombstone {
        /// Winning generation.
        gen: u64,
    },
}

/// `recovery/shadow/`.
pub fn shadow_dir(paths: &StorePaths) -> PathBuf {
    paths.recovery_dir().join("shadow")
}

/// `recovery/shadow/{hex16}.rsh`.
pub fn shadow_path(paths: &StorePaths, segment_id: &[u8; 16]) -> PathBuf {
    shadow_dir(paths).join(format!("{}.rsh", hex16(segment_id)))
}

/// Streaming sequential Shadow builder (records sorted at [`ShadowWriter::finish`]).
#[derive(Debug)]
pub struct ShadowWriter {
    store_id: [u8; 16],
    segment_id: [u8; 16],
    generation: u64,
    records: Vec<ShadowRecord>,
}

impl ShadowWriter {
    /// Start a Shadow for `(store_id, segment_id, generation)`.
    pub fn new(store_id: [u8; 16], segment_id: [u8; 16], generation: u64) -> Self {
        Self {
            store_id,
            segment_id,
            generation,
            records: Vec::new(),
        }
    }

    /// Append a put (order need not be sorted; finish sorts).
    pub fn push_put(&mut self, key: Vec<u8>, gen: u64, value: Vec<u8>) {
        self.records.push(ShadowRecord::Put { key, gen, value });
    }

    /// Append a tombstone.
    pub fn push_tombstone(&mut self, key: Vec<u8>, gen: u64) {
        self.records.push(ShadowRecord::Tombstone { key, gen });
    }

    /// Number of records pushed so far.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether no records were pushed.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Finish into complete wire bytes (sorted, hashed trailer).
    pub fn finish(mut self) -> Vec<u8> {
        self.records
            .sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        encode_shadow(self.store_id, self.segment_id, self.generation, &self.records)
    }
}

/// Encode a complete Shadow file (records should already be unique-intent;
/// encode sorts for determinism).
pub fn encode_shadow(
    store_id: [u8; 16],
    segment_id: [u8; 16],
    generation: u64,
    records: &[ShadowRecord],
) -> Vec<u8> {
    let mut sorted: Vec<&ShadowRecord> = records.iter().collect();
    sorted.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));

    let mut out = Vec::with_capacity(HEADER_LEN + 64 * sorted.len() + HASH_LEN);
    out.extend_from_slice(RSH_MAGIC);
    out.extend_from_slice(&store_id);
    out.extend_from_slice(&segment_id);
    out.extend_from_slice(&generation.to_le_bytes());
    out.extend_from_slice(&(sorted.len() as u32).to_le_bytes());

    for rec in sorted {
        match rec {
            ShadowRecord::Put { key, gen, value } => {
                out.push(TAG_PUT);
                out.extend_from_slice(&(key.len() as u32).to_le_bytes());
                out.extend_from_slice(key);
                out.extend_from_slice(&gen.to_le_bytes());
                out.extend_from_slice(&(value.len() as u32).to_le_bytes());
                out.extend_from_slice(value);
                let h = record_hash_put(key, *gen, value);
                out.extend_from_slice(&h);
            }
            ShadowRecord::Tombstone { key, gen } => {
                out.push(TAG_TOMBSTONE);
                out.extend_from_slice(&(key.len() as u32).to_le_bytes());
                out.extend_from_slice(key);
                out.extend_from_slice(&gen.to_le_bytes());
                let h = record_hash_tombstone(key, *gen);
                out.extend_from_slice(&h);
            }
        }
    }

    let mut hasher = Hasher::new();
    hasher.update(&out);
    out.extend_from_slice(hasher.finalize().as_bytes());
    out
}

/// Decode and verify a Shadow blob. Incomplete / corrupt → [`ShadowLoad`].
pub fn decode_shadow(bytes: &[u8], expect_store: Option<[u8; 16]>) -> ShadowLoad {
    if bytes.len() < HEADER_LEN + HASH_LEN {
        return if bytes.is_empty() {
            ShadowLoad::Missing
        } else {
            ShadowLoad::Incomplete
        };
    }

    if &bytes[0..8] != RSH_MAGIC.as_slice() {
        return ShadowLoad::Corrupt {
            detail: "bad Recovery Shadow magic".into(),
        };
    }

    let store_id: [u8; 16] = match bytes[8..24].try_into() {
        Ok(v) => v,
        Err(_) => {
            return ShadowLoad::Corrupt {
                detail: "store_id truncated".into(),
            }
        }
    };
    if let Some(exp) = expect_store {
        if store_id != exp {
            return ShadowLoad::Corrupt {
                detail: "store_id mismatch".into(),
            };
        }
    }
    let segment_id: [u8; 16] = match bytes[24..40].try_into() {
        Ok(v) => v,
        Err(_) => {
            return ShadowLoad::Corrupt {
                detail: "segment_id truncated".into(),
            }
        }
    };
    let generation = u64::from_le_bytes(match bytes[40..48].try_into() {
        Ok(v) => v,
        Err(_) => {
            return ShadowLoad::Corrupt {
                detail: "generation truncated".into(),
            }
        }
    });
    let n = u32::from_le_bytes(match bytes[48..52].try_into() {
        Ok(v) => v,
        Err(_) => {
            return ShadowLoad::Corrupt {
                detail: "n_records truncated".into(),
            }
        }
    }) as usize;

    let body_end = bytes.len() - HASH_LEN;
    let mut cursor = HEADER_LEN;
    let mut records = Vec::with_capacity(n);
    for _ in 0..n {
        if cursor >= body_end {
            return ShadowLoad::Incomplete;
        }
        let tag = bytes[cursor];
        cursor += 1;
        if cursor + 4 > body_end {
            return ShadowLoad::Incomplete;
        }
        let key_len =
            u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0; 4])) as usize;
        cursor += 4;
        if cursor + key_len + 8 > body_end {
            return ShadowLoad::Incomplete;
        }
        let key = bytes[cursor..cursor + key_len].to_vec();
        cursor += key_len;
        let gen = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;

        match tag {
            TAG_PUT => {
                if cursor + 4 > body_end {
                    return ShadowLoad::Incomplete;
                }
                let value_len =
                    u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0; 4]))
                        as usize;
                cursor += 4;
                if cursor + value_len + HASH_LEN > body_end {
                    return ShadowLoad::Incomplete;
                }
                let value = bytes[cursor..cursor + value_len].to_vec();
                cursor += value_len;
                let got: [u8; 32] = match bytes[cursor..cursor + HASH_LEN].try_into() {
                    Ok(v) => v,
                    Err(_) => {
                        return ShadowLoad::Incomplete;
                    }
                };
                cursor += HASH_LEN;
                let want = record_hash_put(&key, gen, &value);
                if got != want {
                    return ShadowLoad::Corrupt {
                        detail: "put record_hash mismatch".into(),
                    };
                }
                records.push(ShadowRecord::Put { key, gen, value });
            }
            TAG_TOMBSTONE => {
                if cursor + HASH_LEN > body_end {
                    return ShadowLoad::Incomplete;
                }
                let got: [u8; 32] = match bytes[cursor..cursor + HASH_LEN].try_into() {
                    Ok(v) => v,
                    Err(_) => {
                        return ShadowLoad::Incomplete;
                    }
                };
                cursor += HASH_LEN;
                let want = record_hash_tombstone(&key, gen);
                if got != want {
                    return ShadowLoad::Corrupt {
                        detail: "tombstone record_hash mismatch".into(),
                    };
                }
                records.push(ShadowRecord::Tombstone { key, gen });
            }
            _ => {
                return ShadowLoad::Corrupt {
                    detail: format!("unknown record tag {tag}"),
                };
            }
        }
    }

    if cursor != body_end {
        return ShadowLoad::Corrupt {
            detail: "trailing garbage before content_hash".into(),
        };
    }

    let mut hasher = Hasher::new();
    hasher.update(&bytes[..body_end]);
    let want = *hasher.finalize().as_bytes();
    let got: [u8; 32] = match bytes[body_end..].try_into() {
        Ok(v) => v,
        Err(_) => {
            return ShadowLoad::Incomplete;
        }
    };
    if got != want {
        return ShadowLoad::Corrupt {
            detail: "content_hash mismatch".into(),
        };
    }

    ShadowLoad::Ok(DecodedShadow {
        store_id,
        segment_id,
        generation,
        records,
    })
}

/// Project exact latest generation per subject (tombstones suppress older puts).
pub fn project_live(records: &[ShadowRecord]) -> BTreeMap<Vec<u8>, LiveState> {
    let mut out: BTreeMap<Vec<u8>, LiveState> = BTreeMap::new();
    for rec in records {
        let key = rec.key().to_vec();
        let gen = rec.gen();
        let replace = match out.get(&key) {
            None => true,
            Some(LiveState::Put { gen: g, .. } | LiveState::Tombstone { gen: g }) => gen > *g,
        };
        if !replace {
            continue;
        }
        match rec {
            ShadowRecord::Put { value, .. } => {
                out.insert(
                    key,
                    LiveState::Put {
                        value: value.clone(),
                        gen,
                    },
                );
            }
            ShadowRecord::Tombstone { .. } => {
                out.insert(key, LiveState::Tombstone { gen });
            }
        }
    }
    out
}

/// Load Shadow from disk (missing / incomplete / corrupt / ok).
pub fn try_load_shadow(
    path: &Path,
    expect_store: Option<[u8; 16]>,
) -> Result<ShadowLoad, StoreError> {
    if !path.is_file() {
        return Ok(ShadowLoad::Missing);
    }
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ShadowLoad::Missing),
        Err(e) => return Err(StoreError::Io(e)),
    };
    if bytes.is_empty() {
        return Ok(ShadowLoad::Incomplete);
    }
    Ok(decode_shadow(&bytes, expect_store))
}

/// Atomically publish a complete Shadow (temp → sync → rename → dir sync).
pub fn publish_shadow(
    paths: &StorePaths,
    segment_id: &[u8; 16],
    bytes: &[u8],
) -> Result<(), StoreError> {
    // Refuse to publish incomplete/corrupt blobs as P★ candidates.
    match decode_shadow(bytes, None) {
        ShadowLoad::Ok(decoded) => {
            if &decoded.segment_id != segment_id {
                return Err(StoreError::CorruptMeta(
                    "recovery shadow segment_id does not match publish path",
                ));
            }
        }
        ShadowLoad::Missing | ShadowLoad::Incomplete => {
            return Err(StoreError::CorruptMeta(
                "refusing to publish incomplete recovery shadow",
            ));
        }
        ShadowLoad::Corrupt { .. } => {
            return Err(StoreError::CorruptMeta(
                "refusing to publish corrupt recovery shadow",
            ));
        }
    }
    fs::create_dir_all(shadow_dir(paths))?;
    let path = shadow_path(paths, segment_id);
    atomic_file::write_atomic(&path, bytes)?;
    Ok(())
}

fn record_hash_put(key: &[u8], gen: u64, value: &[u8]) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(&[TAG_PUT]);
    h.update(key);
    h.update(&gen.to_le_bytes());
    h.update(value);
    *h.finalize().as_bytes()
}

fn record_hash_tombstone(key: &[u8], gen: u64) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(&[TAG_TOMBSTONE]);
    h.update(key);
    h.update(&gen.to_le_bytes());
    *h.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sid(n: u8) -> [u8; 16] {
        let mut id = [0u8; 16];
        id[0] = n;
        id
    }

    #[test]
    fn roundtrip_put_and_tombstone() {
        let mut w = ShadowWriter::new(sid(1), sid(2), 7);
        w.push_put(b"a".to_vec(), 1, b"v1".to_vec());
        w.push_tombstone(b"a".to_vec(), 2);
        w.push_put(b"b".to_vec(), 1, b"vb".to_vec());
        let bytes = w.finish();
        match decode_shadow(&bytes, Some(sid(1))) {
            ShadowLoad::Ok(d) => {
                assert_eq!(d.generation, 7);
                assert_eq!(d.segment_id, sid(2));
                let live = project_live(&d.records);
                assert_eq!(live.get(&b"a"[..]), Some(&LiveState::Tombstone { gen: 2 }));
                assert_eq!(
                    live.get(&b"b"[..]),
                    Some(&LiveState::Put {
                        value: b"vb".to_vec(),
                        gen: 1
                    })
                );
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn tombstone_prevents_resurrection_across_merge() {
        // Older put gen=1, tombstone gen=2 wins — deleted must not reappear.
        let records = vec![
            ShadowRecord::Put {
                key: b"k".to_vec(),
                gen: 1,
                value: b"old".to_vec(),
            },
            ShadowRecord::Tombstone {
                key: b"k".to_vec(),
                gen: 2,
            },
        ];
        let live = project_live(&records);
        assert!(matches!(
            live.get(&b"k"[..]),
            Some(LiveState::Tombstone { gen: 2 })
        ));
    }

    #[test]
    fn truncated_file_is_incomplete_never_ok() {
        let bytes = encode_shadow(sid(1), sid(2), 1, &[]);
        let torn = &bytes[..bytes.len().saturating_sub(10)];
        assert!(matches!(decode_shadow(torn, None), ShadowLoad::Incomplete));
    }

    #[test]
    fn publish_atomic_and_load() {
        let dir = tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        paths.create_dirs().unwrap();
        let mut w = ShadowWriter::new(sid(9), sid(3), 4);
        w.push_put(b"x".to_vec(), 1, b"y".to_vec());
        let bytes = w.finish();
        publish_shadow(&paths, &sid(3), &bytes).unwrap();
        let path = shadow_path(&paths, &sid(3));
        match try_load_shadow(&path, Some(sid(9))).unwrap() {
            ShadowLoad::Ok(d) => assert_eq!(d.records.len(), 1),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn refuse_publish_incomplete() {
        let dir = tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        paths.create_dirs().unwrap();
        let err = publish_shadow(&paths, &sid(1), b"RSHD0001torn").unwrap_err();
        assert!(matches!(err, StoreError::CorruptMeta(_)));
    }
}
