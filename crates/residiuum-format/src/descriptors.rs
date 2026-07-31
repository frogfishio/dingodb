//! Heap / collection / stream descriptor CBOR bodies (`HEAP_SPEC` §34.5).

use crate::cbor_envelope::{
    decode_deterministic_uint_map, encode_deterministic_uint_map, CborEnvelopeError, CborValue,
};
use thiserror::Error;

/// Profile string embedded in heap descriptors.
pub const HEAP_DESCRIPTOR_PROFILE: &str = "residiuum-heap-v1";

/// Descriptor codec errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DescriptorError {
    /// CBOR shape invalid.
    #[error("descriptor cbor: {0}")]
    Cbor(#[from] CborEnvelopeError),
    /// Semantic field invalid.
    #[error("descriptor field invalid: {0}")]
    Invalid(&'static str),
    /// Body exceeds 65,536 bytes.
    #[error("descriptor body too large")]
    TooLarge,
}

/// Administrative state on a heap descriptor (§34.5 / §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeapDescriptorState {
    /// Active.
    Active = 1,
    /// Read-only.
    ReadOnly = 2,
    /// Suspended.
    Suspended = 3,
    /// Retired.
    Retired = 4,
    /// Purging.
    Purging = 5,
    /// Purged.
    Purged = 6,
}

impl HeapDescriptorState {
    /// Parse wire value.
    pub fn from_u64(v: u64) -> Result<Self, DescriptorError> {
        match v {
            1 => Ok(Self::Active),
            2 => Ok(Self::ReadOnly),
            3 => Ok(Self::Suspended),
            4 => Ok(Self::Retired),
            5 => Ok(Self::Purging),
            6 => Ok(Self::Purged),
            _ => Err(DescriptorError::Invalid("heap state")),
        }
    }
}

/// Collection/stream descriptor state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ObjectDescriptorState {
    /// Active.
    Active = 1,
    /// Retired.
    Retired = 2,
}

impl ObjectDescriptorState {
    /// Parse wire value.
    pub fn from_u64(v: u64) -> Result<Self, DescriptorError> {
        match v {
            1 => Ok(Self::Active),
            2 => Ok(Self::Retired),
            _ => Err(DescriptorError::Invalid("object state")),
        }
    }
}

/// Heap descriptor body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeapDescriptor {
    /// Origin deployment id.
    pub origin_deployment_id: [u8; 16],
    /// Heap id.
    pub heap_id: [u8; 16],
    /// Creation event id.
    pub creation_event_id: [u8; 16],
    /// Created-at unix seconds.
    pub created_at: u64,
    /// Predecessor descriptor hash (None for sequence 1).
    pub predecessor_hash: Option<[u8; 32]>,
    /// Sequence starting at 1.
    pub sequence: u64,
    /// Administrative state.
    pub state: HeapDescriptorState,
    /// Canonical name.
    pub name: String,
    /// Live aliases.
    pub aliases: Vec<String>,
}

/// Collection or stream descriptor body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDescriptor {
    /// Owner heap.
    pub heap_id: [u8; 16],
    /// Immutable object id.
    pub object_id: [u8; 16],
    /// Creation event id.
    pub creation_event_id: [u8; 16],
    /// Created-at unix seconds.
    pub created_at: u64,
    /// Predecessor hash.
    pub predecessor_hash: Option<[u8; 32]>,
    /// Sequence.
    pub sequence: u64,
    /// Name.
    pub name: String,
    /// Aliases.
    pub aliases: Vec<String>,
    /// State.
    pub state: ObjectDescriptorState,
}

fn validate_name(name: &str) -> Result<(), DescriptorError> {
    let b = name.as_bytes();
    if b.is_empty() || b.len() > 255 {
        return Err(DescriptorError::Invalid("name length"));
    }
    if b.iter().any(|&c| c == 0 || c < 0x20) {
        return Err(DescriptorError::Invalid("name control char"));
    }
    Ok(())
}

fn expect_uint(v: &CborValue) -> Result<u64, DescriptorError> {
    match v {
        CborValue::Uint(u) => Ok(*u),
        _ => Err(DescriptorError::Invalid("uint")),
    }
}

fn expect_text(v: &CborValue) -> Result<String, DescriptorError> {
    match v {
        CborValue::Text(s) => Ok(s.clone()),
        _ => Err(DescriptorError::Invalid("text")),
    }
}

fn expect_b16(v: &CborValue) -> Result<[u8; 16], DescriptorError> {
    match v {
        CborValue::Bytes(b) if b.len() == 16 => {
            let mut a = [0u8; 16];
            a.copy_from_slice(b);
            Ok(a)
        }
        _ => Err(DescriptorError::Invalid("bstr16")),
    }
}

fn expect_b32(v: &CborValue) -> Result<[u8; 32], DescriptorError> {
    match v {
        CborValue::Bytes(b) if b.len() == 32 => {
            let mut a = [0u8; 32];
            a.copy_from_slice(b);
            Ok(a)
        }
        _ => Err(DescriptorError::Invalid("bstr32")),
    }
}

fn expect_aliases(v: &CborValue) -> Result<Vec<String>, DescriptorError> {
    match v {
        CborValue::Array(items) => {
            if items.len() > 64 {
                return Err(DescriptorError::Invalid("too many aliases"));
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                let s = expect_text(item)?;
                validate_name(&s)?;
                out.push(s);
            }
            let mut sorted = out.clone();
            sorted.sort();
            if sorted != out {
                return Err(DescriptorError::Invalid("aliases not sorted"));
            }
            for w in out.windows(2) {
                if w[0] == w[1] {
                    return Err(DescriptorError::Invalid("duplicate alias"));
                }
            }
            Ok(out)
        }
        _ => Err(DescriptorError::Invalid("aliases")),
    }
}

/// Encode a heap descriptor body.
pub fn encode_heap_descriptor(d: &HeapDescriptor) -> Result<Vec<u8>, DescriptorError> {
    validate_name(&d.name)?;
    if d.sequence == 0 {
        return Err(DescriptorError::Invalid("sequence"));
    }
    if d.sequence == 1 && d.predecessor_hash.is_some() {
        return Err(DescriptorError::Invalid("seq1 predecessor"));
    }
    if d.sequence == 1 && !d.aliases.is_empty() {
        return Err(DescriptorError::Invalid("seq1 aliases"));
    }
    for a in &d.aliases {
        validate_name(a)?;
    }
    let pred = match d.predecessor_hash {
        None => CborValue::Null,
        Some(h) => CborValue::Bytes(h.to_vec()),
    };
    let aliases = CborValue::Array(d.aliases.iter().cloned().map(CborValue::Text).collect());
    let entries = [
        (1u64, CborValue::Uint(1)),
        (2, CborValue::Bytes(d.origin_deployment_id.to_vec())),
        (3, CborValue::Bytes(d.heap_id.to_vec())),
        (4, CborValue::Bytes(d.creation_event_id.to_vec())),
        (5, CborValue::Uint(d.created_at)),
        (6, CborValue::Text(HEAP_DESCRIPTOR_PROFILE.into())),
        (7, pred),
        (8, CborValue::Uint(d.sequence)),
        (9, CborValue::Uint(d.state as u64)),
        (10, CborValue::Text(d.name.clone())),
        (11, aliases),
    ];
    let out = encode_deterministic_uint_map(&entries)?;
    if out.len() > 65_536 {
        return Err(DescriptorError::TooLarge);
    }
    Ok(out)
}

/// Decode a heap descriptor body.
pub fn decode_heap_descriptor(bytes: &[u8]) -> Result<HeapDescriptor, DescriptorError> {
    if bytes.len() > 65_536 {
        return Err(DescriptorError::TooLarge);
    }
    let map = decode_deterministic_uint_map(bytes)?;
    if map.len() != 11 {
        return Err(DescriptorError::Invalid("field count"));
    }
    let mut version = None;
    let mut origin = None;
    let mut heap = None;
    let mut creation = None;
    let mut created_at = None;
    let mut profile = None;
    let mut pred = None;
    let mut sequence = None;
    let mut state = None;
    let mut name = None;
    let mut aliases = None;
    for (k, v) in map {
        match k {
            1 => version = Some(expect_uint(&v)?),
            2 => origin = Some(expect_b16(&v)?),
            3 => heap = Some(expect_b16(&v)?),
            4 => creation = Some(expect_b16(&v)?),
            5 => created_at = Some(expect_uint(&v)?),
            6 => profile = Some(expect_text(&v)?),
            7 => {
                pred = Some(match v {
                    CborValue::Null => None,
                    other => Some(expect_b32(&other)?),
                })
            }
            8 => sequence = Some(expect_uint(&v)?),
            9 => state = Some(HeapDescriptorState::from_u64(expect_uint(&v)?)?),
            10 => name = Some(expect_text(&v)?),
            11 => aliases = Some(expect_aliases(&v)?),
            _ => return Err(DescriptorError::Invalid("unknown key")),
        }
    }
    if version != Some(1) {
        return Err(DescriptorError::Invalid("version"));
    }
    if profile.as_deref() != Some(HEAP_DESCRIPTOR_PROFILE) {
        return Err(DescriptorError::Invalid("profile"));
    }
    let d = HeapDescriptor {
        origin_deployment_id: origin.ok_or(DescriptorError::Invalid("origin"))?,
        heap_id: heap.ok_or(DescriptorError::Invalid("heap"))?,
        creation_event_id: creation.ok_or(DescriptorError::Invalid("creation"))?,
        created_at: created_at.ok_or(DescriptorError::Invalid("created_at"))?,
        predecessor_hash: pred.ok_or(DescriptorError::Invalid("pred"))?,
        sequence: sequence.ok_or(DescriptorError::Invalid("sequence"))?,
        state: state.ok_or(DescriptorError::Invalid("state"))?,
        name: name.ok_or(DescriptorError::Invalid("name"))?,
        aliases: aliases.ok_or(DescriptorError::Invalid("aliases"))?,
    };
    validate_name(&d.name)?;
    if d.sequence == 0 {
        return Err(DescriptorError::Invalid("sequence"));
    }
    if d.sequence == 1 && (d.predecessor_hash.is_some() || !d.aliases.is_empty()) {
        return Err(DescriptorError::Invalid("seq1 shape"));
    }
    Ok(d)
}

/// Encode collection/stream descriptor body.
pub fn encode_object_descriptor(d: &ObjectDescriptor) -> Result<Vec<u8>, DescriptorError> {
    validate_name(&d.name)?;
    if d.sequence == 0 {
        return Err(DescriptorError::Invalid("sequence"));
    }
    if d.sequence == 1 && d.predecessor_hash.is_some() {
        return Err(DescriptorError::Invalid("seq1 predecessor"));
    }
    for a in &d.aliases {
        validate_name(a)?;
    }
    let pred = match d.predecessor_hash {
        None => CborValue::Null,
        Some(h) => CborValue::Bytes(h.to_vec()),
    };
    let aliases = CborValue::Array(d.aliases.iter().cloned().map(CborValue::Text).collect());
    let entries = [
        (1u64, CborValue::Uint(1)),
        (2, CborValue::Bytes(d.heap_id.to_vec())),
        (3, CborValue::Bytes(d.object_id.to_vec())),
        (4, CborValue::Bytes(d.creation_event_id.to_vec())),
        (5, CborValue::Uint(d.created_at)),
        (6, CborValue::Text(HEAP_DESCRIPTOR_PROFILE.into())),
        (7, pred),
        (8, CborValue::Uint(d.sequence)),
        (9, CborValue::Text(d.name.clone())),
        (10, aliases),
        (11, CborValue::Uint(d.state as u64)),
        (12, CborValue::Map(vec![])),
    ];
    let out = encode_deterministic_uint_map(&entries)?;
    if out.len() > 65_536 {
        return Err(DescriptorError::TooLarge);
    }
    Ok(out)
}

/// Decode collection/stream descriptor body.
pub fn decode_object_descriptor(bytes: &[u8]) -> Result<ObjectDescriptor, DescriptorError> {
    if bytes.len() > 65_536 {
        return Err(DescriptorError::TooLarge);
    }
    let map = decode_deterministic_uint_map(bytes)?;
    if map.len() != 12 {
        return Err(DescriptorError::Invalid("field count"));
    }
    let mut version = None;
    let mut heap = None;
    let mut object = None;
    let mut creation = None;
    let mut created_at = None;
    let mut profile = None;
    let mut pred = None;
    let mut sequence = None;
    let mut name = None;
    let mut aliases = None;
    let mut state = None;
    let mut options_ok = false;
    for (k, v) in map {
        match k {
            1 => version = Some(expect_uint(&v)?),
            2 => heap = Some(expect_b16(&v)?),
            3 => object = Some(expect_b16(&v)?),
            4 => creation = Some(expect_b16(&v)?),
            5 => created_at = Some(expect_uint(&v)?),
            6 => profile = Some(expect_text(&v)?),
            7 => {
                pred = Some(match v {
                    CborValue::Null => None,
                    other => Some(expect_b32(&other)?),
                })
            }
            8 => sequence = Some(expect_uint(&v)?),
            9 => name = Some(expect_text(&v)?),
            10 => aliases = Some(expect_aliases(&v)?),
            11 => state = Some(ObjectDescriptorState::from_u64(expect_uint(&v)?)?),
            12 => match v {
                CborValue::Map(m) if m.is_empty() => options_ok = true,
                _ => return Err(DescriptorError::Invalid("options")),
            },
            _ => return Err(DescriptorError::Invalid("unknown key")),
        }
    }
    if version != Some(1) || profile.as_deref() != Some(HEAP_DESCRIPTOR_PROFILE) || !options_ok {
        return Err(DescriptorError::Invalid("header"));
    }
    let d = ObjectDescriptor {
        heap_id: heap.ok_or(DescriptorError::Invalid("heap"))?,
        object_id: object.ok_or(DescriptorError::Invalid("object"))?,
        creation_event_id: creation.ok_or(DescriptorError::Invalid("creation"))?,
        created_at: created_at.ok_or(DescriptorError::Invalid("created_at"))?,
        predecessor_hash: pred.ok_or(DescriptorError::Invalid("pred"))?,
        sequence: sequence.ok_or(DescriptorError::Invalid("sequence"))?,
        name: name.ok_or(DescriptorError::Invalid("name"))?,
        aliases: aliases.ok_or(DescriptorError::Invalid("aliases"))?,
        state: state.ok_or(DescriptorError::Invalid("state"))?,
    };
    validate_name(&d.name)?;
    if d.sequence == 0 || (d.sequence == 1 && d.predecessor_hash.is_some()) {
        return Err(DescriptorError::Invalid("sequence shape"));
    }
    Ok(d)
}

/// §34.7 descriptor hash: BLAKE3-256(`RESIDIUUM-HEAP-DESCRIPTOR-V1` || 0x00 || body).
pub fn descriptor_hash(body: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"RESIDIUUM-HEAP-DESCRIPTOR-V1");
    hasher.update(&[0u8]);
    hasher.update(body);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_descriptor_roundtrip_and_hash() {
        let d = HeapDescriptor {
            origin_deployment_id: [1u8; 16],
            heap_id: [2u8; 16],
            creation_event_id: [3u8; 16],
            created_at: 1_700_000_000,
            predecessor_hash: None,
            sequence: 1,
            state: HeapDescriptorState::Active,
            name: "accounts".into(),
            aliases: vec![],
        };
        let body = encode_heap_descriptor(&d).unwrap();
        let back = decode_heap_descriptor(&body).unwrap();
        assert_eq!(back, d);
        let h = descriptor_hash(&body);
        assert_ne!(h, [0u8; 32]);
    }
}
