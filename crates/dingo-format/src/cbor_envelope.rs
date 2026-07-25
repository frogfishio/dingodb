//! Deterministic CBOR envelope rules (FORMAT_SPEC §4.4, §5 condition 6).
//!
//! Wire version 1 envelopes must be a single definite-length map with
//! unsigned-integer keys, shortest integer encodings, no indefinite items,
//! unique keys sorted by encoded key bytes, and valid UTF-8 text.

use thiserror::Error;

/// Empty definite map (`{}`) — the minimal valid envelope.
pub const EMPTY_ENVELOPE: &[u8] = &[0xa0];

/// Failure to satisfy deterministic envelope CBOR rules.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CborEnvelopeError {
    /// Buffer ended before a complete item.
    #[error("truncated CBOR item")]
    Truncated,
    /// Trailing bytes after the top-level map.
    #[error("trailing bytes after envelope map")]
    TrailingBytes,
    /// Top-level value is not a definite map.
    #[error("envelope must be a single definite-length map")]
    NotMap,
    /// Indefinite-length item encountered.
    #[error("indefinite-length CBOR is not allowed")]
    Indefinite,
    /// Non-shortest integer or length encoding.
    #[error("non-shortest CBOR integer encoding")]
    NonShortest,
    /// Duplicate map key.
    #[error("duplicate envelope map key")]
    DuplicateKey,
    /// Map keys are not sorted by encoded byte order.
    #[error("envelope map keys are not sorted")]
    UnsortedKeys,
    /// Map key is not an unsigned integer.
    #[error("envelope map key must be an unsigned integer")]
    NonUintKey,
    /// Text string is not valid UTF-8.
    #[error("envelope text is not valid UTF-8")]
    InvalidUtf8,
    /// Unsupported major type or simple value in this profile.
    #[error("unsupported CBOR major type or simple value in envelope")]
    Unsupported,
    /// Floating-point values are rejected (non-deterministic).
    #[error("CBOR floats are not allowed in envelopes")]
    FloatRejected,
    /// CBOR tags are not used in wire v1 envelopes.
    #[error("CBOR tags are not allowed in envelopes")]
    TagRejected,
}

/// Verify that `bytes` is exactly one deterministic CBOR map suitable as a
/// wire version 1 envelope (FORMAT_SPEC §4.4 / §5 condition 6).
pub fn validate_deterministic_cbor_envelope(bytes: &[u8]) -> Result<(), CborEnvelopeError> {
    let mut cur = Cursor::new(bytes);
    cur.read_envelope_map()?;
    if !cur.is_empty() {
        return Err(CborEnvelopeError::TrailingBytes);
    }
    Ok(())
}

/// Encode a definite map with unsigned integer keys in deterministic form.
///
/// Entries are sorted by key (which matches encoded-byte order for major-type-0
/// keys). Duplicate keys are rejected.
pub fn encode_deterministic_uint_map(
    entries: &[(u64, CborValue)],
) -> Result<Vec<u8>, CborEnvelopeError> {
    let mut sorted = entries.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    for w in sorted.windows(2) {
        if w[0].0 == w[1].0 {
            return Err(CborEnvelopeError::DuplicateKey);
        }
    }
    let mut out = Vec::new();
    write_map_header(&mut out, sorted.len())?;
    for (k, v) in &sorted {
        write_uint(&mut out, *k);
        write_value(&mut out, v)?;
    }
    // Self-check: encoded bytes must validate.
    validate_deterministic_cbor_envelope(&out)?;
    Ok(out)
}

/// Decode a deterministic uint-keyed map into owned values.
pub fn decode_deterministic_uint_map(
    bytes: &[u8],
) -> Result<Vec<(u64, CborValue)>, CborEnvelopeError> {
    validate_deterministic_cbor_envelope(bytes)?;
    let mut cur = Cursor::new(bytes);
    let n = cur.read_map_len()?;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let key = cur.read_uint()?;
        let val = cur.read_value()?;
        out.push((key, val));
    }
    Ok(out)
}

/// Minimal CBOR value set used for envelope construction and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CborValue {
    /// Unsigned integer.
    Uint(u64),
    /// Negative integer with CBOR argument `n` meaning value `-1 - n`.
    Nint(u64),
    /// Byte string.
    Bytes(Vec<u8>),
    /// UTF-8 text string.
    Text(String),
    /// Definite array.
    Array(Vec<CborValue>),
    /// Nested definite map with unsigned integer keys (sorted on encode).
    Map(Vec<(u64, CborValue)>),
    /// Boolean.
    Bool(bool),
    /// Null.
    Null,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], CborEnvelopeError> {
        if self.remaining() < n {
            return Err(CborEnvelopeError::Truncated);
        }
        let slice = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn peek(&self) -> Result<u8, CborEnvelopeError> {
        self.bytes
            .get(self.pos)
            .copied()
            .ok_or(CborEnvelopeError::Truncated)
    }

    fn read_envelope_map(&mut self) -> Result<(), CborEnvelopeError> {
        let head = self.peek()?;
        let major = head >> 5;
        if major != 5 {
            return Err(CborEnvelopeError::NotMap);
        }
        let n = self.read_map_len()?;
        let mut prev_key_enc: Option<Vec<u8>> = None;
        let mut seen: Vec<u64> = Vec::with_capacity(n);
        for _ in 0..n {
            let key_start = self.pos;
            let key = self.read_uint()?;
            let key_enc = self.bytes[key_start..self.pos].to_vec();
            if seen.contains(&key) {
                return Err(CborEnvelopeError::DuplicateKey);
            }
            seen.push(key);
            if let Some(prev) = &prev_key_enc {
                if key_enc.as_slice() <= prev.as_slice() {
                    return Err(CborEnvelopeError::UnsortedKeys);
                }
            }
            prev_key_enc = Some(key_enc);
            self.skip_value()?;
        }
        Ok(())
    }

    fn read_map_len(&mut self) -> Result<usize, CborEnvelopeError> {
        let (major, arg) = self.read_head()?;
        if major != 5 {
            return Err(CborEnvelopeError::NotMap);
        }
        Ok(arg as usize)
    }

    fn read_uint(&mut self) -> Result<u64, CborEnvelopeError> {
        let (major, arg) = self.read_head()?;
        if major != 0 {
            return Err(CborEnvelopeError::NonUintKey);
        }
        Ok(arg)
    }

    fn read_value(&mut self) -> Result<CborValue, CborEnvelopeError> {
        let head = self.peek()?;
        let major = head >> 5;
        match major {
            0 => Ok(CborValue::Uint(self.read_uint_any()?)),
            1 => {
                let (m, arg) = self.read_head()?;
                debug_assert_eq!(m, 1);
                Ok(CborValue::Nint(arg))
            }
            2 => {
                let data = self.read_bstr()?;
                Ok(CborValue::Bytes(data.to_vec()))
            }
            3 => {
                let data = self.read_tstr()?;
                Ok(CborValue::Text(data.to_string()))
            }
            4 => {
                let n = self.read_array_len()?;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.read_value()?);
                }
                Ok(CborValue::Array(items))
            }
            5 => {
                let n = self.read_map_len()?;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    let k = self.read_uint()?;
                    let v = self.read_value()?;
                    items.push((k, v));
                }
                Ok(CborValue::Map(items))
            }
            6 => Err(CborEnvelopeError::TagRejected),
            7 => self.read_simple_value(),
            _ => Err(CborEnvelopeError::Unsupported),
        }
    }

    fn skip_value(&mut self) -> Result<(), CborEnvelopeError> {
        let head = self.peek()?;
        let major = head >> 5;
        match major {
            0 | 1 => {
                let _ = self.read_head()?;
                Ok(())
            }
            2 => {
                let _ = self.read_bstr()?;
                Ok(())
            }
            3 => {
                let _ = self.read_tstr()?;
                Ok(())
            }
            4 => {
                let n = self.read_array_len()?;
                for _ in 0..n {
                    self.skip_value()?;
                }
                Ok(())
            }
            5 => {
                // Nested maps obey the same deterministic rules as the top level.
                let n = self.read_map_len()?;
                let mut prev_key_enc: Option<Vec<u8>> = None;
                let mut seen: Vec<u64> = Vec::with_capacity(n);
                for _ in 0..n {
                    let key_start = self.pos;
                    let key = self.read_uint()?;
                    let key_enc = self.bytes[key_start..self.pos].to_vec();
                    if seen.contains(&key) {
                        return Err(CborEnvelopeError::DuplicateKey);
                    }
                    seen.push(key);
                    if let Some(prev) = &prev_key_enc {
                        if key_enc.as_slice() <= prev.as_slice() {
                            return Err(CborEnvelopeError::UnsortedKeys);
                        }
                    }
                    prev_key_enc = Some(key_enc);
                    self.skip_value()?;
                }
                Ok(())
            }
            6 => Err(CborEnvelopeError::TagRejected),
            7 => {
                let _ = self.read_simple_value()?;
                Ok(())
            }
            _ => Err(CborEnvelopeError::Unsupported),
        }
    }

    fn read_simple_value(&mut self) -> Result<CborValue, CborEnvelopeError> {
        let head = self.take(1)?[0];
        let ai = head & 0x1f;
        match ai {
            20 => Ok(CborValue::Bool(false)),
            21 => Ok(CborValue::Bool(true)),
            22 => Ok(CborValue::Null),
            23 => Err(CborEnvelopeError::Unsupported), // undefined
            24 => {
                // one-byte simple; only 0..255 but 0..31 reserved for direct
                let _ = self.take(1)?;
                Err(CborEnvelopeError::Unsupported)
            }
            25 | 26 | 27 => Err(CborEnvelopeError::FloatRejected),
            31 => Err(CborEnvelopeError::Indefinite),
            _ => Err(CborEnvelopeError::Unsupported),
        }
    }

    fn read_array_len(&mut self) -> Result<usize, CborEnvelopeError> {
        let (major, arg) = self.read_head()?;
        if major != 4 {
            return Err(CborEnvelopeError::Unsupported);
        }
        Ok(arg as usize)
    }

    fn read_bstr(&mut self) -> Result<&'a [u8], CborEnvelopeError> {
        let (major, arg) = self.read_head()?;
        if major != 2 {
            return Err(CborEnvelopeError::Unsupported);
        }
        self.take(arg as usize)
    }

    fn read_tstr(&mut self) -> Result<&'a str, CborEnvelopeError> {
        let (major, arg) = self.read_head()?;
        if major != 3 {
            return Err(CborEnvelopeError::Unsupported);
        }
        let raw = self.take(arg as usize)?;
        std::str::from_utf8(raw).map_err(|_| CborEnvelopeError::InvalidUtf8)
    }

    fn read_uint_any(&mut self) -> Result<u64, CborEnvelopeError> {
        let (major, arg) = self.read_head()?;
        if major != 0 {
            return Err(CborEnvelopeError::Unsupported);
        }
        Ok(arg)
    }

    /// Read a CBOR head and additional argument with shortest-form enforcement.
    /// Returns `(major_type, argument)`. Rejects indefinite (`ai == 31`).
    fn read_head(&mut self) -> Result<(u8, u64), CborEnvelopeError> {
        let head = self.take(1)?[0];
        let major = head >> 5;
        let ai = head & 0x1f;
        let arg = match ai {
            n @ 0..=23 => n as u64,
            24 => {
                let b = self.take(1)?[0] as u64;
                if b < 24 {
                    return Err(CborEnvelopeError::NonShortest);
                }
                b
            }
            25 => {
                let raw = self.take(2)?;
                let v = u16::from_be_bytes([raw[0], raw[1]]) as u64;
                if v < 256 {
                    return Err(CborEnvelopeError::NonShortest);
                }
                v
            }
            26 => {
                let raw = self.take(4)?;
                let v = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as u64;
                if v < 65536 {
                    return Err(CborEnvelopeError::NonShortest);
                }
                v
            }
            27 => {
                let raw = self.take(8)?;
                let v = u64::from_be_bytes([
                    raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
                ]);
                if v < 0x1_0000_0000 {
                    return Err(CborEnvelopeError::NonShortest);
                }
                v
            }
            31 => return Err(CborEnvelopeError::Indefinite),
            _ => return Err(CborEnvelopeError::Unsupported),
        };
        Ok((major, arg))
    }
}

fn write_map_header(out: &mut Vec<u8>, len: usize) -> Result<(), CborEnvelopeError> {
    let n = u64::try_from(len).map_err(|_| CborEnvelopeError::Unsupported)?;
    write_type_arg(out, 5, n);
    Ok(())
}

fn write_value(out: &mut Vec<u8>, v: &CborValue) -> Result<(), CborEnvelopeError> {
    match v {
        CborValue::Uint(n) => write_uint(out, *n),
        CborValue::Nint(n) => write_type_arg(out, 1, *n),
        CborValue::Bytes(b) => {
            write_type_arg(out, 2, b.len() as u64);
            out.extend_from_slice(b);
        }
        CborValue::Text(s) => {
            write_type_arg(out, 3, s.len() as u64);
            out.extend_from_slice(s.as_bytes());
        }
        CborValue::Array(items) => {
            write_type_arg(out, 4, items.len() as u64);
            for item in items {
                write_value(out, item)?;
            }
        }
        CborValue::Map(entries) => {
            let encoded = encode_deterministic_uint_map(entries)?;
            out.extend_from_slice(&encoded);
        }
        CborValue::Bool(false) => out.push(0xf4),
        CborValue::Bool(true) => out.push(0xf5),
        CborValue::Null => out.push(0xf6),
    }
    Ok(())
}

fn write_uint(out: &mut Vec<u8>, n: u64) {
    write_type_arg(out, 0, n);
}

fn write_type_arg(out: &mut Vec<u8>, major: u8, arg: u64) {
    let mt = major << 5;
    if arg <= 23 {
        out.push(mt | (arg as u8));
    } else if arg <= 0xff {
        out.push(mt | 24);
        out.push(arg as u8);
    } else if arg <= 0xffff {
        out.push(mt | 25);
        out.extend_from_slice(&(arg as u16).to_be_bytes());
    } else if arg <= 0xffff_ffff {
        out.push(mt | 26);
        out.extend_from_slice(&(arg as u32).to_be_bytes());
    } else {
        out.push(mt | 27);
        out.extend_from_slice(&arg.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_map_is_valid() {
        validate_deterministic_cbor_envelope(EMPTY_ENVELOPE).unwrap();
        validate_deterministic_cbor_envelope(&[]).unwrap_err();
    }

    #[test]
    fn sorted_uint_keys_roundtrip() {
        let bytes = encode_deterministic_uint_map(&[
            (3, CborValue::Bytes(vec![1, 2, 3])),
            (1, CborValue::Uint(9)),
            (2, CborValue::Text("ok".into())),
        ])
        .unwrap();
        // Keys sorted: 1, 2, 3
        assert_eq!(bytes[0], 0xa3);
        let decoded = decode_deterministic_uint_map(&bytes).unwrap();
        assert_eq!(decoded[0].0, 1);
        assert_eq!(decoded[1].0, 2);
        assert_eq!(decoded[2].0, 3);
    }

    #[test]
    fn rejects_unsorted_keys() {
        // map{2: 0, 1: 0} encoded with keys out of order
        let bad = [0xa2, 0x02, 0x00, 0x01, 0x00];
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::UnsortedKeys)
        );
    }

    #[test]
    fn rejects_duplicate_keys() {
        let bad = [0xa2, 0x01, 0x00, 0x01, 0x01];
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::DuplicateKey)
        );
    }

    #[test]
    fn rejects_non_shortest_uint() {
        // uint 1 encoded as 0x18 0x01 instead of 0x01
        let bad = [0xa1, 0x18, 0x01, 0x00];
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::NonShortest)
        );
    }

    #[test]
    fn rejects_indefinite_map() {
        let bad = [0xbf, 0xff]; // indef map + break
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::Indefinite)
        );
    }

    #[test]
    fn rejects_text_key() {
        // map{"a": 1}
        let bad = [0xa1, 0x61, b'a', 0x01];
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::NonUintKey)
        );
    }

    #[test]
    fn rejects_invalid_utf8_text() {
        // map{1: text(1 byte 0xff)}
        let bad = [0xa1, 0x01, 0x61, 0xff];
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::InvalidUtf8)
        );
    }

    #[test]
    fn rejects_trailing_bytes() {
        let bad = [0xa0, 0x00];
        assert_eq!(
            validate_deterministic_cbor_envelope(&bad),
            Err(CborEnvelopeError::TrailingBytes)
        );
    }
}
