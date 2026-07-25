# dingo-format

Survival wire format for DingoDB: frame encode/decode, structural integrity
(CRC32C + BLAKE3-256), deterministic CBOR envelopes, in-memory active segments +
seal, forward and reverse salvage scanning, event-id conflict analysis, and
draft chunk reassembly helpers.

Normative source: repository root [`FORMAT_SPEC.md`](../../FORMAT_SPEC.md).
Delivery: Stage **2a–2d** (complete) in [`DELIVERY_PLAN.md`](../../DELIVERY_PLAN.md).

## Status

**Shipped** for Stage 2. Wire profile label `WIRE_PROFILE_LABEL` = `1.0-draft`
(`wire_major = 1`, `wire_minor = 0`). Not frozen as production wire major 1
until the project declares a soak freeze; a breaking on-disk change would require
a major bump and dual-read support.

Implemented: frames, segment seal, scanners, the FORMAT_SPEC §13 destructive
corpus (`tests/section13_corpus.rs`), and deterministic CBOR envelope validation
(§5 condition 6).

## Surface

| Area | API highlights |
|------|----------------|
| Frame codec | `encode_frame`, `decode_frame`, `verify_frame_at` |
| Envelopes | `validate_deterministic_cbor_envelope`, `EMPTY_ENVELOPE`, uint-map encode/decode |
| Integrity | CRC32C prefix/envelope + suffix; BLAKE3-256 body |
| Segment | `ActiveSegment::create` / `append` / `seal` → `SealedSegment` |
| Scanner | `scan_forward` / `scan_reverse` → `ScanReport` |
| Events | `group_by_event_id` → unique / replicas / conflicting |
| Chunks | `reassemble_chunks` → complete / partial / unavailable / conflicting |
| Meta | Fixed descriptor/summary/chunk body layouts; `WIRE_PROFILE_LABEL` |

Envelopes must be a single definite-length CBOR map with unsigned integer keys,
shortest integer encodings, sorted unique keys, and valid UTF-8 text
(FORMAT_SPEC §4.4). The empty map `0xa0` (`EMPTY_ENVELOPE`) is the minimal
valid envelope.

## Out of scope (this crate)

- Durable storage IO / immutability enforcement — see [`dingo-store`](../dingo-store)
- Compression or encryption transforms
- Full production chunk manifests (reassembly helpers only; store owns chunked puts)
- Required-field checks per frame kind (FORMAT_SPEC §5 condition 11 partial)

## Quick example

```rust
use dingo_format::{
    scan_forward, ActiveSegment, FrameKind, SafetyLimits, SegmentId, EMPTY_ENVELOPE,
};

let ids = SegmentId::new([1u8; 16], [2u8; 16]);
let mut seg = ActiveSegment::create(ids, SafetyLimits::default(), 0)?;
seg.append(FrameKind::ItemEvent, EMPTY_ENVELOPE, b"hello", [9u8; 16])?;
let sealed = seg.seal()?;

let report = scan_forward(sealed.as_bytes(), SafetyLimits::default());
assert!(report.verified_count() >= 3); // descriptor, item, summary
# Ok::<(), dingo_format::SegmentError>(())
```
