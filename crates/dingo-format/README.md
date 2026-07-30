# dingo-format

**Survival wire format** for ResiduumDB: frame encode/decode, structural integrity
(CRC32C + BLAKE3-256), deterministic CBOR envelopes, in-memory active segments
and seal, forward and reverse salvage scanning, event-id conflict analysis, and
chunk reassembly helpers.

This crate is pure format logic — **no durable storage IO**. The filesystem
store that writes and recovers segments is
[`dingo-store`](https://crates.io/crates/dingo-store).

## When to use this crate

| You want… | Use |
|-----------|-----|
| Open a database and put/get data | [`dingo-sdk`](https://crates.io/crates/dingo-sdk) |
| Single-node store (segments on disk) | [`dingo-store`](https://crates.io/crates/dingo-store) |
| Encode/decode/scan frames independently | **`dingo-format`** (this crate) |
| Network RPC framing | [`dingo-client`](https://crates.io/crates/dingo-client) |

## Install

```toml
[dependencies]
dingo-format = "0.1"
```

Or: `cargo add dingo-format`

## Status

**Shipped** for Stage 2. Wire profile label `WIRE_PROFILE_LABEL` = `1.0-draft`
(`wire_major = 1`, `wire_minor = 0`). Not frozen as production wire major 1
until the project declares a soak freeze; a breaking on-disk change would
require a major bump and dual-read support.

Implemented: frames, segment seal, scanners, the FORMAT_SPEC §13 destructive
corpus, and deterministic CBOR envelope validation.

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

## API surface

| Area | Highlights |
|------|------------|
| Frame codec | `encode_frame`, `decode_frame`, `verify_frame_at` |
| Envelopes | `validate_deterministic_cbor_envelope`, `EMPTY_ENVELOPE`, uint-map encode/decode |
| Integrity | CRC32C prefix/envelope + suffix; BLAKE3-256 body |
| Segment | `ActiveSegment::create` / `append` / `seal` → `SealedSegment` |
| Scanner | `scan_forward` / `scan_reverse` → `ScanReport` (verified islands + holes) |
| Events | `group_by_event_id` → unique / replicas / conflicting |
| Chunks | `reassemble_chunks` → complete / partial / unavailable / conflicting |
| Meta | Fixed descriptor/summary/chunk body layouts; `WIRE_PROFILE_LABEL` |

Envelopes must be a single definite-length CBOR map with unsigned integer keys,
shortest integer encodings, sorted unique keys, and valid UTF-8 text. The empty
map `0xa0` (`EMPTY_ENVELOPE`) is the minimal valid envelope.

## Design rule

> What is gone is gone. What remains still lives.

Frames are independently delimited and verified. Damage produces localized
holes; surviving verified islands remain recoverable without a global catalog.

## Out of scope (this crate)

- Durable storage IO / immutability enforcement — see
  [`dingo-store`](https://crates.io/crates/dingo-store)
- Compression or encryption transforms
- Full production chunk manifests (reassembly helpers only; the store owns
  chunked puts)
- Required-field checks per frame kind (partial)

## Related crates

| Crate | License | Role |
|-------|---------|------|
| [`dingo-store`](https://crates.io/crates/dingo-store) | MPL-2.0 | Filesystem store built on this format |
| [`dingo-client`](https://crates.io/crates/dingo-client) | MIT | Network RPC framing (separate from on-disk format) |
| [`dingo-examine`](https://crates.io/crates/dingo-examine) | MPL-2.0 | SDA examination over recovered frames |

## Documentation

- Format spec: [FORMAT_SPEC.md](https://github.com/frogfishio/dingodb/blob/main/FORMAT_SPEC.md)
- Architecture: [OVERVIEW.md](https://github.com/frogfishio/dingodb/blob/main/OVERVIEW.md)

## License

MIT.

Part of [ResiduumDB](https://github.com/frogfishio/dingodb).
