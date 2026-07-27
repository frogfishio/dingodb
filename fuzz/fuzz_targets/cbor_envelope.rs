//! DEF-091 — fuzz deterministic CBOR envelope validation (FORMAT_SPEC §4.4).

#![no_main]

use dingo_format::validate_deterministic_cbor_envelope;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = validate_deterministic_cbor_envelope(data);
});
