# Security policy (DEF-063-A)

This document is the **public vulnerability disclosure process** for ResiduumDB
/ DingoDB. It is normative process text for operators and researchers.

Related:

| Document | Role |
|----------|------|
| [doc/SUPPORTED_VERSIONS.md](doc/SUPPORTED_VERSIONS.md) | Support windows and upgrade expectations |
| [doc/THREAT_MODEL.md](doc/THREAT_MODEL.md) | In-tree threat model (`dingo-threat-model-v0`) |
| [doc/SECURITY_AUDIT_PACKAGE.md](doc/SECURITY_AUDIT_PACKAGE.md) | Evidence pack for independent auditors |
| [doc/HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md](doc/HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md) | Heap isolation review brief (H6) |
| [fuzz/README.md](fuzz/README.md) | Continuous fuzz policy (DEF-091 / DEF-091-F) |

## Status of the product

**ResiduumDB is not production-ready.** Do not treat any release as having
passed independent security acceptance until:

1. an independent audit of the claimed surface is complete;
2. critical/high findings are remediated or accepted with published residual;
3. [CAPABILITY_MATRIX.md](doc/CAPABILITY_MATRIX.md) and maturity labels match
   the audited surface.

Reporting vulnerabilities is still welcome and will be handled under this
policy.

## Reporting a vulnerability

**Do not open a public GitHub issue for security-sensitive findings.**

Prefer a private report that includes:

1. Affected version / commit (`git rev-parse HEAD`, crate versions if known).
2. Surface (store media, RPC, CLI, cluster experimental path, salvage, backup,
   Heap, parsers, supply chain, etc.).
3. Impact (confidentiality / integrity / availability / privilege).
4. Reproduction steps, minimized PoC, and any crash/corpus files.
5. Whether you plan public disclosure and your preferred timeline.

### Contact channel

Until a dedicated security mailbox is published by the project maintainers,
report privately via **one** of:

1. **Private maintainer contact** for this repository (repository owner on the
   hosting forge — use the forge’s private vulnerability reporting feature when
   available, e.g. GitHub Security Advisories).
2. If the forge private-reporting feature is unavailable, open a **minimal
   non-sensitive** issue titled `SECURITY: private channel request` with **no
   exploit details**, asking for a secure contact path — then wait for a
   maintainer reply before sending technical details.

Do **not** attach exploit PoCs, credentials, or customer data to public issues.

## Coordinated disclosure

- We aim to acknowledge receipt within **7 calendar days**.
- We aim to provide an initial triage (severity / not applicable / needs more
  info) within **14 calendar days** of a complete report.
- For confirmed issues affecting published versions, we aim for a fix or
  published mitigation guidance within **90 days** of confirmation, or a
  documented exception with residual risk language.
- We prefer coordinated disclosure: please wait for a fix or agreed date before
  public write-ups. If we do not respond within the acknowledgment window after
  a good-faith private report, you may escalate by requesting a secure channel
  again via a non-sensitive public issue.

## Scope (in)

- Untrusted parsers and protocol surfaces (format frames, CBOR, SDA, RPC
  framing, manifests, continuation tokens) — see fuzz inventory.
- Local store integrity, exclusive writer ownership, salvage/doctor honesty.
- Authentication / authorization for shipped TLS and Heap paths (with stated
  maturity limits).
- Backup / restore / migrate control documents as hostile input.
- Supply-chain issues in released crates or published packages.

## Scope (out / lower priority)

- Denial-of-service via unbounded legitimate workload without a protocol bug
  (still useful feedback; may not be a security advisory).
- Issues only in experimental surfaces clearly labelled non-production
  (`serve-cluster`, unfinished Studio, etc.) — still reportable; severity may
  be limited by lack of production claims.
- Social engineering of operators outside the software.
- Physical access or full host compromise of a running node (documented as
  outside logical isolation claims).

## Safe harbor

Good-faith security research under this policy is welcome. Do not:

- access or modify data you do not own;
- degrade availability of systems you do not operate;
- pivot into third-party infrastructure.

## Advisories

Security advisories, when issued, will identify:

- affected versions and fixed versions (or “no release yet — use commit X”);
- severity and impact summary without unnecessary exploit detail;
- required operator actions (upgrade, rotate keys, re-run salvage, etc.).

## Supported versions

See [doc/SUPPORTED_VERSIONS.md](doc/SUPPORTED_VERSIONS.md). Only versions listed
as supported receive security fixes as a matter of policy.

## Residual (honest)

Independent **external** audit has **not** been completed under this labor cut.
Threat model + disclosure process + audit package are prerequisites; they do
not replace a signed external review (DEF-063 acceptance).
