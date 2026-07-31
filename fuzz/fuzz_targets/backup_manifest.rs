//! DEF-091-F — fuzz backup manifest JSON (untrusted package control document).

#![no_main]

use residuum_store::BackupManifest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Bound JSON size for the harness.
    let slice = if data.len() > 256 * 1024 {
        &data[..256 * 1024]
    } else {
        data
    };
    let _: Result<BackupManifest, _> = serde_json::from_slice(slice);
});
