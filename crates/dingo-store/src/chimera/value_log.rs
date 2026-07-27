//! Append-only large-value log (FINAL DESIGN §1 large class).
//!
//! Independent of FORMAT_SPEC payload-chunks (`chunk_payload`): this log is the
//! Chimera physical placement for large values addressed by
//! [`super::ValueLocator::LargeValueLog`]. Chunk frames remain the wire/format
//! path; the value log is the workload-compiled extent form.
//!
//! Layout of one log file (concatenated records):
//!
//! ```text
//! for each record:
//!   magic[4] = b"DVL1"
//!   generation:u32 LE
//!   flags:u32 LE
//!   value_len:u64 LE
//!   checksum:u32 LE   // CRC32C of value
//!   value[value_len]
//! ```

use crate::error::StoreError;
use std::io;

/// Per-record magic.
pub const VALUE_LOG_MAGIC: &[u8; 4] = b"DVL1";
/// Header size before value bytes.
pub const VALUE_LOG_HEADER_LEN: usize = 4 + 4 + 4 + 8 + 4;

/// One large-value log record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueLogRecord {
    /// Relocation generation.
    pub generation: u32,
    /// Reserved flags (0 today).
    pub flags: u32,
    /// Payload bytes.
    pub value: Vec<u8>,
}

impl ValueLogRecord {
    /// Construct a raw record.
    pub fn new(generation: u32, value: Vec<u8>) -> Self {
        Self {
            generation,
            flags: 0,
            value,
        }
    }

    /// Encoded size on the wire.
    pub fn encoded_len(&self) -> usize {
        VALUE_LOG_HEADER_LEN + self.value.len()
    }

    /// Encode one record.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.encoded_len());
        encode_record(self, &mut out);
        out
    }
}

/// Append-only in-memory log used by tests and the compiler planner.
#[derive(Debug, Clone, Default)]
pub struct ValueLog {
    /// Concatenated encoded records.
    bytes: Vec<u8>,
}

impl ValueLog {
    /// Empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Current end offset (next append position).
    pub fn len_bytes(&self) -> u64 {
        self.bytes.len() as u64
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Append a record; returns `(offset, len)` for a locator.
    pub fn append(&mut self, record: &ValueLogRecord) -> (u64, u64) {
        let offset = self.bytes.len() as u64;
        encode_record(record, &mut self.bytes);
        let len = record.encoded_len() as u64;
        (offset, len)
    }

    /// Raw bytes (for persistence).
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Load from bytes (does not validate until reads).
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Read a record at `offset` with expected encoded `len` (from locator).
    pub fn read_at(&self, offset: u64, len: u64) -> Result<ValueLogRecord, StoreError> {
        read_record_at(&self.bytes, offset, len)
    }
}

fn crc32c(mut data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    while !data.is_empty() {
        crc ^= u32::from(data[0]);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
        data = &data[1..];
    }
    !crc
}

fn encode_record(r: &ValueLogRecord, out: &mut Vec<u8>) {
    let sum = crc32c(&r.value);
    let value_len = r.value.len() as u64;
    out.extend_from_slice(VALUE_LOG_MAGIC);
    out.extend_from_slice(&r.generation.to_le_bytes());
    out.extend_from_slice(&r.flags.to_le_bytes());
    out.extend_from_slice(&value_len.to_le_bytes());
    out.extend_from_slice(&sum.to_le_bytes());
    out.extend_from_slice(&r.value);
}

fn read_record_at(bytes: &[u8], offset: u64, len: u64) -> Result<ValueLogRecord, StoreError> {
    let start = offset as usize;
    let end = offset
        .checked_add(len)
        .ok_or_else(|| bad("offset overflow"))? as usize;
    if end > bytes.len() || len < VALUE_LOG_HEADER_LEN as u64 {
        return Err(bad("value log range out of bounds"));
    }
    let slice = &bytes[start..end];
    if &slice[0..4] != VALUE_LOG_MAGIC.as_slice() {
        return Err(bad("value log magic mismatch"));
    }
    let generation = u32::from_le_bytes(slice[4..8].try_into().unwrap());
    let flags = u32::from_le_bytes(slice[8..12].try_into().unwrap());
    let value_len = u64::from_le_bytes(slice[12..20].try_into().unwrap()) as usize;
    let expect = u32::from_le_bytes(slice[20..24].try_into().unwrap());
    if VALUE_LOG_HEADER_LEN + value_len != slice.len() {
        return Err(bad("value log length mismatch"));
    }
    let value = slice[VALUE_LOG_HEADER_LEN..].to_vec();
    if crc32c(&value) != expect {
        return Err(bad("value log checksum mismatch"));
    }
    Ok(ValueLogRecord {
        generation,
        flags,
        value,
    })
}

/// Decode a record from a full encoded buffer (offset 0, full len).
pub fn decode_record(bytes: &[u8]) -> Result<ValueLogRecord, StoreError> {
    read_record_at(bytes, 0, bytes.len() as u64)
}

fn bad(msg: &'static str) -> StoreError {
    StoreError::Io(io::Error::new(io::ErrorKind::InvalidData, msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_read_roundtrip() {
        let mut log = ValueLog::new();
        let r1 = ValueLogRecord::new(1, b"hello-large".to_vec());
        let r2 = ValueLogRecord::new(2, vec![9u8; 1000]);
        let (o1, l1) = log.append(&r1);
        let (o2, l2) = log.append(&r2);
        assert_eq!(log.read_at(o1, l1).unwrap().value, b"hello-large");
        assert_eq!(log.read_at(o2, l2).unwrap().value, vec![9u8; 1000]);
        assert_eq!(log.read_at(o2, l2).unwrap().generation, 2);
    }

    #[test]
    fn corrupt_rejected() {
        let mut log = ValueLog::new();
        let (o, l) = log.append(&ValueLogRecord::new(1, b"xyz".to_vec()));
        let mut bytes = log.as_bytes().to_vec();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        let log = ValueLog::from_bytes(bytes);
        assert!(log.read_at(o, l).is_err());
    }
}
