//! DEF-091-F — fuzz length-prefixed RPC frame reader (untrusted network bytes).
//!
//! Must never panic or allocate unbounded: oversized lengths refuse before alloc.

#![no_main]

use dingo_client::read_frame;
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Cap max_frame small so the harness stresses the refuse-before-alloc path.
    let max_frame = 4 * 1024;
    let mut cur = Cursor::new(data);
    let _ = read_frame(&mut cur, max_frame);
    // Second pass with legacy detection when any bytes remain.
    let mut cur2 = Cursor::new(data);
    let _ = dingo_client::read_frame_or_detect_legacy(&mut cur2, max_frame);
});
