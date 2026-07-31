//! DEF-091-F — fuzz chunk manifest decode (untrusted stored body bytes).
//!
//! Hostile chunk_count must not allocate multi-GiB vectors (capacity bound).

#![no_main]

use residiuum_store::{decode_chunk_manifest, is_chunk_manifest};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = is_chunk_manifest(data);
    let _ = decode_chunk_manifest(data);
});
