//! DEF-091-F — fuzz store continuation-token decode (untrusted client token).
//!
//! Uses a process-local secret keyring so MAC failures are expected; must not panic.

#![no_main]

use dingo_store::{verify_continuation_token, ContinuationKeyring};
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

fuzz_target!(|data: &[u8]| {
    let store_id = [7u8; 16];
    static RING: OnceLock<ContinuationKeyring> = OnceLock::new();
    let ring = RING.get_or_init(|| ContinuationKeyring::mint_new().expect("mint keyring"));
    let _ = verify_continuation_token(&store_id, ring, data);
});
