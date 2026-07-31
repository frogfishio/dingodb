# ResiduumDB threat model (DEF-063 first cut)

Status: **draft / in-tree first cut** (2026-07-27)  
Scope: surfaces required by DEF-063 Work list  
Not a substitute for independent security review (still required before production).

Companion: [DEFECTS.md](../DEFECTS.md) DEF-063, [FORMAT_SPEC.md](../FORMAT_SPEC.md),
[CLUSTER_SPEC.md](../CLUSTER_SPEC.md), [doc/RUNBOOK_RETENTION.md](RUNBOOK_RETENTION.md).

## 1. Assets

| Asset | Why it matters |
|-------|----------------|
| Authoritative event frames on durable media | Source of truth; salvage must preserve them |
| Primary / derived indexes and catalogs | Can lag media but must never invent commitment |
| Cluster Raft hard state + logs | Term, vote, membership, commit evidence |
| Wire / RPC traffic | Integrity and confidentiality of client and peer traffic |
| Credentials (TLS keys, auth tokens, secret refs) | Authentication and authorization boundaries |
| Backup packages and scrub findings | Recovery and integrity evidence |
| Operator control plane (config, endpoints, purge) | Privilege boundary for destructive ops |
| Supply chain (crates, CI artifacts, signed packages) | Trust in what operators actually run |

## 2. Trust boundaries

```text
[untrusted client / network]
        |  TLS / authz / admission (DEF-032–034)
        v
[residuum serve / serve-cluster process]
        |  local FS paths, flock writer ownership
        v
[store root: segments, meta, derived]
        |
        +--> [peer nodes]  mTLS (when cluster experimental path is enabled)
        +--> [backup / scrub / migrate tools]  separate process, same media
        +--> [salvage / doctor]  read-only evidence tools
```

- **Network clients** are hostile by default (malformed frames, replay, auth
  brute force, expensive queries).
- **Local filesystem** is trusted for exclusive-writer process identity only when
  flock/ownership holds; multi-writer or NFS is out of scope until fenced.
- **Peer nodes** are partially trusted: Raft + placement fencing bound
  authority; a compromised peer can withhold or equivocate within its role.
- **Operators** with admin rights can purge and reconfigure; that is intentional
  and must be audited (follow-on).

## 3. Adversaries and goals

| Adversary | Goals |
|-----------|--------|
| Remote unauthenticated | Crash process, DoS, information leak, bypass auth |
| Authenticated low-privilege client | Read/write outside ACL, exhaust resources, replay ops |
| Network MITM (no TLS / broken TLS) | Observe or modify traffic; inject frames |
| Malicious replica / partition | Split brain, stale leadership, forge commit evidence |
| Malicious / corrupted media | Silent data loss, salvage confusion, backup poison |
| Supply-chain attacker | Malicious dependency or release artifact |
| Insider with FS access | Rewrite segments under a live writer (mitigated by exclusive lock + hashes) |

## 4. Surfaces (must fuzz / review)

Aligned with DEF-091 targets and DEF-063 “hostile-input surfaces”:

| Surface | Primary crate | Status (this cut) |
|---------|---------------|-------------------|
| Frame decode / verify | `residuum-format` | Property tests + fuzz target `decode_frame` |
| Forward / reverse salvage scan | `residuum-format` | Property tests + fuzz targets |
| Deterministic CBOR envelopes | `residuum-format` | Property tests + fuzz target `cbor_envelope` |
| Segment descriptor / trailer | `residuum-format` | Unit / corpus tests; fuzz follow-on |
| Chunk manifest reassembly | `residuum-format` | Unit tests; fuzz follow-on |
| Store envelopes / indexes / catalogs | `residuum-store` | Integration tests; fuzz follow-on |
| SDA lexer / parser / eval | `sda-core` | Conformance JSON; property/fuzz follow-on |
| RPC / URL / protocol parsers | `residuum-client` / server | Protocol fixtures; fuzz follow-on |
| Cluster control metadata | `residuum-cluster` | Stage tests; fuzz follow-on |
| Salvage / migration / backup manifests | `residuum-store` | DEF-050–052 tests; fuzz follow-on |

## 5. Security controls already in tree (partial)

| Control | DEF / evidence |
|---------|----------------|
| TLS outside loopback; mTLS peer path | DEF-032 |
| Authz separates data / admin / salvage / purge | DEF-033 |
| Admission: rate, auth failures, churn, expensive ops, op-id replay | DEF-034 |
| Structured logs redact secrets/payloads by default (serve) | DEF-060 |
| Liveness / readiness / detail health | DEF-061 |
| Unsafe config combinations rejected | DEF-054 |
| Exclusive writer flock | DEF-020 |
| Frame integrity (CRC + BLAKE3 body hash) | FORMAT_SPEC §5 |
| Operation-id write dedup | DEF-010 |

## 6. Open risks (not closed by this document)

1. **No independent audit** — DEF-063 acceptance requires external review and
   zero unresolved critical/high findings on the claimed surface.
2. **Long-run fuzz residual** — scheduled nightly smoke + PR property bar exist
   (DEF-091-F labor); OSS-Fuzz / multi-hour accumulation still open.
3. **Network multi-node still experimental** — serve-cluster is not production;
   multiproc OS short soak/rolling restart exists (DEF-041-N labor); full Jepsen
   PORC vs live TCP + multi-hour soak remain.
4. **Wire still `1.0-draft`** — freeze checklist published
   ([WIRE_MAJOR1_FREEZE.md](WIRE_MAJOR1_FREEZE.md)); residual soak, long fuzz,
   external review still block stable label (DEF-053).
5. **Native S3/GCS / erasure / encryption** — archive path is mirror/scaffold;
   hostile cloud I/O not yet in scope of production claims.
6. **Client-side logging** — redaction parity is a follow-on to DEF-060.

### Process risks closed by DEF-063-A labor (still need audit execution)

| Item | Location |
|------|----------|
| Vulnerability disclosure process | [SECURITY.md](../SECURITY.md) |
| Supported-version policy | [SUPPORTED_VERSIONS.md](SUPPORTED_VERSIONS.md) |
| Auditor evidence package | [SECURITY_AUDIT_PACKAGE.md](SECURITY_AUDIT_PACKAGE.md) |

These do **not** close “no independent audit.” They remove the “process not
published” gap and package evidence for engagement.

## 7. Recommended next security labor

1. Commission independent review using [SECURITY_AUDIT_PACKAGE.md](SECURITY_AUDIT_PACKAGE.md).
2. Remediate critical/high findings; update this model’s residual list.
3. Multi-process partition + long soak evidence (DEF-041-N) before any
   “production cluster” language.
4. Close DEF-053 residual gates (audit, long fuzz, soak, clean-room fixtures)
   then freeze wire before stable wire claims
   ([WIRE_MAJOR1_FREEZE.md](WIRE_MAJOR1_FREEZE.md)).

## 8. Document control

| Field | Value |
|-------|--------|
| Profile | `dingo-threat-model-v0` (draft) |
| Freeze | Not frozen; revises with each major surface landing |
| Owners | Engineering program (DEF-063) |