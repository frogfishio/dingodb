//! DEF-091-F — fuzz item envelope CBOR decode (untrusted stored envelope).

#![no_main]

use residiuum_store::decode_item_envelope;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = decode_item_envelope(data);
});
