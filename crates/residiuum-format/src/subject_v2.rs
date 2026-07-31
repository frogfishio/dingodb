//! Subject version 2 (`HEAP_SPEC` §34.4).

use thiserror::Error;

/// Object kind byte inside a v2 subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SubjectObjectKind {
    /// Heap metadata (descriptors / migration evidence).
    HeapMetadata = 0x00,
    /// Collection-scoped object.
    Collection = 0x01,
    /// Stream-scoped object.
    Stream = 0x02,
}

impl SubjectObjectKind {
    /// Parse object kind.
    pub fn from_u8(v: u8) -> Result<Self, SubjectV2Error> {
        match v {
            0x00 => Ok(Self::HeapMetadata),
            0x01 => Ok(Self::Collection),
            0x02 => Ok(Self::Stream),
            _ => Err(SubjectV2Error::InvalidObjectKind),
        }
    }
}

/// Borrowed decode of a subject v2 buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubjectV2<'a> {
    /// Heap owner.
    pub heap_id: &'a [u8; 16],
    /// Object kind.
    pub object_kind: SubjectObjectKind,
    /// Collection/stream id, or all-zero for heap metadata.
    pub object_id: &'a [u8; 16],
    /// Application / metadata key bytes.
    pub key: &'a [u8],
}

/// Subject v2 codec errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubjectV2Error {
    /// Buffer too short or length mismatch.
    #[error("subject v2 length invalid")]
    InvalidLength,
    /// Version byte is not 0x02.
    #[error("subject version is not v2")]
    WrongVersion,
    /// Unknown object kind.
    #[error("invalid subject object kind")]
    InvalidObjectKind,
    /// Key longer than 2048 bytes.
    #[error("subject key too long")]
    KeyTooLong,
    /// Heap-metadata object id must be zero.
    #[error("heap metadata object id must be zero")]
    NonZeroMetadataObjectId,
}

const HEADER_LEN: usize = 36;

/// Encode a subject v2 buffer.
pub fn encode_subject_v2(
    heap_id: &[u8; 16],
    object_kind: SubjectObjectKind,
    object_id: &[u8; 16],
    key: &[u8],
) -> Result<Vec<u8>, SubjectV2Error> {
    if key.len() > 2048 {
        return Err(SubjectV2Error::KeyTooLong);
    }
    if object_kind == SubjectObjectKind::HeapMetadata && object_id != &[0u8; 16] {
        return Err(SubjectV2Error::NonZeroMetadataObjectId);
    }
    let mut out = Vec::with_capacity(HEADER_LEN + key.len());
    out.push(0x02);
    out.extend_from_slice(heap_id);
    out.push(object_kind as u8);
    out.extend_from_slice(object_id);
    out.extend_from_slice(&(key.len() as u16).to_be_bytes());
    out.extend_from_slice(key);
    Ok(out)
}

/// Decode a borrowed subject v2.
pub fn decode_subject_v2(bytes: &[u8]) -> Result<SubjectV2<'_>, SubjectV2Error> {
    if bytes.len() < HEADER_LEN {
        return Err(SubjectV2Error::InvalidLength);
    }
    if bytes[0] != 0x02 {
        return Err(SubjectV2Error::WrongVersion);
    }
    let heap_id: &[u8; 16] = bytes[1..17].try_into().unwrap();
    let object_kind = SubjectObjectKind::from_u8(bytes[17])?;
    let object_id: &[u8; 16] = bytes[18..34].try_into().unwrap();
    let key_len = u16::from_be_bytes([bytes[34], bytes[35]]) as usize;
    if bytes.len() != HEADER_LEN + key_len {
        return Err(SubjectV2Error::InvalidLength);
    }
    if key_len > 2048 {
        return Err(SubjectV2Error::KeyTooLong);
    }
    if object_kind == SubjectObjectKind::HeapMetadata && object_id != &[0u8; 16] {
        return Err(SubjectV2Error::NonZeroMetadataObjectId);
    }
    Ok(SubjectV2 {
        heap_id,
        object_kind,
        object_id,
        key: &bytes[HEADER_LEN..],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let heap = [0x11u8; 16];
        let coll = [0x22u8; 16];
        let key = b"user-1";
        let enc = encode_subject_v2(&heap, SubjectObjectKind::Collection, &coll, key).unwrap();
        let dec = decode_subject_v2(&enc).unwrap();
        assert_eq!(dec.heap_id, &heap);
        assert_eq!(dec.object_id, &coll);
        assert_eq!(dec.key, key);
    }

    #[test]
    fn rejects_trailing_bytes() {
        let heap = [0x11u8; 16];
        let enc =
            encode_subject_v2(&heap, SubjectObjectKind::HeapMetadata, &[0u8; 16], &[0x01]).unwrap();
        let mut bad = enc;
        bad.push(0xff);
        assert!(decode_subject_v2(&bad).is_err());
    }
}
