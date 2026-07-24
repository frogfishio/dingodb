//! Draft item-event envelope layout (Stage 3a).
//!
//! Wire major 1 envelopes are specified as deterministic CBOR (FORMAT_SPEC §4.4).
//! Until that validation lands, Stage 3 uses a fixed little-endian layout that
//! is self-describing enough for put/delete recovery and remains opaque to
//! `dingo-format` scanners.
//!
//! Layout:
//! ```text
//! magic[8] = b"DENV0001"
//! store_id[16]
//! segment_id[16]
//! item_id[16]
//! event_kind:u8
//! created_ns:u64 LE
//! subject_len:u16 LE
//! subject[subject_len]
//! ```

/// Magic identifying draft item envelopes.
pub const ENVELOPE_MAGIC: &[u8; 8] = b"DENV0001";

/// Maximum subject length in this draft (fits in u16; also bounds envelopes).
pub const MAX_SUBJECT_LEN: usize = 4096;

/// Core event kinds for Stage 3 (OVERVIEW §5.4 subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventKind {
    /// Associate a payload with a logical subject.
    Put = 1,
    /// Record a logical deletion.
    Delete = 2,
}

impl EventKind {
    /// Parse a raw kind byte.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Put),
            2 => Some(Self::Delete),
            _ => None,
        }
    }

    /// Wire byte.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Stable name for diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Put => "put",
            Self::Delete => "delete",
        }
    }
}

/// Decoded draft item envelope fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEnvelope {
    /// Store identifier.
    pub store_id: [u8; 16],
    /// Segment that holds this frame (at write time).
    pub segment_id: [u8; 16],
    /// Stable item identifier for this subject lineage.
    pub item_id: [u8; 16],
    /// Event kind.
    pub event_kind: EventKind,
    /// Writer-supplied creation timestamp (nanoseconds); 0 if unknown.
    pub created_ns: u64,
    /// Logical subject key bytes (UTF-8 for string APIs).
    pub subject: Vec<u8>,
}

/// Encode a draft item envelope.
pub fn encode_item_envelope(env: &ItemEnvelope) -> Result<Vec<u8>, &'static str> {
    if env.subject.len() > MAX_SUBJECT_LEN {
        return Err("subject too long");
    }
    let subject_len = u16::try_from(env.subject.len()).map_err(|_| "subject too long")?;
    let mut out = Vec::with_capacity(8 + 16 * 3 + 1 + 8 + 2 + env.subject.len());
    out.extend_from_slice(ENVELOPE_MAGIC);
    out.extend_from_slice(&env.store_id);
    out.extend_from_slice(&env.segment_id);
    out.extend_from_slice(&env.item_id);
    out.push(env.event_kind.as_u8());
    out.extend_from_slice(&env.created_ns.to_le_bytes());
    out.extend_from_slice(&subject_len.to_le_bytes());
    out.extend_from_slice(&env.subject);
    Ok(out)
}

/// Decode a draft item envelope. Returns `None` if magic or lengths fail.
pub fn decode_item_envelope(bytes: &[u8]) -> Option<ItemEnvelope> {
    if bytes.len() < 8 + 16 * 3 + 1 + 8 + 2 {
        return None;
    }
    if &bytes[0..8] != ENVELOPE_MAGIC.as_slice() {
        return None;
    }
    let store_id: [u8; 16] = bytes[8..24].try_into().ok()?;
    let segment_id: [u8; 16] = bytes[24..40].try_into().ok()?;
    let item_id: [u8; 16] = bytes[40..56].try_into().ok()?;
    let event_kind = EventKind::from_u8(bytes[56])?;
    let created_ns = u64::from_le_bytes(bytes[57..65].try_into().ok()?);
    let subject_len = u16::from_le_bytes(bytes[65..67].try_into().ok()?) as usize;
    if subject_len > MAX_SUBJECT_LEN {
        return None;
    }
    let subject_start: usize = 67;
    let subject_end = subject_start.checked_add(subject_len)?;
    if bytes.len() != subject_end {
        return None;
    }
    let subject = bytes[subject_start..subject_end].to_vec();
    Some(ItemEnvelope {
        store_id,
        segment_id,
        item_id,
        event_kind,
        created_ns,
        subject,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let env = ItemEnvelope {
            store_id: [1u8; 16],
            segment_id: [2u8; 16],
            item_id: [3u8; 16],
            event_kind: EventKind::Put,
            created_ns: 99,
            subject: b"user-42".to_vec(),
        };
        let bytes = encode_item_envelope(&env).unwrap();
        let decoded = decode_item_envelope(&bytes).unwrap();
        assert_eq!(decoded, env);
    }

    #[test]
    fn delete_kind_roundtrip() {
        let env = ItemEnvelope {
            store_id: [0u8; 16],
            segment_id: [0u8; 16],
            item_id: [9u8; 16],
            event_kind: EventKind::Delete,
            created_ns: 0,
            subject: b"k".to_vec(),
        };
        let decoded = decode_item_envelope(&encode_item_envelope(&env).unwrap()).unwrap();
        assert_eq!(decoded.event_kind, EventKind::Delete);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = encode_item_envelope(&ItemEnvelope {
            store_id: [0u8; 16],
            segment_id: [0u8; 16],
            item_id: [0u8; 16],
            event_kind: EventKind::Put,
            created_ns: 0,
            subject: b"x".to_vec(),
        })
        .unwrap();
        bytes[0] = b'X';
        assert!(decode_item_envelope(&bytes).is_none());
    }
}
