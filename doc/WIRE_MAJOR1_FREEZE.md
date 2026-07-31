# Wire major-1 freeze qualification (DEF-053)

Policy id: `dingo-wire-major1-freeze-v1`  
Date: 2026-07-31  
Status: **freeze NOT declared** — `WIRE_PROFILE_LABEL` remains `1.0-draft`  
Companion: [FORMAT_SPEC.md](../FORMAT_SPEC.md), [DEFECTS.md](../DEFECTS.md) §DEF-053,
[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md), [SUPPORTED_VERSIONS.md](SUPPORTED_VERSIONS.md),
`residuum-format::compat`

This document is the **freeze gate checklist**, **gap inventory**, and
**compatibility policy** for on-disk / survival wire major 1. Completing this
package is necessary but **not sufficient** for freeze acceptance.

**Do not** change `WIRE_PROFILE_LABEL` from `1.0-draft` to a stable label until
every gate in §2 is **Met** and a principal freeze declaration is recorded here.

---

## 1. What freezes (and what does not)

| Artifact | In this freeze? | Label today |
|----------|-----------------|-------------|
| On-disk frame layout (prefix/envelope/body/suffix) | **Yes** (candidate) | `WIRE_PROFILE_LABEL = 1.0-draft` |
| Magics, CRC32C + BLAKE3-256 integrity | **Yes** (candidate) | same |
| Deterministic CBOR envelope rules | **Yes** (candidate) | same |
| Core frame kinds 0–13 assignments | **Yes** (candidate) | `FrameKind` in `residuum-format` |
| Chunk body layout helpers | **Yes** (candidate helpers) | store owns full manifests |
| Reader/writer support matrix | Policy under freeze | `residuum-format::compat` |
| Network RPC (`dingo-rpc-v1`) | **Separate** | `RPC_WIRE_LABEL = 1.0-draft` |
| Collection SDK API | **Separate** | `SDK_API_VERSION = 1.0` |
| Crate / workspace semver | **Separate** | monorepo `VERSION` |

Network RPC draft is related (DEF-031) but is **not** closed by freezing the
survival frame format alone. After frame freeze, RPC may still remain draft
until its own fixtures and external review close.

---

## 2. Freeze criteria (go / no-go)

A freeze declaration requires **all** rows Met. Partial labor does not authorize
relabel.

| ID | Criterion | Status | Evidence / residual |
|----|-----------|--------|---------------------|
| F1 | Framing layout + magics + field offsets match FORMAT_SPEC §4 | **Met (impl)** | `residuum-format::frame`; `frame_codec` tests |
| F2 | Integrity: CRC32C prefix/suffix + BLAKE3-256 body (FORMAT_SPEC §3) | **Met (impl)** | `integrity`; encode/decode round-trips |
| F3 | Safety limits reject adversarial lengths before allocation | **Met (impl)** | `SafetyLimits::draft_defaults`; checked frame len |
| F4 | Deterministic CBOR envelope validation | **Met (impl)** | `cbor_envelope`; fuzz target `cbor_envelope` |
| F5 | Chunk body encode/reassemble helpers + generation-exact store path | **Partial** | format helpers + DEF-098 labor; store manifests continuous |
| F6 | Conflict identity: same `event_id`, differing body → conflicting | **Met (impl)** | `group_by_event_id`; FORMAT_SPEC §9 tests in §13 corpus |
| F7 | Recovery ordering: salvage forward/reverse; later intact frames after damage | **Met (impl)** | FORMAT_SPEC §13 corpus (`section13_corpus`) |
| F8 | FORMAT_SPEC §13 required wire tests automated in CI/nightly | **Met (impl)** | nightly runs `section13_corpus`; PR via `quality.sh` paths |
| F9 | Fuzzing of untrusted format parsers (schedule + crash policy) | **Partial** | DEF-091 / 091-F: nightly 30s smoke; **not** OSS-Fuzz / multi-hour |
| F10 | Multi-implementation / clean-room fixtures published | **Partial** | HP-000 format vectors, §13 corpus, heap vectors; **no** second independent encoder |
| F11 | Production-scale soak + long corruption campaigns | **Open** | Nightly corruption suite exists; multi-hour / multi-media soak residual (ties DEF-041-N / DEF-090) |
| F12 | External review of framing, integrity, limits, chunks, envelopes, conflict, recovery | **Open** | [SECURITY_AUDIT_PACKAGE.md](SECURITY_AUDIT_PACKAGE.md) prepared; audit **not** completed |
| F13 | Canonical encodings inventory published | **Met (this cut)** | §3 below |
| F14 | Compatibility / support-window policy published | **Met (this cut)** | §4 below + `residuum-format::compat` |
| F15 | Golden corpus upgrade/downgrade across promised windows | **Open** | Single-gen migrate (DEF-052) shipped; multi-major dual-read not yet |
| F16 | Explicit freeze declaration + stable `WIRE_PROFILE_LABEL` | **Open** | Label stays `1.0-draft` until F1–F15 close |

**Go decision today: NO-GO.** Keep draft label.

---

## 3. Canonical encodings inventory (draft major 1)

These are the encodings this build treats as canonical for wire major 1
**candidate** bytes. Until freeze, treat as draft-stable for interop tests, not
as a long-term interoperability promise.

### 3.1 Frame structural constants

| Name | Value | Source |
|------|-------|--------|
| `START_MAGIC` | ASCII `DINGOFRM` (8 B) | `residuum-format::START_MAGIC` |
| `END_MAGIC` | ASCII `DINGOEND` (8 B) | `residuum-format::END_MAGIC` |
| Prefix length | 64 B | `FRAME_PREFIX_LEN` |
| Suffix length | 56 B | `FRAME_SUFFIX_LEN` |
| Byte order | little-endian for multi-byte integers | FORMAT_SPEC §2 |
| `WIRE_MAJOR` / `WIRE_MINOR` | `1` / `0` | frame header |
| `WIRE_PROFILE_LABEL` | `1.0-draft` | crate root |

### 3.2 Integrity

| Algorithm | Role | API |
|-----------|------|-----|
| CRC32C | Fast reject prefix+envelope and suffix | `prefix_crc32c`, `suffix_crc32c` |
| BLAKE3-256 | Body content integrity | `body_hash` (`BODY_HASH_LEN = 32`) |

CRC32C is damage detection, not authenticity. BLAKE3-256 is integrity evidence,
not authorship.

### 3.3 Envelope

- Encoding: **deterministic CBOR** definite-length map, unsigned integer keys,
  shortest integer encodings, sorted unique keys, valid UTF-8 text.
- Minimal empty envelope: single byte `0xa0` (`EMPTY_ENVELOPE`).
- API: `validate_deterministic_cbor_envelope`, uint-map encode/decode.

### 3.4 Core frame kinds (assigned)

| Value | Kind |
|------:|------|
| 0 | invalid/reserved |
| 1–9 | store/segment/item/chunk/batch/summary/purge/padding |
| 10–13 | heap/collection/stream descriptors + heap migration evidence |
| 14–16 | evidence ledger (spec-assigned; may be sparse in impl) |
| 17–127 | reserved core |
| 128–255 | application/profile extension |

Unknown kinds remain recoverable as opaque verified frames.

### 3.5 Flags (assigned)

| Bit | Name |
|----:|------|
| 0 | compressed |
| 1 | encrypted |
| 2 | chunked |
| 3 | canonical |
| 4 | repair |
| 5–7 | reserved (writers zero; readers tolerate unknown) |

**Honesty:** compressed/encrypted flags are format vocabulary; full transform
pipelines are not a freeze requirement for “frames survive salvage.”

### 3.6 Safety limits (draft defaults)

| Limit | Default |
|-------|---------|
| `max_envelope_len` | 64 KiB |
| `max_body_len` | 16 MiB |
| `max_frame_len` | 17 MiB |

Freeze may keep these as defaults; profiles may tighten. Raising defaults after
freeze requires a compatibility note; lowering defaults must not brick historical
media that used the previous bound within the support window.

### 3.7 Multi-implementation fixtures (current set)

| Fixture / suite | Location | Role |
|-----------------|----------|------|
| FORMAT_SPEC §13 destructive corpus | `crates/residuum-format/tests/section13_corpus.rs` | Required wire tests |
| Frame codec unit suite | `crates/residuum-format/tests/frame_codec.rs` | Encode/decode edges |
| Segment + scan suite | `crates/residuum-format/tests/segment_and_scan.rs` | Seal / salvage |
| Property / hostile tests | `crates/residuum-format/tests/stage_def_091_properties.rs` | DEF-091 |
| HP-000 format vectors | `spec/heap/vectors-v1.json` + `emit_hp000_format_vectors` | Heap/format golden |
| Protocol RPC goldens | `crates/residuum-sdk/tests/fixtures/protocol/` | Network (separate label) |
| cargo-fuzz targets | `fuzz/fuzz_targets/{decode_frame,cbor_envelope,scan_*,heap_ownership}` | Hostile input |

**Residual for true multi-impl:** a second language or clean-room encoder that
round-trips the same golden byte vectors without linking `residuum-format`.

---

## 4. Compatibility policy (draft support window)

### 4.1 Versioning rules (FORMAT_SPEC §12 + DEF-052)

1. **Major** may change framing semantics. Introducing a new writer major
   requires **dual-read** of the prior major until the support window ends.
2. **Minor** may add kinds, flags, or envelope fields while same-major older
   readers still locate, bound, verify, and retain unknown frames.
3. **Draft data** written under `1.0-draft` MUST be labeled draft in tests and
   packaging until freeze (§12). After freeze, historical draft corpora remain
   readable for the declared window (expected: entire major-1 life if bytes
   did not change).
4. **Unsupported** majors: preserve bytes as opaque evidence; do not silent
   rewrite (DEF-052 migrate).

### 4.2 Declared matrix (this build)

Source of truth: `residuum_format::wire_compat_matrix()` / `wire_support_summary()`.

| Major | Minors | Read | Write | Status |
|------:|--------|------|-------|--------|
| 1 | 0..∞ (open) | yes | yes | Current (draft profile) |

No second major is declared. Future major-2 must add a ReadOnly/Deprecated
row for major-1 before any major-2 writer ships.

### 4.3 Migration

- Profile: `dingo-migrate-v1` (DEF-052 single-generation cut).
- Phases: preflight → plan → apply → verify; rollback of incomplete.
- Evidence-preserving copy only; never in-place blind rewrite.
- Residual: multi-major dual-read rewrite jobs; rolling mixed-cluster upgrades.

### 4.4 Operator expectations under draft

1. Pin commits/tags; do not assume decade-long binary stability yet.
2. On upgrade, run doctor / migrate preflight as documented.
3. After freeze, migration notes will state whether `1.0-draft` media bit-equals
   frozen major-1 or requires a labeled rewrite.
4. Treat experimental multi-node paths as non-production regardless of wire
   freeze.

### 4.5 Relabel procedure (when freeze is authorized)

Only after §2 is all Met and principal accept:

1. Set `WIRE_PROFILE_LABEL` to a stable string (expected `1.0` unless semantics
   changed — then bump major and dual-read).
2. Update FORMAT_SPEC status from draft to frozen major 1.
3. Update CAPABILITY_MATRIX, SUPPORTED_VERSIONS, CONTRIBUTING, crate READMEs.
4. Tag golden archives with the freeze label.
5. Record freeze date and git revision in this document’s control table.
6. Open follow-on cards for dual-read retention and any deferred minors.

---

## 5. Executable evidence (how to re-check)

| Check | Command |
|-------|---------|
| Format unit + §13 corpus | `cargo test -p residuum-format` |
| §13 only | `cargo test -p residuum-format --test section13_corpus` |
| Compat / freeze guard | `cargo test -p residuum-format compat` |
| DEF-091 properties | `cargo test -p residuum-format --test stage_def_091_properties` |
| Fuzz property bar | `DINGO_FUZZ_SKIP_CARGO_FUZZ=1 ./scripts/fuzz-smoke.sh` |
| Migrate cut | `cargo test -p residuum-store --features legacy-raw-store --test stage_def_052_migrate` |
| Freeze readiness API | `wire_freeze_readiness()` / `wire_is_frozen()` in `residuum-format::compat` |

---

## 6. Gap summary (residual program)

| Gap | Blocks freeze? | Related work |
|-----|----------------|--------------|
| External wire/security review incomplete | **Yes** (F12) | DEF-063-A package; commission audit |
| Long fuzz / OSS-Fuzz accumulation | **Yes** (F9 residual) | DEF-091-F residual |
| Production-scale soak + long corruption | **Yes** (F11) | DEF-041-N residual, nightly expansion |
| Clean-room second implementation | **Yes** (F10 residual) | Future labor / DEL-0 golden expansion |
| Multi-major dual-read + golden cross-window | **Yes** (F15) | DEF-052 remaining |
| Stable label assignment | **Yes** (F16) | This gate after all Met |

---

## 7. Document control

| Field | Value |
|-------|--------|
| Policy id | `dingo-wire-major1-freeze-v1` |
| Labor cut | DEF-053 freeze gap inventory + policy (2026-07-31) |
| Freeze declared | **No** |
| Stable label | not assigned (`1.0-draft` retained) |
| Code guard | `residuum_format::wire_is_frozen() == false` while draft |

Revisions: append dated notes; do not silently edit Met→Open without a defect.
