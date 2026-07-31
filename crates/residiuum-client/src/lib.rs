//! Thin Residiuum network client primitives (MIT).
//!
//! Framed RPC protocol (`dingo-rpc-v1`) and handshake only. Collection APIs:
//! [`residiuum_sdk`](https://docs.rs/residiuum-sdk). TCP serve: `residiuum-server`.

#![deny(missing_docs)]

mod error;
mod heap_handshake;
mod protocol;

pub use error::{Error, ErrorCode};
pub use heap_handshake::{
    b64u_decode, b64u_encode, HeapAuth, HeapChallenge, HeapReject, HeapWelcome, FEATURE_HEAP_KEY_V1,
    HEAP_AUDIENCE_DATA_V1, HEAP_AUTH_MAX_BYTES,
};
pub use protocol::{
    client_handshake, encode_frame, negotiate_features, negotiate_features_with_optional,
    negotiate_max_frame, negotiate_qualified_features, parse_handshake, read_frame,
    read_frame_or_detect_legacy, server_handshake, write_frame, write_json_frame, write_reject_frame,
    Handshake, HandshakeMsg, NegotiatedSession, DEFAULT_MAX_FRAME_BYTES, DEFAULT_MAX_RPC_LINE_BYTES,
    DEFAULT_SERVER_PROFILE, FEATURE_IDEMPOTENCY_V1, FEATURE_JSON_RPC_V1, FEATURE_RECEIPTS_V1,
    HANDSHAKE_MAX_FRAME_BYTES, PROTOCOL_MAJOR, PROTOCOL_MINOR, PROTOCOL_PROFILE,
    REQUIRED_DELETE_RECEIPT_FIELDS, REQUIRED_FEATURES, REQUIRED_WRITE_RECEIPT_FIELDS, RPC_WIRE_LABEL,
};
