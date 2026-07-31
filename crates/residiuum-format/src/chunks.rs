//! Chunk reassembly and partial extent maps (FORMAT_SPEC §8).
//!
//! Each chunk remains independently verifiable. Reassembly never silently fills
//! missing extents with zeros or neighboring bytes. This module operates on
//! already-verified chunk pieces (salvage layer); durable chunk manifests land
//! with later store stages.

use crate::integrity::{body_hash, BODY_HASH_LEN};
use crate::scan::ByteRange;
use std::collections::BTreeMap;

/// One independently verified payload chunk contribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPiece {
    /// Stable item / logical object identifier (usually the parent event id).
    pub item_id: [u8; 16],
    /// Zero-based index in the ordered chunk map.
    pub index: u32,
    /// Declared total number of chunks for this item.
    pub total: u32,
    /// Logical length of this piece (may equal `body.len()` for raw chunks).
    pub logical_len: u64,
    /// Verified stored body bytes for this chunk.
    pub body: Vec<u8>,
}

/// Draft fixed body layout for a payload chunk (Stage 2d test / intermediate).
///
/// ```text
/// item_id[16]  index:u32  total:u32  logical_len:u64  payload...
/// ```
pub const CHUNK_BODY_HEADER_LEN: usize = 16 + 4 + 4 + 8;

/// Encode a draft chunk body.
pub fn encode_chunk_body(piece: &ChunkPiece) -> Vec<u8> {
    let mut out = Vec::with_capacity(CHUNK_BODY_HEADER_LEN + piece.body.len());
    out.extend_from_slice(&piece.item_id);
    out.extend_from_slice(&piece.index.to_le_bytes());
    out.extend_from_slice(&piece.total.to_le_bytes());
    out.extend_from_slice(&piece.logical_len.to_le_bytes());
    out.extend_from_slice(&piece.body);
    out
}

/// Decode a draft chunk body into a [`ChunkPiece`].
pub fn decode_chunk_body(body: &[u8]) -> Option<ChunkPiece> {
    if body.len() < CHUNK_BODY_HEADER_LEN {
        return None;
    }
    let item_id: [u8; 16] = body[0..16].try_into().ok()?;
    let index = u32::from_le_bytes(body[16..20].try_into().ok()?);
    let total = u32::from_le_bytes(body[20..24].try_into().ok()?);
    let logical_len = u64::from_le_bytes(body[24..32].try_into().ok()?);
    let payload = body[32..].to_vec();
    Some(ChunkPiece {
        item_id,
        index,
        total,
        logical_len,
        body: payload,
    })
}

/// One known extent in a partial or complete map (byte offsets in the logical stream).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalExtent {
    /// Chunk index that produced this extent.
    pub index: u32,
    /// Logical byte range covered by this chunk in the reassembled stream.
    pub range: ByteRange,
    /// Whether the chunk body is present and verified for this slot.
    pub present: bool,
}

/// Reassembly result for one item (FORMAT_SPEC §8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReassemblyState {
    /// Every required chunk verifies and the full content hash matches when provided.
    Complete {
        /// Concatenated logical body in index order.
        body: Vec<u8>,
        /// BLAKE3-256 of the concatenated body.
        content_hash: [u8; BODY_HASH_LEN],
    },
    /// At least one chunk verifies and at least one required chunk is missing.
    Partial {
        /// Ordered extent map (every index `0..total` appears once).
        extents: Vec<LogicalExtent>,
        /// Missing chunk indices.
        missing: Vec<u32>,
    },
    /// No payload chunk is available for this item.
    Unavailable,
    /// More than one verified chunk claims the same manifest position with different content.
    Conflicting {
        /// Index where conflicting pieces were observed.
        index: u32,
        /// Distinct body digests observed at that index.
        body_hashes: Vec<[u8; BODY_HASH_LEN]>,
    },
}

/// Reassemble verified chunk pieces for a single `item_id` (FORMAT_SPEC §8).
///
/// `expected_content_hash`, when `Some`, is checked only for the `Complete` path.
/// Missing pieces never synthesize zeros; the result is `Partial` or `Unavailable`.
pub fn reassemble_chunks(
    pieces: &[ChunkPiece],
    expected_content_hash: Option<[u8; BODY_HASH_LEN]>,
) -> ReassemblyState {
    if pieces.is_empty() {
        return ReassemblyState::Unavailable;
    }

    let total = pieces[0].total;
    if total == 0 {
        return ReassemblyState::Unavailable;
    }
    if pieces
        .iter()
        .any(|p| p.total != total || p.item_id != pieces[0].item_id)
    {
        // Inconsistent manifest declarations: treat as conflict at the first mismatch.
        return ReassemblyState::Conflicting {
            index: pieces
                .iter()
                .find(|p| p.total != total || p.item_id != pieces[0].item_id)
                .map(|p| p.index)
                .unwrap_or(0),
            body_hashes: pieces.iter().map(|p| body_hash(&p.body)).collect(),
        };
    }

    // Group by index; detect conflicting content at the same index.
    let mut by_index: BTreeMap<u32, Vec<&ChunkPiece>> = BTreeMap::new();
    for p in pieces {
        if p.index >= total {
            return ReassemblyState::Conflicting {
                index: p.index,
                body_hashes: vec![body_hash(&p.body)],
            };
        }
        by_index.entry(p.index).or_default().push(p);
    }

    for (index, group) in &by_index {
        let first = body_hash(&group[0].body);
        if group.iter().any(|p| body_hash(&p.body) != first) {
            let mut hashes: Vec<_> = group.iter().map(|p| body_hash(&p.body)).collect();
            hashes.sort();
            hashes.dedup();
            return ReassemblyState::Conflicting {
                index: *index,
                body_hashes: hashes,
            };
        }
    }

    // Build ordered extent map using logical lengths of the first piece per index.
    let mut extents = Vec::with_capacity(total as usize);
    let mut missing = Vec::new();
    let mut cursor = 0u64;
    let mut complete_body = Vec::new();
    let mut all_present = true;

    for i in 0..total {
        match by_index.get(&i) {
            Some(group) => {
                let p = group[0];
                let start = cursor;
                let end = cursor.saturating_add(p.logical_len);
                extents.push(LogicalExtent {
                    index: i,
                    range: ByteRange::new(start, end),
                    present: true,
                });
                complete_body.extend_from_slice(&p.body);
                cursor = end;
            }
            None => {
                all_present = false;
                missing.push(i);
                // Unknown length for missing chunk: record a zero-length placeholder
                // range at the current cursor so the map stays ordered by index.
                extents.push(LogicalExtent {
                    index: i,
                    range: ByteRange::new(cursor, cursor),
                    present: false,
                });
            }
        }
    }

    if !all_present {
        return ReassemblyState::Partial { extents, missing };
    }

    let content_hash = body_hash(&complete_body);
    if let Some(expected) = expected_content_hash {
        if expected != content_hash {
            // Full set present but content hash disagrees — treat as conflict on index 0.
            return ReassemblyState::Conflicting {
                index: 0,
                body_hashes: vec![content_hash, expected],
            };
        }
    }

    ReassemblyState::Complete {
        body: complete_body,
        content_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(item: u8, index: u32, total: u32, body: &[u8]) -> ChunkPiece {
        let mut item_id = [0u8; 16];
        item_id[0] = item;
        ChunkPiece {
            item_id,
            index,
            total,
            logical_len: body.len() as u64,
            body: body.to_vec(),
        }
    }

    #[test]
    fn complete_two_chunks() {
        let pieces = vec![piece(1, 0, 2, b"hel"), piece(1, 1, 2, b"lo")];
        match reassemble_chunks(&pieces, None) {
            ReassemblyState::Complete { body, content_hash } => {
                assert_eq!(body, b"hello");
                assert_eq!(content_hash, body_hash(b"hello"));
            }
            other => panic!("expected complete: {other:?}"),
        }
    }

    #[test]
    fn partial_missing_middle() {
        let pieces = vec![piece(1, 0, 3, b"A"), piece(1, 2, 3, b"C")];
        match reassemble_chunks(&pieces, None) {
            ReassemblyState::Partial { extents, missing } => {
                assert_eq!(missing, vec![1]);
                assert_eq!(extents.len(), 3);
                assert!(extents[0].present);
                assert!(!extents[1].present);
                assert!(extents[2].present);
            }
            other => panic!("expected partial: {other:?}"),
        }
    }

    #[test]
    fn conflicting_same_index() {
        let pieces = vec![piece(1, 0, 1, b"a"), piece(1, 0, 1, b"b")];
        assert!(matches!(
            reassemble_chunks(&pieces, None),
            ReassemblyState::Conflicting { index: 0, .. }
        ));
    }

    #[test]
    fn unavailable_empty() {
        assert_eq!(reassemble_chunks(&[], None), ReassemblyState::Unavailable);
    }

    #[test]
    fn draft_body_roundtrip() {
        let p = piece(7, 1, 4, b"payload");
        let encoded = encode_chunk_body(&p);
        let decoded = decode_chunk_body(&encoded).unwrap();
        assert_eq!(decoded, p);
    }
}
