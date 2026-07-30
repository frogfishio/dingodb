# DingoDB architecture map

This file points implementers at the **normative specs** and the **crate layout**.
It is not a second architecture document.

## Product thesis

> Put anything in. Keep it at scale. Damage it. Find what survived.

Governing recovery rule: *What is gone is gone. What remains still lives.*

## Normative documents

| Concern | Document |
|---------|----------|
| Database identity, trust, security, encryption, lifecycle, ownership | [DATABASE_DOCTRINE.md](DATABASE_DOCTRINE.md) |
| Logical heap identity, containment, and access isolation | [HEAP_SPEC.md](HEAP_SPEC.md) |
| System architecture, storage model, recovery, quality bars | [OVERVIEW.md](OVERVIEW.md) |
| Survival wire format, frames, segments, scanner tests | [FORMAT_SPEC.md](FORMAT_SPEC.md) |
| Core storage invariants, failure model, and qualification suite | [CORE_STORAGE_QUALIFICATION_SPEC.md](CORE_STORAGE_QUALIFICATION_SPEC.md), [implementation plan](doc/CORE_STORAGE_QUALIFICATION_IMPLEMENTATION_PLAN.md) |
| Everyday API, CLI, progressive disclosure | [DX_SPEC.md](DX_SPEC.md) |
| First Heap-bound Rust application API and DQL delivery package | [doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) |
| Missing application APIs and product-capability closure | [PRODUCT_DEFICIENCIES.md](PRODUCT_DEFICIENCIES.md) |
| Immediate post-qualification application baseline packages | [MUST_ADD.md](MUST_ADD.md) |
| Structured Data Algebra (standalone language) | [SDA_SPEC.md](SDA_SPEC.md) |
| Dingo Query Language (v1 design; shipped parser is v0.1 subset) | [DQL_SPEC.md](DQL_SPEC.md), current-subset guide [doc/DQL/USER_GUIDE.md](doc/DQL/USER_GUIDE.md) |
| Exact ranked query access and rank/select substrate | [DIRECT_ACCESS_SPEC.md](DIRECT_ACCESS_SPEC.md) |
| Filter-conditioned sorting without prefix enumeration | [ORDER_WAVELET_SPEC.md](ORDER_WAVELET_SPEC.md) |
| Shared total predicate semantics for DQL and DRE | [DINGO_PREDICATE_SPEC.md](DINGO_PREDICATE_SPEC.md) |
| Dingo Rule Expression (DRE) constraint language and Invariant Core | [DRE_SPEC.md](DRE_SPEC.md) |
| Collection-owned behaviour and default scope confinement | [COLLECTION_CONTRACT_SPEC.md](COLLECTION_CONTRACT_SPEC.md) |
| Bounded serializable state transitions and relationship integrity | [ATOMICS_SPEC.md](ATOMICS_SPEC.md) |
| Durable security and administrative evidence | [EVIDENCE_LEDGER_SPEC.md](EVIDENCE_LEDGER_SPEC.md) |
| Operational telemetry collection and Ratatouille export | [TELEMETRY_SPEC.md](TELEMETRY_SPEC.md) |
| First-party desktop database IDE | [STUDIO_SPEC.md](STUDIO_SPEC.md), [implementation plan](doc/STUDIO_IMPLEMENTATION_PLAN.md) |
| Testing, assurance levels, claim evidence, and release verification | [TESTING_STRATEGY.md](TESTING_STRATEGY.md), [implementation plan](doc/VERIFICATION_IMPLEMENTATION_PLAN.md), [status](doc/VERIFICATION_STATUS.md) |
| SQL-ish+ executable surface and SQL→DQL compiler | [SQL_TO_DQL_SPEC.md](SQL_TO_DQL_SPEC.md) |
| JSON Schema Draft 2020-12 import into DRE | [JSON_SCHEMA_TO_DRE_SPEC.md](JSON_SCHEMA_TO_DRE_SPEC.md) |
| Query dialects (dql / sda / json / mongo / sql / … → pure SDA) | [doc/SDA/DIALECTS.md](doc/SDA/DIALECTS.md) |
| SDA examination of recovered DingoDB units | [SDA_PROFILE.md](SDA_PROFILE.md) |
| Enrichment algebra (ENR1 kernel in `dingo-sda`; ENR2 candidates design-only) | [crates/enr-core/README.md](crates/enr-core/README.md), [ENR1.md](crates/enr-core/ENR1.md), [ENR2.md](crates/enr-core/ENR2.md); profile `sda-enr1-v0.1` |
| Cluster federation and coverage | [CLUSTER_SPEC.md](CLUSTER_SPEC.md) |
| Product framing | [USP.md](USP.md) |
| Public product website | [WEBSITE_SPEC.md](WEBSITE_SPEC.md) |
| Public documentation website | [DOCS_SITE_SPEC.md](DOCS_SITE_SPEC.md) |
| Three-stage competitive goals and exit gates | [COMPETITIVE_GOALS.md](COMPETITIVE_GOALS.md) |
| Definitive execution priority, stages, and current starting queue | [MASTER_DELIVERY_PLAN.md](MASTER_DELIVERY_PLAN.md) |
| Staged delivery and exit criteria | [DELIVERY_PLAN.md](DELIVERY_PLAN.md) |
| Doctrine implementation gap map | [doc/DOCTRINE_GAPS.md](doc/DOCTRINE_GAPS.md) |
| Post-Heap implementation sequence and package gates | [NEXT_BUILD_PLAN.md](NEXT_BUILD_PLAN.md), [doc/NEXT_BUILD_STATUS.md](doc/NEXT_BUILD_STATUS.md) |

Prefer amending a named section of a normative doc before inventing new behavior.

## Delivery stages (summary)

See [DELIVERY_PLAN.md](DELIVERY_PLAN.md) for full exit criteria.

| Stage | Focus | Status |
|-------|--------|--------|
| 0 | Repo + CI harness | **done** (workspace, CI, language decision) |
| 1 | SDA standalone (pure) | **done** — full §14 MUST lock; corpus tag `sda-standalone-v1.0` |
| 2 | Wire format + salvage scanner | **2a–2d** — frames, seal, fwd/rev scan, §13 corpus, deterministic CBOR envelopes |
| 3 | Single-node store | **3a–3c** — put/get/delete, §16 suite, descriptor + index cache |
| 4 | Collection SDK | **4a–4d** — `dingo-sdk` open, JSON/bytes, scan/stream, filters, `ErrorCode` |
| 5 | SDA examination profile | **done** — `dingo-examine` ExaminationUnit + SDA over salvage |
| 6 | Indexes, catalogs, chunks | **done** — secondary indexes, history, chunks, compact, checkpoints |
| 7 | CLI doctor/salvage + server | **done** — `dingo-cli`, connect options (auth/deadline/retry), nightly packaging |
| 8 | Cluster federation | **8a–8f done** — partitions, coverage, Raft, convergent-append, SDK routing, find coverage, rebalance |
| 9 | Tiering / archive | **done** — filesystem media roots, segment move/copy, hierarchical catalogs, offline coverage, retention runbook |

## Crate layout (current)

```text
dingodb/
  crates/
    sda-core/       # package name dingo-sda; SDA+ENR1 hybrid pure eval (Stage 1) — MIT
    sda-cli/        # package name dingo-sda-cli; `dingo-sda` binary (Stage 1) — MIT
    enr-core/       # ENR1/ENR2 specs; ENR1 runtime lives in dingo-sda (one compile path)
    dingo-format/   # frames, CBOR envelopes, seal, scan, §13 corpus (Stage 2a–2d) — MIT
    dingo-client/   # framed RPC + handshake only — MIT
    dingo-store/    # single-node append store (Stages 3 + 6 + 7 inspect/salvage_to) — MPL-2.0
    dingo-sdk/      # collection API + remote connect (Stages 4 + 6 + 7); cluster via feature — MPL-2.0
    dingo-server/   # accept loop, authz, admission, Raft RPC glue, serve_* — AGPL
    dingo-examine/  # ExaminationUnit + SDA over salvage (Stage 5) — MPL-2.0
    dingo-cli/      # `dingo` binary: put/get, doctor, salvage, backup/restore, scrub, migrate, serve (Stage 7) — AGPL
    dingo-cluster/  # partitions, coverage, multi-node + Raft + find + rebalance (Stage 8a–8f) — AGPL
```

Crate ownership:

| Stage | Crate | Role |
|-------|-------|------|
| 2 | `dingo-format` | **Present** — frames, deterministic CBOR envelopes, seal, scanner, §13 corpus (2a–2d) |
| — | `dingo-client` | **Present** — MIT wire framing + handshake (`dingo-rpc-v1`) |
| 3+6+7 | `dingo-store` | **Present** — put/get/delete, salvage, open_inspect, salvage_to, backup_to/restore (DEF-050), scrub (DEF-051), migrate (DEF-052), catalogs, chunks, history, compact |
| 4+6+7+8d–8e | `dingo-sdk` | **Present** — collections, filters, indexes, history, remote RPC; `cluster` feature for open_cluster |
| 5 | `dingo-examine` | **Present** — ExaminationUnit projection, salvage stream, SDA filter/map, bounded pages |
| 7 | `dingo-server` | **Present** — bounded serve, authz, admission, TLS bind policy, network Raft glue |
| 7 | `dingo-cli` | **Present** — `dingo` put/get/list/doctor/salvage/backup/restore/scrub/migrate/serve (serve via `dingo-server`) |
| 8 | `dingo-cluster` | **Present (8a–8f)** — partitions, coverage, Raft, convergent-append, find honesty, rebalance |

Rule of thumb from the delivery plan: **vertical slices over empty package trees.**

## Language decisions (Stage 0)

| Choice | Decision |
|--------|----------|
| Core implementation language | **Rust** |
| First embedded surface | Rust library API; TypeScript-like examples in DX_SPEC remain the product shape |
| First CLI | `dingo-sda` (Stage 1) + `dingo` (Stage 7) |
| SDA packaging | `dingo-sda` (lib) + `dingo-sda-cli` (`dingo-sda` binary); SDA+ENR1 hybrid; no storage IO |
| Wire format versioning | Draft `1.0-draft`; reader/writer matrix + migrate phases (DEF-052); freeze is DEF-053 |
| Process configuration | Versioned `dingo-config-v1` validate-before-serve (DEF-054); live reload follow-on |
| Operational telemetry | [Ratatouille-only bounded firehose](TELEMETRY_SPEC.md); no request-path file/stdout logging |
| Formal audit | Dingo Evidence Ledger; durable, Heap-confined, independently verifiable |
| Metrics / health | Versioned `dingo-metrics-v1` scrape + `dingo-health-v1` live/ready/detail RPCs (DEF-061); store/cluster gauges follow-on |
| License | Multi-tier: MIT / MPL-2.0 / AGPL-3.0-or-later (see `doc/LICENSING.md`) |

## SDA import convention

- Package name on crates.io: **`dingo-sda`** (never bare `sda` / `sda-lib`)
- CLI package: **`dingo-sda-cli`**, binary **`dingo-sda`**
- Workspace dependency key: `sda-core` → Rust path `sda_core::…` (dependents)
- Inside the library package / its integration tests: `dingo_sda::…`
- Product shape: SDA + additive ENR1 hybrid for DingoDB, not a generic pure-SDA claim

## Product follow-ons (in-tree v0.23 — not production)

Stages **0–9** are implemented in-tree. Product follow-ons 1–4:

1. **S3/GCS filesystem mirrors** — `MediaLocator` + `CloudMirrorConfig`
   (`DINGO_S3_ROOT` / `DINGO_GS_ROOT`); `object:local:` stand-in unchanged.
   These are **mirrors**, not native cloud backends.
2. **Network multi-hop routing + experimental Raft** — `dingo serve-cluster` +
   live `endpoints.json` reload; `RemoteClient` routes keyed ops and refreshes
   on transport failure; demo `scripts/demos/08_kill_a_node.sh`. Requires
   `--experimental-network-cluster`. When Raft attaches (default), collection
   put/delete use partition propose (DEF-037) and control-plane `raft_*` RPCs
   (DEF-036); acks report `committed` only after quorum + local apply.
   Directory-only fallback if attach fails. Deterministic multi-replica tests
   still prefer in-process `Dingo::open_cluster`.
3. **Freeze / packaging labels** — `SDK_API_VERSION` (`1.0`),
   `CLUSTER_PROFILE_VERSION` (`v1` in-process), `WIRE_PROFILE_LABEL`
   (`1.0-draft`), plus `CLUSTER_COMMIT_PROFILE` (`dingo-cluster-commit-v1`).
   Distinct from crate semver `0.2.0`.
4. **Nice-to-haves** — `LifecyclePolicy`, erasure manifest scaffold,
   [doc/BENCHMARK_DISCLOSURE.md](doc/BENCHMARK_DISCLOSURE.md) (OVERVIEW §12.2).

Network Raft control plane, data-plane commit, durable rebalance jobs,
in-process anti-entropy repair, and seeded in-process verification are in-tree
on the experimental path (DEF-035–041). Production local-cluster gates
(multi-process Jepsen / long soak) remain DEF-041 follow-ons. Operator path today:
development `dingo serve`, experimental `serve-cluster` with Raft when attached,
and offline node salvage. Maturity labels:
[doc/CAPABILITY_MATRIX.md](doc/CAPABILITY_MATRIX.md), [DEFECTS.md](DEFECTS.md).
Work horizon (stage plan vs remaining gates):
[doc/WORK_HORIZON.md](doc/WORK_HORIZON.md).

## Stage 9 (landed)

Filesystem hot/warm/cold/archive media roots, segment move/copy with stable
identities, hierarchical segment catalogs, offline-tier coverage honesty, and
[doc/RUNBOOK_RETENTION.md](doc/RUNBOOK_RETENTION.md).

Object-style addressing: parse `MediaLocator` (`file` / `object:local` / `s3` /
`gs`); local object media and mirrored cloud roots work under the placement API.
