# CSQ-11 released-writer fixture repository

## Purpose

Immutable Residiuum-identity fixtures for **old-writer → new-reader** edges
(`CSQ-COMPAT-001`). Compatibility is Residiuum format/store version edges and
claimed platforms — **not** dual-read of pre-reset product identities.

## Rules

1. Fixtures must be produced by a Residiuum-released (or frozen post-reset)
   writer binary only.
2. Record: binary hash, generation command, wire profile label, store meta
   version, and the advertised edge (writer major → reader major).
3. Pre-reset product/protocol identity fixtures are **invalid** for this gate.
   Failure to read them is expected, not a defect
   (`REBRAND_PROTOCOL_IDENTITY_RESET.md` §§3–5).
4. Unsupported edges (`CSQ-COMPAT-002`) must fail without modifying the source
   tree.

## First labor cut status

The self-edge (current writer → current reader) is exercised in
`crates/residiuum-store/tests/csq11_compat_scale_soak.rs`. Multi-version
released-binary fixture blobs land here when a second Residiuum wire/store
generation is claimed.