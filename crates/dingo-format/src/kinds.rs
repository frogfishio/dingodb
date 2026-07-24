//! Frame kinds and flags (FORMAT_SPEC §4.2–§4.3).

/// Frame kind byte values for wire major 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FrameKind {
    /// Invalid / reserved.
    Invalid = 0,
    /// Store descriptor.
    StoreDescriptor = 1,
    /// Segment descriptor.
    SegmentDescriptor = 2,
    /// Item event.
    ItemEvent = 3,
    /// Payload chunk.
    PayloadChunk = 4,
    /// Batch prepare.
    BatchPrepare = 5,
    /// Batch commit.
    BatchCommit = 6,
    /// Segment summary.
    SegmentSummary = 7,
    /// Purge attestation.
    PurgeAttestation = 8,
    /// Explicit padding.
    Padding = 9,
}

impl FrameKind {
    /// Interpret a raw kind byte. Unknown values remain recoverable as opaque frames.
    pub fn from_u8(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::Invalid),
            1 => Ok(Self::StoreDescriptor),
            2 => Ok(Self::SegmentDescriptor),
            3 => Ok(Self::ItemEvent),
            4 => Ok(Self::PayloadChunk),
            5 => Ok(Self::BatchPrepare),
            6 => Ok(Self::BatchCommit),
            7 => Ok(Self::SegmentSummary),
            8 => Ok(Self::PurgeAttestation),
            9 => Ok(Self::Padding),
            other => Err(other),
        }
    }

    /// Raw wire byte.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Flag bits for wire major 1 (FORMAT_SPEC §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FrameFlags(pub u8);

impl FrameFlags {
    /// Body uses envelope-declared compression.
    pub const COMPRESSED: u8 = 1 << 0;
    /// Body uses envelope-declared authenticated encryption.
    pub const ENCRYPTED: u8 = 1 << 1;
    /// Body is or references a chunked payload.
    pub const CHUNKED: u8 = 1 << 2;
    /// Structured body uses its declared canonical encoding.
    pub const CANONICAL: u8 = 1 << 3;
    /// Frame was produced by an evidence-recorded repair.
    pub const REPAIR: u8 = 1 << 4;
    /// Bits writers must set to zero in this draft.
    pub const RESERVED_MASK: u8 = 0b1110_0000;

    /// Construct from raw flags byte.
    pub fn new(raw: u8) -> Self {
        Self(raw)
    }

    /// Raw flags byte.
    pub fn as_u8(self) -> u8 {
        self.0
    }

    /// Whether any reserved bit is set.
    pub fn has_reserved_bits(self) -> bool {
        self.0 & Self::RESERVED_MASK != 0
    }

    /// Whether the compressed flag is set.
    pub fn compressed(self) -> bool {
        self.0 & Self::COMPRESSED != 0
    }

    /// Whether the encrypted flag is set.
    pub fn encrypted(self) -> bool {
        self.0 & Self::ENCRYPTED != 0
    }

    /// Whether the chunked flag is set.
    pub fn chunked(self) -> bool {
        self.0 & Self::CHUNKED != 0
    }
}
