//! Stage 2a integration tests: frame codec + integrity (FORMAT_SPEC §4, §5, §13 starters).

use dingo_format::{
    body_hash, decode_frame, encode_frame, FrameHeader, FrameKind, FrameParts, FrameVerifyError,
    SafetyLimits, EMPTY_ENVELOPE, END_MAGIC, FRAME_PREFIX_LEN, FRAME_SUFFIX_LEN, START_MAGIC,
    WIRE_MAJOR, WIRE_MINOR,
};

fn event_id(n: u8) -> [u8; 16] {
    let mut id = [0u8; 16];
    id[0] = n;
    id
}

fn parts(kind: FrameKind, envelope: &[u8], body: &[u8]) -> FrameParts {
    FrameParts {
        header: FrameHeader {
            wire_major: WIRE_MAJOR,
            wire_minor: WIRE_MINOR,
            frame_kind: kind.as_u8(),
            flags: Default::default(),
            envelope_len: envelope.len() as u32,
            body_len: body.len() as u64,
            logical_len: body.len() as u64,
            writer_sequence: 7,
            event_id: event_id(1),
        },
        envelope: envelope.to_vec(),
        body: body.to_vec(),
    }
}

#[test]
fn prefix_and_suffix_field_boundaries() {
    let envelope = EMPTY_ENVELOPE.to_vec();
    let body = vec![0u8; 0];
    let encoded = encode_frame(&parts(FrameKind::Padding, &envelope, &body)).unwrap();
    assert_eq!(
        encoded.len(),
        FRAME_PREFIX_LEN + EMPTY_ENVELOPE.len() + FRAME_SUFFIX_LEN
    );
    assert_eq!(&encoded[0..8], START_MAGIC);
    assert_eq!(encoded[8], WIRE_MAJOR);
    assert_eq!(encoded[9], WIRE_MINOR);
    assert_eq!(encoded[10], FrameKind::Padding.as_u8());
    assert_eq!(&encoded[encoded.len() - FRAME_SUFFIX_LEN..][..8], END_MAGIC);

    let frame_len = u64::from_le_bytes(
        encoded[encoded.len() - FRAME_SUFFIX_LEN + 8..encoded.len() - FRAME_SUFFIX_LEN + 16]
            .try_into()
            .unwrap(),
    );
    assert_eq!(frame_len, encoded.len() as u64);
}

#[test]
fn max_small_envelope_and_body_roundtrip() {
    // Deterministic map with many uint keys → non-trivial envelope size.
    use dingo_format::{encode_deterministic_uint_map, CborValue};
    let entries: Vec<_> = (1u64..=20)
        .map(|k| (k, CborValue::Uint(k.wrapping_mul(3))))
        .collect();
    let envelope = encode_deterministic_uint_map(&entries).unwrap();
    let body = (0u8..128).map(|i| i.wrapping_mul(3)).collect::<Vec<_>>();
    let encoded = encode_frame(&parts(FrameKind::ItemEvent, &envelope, &body)).unwrap();
    let decoded = decode_frame(&encoded, SafetyLimits::default()).unwrap();
    assert_eq!(decoded.envelope, envelope);
    assert_eq!(decoded.body, body);
    assert_eq!(decoded.header.writer_sequence, 7);
    assert_eq!(decoded.header.event_id, event_id(1));
}

#[test]
fn every_known_kind_roundtrips() {
    for kind in [
        FrameKind::StoreDescriptor,
        FrameKind::SegmentDescriptor,
        FrameKind::ItemEvent,
        FrameKind::PayloadChunk,
        FrameKind::BatchPrepare,
        FrameKind::BatchCommit,
        FrameKind::SegmentSummary,
        FrameKind::PurgeAttestation,
        FrameKind::Padding,
    ] {
        let encoded = encode_frame(&parts(kind, b"\xa0", b"k")).unwrap();
        let decoded = decode_frame(&encoded, SafetyLimits::default()).unwrap();
        assert_eq!(decoded.header.known_kind(), Some(kind));
    }
}

#[test]
fn unknown_extension_kind_is_opaque_but_verified() {
    let mut p = parts(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"ext");
    p.header.frame_kind = 200; // application/profile extension range
    let encoded = encode_frame(&p).unwrap();
    let decoded = decode_frame(&encoded, SafetyLimits::default()).unwrap();
    assert_eq!(decoded.header.frame_kind, 200);
    assert_eq!(decoded.header.known_kind(), None);
    assert_eq!(decoded.body, b"ext");
}

#[test]
fn one_byte_body_corruption_rejects() {
    let mut encoded =
        encode_frame(&parts(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"BODYDATA")).unwrap();
    let body_at = FRAME_PREFIX_LEN + EMPTY_ENVELOPE.len();
    encoded[body_at] ^= 0x5a;
    assert_eq!(
        decode_frame(&encoded, SafetyLimits::default()).unwrap_err(),
        FrameVerifyError::BadBodyHash
    );
}

#[test]
fn one_byte_envelope_corruption_rejects_prefix_crc() {
    // map{1:0} so envelope is multi-byte and flipping the first byte breaks CRC first.
    let env = [0xa1u8, 0x01, 0x00];
    let mut encoded = encode_frame(&parts(FrameKind::ItemEvent, &env, b"body")).unwrap();
    encoded[FRAME_PREFIX_LEN] ^= 0xff;
    assert_eq!(
        decode_frame(&encoded, SafetyLimits::default()).unwrap_err(),
        FrameVerifyError::BadPrefixCrc
    );
}

#[test]
fn one_byte_truncation_positions_reject() {
    let encoded = encode_frame(&parts(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"yyy")).unwrap();
    // Truncate at every position from just under full length down through prefix.
    for keep in (FRAME_PREFIX_LEN..encoded.len()).rev().take(8) {
        let err = decode_frame(&encoded[..keep], SafetyLimits::default()).unwrap_err();
        assert!(
            matches!(
                err,
                FrameVerifyError::Truncated { .. }
                    | FrameVerifyError::BadEndMagic
                    | FrameVerifyError::BadSuffixCrc
                    | FrameVerifyError::BadBodyHash
                    | FrameVerifyError::FrameLenMismatch { .. }
            ),
            "keep={keep} err={err:?}"
        );
    }
    // Short of a prefix always truncates.
    assert!(matches!(
        decode_frame(&encoded[..FRAME_PREFIX_LEN - 1], SafetyLimits::default()).unwrap_err(),
        FrameVerifyError::Truncated { .. }
    ));
}

#[test]
fn false_magic_in_body_does_not_validate_as_standalone_frame() {
    // A body that contains start magic must not decode as a frame by itself.
    let mut body = Vec::new();
    body.extend_from_slice(START_MAGIC);
    body.extend_from_slice(&[0u8; 100]);
    assert!(matches!(
        decode_frame(&body, SafetyLimits::default()).unwrap_err(),
        FrameVerifyError::BadStartMagic
            | FrameVerifyError::LengthsOutOfLimits
            | FrameVerifyError::Truncated { .. }
            | FrameVerifyError::UnsupportedWireMajor(_)
            | FrameVerifyError::BadPrefixCrc
    ));
}

#[test]
fn body_hash_matches_blake3() {
    let body = b"canonical-body-bytes";
    let encoded = encode_frame(&parts(FrameKind::PayloadChunk, EMPTY_ENVELOPE, body)).unwrap();
    let decoded = decode_frame(&encoded, SafetyLimits::default()).unwrap();
    assert_eq!(decoded.body_hash, body_hash(body));
}

#[test]
fn corrupt_length_fields_fail_closed() {
    let mut encoded =
        encode_frame(&parts(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"cd")).unwrap();
    // Blow up envelope_len in prefix without fixing CRC — reject via CRC or limits.
    encoded[12] = 0xff;
    encoded[13] = 0xff;
    let err = decode_frame(&encoded, SafetyLimits::default()).unwrap_err();
    assert!(
        matches!(
            err,
            FrameVerifyError::BadPrefixCrc
                | FrameVerifyError::LengthsOutOfLimits
                | FrameVerifyError::Truncated { .. }
        ),
        "err={err:?}"
    );
}
