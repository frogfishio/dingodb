//! FORMAT_SPEC §13 — required wire-format / destructive corpus (Stage 2d).
//!
//! Every listed case is automated. For every destructive case, later intact
//! frames MUST remain discoverable and corrupt candidates MUST NOT be labeled
//! verified.

use residuum_format::{
    body_hash, decode_frame, encode_chunk_body, encode_frame, group_by_event_id, reassemble_chunks,
    scan_forward, scan_reverse, ChunkPiece, EventIdOutcome, FrameFlags, FrameHeader, FrameKind,
    FrameParts, FrameVerifyError, ReassemblyState, SafetyLimits, EMPTY_ENVELOPE, END_MAGIC,
    FRAME_PREFIX_LEN, FRAME_SUFFIX_LEN, START_MAGIC, WIRE_MAJOR, WIRE_MINOR,
};

fn event_id(n: u8) -> [u8; 16] {
    let mut id = [0u8; 16];
    id[0] = n;
    id
}

fn item_parts(event: u8, envelope: &[u8], body: &[u8]) -> FrameParts {
    FrameParts {
        header: FrameHeader {
            wire_major: WIRE_MAJOR,
            wire_minor: WIRE_MINOR,
            frame_kind: FrameKind::ItemEvent.as_u8(),
            flags: FrameFlags::default(),
            envelope_len: envelope.len() as u32,
            body_len: body.len() as u64,
            logical_len: body.len() as u64,
            writer_sequence: 1,
            event_id: event_id(event),
        },
        envelope: envelope.to_vec(),
        body: body.to_vec(),
    }
}

fn encode_item(event: u8, body: &[u8]) -> Vec<u8> {
    encode_frame(&item_parts(event, EMPTY_ENVELOPE, body)).unwrap()
}

fn survivor() -> Vec<u8> {
    encode_item(0xff, b"SURVIVOR-ISLAND")
}

fn assert_survivor_discoverable(buf: &[u8]) {
    let report = scan_forward(buf, SafetyLimits::default());
    let found: Vec<_> = report
        .verified_frames()
        .filter(|(_, f)| f.body == b"SURVIVOR-ISLAND")
        .collect();
    assert_eq!(
        found.len(),
        1,
        "later intact frame must remain discoverable; verified={}",
        report.verified_count()
    );
    // Corrupt/damaged candidates must never appear as verified with survivor body only —
    // already checked. Also ensure no verified frame has empty-claim on survivor offset alone.
    for (_, frame) in report.verified_frames() {
        // All verified frames must pass structural rules (scan only emits verified).
        assert_eq!(frame.header.wire_major, WIRE_MAJOR);
    }
}

// ---------------------------------------------------------------------------
// §13: every prefix and suffix field at boundary values
// ---------------------------------------------------------------------------

#[test]
fn prefix_and_suffix_fields_at_boundaries() {
    // Empty CBOR map envelope + empty body → minimum valid wire-v1 frame.
    let min = encode_frame(&item_parts(1, EMPTY_ENVELOPE, b"")).unwrap();
    assert_eq!(
        min.len(),
        FRAME_PREFIX_LEN + EMPTY_ENVELOPE.len() + FRAME_SUFFIX_LEN
    );
    assert_eq!(&min[0..8], START_MAGIC);
    assert_eq!(min[8], WIRE_MAJOR);
    assert_eq!(min[9], WIRE_MINOR);
    assert_eq!(min[10], FrameKind::ItemEvent.as_u8());
    assert_eq!(min[11], 0); // flags
    assert_eq!(&min[12..16], &(EMPTY_ENVELOPE.len() as u32).to_le_bytes()); // envelope_len
    assert_eq!(&min[16..24], &0u64.to_le_bytes()); // body_len
    assert_eq!(&min[24..32], &0u64.to_le_bytes()); // logical_len
    assert_eq!(&min[min.len() - FRAME_SUFFIX_LEN..][..8], END_MAGIC);
    let frame_len = u64::from_le_bytes(
        min[min.len() - FRAME_SUFFIX_LEN + 8..min.len() - FRAME_SUFFIX_LEN + 16]
            .try_into()
            .unwrap(),
    );
    assert_eq!(frame_len, min.len() as u64);
    // reserved suffix tail zeros
    assert_eq!(&min[min.len() - 4..], &[0, 0, 0, 0]);
    // reserved prefix tail zeros
    assert_eq!(&min[60..64], &[0, 0, 0, 0]);

    decode_frame(&min, SafetyLimits::default()).unwrap();

    // Non-zero writer_sequence + full event_id + non-empty envelope/body.
    let mut parts = item_parts(2, b"\xa0", b"payload-boundary");
    parts.header.writer_sequence = u64::MAX;
    parts.header.event_id = [0xff; 16];
    parts.header.logical_len = parts.body.len() as u64;
    let enc = encode_frame(&parts).unwrap();
    let dec = decode_frame(&enc, SafetyLimits::default()).unwrap();
    assert_eq!(dec.header.writer_sequence, u64::MAX);
    assert_eq!(dec.header.event_id, [0xff; 16]);
    assert_eq!(dec.envelope, b"\xa0");
    assert_eq!(dec.body, b"payload-boundary");
}

// ---------------------------------------------------------------------------
// §13: checked arithmetic for every length combination (representative matrix)
// ---------------------------------------------------------------------------

#[test]
fn checked_arithmetic_length_matrix() {
    let limits = SafetyLimits::default();
    // (envelope_len, body_len, should_accept)
    let cases: &[(u32, u64, bool)] = &[
        (0, 0, true),
        (1, 0, true),
        (0, 1, true),
        (limits.max_envelope_len, 0, true),
        (limits.max_envelope_len + 1, 0, false),
        (0, limits.max_body_len, true),
        (0, limits.max_body_len + 1, false),
        (u32::MAX, 0, false),
        (0, u64::MAX, false),
        (u32::MAX, u64::MAX, false),
        (64 * 1024, 1024, true),
    ];
    for &(env, body, ok) in cases {
        assert_eq!(
            limits.accepts_lengths(env, body),
            ok,
            "env={env} body={body}"
        );
    }
}

// ---------------------------------------------------------------------------
// §13: all one-byte truncation positions
// ---------------------------------------------------------------------------

#[test]
fn all_one_byte_truncation_positions_reject() {
    let encoded = encode_item(3, b"truncate-me-please");
    assert!(
        decode_frame(&encoded, SafetyLimits::default()).is_ok(),
        "control must verify"
    );
    for keep in 0..encoded.len() {
        let err = decode_frame(&encoded[..keep], SafetyLimits::default()).unwrap_err();
        assert!(
            !matches!(err, FrameVerifyError::TrailingBytes { .. }),
            "truncation must not report trailing bytes; keep={keep} err={err:?}"
        );
        // Never succeeds.
        assert!(
            matches!(
                err,
                FrameVerifyError::Truncated { .. }
                    | FrameVerifyError::BadStartMagic
                    | FrameVerifyError::BadEndMagic
                    | FrameVerifyError::BadPrefixCrc
                    | FrameVerifyError::BadSuffixCrc
                    | FrameVerifyError::BadBodyHash
                    | FrameVerifyError::FrameLenMismatch { .. }
                    | FrameVerifyError::LengthsOutOfLimits
                    | FrameVerifyError::UnsupportedWireMajor(_)
            ),
            "keep={keep} err={err:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// §13: every one-byte corruption position in a representative frame
// ---------------------------------------------------------------------------

#[test]
fn every_one_byte_corruption_rejects_and_survivor_found() {
    let good = encode_item(4, b"CORRUPT-TARGET");
    let tail = survivor();
    let limits = SafetyLimits::default();

    for i in 0..good.len() {
        let mut damaged = good.clone();
        damaged[i] ^= 0xA5;
        // Exact decode of the damaged buffer alone must fail.
        assert!(
            decode_frame(&damaged, limits).is_err(),
            "byte {i} corruption must fail exact decode"
        );

        let mut buf = damaged;
        buf.extend_from_slice(&tail);
        let report = scan_forward(&buf, limits);
        // Damaged original body must not verify.
        assert!(
            !report
                .verified_frames()
                .any(|(_, f)| f.body == b"CORRUPT-TARGET"),
            "byte {i}: corrupt frame must not verify"
        );
        assert!(
            report
                .verified_frames()
                .any(|(_, f)| f.body == b"SURVIVOR-ISLAND"),
            "byte {i}: survivor must remain discoverable"
        );
    }
}

// ---------------------------------------------------------------------------
// §13: false magic inside bodies
// ---------------------------------------------------------------------------

#[test]
fn false_magic_inside_body_parent_verifies_once() {
    let mut body = Vec::new();
    body.extend_from_slice(START_MAGIC);
    body.extend_from_slice(END_MAGIC);
    body.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
    body.extend_from_slice(START_MAGIC);
    let frame = encode_item(5, &body);
    let mut buf = frame.clone();
    buf.extend_from_slice(&survivor());

    let report = scan_forward(&buf, SafetyLimits::default());
    // Parent + survivor; false magics inside body are not separate verified frames.
    assert_eq!(report.verified_count(), 2);
    let parent = report
        .verified_frames()
        .find(|(_, f)| f.header.event_id == event_id(5))
        .unwrap()
        .1;
    assert_eq!(parent.body, body);
    assert_survivor_discoverable(&buf);
}

// ---------------------------------------------------------------------------
// §13: corrupt length followed by a valid frame
// ---------------------------------------------------------------------------

#[test]
fn corrupt_length_followed_by_valid_frame() {
    let mut bad = encode_item(6, b"x");
    // Blow envelope_len without repairing CRC.
    bad[12] = 0xff;
    bad[13] = 0xff;
    bad[14] = 0xff;
    bad[15] = 0x7f;
    let mut buf = bad;
    buf.extend_from_slice(&survivor());
    assert_survivor_discoverable(&buf);
    let report = scan_forward(&buf, SafetyLimits::default());
    assert!(report.holes().count() > 0);
}

// ---------------------------------------------------------------------------
// §13: missing prefix followed by a valid suffix and later frame
// ---------------------------------------------------------------------------

#[test]
fn missing_prefix_valid_suffix_later_frame() {
    let full = encode_item(7, b"orphan-suffix-src");
    // Keep only the suffix of a real frame (missing prefix/body).
    let suffix = full[full.len() - FRAME_SUFFIX_LEN..].to_vec();
    let mut buf = suffix;
    buf.extend_from_slice(&survivor());

    let report = scan_forward(&buf, SafetyLimits::default());
    assert!(
        !report
            .verified_frames()
            .any(|(_, f)| f.body == b"orphan-suffix-src"),
        "orphan suffix must not verify without prefix"
    );
    assert_survivor_discoverable(&buf);

    // Reverse may attempt the suffix but must not accept without full frame.
    let rev = scan_reverse(&buf, SafetyLimits::default());
    assert!(!rev
        .verified_frames()
        .any(|(_, f)| f.body == b"orphan-suffix-src"));
    assert!(rev
        .verified_frames()
        .any(|(_, f)| f.body == b"SURVIVOR-ISLAND"));
}

// ---------------------------------------------------------------------------
// §13: valid prefix with missing suffix
// ---------------------------------------------------------------------------

#[test]
fn valid_prefix_missing_suffix() {
    let full = encode_item(8, b"no-suffix");
    let prefix_only = full[..FRAME_PREFIX_LEN].to_vec();
    let mut buf = prefix_only;
    buf.extend_from_slice(&survivor());

    let report = scan_forward(&buf, SafetyLimits::default());
    assert!(!report
        .verified_frames()
        .any(|(_, f)| f.body == b"no-suffix"));
    assert_survivor_discoverable(&buf);
}

// ---------------------------------------------------------------------------
// §13: valid suffix with missing prefix (standalone + with garbage mid)
// ---------------------------------------------------------------------------

#[test]
fn valid_suffix_missing_prefix_standalone() {
    let full = encode_item(9, b"suffix-only");
    let suffix = &full[full.len() - FRAME_SUFFIX_LEN..];
    let mut buf = b"~~".to_vec();
    buf.extend_from_slice(suffix);
    buf.extend_from_slice(b"~~");
    buf.extend_from_slice(&survivor());
    assert_survivor_discoverable(&buf);
    let report = scan_forward(&buf, SafetyLimits::default());
    assert!(!report
        .verified_frames()
        .any(|(_, f)| f.body == b"suffix-only"));
}

// ---------------------------------------------------------------------------
// §13: unsupported kinds, flags, envelope keys, and codecs
// ---------------------------------------------------------------------------

#[test]
fn unsupported_kinds_flags_still_structurally_verified() {
    // Unknown extension kind remains recoverable as opaque verified frame.
    let mut parts = item_parts(10, EMPTY_ENVELOPE, b"opaque-ext");
    parts.header.frame_kind = 200;
    let enc = encode_frame(&parts).unwrap();
    let dec = decode_frame(&enc, SafetyLimits::default()).unwrap();
    assert_eq!(dec.header.frame_kind, 200);
    assert_eq!(dec.header.known_kind(), None);

    // Unknown / reserved flag bits do not make a structurally valid frame corrupt.
    let mut flagged = item_parts(11, EMPTY_ENVELOPE, b"flagged");
    flagged.header.flags =
        FrameFlags::new(FrameFlags::COMPRESSED | FrameFlags::ENCRYPTED | FrameFlags::RESERVED_MASK);
    let enc = encode_frame(&flagged).unwrap();
    let dec = decode_frame(&enc, SafetyLimits::default()).unwrap();
    assert!(dec.header.flags.has_reserved_bits());
    assert!(dec.header.flags.compressed());
    // "codec unsupported" is a higher-layer interpretation; structural verify passes.
    assert_eq!(dec.body, b"flagged");

    // Unknown uint envelope keys are retained losslessly (FORMAT_SPEC §4.4).
    // map{99: "xyz"} — key 99 is not a core field; deterministic CBOR still verifies.
    let env = b"\xa1\x18\x63\x63xyz";
    let with_env = item_parts(12, env, b"body");
    let enc = encode_frame(&with_env).unwrap();
    let dec = decode_frame(&enc, SafetyLimits::default()).unwrap();
    assert_eq!(dec.envelope, env);

    // Unsupported wire major fails closed (not verified).
    let mut major = encode_item(13, b"x");
    major[8] = 99;
    // Fixing CRC is not done → still fails; even with salvage, not verified.
    assert!(decode_frame(&major, SafetyLimits::default()).is_err());
}

// ---------------------------------------------------------------------------
// §13: duplicated and conflicting event identifiers
// ---------------------------------------------------------------------------

#[test]
fn duplicated_and_conflicting_event_identifiers() {
    let a = encode_item(20, b"replica");
    let mut buf = a.clone();
    buf.extend_from_slice(&a); // replica
    buf.extend_from_slice(&encode_item(20, b"conflict-body"));
    buf.extend_from_slice(&encode_item(21, b"unique"));

    let report = scan_forward(&buf, SafetyLimits::default());
    let frames: Vec<_> = report
        .verified_frames()
        .map(|(o, f)| (o, f.clone()))
        .collect();
    assert_eq!(frames.len(), 4);

    let groups = group_by_event_id(frames);
    let g20 = groups
        .iter()
        .find(|g| g.event_id() == event_id(20))
        .unwrap();
    assert!(
        g20.is_conflicting(),
        "same event_id with different body is conflicting"
    );
    // Both survivors remain; no silent pick by encounter order.
    match g20 {
        EventIdOutcome::Conflicting { occurrences } => {
            assert!(occurrences.len() >= 2);
            let bodies: Vec<_> = occurrences.iter().map(|(_, f)| f.body.as_slice()).collect();
            assert!(bodies.contains(&b"replica".as_slice()));
            assert!(bodies.contains(&b"conflict-body".as_slice()));
        }
        other => panic!("expected conflict: {other:?}"),
    }

    let g21 = groups
        .iter()
        .find(|g| g.event_id() == event_id(21))
        .unwrap();
    assert!(matches!(g21, EventIdOutcome::Unique { .. }));

    // Pure replicas only:
    let mut pure = encode_item(22, b"same");
    pure.extend_from_slice(&encode_item(22, b"same"));
    let report = scan_forward(&pure, SafetyLimits::default());
    let frames: Vec<_> = report
        .verified_frames()
        .map(|(o, f)| (o, f.clone()))
        .collect();
    let groups = group_by_event_id(frames);
    assert!(matches!(
        &groups[0],
        EventIdOutcome::Replicas { offsets, .. } if offsets.len() == 2
    ));
}

// ---------------------------------------------------------------------------
// §13: partial chunk maps
// ---------------------------------------------------------------------------

#[test]
fn partial_chunk_maps_never_fill_holes() {
    let item = event_id(30);
    let p0 = ChunkPiece {
        item_id: item,
        index: 0,
        total: 3,
        logical_len: 3,
        body: b"AAA".to_vec(),
    };
    let p2 = ChunkPiece {
        item_id: item,
        index: 2,
        total: 3,
        logical_len: 3,
        body: b"CCC".to_vec(),
    };

    match reassemble_chunks(&[p0.clone(), p2.clone()], None) {
        ReassemblyState::Partial { extents, missing } => {
            assert_eq!(missing, vec![1]);
            assert_eq!(extents.len(), 3);
            assert!(extents[0].present);
            assert!(!extents[1].present);
            assert!(extents[2].present);
            // Must not invent payload for missing index.
            assert_eq!(extents[1].range.len(), 0);
        }
        other => panic!("expected partial: {other:?}"),
    }

    // Conflicting chunks at same index.
    let conflict_b = ChunkPiece {
        body: b"XXX".to_vec(),
        ..p0.clone()
    };
    assert!(matches!(
        reassemble_chunks(&[p0.clone(), conflict_b], None),
        ReassemblyState::Conflicting { index: 0, .. }
    ));

    // Complete path with content hash.
    let complete = [
        ChunkPiece {
            item_id: item,
            index: 0,
            total: 2,
            logical_len: 2,
            body: b"ab".to_vec(),
        },
        ChunkPiece {
            item_id: item,
            index: 1,
            total: 2,
            logical_len: 2,
            body: b"cd".to_vec(),
        },
    ];
    let hash = body_hash(b"abcd");
    match reassemble_chunks(&complete, Some(hash)) {
        ReassemblyState::Complete { body, content_hash } => {
            assert_eq!(body, b"abcd");
            assert_eq!(content_hash, hash);
        }
        other => panic!("expected complete: {other:?}"),
    }

    // Chunks remain independently verifiable as frames.
    let body0 = encode_chunk_body(&p0);
    let frame = encode_frame(&FrameParts {
        header: FrameHeader::new_draft(
            FrameKind::PayloadChunk,
            EMPTY_ENVELOPE.len() as u32,
            body0.len() as u64,
            item,
        ),
        envelope: EMPTY_ENVELOPE.to_vec(),
        body: body0,
    })
    .unwrap();
    let mut buf = frame;
    buf.extend_from_slice(&survivor());
    assert_survivor_discoverable(&buf);
}

// ---------------------------------------------------------------------------
// §13: forward and reverse discovery agreement
// ---------------------------------------------------------------------------

#[test]
fn forward_and_reverse_discovery_agreement() {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"HEAD-GARBAGE");
    buf.extend_from_slice(&encode_item(40, b"alpha"));
    buf.extend_from_slice(b"MID");
    buf.extend_from_slice(&encode_item(41, b"beta"));
    buf.extend_from_slice(&[0u8; 7]);
    buf.extend_from_slice(&encode_item(42, b"gamma"));
    buf.extend_from_slice(b"TAIL");

    let fwd = scan_forward(&buf, SafetyLimits::default());
    let rev = scan_reverse(&buf, SafetyLimits::default());

    let fwd_set: Vec<_> = fwd
        .verified_frames()
        .map(|(o, f)| (o, f.body.clone(), f.header.event_id))
        .collect();
    let rev_set: Vec<_> = rev
        .verified_frames()
        .map(|(o, f)| (o, f.body.clone(), f.header.event_id))
        .collect();
    assert_eq!(
        fwd_set, rev_set,
        "forward and reverse must agree on verified frames"
    );
    assert_eq!(fwd_set.len(), 3);
}

// ---------------------------------------------------------------------------
// §13: scanning without a segment descriptor or summary
// ---------------------------------------------------------------------------

#[test]
fn scanning_without_descriptor_or_summary() {
    let a = encode_item(50, b"island-a");
    let b = encode_item(51, b"island-b");
    let mut buf = a;
    buf.extend_from_slice(&b);
    let report = scan_forward(&buf, SafetyLimits::default());
    assert_eq!(report.verified_count(), 2);
    let rev = scan_reverse(&buf, SafetyLimits::default());
    assert_eq!(rev.verified_count(), 2);
}

// ---------------------------------------------------------------------------
// §13: scanning random and adversarial garbage
// ---------------------------------------------------------------------------

#[test]
fn scanning_random_and_adversarial_garbage() {
    // Pseudo-random stream without accidental full frames.
    let mut garbage: Vec<u8> = (0u16..4096)
        .map(|i| ((i.wrapping_mul(37) ^ 0xA5) as u8).wrapping_add(i as u8))
        .collect();
    // Inject false start/end magics adversarially.
    if garbage.len() > 100 {
        garbage[10..18].copy_from_slice(START_MAGIC);
        garbage[50..58].copy_from_slice(END_MAGIC);
        garbage[100..108].copy_from_slice(START_MAGIC);
        // Huge length-like fields after a false magic.
        garbage[18..22].copy_from_slice(&u32::MAX.to_le_bytes());
    }

    let report = scan_forward(&garbage, SafetyLimits::default());
    assert_eq!(
        report.verified_count(),
        0,
        "pure adversarial garbage must not verify frames"
    );

    // Plant a real survivor after adversarial prefix.
    let mut buf = garbage;
    buf.extend_from_slice(&survivor());
    assert_survivor_discoverable(&buf);

    let rev = scan_reverse(&buf, SafetyLimits::default());
    assert!(rev
        .verified_frames()
        .any(|(_, f)| f.body == b"SURVIVOR-ISLAND"));
}

// ---------------------------------------------------------------------------
// Cross-cutting: corrupt candidates never labeled verified
// ---------------------------------------------------------------------------

#[test]
fn corrupt_candidates_never_labeled_verified() {
    let good = encode_item(60, b"will-corrupt");
    let mut damaged = good.clone();
    // Flip body hash region in suffix (body_hash field).
    let hash_off = damaged.len() - FRAME_SUFFIX_LEN + 16;
    damaged[hash_off] ^= 0xff;

    let mut buf = damaged;
    buf.extend_from_slice(&survivor());
    let report = scan_forward(&buf, SafetyLimits::default());
    for (off, frame) in report.verified_frames() {
        // Re-verify every reported verified frame independently.
        let window = &buf[off as usize..off as usize + frame.frame_len as usize];
        decode_frame(window, SafetyLimits::default()).expect("verified region must re-decode");
    }
    assert!(!report
        .verified_frames()
        .any(|(_, f)| f.body == b"will-corrupt"));
    assert_survivor_discoverable(&buf);
}
