# Security audit evidence package (DEF-063-A)

Status: **labor package for independent engagement**  
Date: 2026-07-31  
Does **not** certify that an audit has been completed.

Use this as the table of contents for external reviewers. Completing this pack
is necessary but not sufficient for DEF-063 acceptance.

## 1. Engagement scope (recommended)

### 1.1 In scope (single-node + shipped network client paths)

| Area | Why |
|------|-----|
| Local store media, writer lock, salvage/doctor honesty | Core data integrity |
| Untrusted parsers (format, CBOR, SDA, RPC frames, manifests, tokens) | Hostile input |
| TLS / authz / admission for `residuum serve` (as implemented) | Network boundary |
| Heap isolation Level-1 claims (if claimed in release) | Authorization TCB |
| Backup / migrate / scrub control documents | Recovery trust |
| Supply chain of published crates | What operators run |

### 1.2 Explicitly experimental / non-production

- `serve-cluster` / multi-node Raft network surface (until DEF-041-N evidence)
- Studio / unfinished DX
- Native cloud object backends still scaffolded

### 1.3 Out of scope unless contracted

- Full side-channel lab
- Social engineering
- Physical host compromise as a logical isolation failure

## 2. Documents to read first

| Document | Role |
|----------|------|
| [SECURITY.md](../SECURITY.md) | Disclosure process |
| [SUPPORTED_VERSIONS.md](SUPPORTED_VERSIONS.md) | Support policy |
| [THREAT_MODEL.md](THREAT_MODEL.md) | Assets, boundaries, adversaries |
| [HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md](HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md) | Heap TCB questions |
| [CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md) | Honest shipping claims |
| [CRASH_AND_RECOVERY_CONTRACT.md](CRASH_AND_RECOVERY_CONTRACT.md) | Operator recovery contract |
| [PRIMARY_INDEX_LIFECYCLE.md](PRIMARY_INDEX_LIFECYCLE.md) | Derived vs authority |
| [../DEFECTS.md](../DEFECTS.md) DEF-032–034, 063, 091, 097, 101 | Security-related defects |

## 3. Executable evidence (CI / local)

| Evidence | How to run |
|----------|------------|
| Delivery / quality bar | `./scripts/quality.sh` |
| Fuzz property bar (no cargo-fuzz required) | `RESIDUUM_FUZZ_SKIP_CARGO_FUZZ=1 ./scripts/fuzz-smoke.sh` |
| Full fuzz smoke (nightly + cargo-fuzz) | `./scripts/fuzz-smoke.sh` or CI job `fuzz_smoke` |
| Crash matrix (CI subset) | `cargo test -p residuum-store --features legacy-raw-store --test stage_def_022_crash_matrix` |
| Cluster in-process verify | `cargo test -p residuum-cluster --test stage_def_041_verify` |
| Writer-lock honesty | `cargo test -p residuum-store --features legacy-raw-store --test stage_def_101_writer_lock` |
| Continuation secret keys | `cargo test -p residuum-store --features legacy-raw-store --test stage_def_097_token_keys` |
| Crash/recovery contract | `./scripts/verify-crash-recovery-contract.sh` |

## 4. Fuzz inventory (DEF-091 / 091-F)

See [fuzz/README.md](../fuzz/README.md). Continuous **schedule** is nightly
30s×target smoke plus PR property bar — not yet OSS-Fuzz multi-hour
accumulation.

| Target | Surface |
|--------|---------|
| decode_frame / cbor_envelope / scan_* / heap_ownership | format |
| sda_parse | SDA |
| rpc_frame | client framing |
| chunk_manifest / item_envelope / backup_manifest / cursor_token | store |

## 5. Known open security program residuals

1. **Independent external audit not completed** (this package prepares it).
2. **Wire still `1.0-draft`** (DEF-053) — freeze checklist:
   [WIRE_MAJOR1_FREEZE.md](WIRE_MAJOR1_FREEZE.md).
3. **Multi-process Jepsen / long soak** residual (DEF-041-N).
4. **OSS-Fuzz / long fuzz budgets** residual (DEF-091-F residual).
5. **Experimental cluster** maturity labels must stay experimental.

## 6. Findings workflow

1. Reporter uses [SECURITY.md](../SECURITY.md).
2. Maintainers triage and track as DEFECTS / Kanban cards.
3. Critical/high on a **claimed** production surface block that claim until
   remediated or residual is published in CAPABILITY_MATRIX / threat model.
4. External audit findings follow the same track; remediation cards should
   link audit report ids.

## 7. Suggested deliverables from external review

- Written report with severity, repro, affected surface, residual risk.
- Explicit mapping to threat-model assets/adversaries.
- Confirmation of what is **out of scope** vs tested.
- Sign-off language suitable for CAPABILITY_MATRIX maturity updates
  (maintainers apply labels; reviewers do not auto-promote maturity).

## 8. Document control

| Field | Value |
|-------|--------|
| Package id | `dingo-security-audit-package-v1` |
| Labor | DEF-063-A process cut |
| External audit | **Not complete** |