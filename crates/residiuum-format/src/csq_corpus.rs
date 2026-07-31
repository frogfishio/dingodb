//! CSQ-3 deterministic, hash-addressed format mutation corpus.
//!
//! Freezes canonical microframes/microsegments and enumerates the finite
//! damage domains required by CORE_STORAGE_QUALIFICATION_SPEC §11.2 for the
//! small-frame bound. Full multi-megabyte campaigns remain under CSQ-7/CSQ-10;
//! this module is the **closed finite corpus generator** that CI must run.

use crate::cbor_envelope::EMPTY_ENVELOPE;
use crate::frame::{encode_frame, FrameHeader, FrameParts};
use crate::kinds::{FrameFlags, FrameKind};
use crate::{WIRE_MAJOR, WIRE_MINOR};
use blake3::Hasher;

/// Stable body for the primary CSQ-3 microframe (Residiuum identity only).
pub const CANONICAL_BODY: &[u8] = b"CSQ3";
/// Later healthy island body used after every damage cell.
pub const SURVIVOR_BODY: &[u8] = b"SURVIVOR-ISLAND";
/// Insertion alphabet for one-byte insertion at every boundary.
pub const INSERTION_ALPHABET: &[u8] = &[0x00, 0xff, 0x41, 0x52]; // NUL, 0xFF, 'A', 'R' (RESID…)
/// Byte replacements applied at every position.
pub const BYTE_REPLACEMENTS: &[u8] = &[0x00, 0xff, 0xa5];
/// Maximum contiguous hole length in the bounded hole corpus.
pub const HOLE_MAX_LEN: usize = 8;
/// Bound for exhaustive hole start positions inside the damaged primary frame.
pub const HOLE_REGION_CAP: usize = 48;

/// Corpus schema / package label.
pub const CORPUS_PROFILE: &str = "residiuum-core-storage-v1";
/// Generator version (bump when mutation enum changes).
pub const CORPUS_GENERATOR: &str = "csq3-corpus-v1";

/// One deterministic mutation cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    /// Flip a single bit at global bit index `bit` (byte = bit/8, mask = 1<<(bit%8)).
    BitFlip {
        /// Global bit index into the base buffer.
        bit: usize,
    },
    /// Replace byte at `index` with `value`.
    ByteReplace {
        /// Byte index.
        index: usize,
        /// Replacement value.
        value: u8,
    },
    /// Keep only the first `len` bytes.
    Truncate {
        /// Prefix length retained.
        len: usize,
    },
    /// Insert `byte` before `index` (index == len appends).
    Insert {
        /// Insertion index.
        index: usize,
        /// Byte inserted.
        byte: u8,
    },
    /// Delete the single byte at `index`.
    Delete {
        /// Byte index removed.
        index: usize,
    },
    /// Zero-fill / remove `[start, end)` (end exclusive) as a contiguous hole.
    Hole {
        /// Inclusive start.
        start: usize,
        /// Exclusive end.
        end: usize,
    },
}

impl Mutation {
    /// Stable hash-addressed cell id (blake3 of generator + mutation encoding).
    pub fn cell_id(&self) -> String {
        let mut h = Hasher::new();
        h.update(CORPUS_GENERATOR.as_bytes());
        h.update(b"|");
        match self {
            Mutation::BitFlip { bit } => {
                h.update(b"bitflip|");
                h.update(&(*bit as u64).to_le_bytes());
            }
            Mutation::ByteReplace { index, value } => {
                h.update(b"byterepl|");
                h.update(&(*index as u64).to_le_bytes());
                h.update(&[*value]);
            }
            Mutation::Truncate { len } => {
                h.update(b"trunc|");
                h.update(&(*len as u64).to_le_bytes());
            }
            Mutation::Insert { index, byte } => {
                h.update(b"insert|");
                h.update(&(*index as u64).to_le_bytes());
                h.update(&[*byte]);
            }
            Mutation::Delete { index } => {
                h.update(b"delete|");
                h.update(&(*index as u64).to_le_bytes());
            }
            Mutation::Hole { start, end } => {
                h.update(b"hole|");
                h.update(&(*start as u64).to_le_bytes());
                h.update(&(*end as u64).to_le_bytes());
            }
        }
        hex32(h.finalize().as_bytes())
    }
}

fn hex32(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(H[(b >> 4) as usize] as char);
        out.push(H[(b & 0xf) as usize] as char);
    }
    out
}

fn event_id(tag: u8) -> [u8; 16] {
    let mut id = [0u8; 16];
    id[0] = tag;
    id[15] = 0x3; // CSQ-3 marker
    id
}

fn item_parts(tag: u8, body: &[u8]) -> FrameParts {
    FrameParts {
        header: FrameHeader {
            wire_major: WIRE_MAJOR,
            wire_minor: WIRE_MINOR,
            frame_kind: FrameKind::ItemEvent.as_u8(),
            flags: FrameFlags::default(),
            envelope_len: EMPTY_ENVELOPE.len() as u32,
            body_len: body.len() as u64,
            logical_len: body.len() as u64,
            writer_sequence: 1,
            event_id: event_id(tag),
        },
        envelope: EMPTY_ENVELOPE.to_vec(),
        body: body.to_vec(),
    }
}

/// Encode the frozen canonical CSQ-3 microframe (primary damage target).
pub fn canonical_microframe() -> Vec<u8> {
    encode_frame(&item_parts(0x31, CANONICAL_BODY)).expect("canonical encode")
}

/// Encode the frozen survivor island microframe.
pub fn survivor_microframe() -> Vec<u8> {
    encode_frame(&item_parts(0xff, SURVIVOR_BODY)).expect("survivor encode")
}

/// Microsegment: primary + survivor (no garbage).
pub fn canonical_microsegment() -> Vec<u8> {
    let mut v = canonical_microframe();
    v.extend_from_slice(&survivor_microframe());
    v
}

/// Blake3 hash of arbitrary bytes (hex).
pub fn content_hash_hex(bytes: &[u8]) -> String {
    let mut h = Hasher::new();
    h.update(bytes);
    hex32(h.finalize().as_bytes())
}

/// Apply a mutation to a base buffer.
pub fn apply_mutation(base: &[u8], m: &Mutation) -> Vec<u8> {
    match *m {
        Mutation::BitFlip { bit } => {
            let mut out = base.to_vec();
            let i = bit / 8;
            let mask = 1u8 << (bit % 8);
            if i < out.len() {
                out[i] ^= mask;
            }
            out
        }
        Mutation::ByteReplace { index, value } => {
            let mut out = base.to_vec();
            if index < out.len() {
                out[index] = value;
            }
            out
        }
        Mutation::Truncate { len } => base[..len.min(base.len())].to_vec(),
        Mutation::Insert { index, byte } => {
            let mut out = Vec::with_capacity(base.len() + 1);
            let i = index.min(base.len());
            out.extend_from_slice(&base[..i]);
            out.push(byte);
            out.extend_from_slice(&base[i..]);
            out
        }
        Mutation::Delete { index } => {
            let mut out = Vec::with_capacity(base.len().saturating_sub(1));
            if index < base.len() {
                out.extend_from_slice(&base[..index]);
                out.extend_from_slice(&base[index + 1..]);
            } else {
                out.extend_from_slice(base);
            }
            out
        }
        Mutation::Hole { start, end } => {
            let mut out = base.to_vec();
            let s = start.min(out.len());
            let e = end.min(out.len()).max(s);
            for b in &mut out[s..e] {
                *b = 0x00;
            }
            out
        }
    }
}

/// Enumerate every single-bit flip over `len` bytes.
pub fn bit_flip_mutations(len: usize) -> impl Iterator<Item = Mutation> {
    (0..len * 8).map(|bit| Mutation::BitFlip { bit })
}

/// Enumerate byte replacements at every index for [`BYTE_REPLACEMENTS`].
pub fn byte_replace_mutations(len: usize) -> impl Iterator<Item = Mutation> {
    (0..len).flat_map(|index| {
        BYTE_REPLACEMENTS
            .iter()
            .copied()
            .map(move |value| Mutation::ByteReplace { index, value })
    })
}

/// Truncate at every byte boundary `0..len` (exclusive of full length).
pub fn truncate_mutations(len: usize) -> impl Iterator<Item = Mutation> {
    (0..len).map(|n| Mutation::Truncate { len: n })
}

/// Insert each alphabet byte at every boundary `0..=len`.
pub fn insert_mutations(len: usize) -> impl Iterator<Item = Mutation> {
    (0..=len).flat_map(|index| {
        INSERTION_ALPHABET
            .iter()
            .copied()
            .map(move |byte| Mutation::Insert { index, byte })
    })
}

/// Delete every single byte.
pub fn delete_mutations(len: usize) -> impl Iterator<Item = Mutation> {
    (0..len).map(|index| Mutation::Delete { index })
}

/// Bounded contiguous hole corpus over the first `cap` bytes (or full len).
pub fn hole_mutations(len: usize) -> impl Iterator<Item = Mutation> {
    let region = len.min(HOLE_REGION_CAP);
    (0..region).flat_map(move |start| {
        (1..=HOLE_MAX_LEN)
            .filter(move |&h| start + h <= region)
            .map(move |h| Mutation::Hole {
                start,
                end: start + h,
            })
    })
}

/// Pairwise multi-fault covering array: every ordered pair of fault *classes*
/// applied at two distinct deterministic sites (not full cartesian of cells).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultClass {
    /// Bit flip class.
    Bit,
    /// Byte replace class.
    Byte,
    /// Truncate class.
    Trunc,
    /// Insert class.
    Insert,
    /// Delete class.
    Delete,
}

/// One pairwise multi-fault schedule entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairFault {
    /// First fault class.
    pub a: FaultClass,
    /// Second fault class.
    pub b: FaultClass,
    /// Concrete mutations (applied left-to-right).
    pub mutations: [Mutation; 2],
}

fn sample_fault(class: FaultClass, len: usize, salt: usize) -> Mutation {
    match class {
        FaultClass::Bit => Mutation::BitFlip {
            bit: (salt * 7) % (len * 8).max(1),
        },
        FaultClass::Byte => Mutation::ByteReplace {
            index: (salt * 3) % len.max(1),
            value: BYTE_REPLACEMENTS[salt % BYTE_REPLACEMENTS.len()],
        },
        FaultClass::Trunc => Mutation::Truncate {
            len: (len / 2).max(1).min(len.saturating_sub(1)),
        },
        FaultClass::Insert => Mutation::Insert {
            index: salt % (len + 1),
            byte: INSERTION_ALPHABET[salt % INSERTION_ALPHABET.len()],
        },
        FaultClass::Delete => Mutation::Delete {
            index: salt % len.max(1),
        },
    }
}

/// Deterministic pairwise covering of fault classes (order matters).
pub fn pairwise_fault_covering(len: usize) -> Vec<PairFault> {
    let classes = [
        FaultClass::Bit,
        FaultClass::Byte,
        FaultClass::Trunc,
        FaultClass::Insert,
        FaultClass::Delete,
    ];
    let mut out = Vec::new();
    let mut salt = 1usize;
    for &a in &classes {
        for &b in &classes {
            if a == FaultClass::Trunc && b != FaultClass::Trunc {
                // Truncate first shrinks the buffer; second salt uses shrunk bound carefully.
            }
            let m0 = sample_fault(a, len, salt);
            salt += 1;
            // Second mutation samples against original len; apply order is defined.
            let m1 = sample_fault(b, len, salt);
            salt += 1;
            out.push(PairFault {
                a,
                b,
                mutations: [m0, m1],
            });
        }
    }
    out
}

/// Apply an ordered pair of mutations (second sees the first result length).
pub fn apply_pair(base: &[u8], pair: &PairFault) -> Vec<u8> {
    let mid = apply_mutation(base, &pair.mutations[0]);
    // Re-sample second mutation indices that may be OOB after truncate/delete.
    let m1 = match &pair.mutations[1] {
        Mutation::BitFlip { bit } if mid.is_empty() => Mutation::BitFlip { bit: 0 },
        Mutation::BitFlip { bit } if *bit >= mid.len() * 8 => Mutation::BitFlip {
            bit: bit % (mid.len() * 8),
        },
        Mutation::ByteReplace { index, value } if mid.is_empty() => Mutation::ByteReplace {
            index: 0,
            value: *value,
        },
        Mutation::ByteReplace { index, value } if *index >= mid.len() => Mutation::ByteReplace {
            index: index % mid.len(),
            value: *value,
        },
        Mutation::Delete { index } if mid.is_empty() => Mutation::Delete { index: 0 },
        Mutation::Delete { index } if *index >= mid.len() => Mutation::Delete {
            index: index % mid.len(),
        },
        Mutation::Insert { index, byte } if *index > mid.len() => Mutation::Insert {
            index: mid.len(),
            byte: *byte,
        },
        Mutation::Truncate { len } => Mutation::Truncate {
            len: (*len).min(mid.len()),
        },
        other => other.clone(),
    };
    apply_mutation(&mid, &m1)
}

/// Manifest row for a frozen artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenArtifact {
    /// Stable name.
    pub name: &'static str,
    /// Blake3 hex of bytes.
    pub blake3_hex: String,
    /// Byte length.
    pub len: usize,
}

/// Compute frozen artifact table for the canonical set.
pub fn frozen_artifacts() -> Vec<FrozenArtifact> {
    let primary = canonical_microframe();
    let survivor = survivor_microframe();
    let segment = canonical_microsegment();
    vec![
        FrozenArtifact {
            name: "canonical_microframe",
            blake3_hex: content_hash_hex(&primary),
            len: primary.len(),
        },
        FrozenArtifact {
            name: "survivor_microframe",
            blake3_hex: content_hash_hex(&survivor),
            len: survivor.len(),
        },
        FrozenArtifact {
            name: "canonical_microsegment",
            blake3_hex: content_hash_hex(&segment),
            len: segment.len(),
        },
    ]
}

/// Encode an unsupported-kind frame that must remain recoverable as opaque.
pub fn unsupported_kind_microframe() -> Vec<u8> {
    let mut parts = item_parts(0x55, b"opaque-ext");
    parts.header.frame_kind = 200;
    encode_frame(&parts).expect("unsupported kind encode")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_is_stable_and_nonempty() {
        let a = canonical_microframe();
        let b = canonical_microframe();
        assert_eq!(a, b);
        assert!(a.len() > 100);
        assert_eq!(&a[0..8], b"RESIDFRM");
        assert!(!content_hash_hex(&a).is_empty());
    }

    #[test]
    fn mutation_cell_ids_are_unique_for_bitflips() {
        let mut ids = std::collections::HashSet::new();
        for m in bit_flip_mutations(4) {
            assert!(ids.insert(m.cell_id()));
        }
        assert_eq!(ids.len(), 32);
    }

    #[test]
    fn apply_bitflip_changes_exactly_one_bit() {
        let base = b"abcd".to_vec();
        let out = apply_mutation(&base, &Mutation::BitFlip { bit: 0 });
        assert_ne!(out, base);
        let mut diff = 0u32;
        for (x, y) in base.iter().zip(out.iter()) {
            diff += (x ^ y).count_ones();
        }
        assert_eq!(diff, 1);
    }
}
