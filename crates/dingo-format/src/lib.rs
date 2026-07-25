//! DingoDB survival wire format (FORMAT_SPEC draft → major-1 candidate).
//!
//! Stage 2: frame codec, active segment + seal, forward/reverse salvage
//! scanning, event-id conflict analysis, and chunk reassembly helpers.
//! Deterministic CBOR envelope validation (FORMAT_SPEC §5 condition 6).
//! No durable storage IO (Stage 3).
//!
//! **Wire freeze:** [`WIRE_MAJOR`] / [`WIRE_MINOR`] are the draft major-1
//! candidate (DELIVERY_PLAN §7). Production soak may still adjust minor; a
//! breaking on-disk change requires a major bump and dual-read support.

#![deny(missing_docs)]

/// Product freeze label for the draft wire major-1 candidate.
///
/// Same values as [`WIRE_MAJOR`].[`WIRE_MINOR`]; string form for packaging
/// manifests and benchmark disclosure (OVERVIEW §12.2).
pub const WIRE_PROFILE_LABEL: &str = "1.0-draft";

mod cbor_envelope;
mod chunks;
mod events;
mod frame;
mod integrity;
mod kinds;
mod limits;
mod scan;
mod segment;

pub use cbor_envelope::{
    decode_deterministic_uint_map, encode_deterministic_uint_map,
    validate_deterministic_cbor_envelope, CborEnvelopeError, CborValue, EMPTY_ENVELOPE,
};
pub use chunks::{
    decode_chunk_body, encode_chunk_body, reassemble_chunks, ChunkPiece, LogicalExtent,
    ReassemblyState, CHUNK_BODY_HEADER_LEN,
};
pub use events::{group_by_event_id, EventIdOutcome};
pub use frame::{
    decode_frame, encode_frame, verify_frame_at, verify_frame_bytes, DecodedFrame, FrameHeader,
    FrameParts, FrameVerifyError, END_MAGIC, FRAME_PREFIX_LEN, FRAME_SUFFIX_LEN, START_MAGIC,
    WIRE_MAJOR, WIRE_MINOR,
};
// WIRE_PROFILE_LABEL is defined at crate root for packaging / disclosure.
pub use integrity::{body_hash, prefix_crc32c, suffix_crc32c, BODY_HASH_LEN};
pub use kinds::{FrameFlags, FrameKind};
pub use limits::SafetyLimits;
pub use scan::{
    find_end_magic_rightmost, find_start_magic, scan_forward, scan_reverse, ByteRange, HoleReason,
    ScanRegion, ScanReport,
};
pub use segment::{
    decode_descriptor_body, decode_store_descriptor_body, decode_summary_body,
    encode_descriptor_body, encode_store_descriptor_body, encode_store_descriptor_frame,
    encode_summary_body, ActiveSegment, SealedSegment, SegmentError, SegmentId,
    DESCRIPTOR_BODY_LEN, STORE_DESCRIPTOR_BODY_LEN, STORE_DESCRIPTOR_FORMAT_TAG, SUMMARY_BODY_LEN,
};
