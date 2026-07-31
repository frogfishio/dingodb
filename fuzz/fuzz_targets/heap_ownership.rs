#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = residuum_format::decode_subject_v2(data);
    let _ = residuum_format::parse_ownership_envelope(data);
    let _ = residuum_format::decode_heap_descriptor(data);
    let _ = residuum_format::decode_object_descriptor(data);
    if data.len() >= 48 {
        let mut heap = [0u8; 16];
        heap.copy_from_slice(&data[..16]);
        let mid = 16 + (data.len() - 16) / 2;
        let _ = residuum_format::admit_frame_to_heap(&heap, &data[16..mid], &data[mid..], None);
    }
});
