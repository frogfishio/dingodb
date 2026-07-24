# dingo-format

Stage 2 survival wire format for DingoDB: frame encode/decode, structural
integrity (CRC32C + BLAKE3-256), in-memory active segments + seal, forward and
reverse salvage scanning, event-id conflict analysis, and draft chunk
reassembly helpers.

Normative source: repository root [`FORMAT_SPEC.md`](../../FORMAT_SPEC.md).

## Status

**Draft wire profile.** Constants and layouts match FORMAT_SPEC draft v0.1
(`wire_major = 1`, `wire_minor = 0`). Not frozen until the project declares
wire major 1 stable.

Stage **2a–2d** are implemented: frames, segment seal, scanners, and the
FORMAT_SPEC §13 destructive corpus (`tests/section13_corpus.rs`).

## Surface (Stages 2a–2d)

| Area | API highlights |
|------|----------------|
| Frame codec | `encode_frame`, `decode_frame`, `verify_frame_at` |
| Integrity | CRC32C prefix/envelope + suffix; BLAKE3-256 body |
| Segment | `ActiveSegment::create` / `append` / `seal` → `SealedSegment` |
| Scanner | `scan_forward` / `scan_reverse` → `ScanReport` |
| Events | `group_by_event_id` → unique / replicas / conflicting |
| Chunks | `reassemble_chunks` → complete / partial / unavailable / conflicting |
| Draft meta | Fixed descriptor/summary/chunk body layouts |

Envelope bytes are still **opaque** (deterministic CBOR rules deferred).

## Non-goals (yet)

- Full deterministic-CBOR envelope validation (FORMAT_SPEC §5 condition 6)
- Durable storage IO / immutability enforcement (Stage 3 `dingo-store`)
- Compression or encryption transforms
- Full production chunk manifests (draft reassembly only)

## Quick example

```rust
use dingo_format::{
    scan_forward, ActiveSegment, FrameKind, SafetyLimits, SegmentId,
};

let ids = SegmentId::new([1u8; 16], [2u8; 16]);
let mut seg = ActiveSegment::create(ids, SafetyLimits::default(), 0)?;
seg.append(FrameKind::ItemEvent, b"", b"hello", [9u8; 16])?;
let sealed = seg.seal()?;

let report = scan_forward(sealed.as_bytes(), SafetyLimits::default());
assert!(report.verified_count() >= 3); // descriptor, item, summary
# Ok::<(), dingo_format::SegmentError>(())
```
