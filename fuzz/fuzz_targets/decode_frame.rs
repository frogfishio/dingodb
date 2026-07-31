//! DEF-091 — fuzz frame decode / verify (FORMAT_SPEC §4–§5).
//!
//! Harness must never panic: every input is either a verified frame or a typed error.

#![no_main]

use residuum_format::{decode_frame, verify_frame_at, SafetyLimits};
use libfuzzer_sys::fuzz_target;

fn limits() -> SafetyLimits {
    // Bound allocations so a single input cannot OOM the fuzzer.
    SafetyLimits {
        max_envelope_len: 8 * 1024,
        max_body_len: 256 * 1024,
        max_frame_len: 280 * 1024,
    }
}

fuzz_target!(|data: &[u8]| {
    let lim = limits();
    let _ = decode_frame(data, lim);
    let _ = verify_frame_at(data, lim);
});
