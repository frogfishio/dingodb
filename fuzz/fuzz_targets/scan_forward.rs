//! DEF-091 — fuzz forward salvage scan (FORMAT_SPEC §7).

#![no_main]

use dingo_format::{scan_forward, SafetyLimits};
use libfuzzer_sys::fuzz_target;

fn limits() -> SafetyLimits {
    SafetyLimits {
        max_envelope_len: 8 * 1024,
        max_body_len: 256 * 1024,
        max_frame_len: 280 * 1024,
    }
}

fuzz_target!(|data: &[u8]| {
    let report = scan_forward(data, limits());
    for region in &report.regions {
        if let dingo_format::ScanRegion::VerifiedFrame { range, .. } = region {
            assert!(range.end as usize <= data.len());
            assert!(range.start <= range.end);
        }
    }
});
