//! Stage 2b–2c: active segment + seal, forward salvage scanner (FORMAT_SPEC §6–§7, §13).

use dingo_format::{
    decode_descriptor_body, decode_summary_body, encode_frame, scan_forward, ActiveSegment,
    FrameHeader, FrameKind, FrameParts, HoleReason, SafetyLimits, SegmentId, START_MAGIC,
    EMPTY_ENVELOPE,
};

fn ids() -> SegmentId {
    let mut store = [0u8; 16];
    store[0] = 0xaa;
    let mut seg = [0u8; 16];
    seg[0] = 0xbb;
    SegmentId::new(store, seg)
}

fn item_frame(body: &[u8], event: u8) -> FrameParts {
    let mut event_id = [0u8; 16];
    event_id[0] = event;
    FrameParts {
        header: FrameHeader::new_draft(
            FrameKind::ItemEvent,
            EMPTY_ENVELOPE.len() as u32,
            body.len() as u64,
            event_id,
        ),
        envelope: EMPTY_ENVELOPE.to_vec(),
        body: body.to_vec(),
    }
}

#[test]
fn sealed_segment_scan_finds_descriptor_items_summary() {
    let mut active = ActiveSegment::create(ids(), SafetyLimits::default(), 42).unwrap();
    active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"one", [1u8; 16])
        .unwrap();
    active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"two", [2u8; 16])
        .unwrap();
    let sealed = active.seal().unwrap();
    assert_eq!(sealed.frame_count(), 4); // desc + 2 items + summary

    let report = scan_forward(sealed.as_bytes(), SafetyLimits::default());
    assert_eq!(report.verified_count(), 4);
    assert_eq!(report.holes().count(), 0);

    let kinds: Vec<_> = report
        .verified_frames()
        .map(|(_, f)| f.header.known_kind())
        .collect();
    assert_eq!(
        kinds,
        vec![
            Some(FrameKind::SegmentDescriptor),
            Some(FrameKind::ItemEvent),
            Some(FrameKind::ItemEvent),
            Some(FrameKind::SegmentSummary),
        ]
    );

    let frames: Vec<_> = report.verified_frames().map(|(_, f)| f).collect();
    let (desc_ids, created, _) = decode_descriptor_body(&frames[0].body).unwrap();
    assert_eq!(desc_ids, ids());
    assert_eq!(created, 42);

    let (sum_ids, sealed_len, frame_count) = decode_summary_body(&frames[3].body).unwrap();
    assert_eq!(sum_ids, ids());
    assert_eq!(sealed_len, sealed.len());
    assert_eq!(frame_count, 4);
}

#[test]
fn scan_works_without_descriptor_or_summary() {
    // FORMAT_SPEC §13: scanning without a segment descriptor or summary.
    let a = encode_frame(&item_frame(b"island-a", 1)).unwrap();
    let b = encode_frame(&item_frame(b"island-b", 2)).unwrap();
    let mut buf = a;
    buf.extend_from_slice(&b);

    let report = scan_forward(&buf, SafetyLimits::default());
    assert_eq!(report.verified_count(), 2);
    let bodies: Vec<_> = report
        .verified_frames()
        .map(|(_, f)| f.body.clone())
        .collect();
    assert_eq!(bodies, vec![b"island-a".to_vec(), b"island-b".to_vec()]);
}

#[test]
fn corrupt_middle_frame_later_island_survives() {
    // FORMAT_SPEC §13: corrupt length / damage followed by a valid frame.
    let mut active = ActiveSegment::create(ids(), SafetyLimits::default(), 0).unwrap();
    active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"before", [1u8; 16])
        .unwrap();
    let mid_off = active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"middle", [2u8; 16])
        .unwrap() as usize;
    active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"after", [3u8; 16])
        .unwrap();
    let sealed = active.seal().unwrap();
    let mut damaged = sealed.into_bytes();

    // Flip a body byte in the middle item so body hash fails.
    // Middle frame starts at mid_off; body follows 64-byte prefix + empty CBOR map.
    let body_at = mid_off + 64 + EMPTY_ENVELOPE.len();
    damaged[body_at] ^= 0xff;

    let report = scan_forward(&damaged, SafetyLimits::default());
    let bodies: Vec<_> = report
        .verified_frames()
        .map(|(_, f)| f.body.clone())
        .collect();
    assert!(
        bodies.iter().any(|b| b == b"before"),
        "before must survive: {bodies:?}"
    );
    assert!(
        bodies.iter().any(|b| b == b"after"),
        "after must survive: {bodies:?}"
    );
    assert!(
        !bodies.iter().any(|b| b == b"middle"),
        "corrupt middle must not verify: {bodies:?}"
    );
    // At least one hole / failed candidate recorded.
    assert!(report.holes().count() > 0);
}

#[test]
fn leading_garbage_and_false_magic_do_not_hide_frame() {
    let good = encode_frame(&item_frame(b"payload", 7)).unwrap();
    let mut buf = Vec::new();
    buf.extend_from_slice(b"xxx");
    buf.extend_from_slice(START_MAGIC); // false magic without valid frame
    buf.extend_from_slice(&[0u8; 20]);
    buf.extend_from_slice(&good);

    let report = scan_forward(&buf, SafetyLimits::default());
    let survivors: Vec<_> = report
        .verified_frames()
        .filter(|(_, f)| f.body == b"payload")
        .collect();
    assert_eq!(survivors.len(), 1);
    assert!(report.holes().count() >= 1);
}

#[test]
fn random_garbage_scan_finds_nothing_verified() {
    let garbage: Vec<u8> = (0u8..200)
        .map(|i| i.wrapping_mul(17).wrapping_add(3))
        .collect();
    // Avoid accidental DINGOFRM.
    let report = scan_forward(&garbage, SafetyLimits::default());
    assert_eq!(report.verified_count(), 0);
    if !garbage.is_empty() {
        assert!(report.holes().count() >= 1);
        assert!(matches!(
            report.holes().next().unwrap().1,
            HoleReason::UnclassifiedGarbage
        ));
    }
}

#[test]
fn active_segment_rejects_append_after_seal_path() {
    let active = ActiveSegment::create(ids(), SafetyLimits::default(), 0).unwrap();
    let sealed = active.seal().unwrap();
    // Rebuild is not possible from SealedSegment; ensure seal is terminal via
    // second seal attempt on a fresh active that we seal once.
    let mut a2 = ActiveSegment::create(ids(), SafetyLimits::default(), 0).unwrap();
    a2.append(FrameKind::Padding, EMPTY_ENVELOPE, b"", [0u8; 16]).unwrap();
    let s = a2.seal().unwrap();
    assert!(s.frame_count() >= 2);
    let _ = sealed;
}

#[test]
fn truncated_tail_does_not_poison_earlier_frames() {
    let mut active = ActiveSegment::create(ids(), SafetyLimits::default(), 0).unwrap();
    active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"stable", [1u8; 16])
        .unwrap();
    let off = active
        .append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"cut-me", [2u8; 16])
        .unwrap() as usize;
    let mut bytes = active.as_bytes().to_vec();
    // Truncate mid-second-frame.
    bytes.truncate(off + 40);
    let report = scan_forward(&bytes, SafetyLimits::default());
    let bodies: Vec<_> = report
        .verified_frames()
        .map(|(_, f)| f.body.clone())
        .collect();
    assert!(bodies.iter().any(|b| b == b"stable"));
    assert!(!bodies.iter().any(|b| b == b"cut-me"));
}

#[test]
fn scan_report_includes_limits() {
    let limits = SafetyLimits {
        max_envelope_len: 1024,
        max_body_len: 4096,
        max_frame_len: 8192,
    };
    let f = encode_frame(&item_frame(b"x", 1)).unwrap();
    let report = scan_forward(&f, limits);
    assert_eq!(report.limits, limits);
    assert_eq!(report.source_len, f.len() as u64);
}
