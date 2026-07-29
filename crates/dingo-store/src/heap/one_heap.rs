//! One-heap frame admission for store façades (`HEAP_SPEC` HP-002).

use crate::error::StoreError;
use dingo_format::{
    admit_frame_to_heap, encode_heap_binding_envelope, AdmitDecision, OwnershipEvidence,
};

/// Convert an [`AdmitDecision`] into a store result.
pub fn require_admit(
    bound_heap: &[u8; 16],
    segment_descriptor_envelope: &[u8],
    frame_envelope: &[u8],
    subject: Option<&[u8]>,
) -> Result<OwnershipEvidence, StoreError> {
    match admit_frame_to_heap(
        bound_heap,
        segment_descriptor_envelope,
        frame_envelope,
        subject,
    ) {
        AdmitDecision::Admit { ownership } => Ok(ownership),
        AdmitDecision::RejectUnknown => Err(StoreError::HeapAdmit("unknown ownership".into())),
        AdmitDecision::RejectConflict => Err(StoreError::HeapAdmit("ownership conflict".into())),
        AdmitDecision::RejectWrongHeap { claimed } => Err(StoreError::HeapAdmit(format!(
            "wrong heap {}",
            hex16(&claimed)
        ))),
        AdmitDecision::RejectSubjectMismatch => {
            Err(StoreError::HeapAdmit("subject mismatch".into()))
        }
        AdmitDecision::RejectMalformed => Err(StoreError::HeapAdmit("malformed ownership".into())),
    }
}

/// Build the canonical heap-binding envelope for a segment descriptor.
pub fn heap_binding_envelope(heap_id: &[u8; 16]) -> Result<Vec<u8>, StoreError> {
    encode_heap_binding_envelope(heap_id).map_err(|e| StoreError::HeapAdmit(e.to_string()))
}

fn hex16(id: &[u8; 16]) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_cross_heap_and_accepts_match() {
        let a = [0x11u8; 16];
        let b = [0x22u8; 16];
        let ea = heap_binding_envelope(&a).unwrap();
        let eb = heap_binding_envelope(&b).unwrap();
        assert!(require_admit(&a, &ea, &ea, None).is_ok());
        assert!(require_admit(&a, &ea, &eb, None).is_err());
    }
}
