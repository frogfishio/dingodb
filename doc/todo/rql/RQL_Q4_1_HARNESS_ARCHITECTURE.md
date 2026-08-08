# RQL-Q4.1 — Harness architecture: lanes, fixtures, evidence format

Status: **labor complete → board `in_review`** (2026-08-08) · package **not accepted**  
Package: RQL-Q4 · Feature `019fda4c-59bf-7320-a0cb-35f92c50fc45` · Task Q4.1  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §7  
Lanes: [RQL_Q0_LANES_EXCLUSIONS.md](./RQL_Q0_LANES_EXCLUSIONS.md)  
Env pins: [RQL_Q0_ENV_MANIFEST.md](./RQL_Q0_ENV_MANIFEST.md)  
Equivalence: [RQL_Q0_RESULT_EQUIVALENCE.md](./RQL_Q0_RESULT_EQUIVALENCE.md)

## 1. Goal

Design harness **crate/module ownership** and ship **skeleton interfaces** so Q4.2
(dataset cells) and Q4.3 (metrics + engine adapters) land without re-architecting.
Principal accepts design + skeleton **before** full cell matrix / competitive claims.

## 2. Ownership map

| Concern | Owner | Notes |
|---|---|---|
| Lane pairing + engine ids | `residiuum_rql_qual::lane` | Frozen Lane E / Lane S only |
| Corpus case index | `residiuum_rql_qual::fixture` | Loads `spec/rql/qualification/corpus-v1/` |
| Logical fixture bytes | `tools/rql_q1/materialise_fixture.py` | Unchanged; harness records handles |
| Engine execute | `residiuum_rql_qual::engine` | Trait + stubs; product optional feature |
| Result digests | `residiuum_rql_qual::canonicalize` | Q0 dimensions; not row-count alone |
| §7.4 metrics types | `residiuum_rql_qual::metrics` | Collectors → Q4.3 |
| Evidence + fingerprint | `residiuum_rql_qual::evidence` | Bundle writer + dirty-tree guard |
| Mandatory cell registry | `residiuum_rql_qual::cells` | 12 programme cells + concurrency slots |
| Store PQH | `residiuum-perf` | **Separate** — write-path qual, not RQL |

**Crate:** `crates/residiuum-rql-qual` (`publish = false`, MPL-2.0).  
**Not** a product dependency of `residiuum-sdk` / server.

## 3. Lanes (hard)

```text
Lane E (embedded):     ResidiuumEmbedded  vs  CouchbaseLiteEmbedded
Lane S (local c/s):    ResidiuumServer    vs  MongoLocal
```

- Cross-lane pairings are **rejected** by `LanePairing::validate`.
- Geometric means across lanes are **forbidden** for Gate-1.
- Full enrich cells: `server_lane_ineligible` by default (Q0.A4 Full-over-wire).

## 4. Engine adapters

| Engine | Q4.1 status | Next |
|---|---|---|
| Residiuum embedded | Optional feature `residiuum-embedded`; execute residual until Q4.2 runner | Q4.2/Q4.3 product path |
| Residiuum server | Stub `NotConfigured` | Q4.3 loopback + op 118 |
| Mongo local | Stub (pin **8.2.12**) | Q4.3 `mongodb` 3.8.0 driver |
| CBL embedded | Stub (pin **4.1.0**, Full Sync) | Q4.3 native binding |

Stubs **never invent digests**. They return stable
`adapter_not_configured:*` refuse codes.

## 5. Evidence format

| Schema | Path |
|---|---|
| Env fingerprint | `spec/rql/qualification/harness-v1/env-fingerprint-v1.schema.json` |
| Cell result | `spec/rql/qualification/harness-v1/cell-result-v1.schema.json` |
| Evidence bundle | `spec/rql/qualification/harness-v1/evidence-bundle-v1.schema.json` |

Fingerprint records: `git_sha`, `dirty`, Residiuum `VERSION`, rustc, OS/arch,
Mongo/CBL pins, `cbl_full_sync=true`, named query defaults
(`Available` / `Complete` / page 64). Dirty trees fail campaign start unless
principal `dirty_waiver`.

Bundle includes raw cell records, notes, and `content_hash` (SHA-256 of body
without the hash field).

## 6. Mandatory cells (registry only)

Programme §7.2 cells 1–12 are named in `MandatoryCell::ALL`. Dataset generators
and runners are **Q4.2**. Metrics collection is **Q4.3**.

Concurrency levels: `1, 2, 4, 8` + one host-declared oversubscribed slot.

## 7. Evidence (labor)

| Command | Result |
|---|---|
| `cargo test -p residiuum-rql-qual` | structural unit tests |
| `bash scripts/verify-rql-q4-harness.sh` | crate + schemas + architecture report |

Architecture report:
`spec/rql/qualification/harness-v1/q4_1_architecture_report.json`

## 8. Non-claims

- Not Gate-1; **not RQL-Q4 package accept**.
- Not competitive baseline (Q5).
- Scaffold does not measure product latency or publish comparator wins.
- Q3 package accept still principal; harness scaffold may proceed in parallel.

## 9. Exit checklist (Q4.1)

- [x] Design doc (this file)
- [x] Crate module ownership skeleton
- [x] Lanes enforced; cross-lane rejected
- [x] Engine adapter trait + honest stubs
- [x] Canonical digests (keys/values/order/coverage)
- [x] §7.4 metric type list + required keys
- [x] Evidence fingerprint + bundle writer
- [x] Machine schemas under `harness-v1/`
- [x] One-command structural verify
- [ ] Principal design accept (not labor)

## 10. Next

- **Q4.2** — dataset generators + mandatory cell runners  
- **Q4.3** — metrics collectors + Mongo/CBL/server adapters + evidence publication path  
