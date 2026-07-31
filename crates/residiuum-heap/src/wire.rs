//! Numeric CBOR/COSE labels and protocol constants (`HEAP_SPEC` §31).

#![allow(dead_code)] // Public wire constants consumed by later packages / store.

/// Profile version for heap-key objects.
pub const PROFILE_VERSION: u64 = 1;

/// Qualified data-plane audience.
pub const AUDIENCE_DATA_V1: &str = "dingo:data:v1";

/// COSE content type for HeapKey certificates.
pub const CONTENT_TYPE_CERTIFICATE: &str = "application/dingo.heap-key+cbor";

/// COSE content type for holder proofs.
pub const CONTENT_TYPE_HOLDER_PROOF: &str = "application/dingo.heap-proof+cbor";

/// External AAD for certificate Sig_structure.
pub const EXTERNAL_AAD_CERTIFICATE: &[u8] = b"RESIDIUUM-HEAPKEY-CERTIFICATE-V1";

/// External AAD for holder-proof Sig_structure.
pub const EXTERNAL_AAD_HOLDER_PROOF: &[u8] = b"RESIDIUUM-HEAPKEY-HOLDER-PROOF-V1";

/// COSE `alg` = EdDSA.
pub const COSE_ALG_EDDSA: i64 = -8;

/// Envelope key: heap_id.
pub const ENV_HEAP_ID: u64 = 31;
/// Envelope key: collection_id.
pub const ENV_COLLECTION_ID: u64 = 32;
/// Envelope key: stream_id.
pub const ENV_STREAM_ID: u64 = 33;
/// Envelope key: ownership_profile.
pub const ENV_OWNERSHIP_PROFILE: u64 = 34;
/// Envelope key: source_heap_id.
pub const ENV_SOURCE_HEAP_ID: u64 = 35;
/// Envelope key: source_object_id.
pub const ENV_SOURCE_OBJECT_ID: u64 = 36;

/// Ownership profile value for `dingo-heap-v1`.
pub const OWNERSHIP_PROFILE_V1: u64 = 1;

/// Maximum certificate lifetime (seconds).
pub const CERT_MAX_LIFETIME_S: u64 = 7_776_000;

/// Maximum certificate COSE bytes.
pub const CERT_MAX_BYTES: usize = 16_384;
/// Maximum proof COSE bytes.
pub const PROOF_MAX_BYTES: usize = 4_096;
