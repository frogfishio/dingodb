//! DEF-091-F — fuzz SDA lexer/parser (untrusted query program text).
//!
//! Must never panic: every UTF-8 / non-UTF-8 byte string is either a program or
//! a typed parse/lex error.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Bound work: drop huge inputs after a soft cap (parser already fails closed).
    let slice = if data.len() > 64 * 1024 {
        &data[..64 * 1024]
    } else {
        data
    };
    // Lossy UTF-8 covers hostile non-text bytes.
    let src = String::from_utf8_lossy(slice);
    let _ = residuum_sda::Program::parse(&src);
});
