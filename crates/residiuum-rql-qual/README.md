# residiuum-rql-qual

**RQL-Q4 cross-engine qualification harness** (unpublished development tooling).

Authority: `doc/todo/rql/RQL_QUERY_QUALIFICATION_PROGRAM.md` §7 ·  
design: `doc/todo/rql/RQL_Q4_1_HARNESS_ARCHITECTURE.md` ·  
lanes: `doc/todo/rql/RQL_Q0_LANES_EXCLUSIONS.md` ·  
env: `doc/todo/rql/RQL_Q0_ENV_MANIFEST.md`

## Honesty

- **Not Gate-1.** Scaffold + structural tests only until Q4 package accept.
- **No competitive claims** from this crate until principal accepts harness design
  and Q3-green families are admitted into measured cells.
- Do **not** score Lane E (embedded) against Mongo TCP as one contest — lanes are separate.
- Comparator engine adapters (Mongo, CBL) are **stubs** until Q4.3; they return
  `AdapterStatus::NotConfigured` and never invent digests.

## Modules

| Module | Ownership |
|---|---|
| `lane` | Lane E / Lane S pairing; engine ids |
| `fixture` | Corpus case load + logical fixture handles |
| `engine` | `EngineAdapter` trait + status |
| `canonicalize` | Result digests per Q0 equivalence dimensions |
| `metrics` | §7.4 metric envelopes (types only in Q4.1) |
| `evidence` | Env fingerprint + evidence bundle writer |
| `cells` | Mandatory measured cell registry (ids) |
| `dataset` | §7.1 axes (shape, payload, memory ratio, dist, card, sel) |
| `generator` | Deterministic logical docs + content hash |
| `lifecycle` | §7.3 classes; cold/reopen honesty |
| `cell_plan` | Cell plans + concurrency/selectivity/lifecycle matrices |
| `shared_work` | Cross-engine logical fixture identity (content hash) |
| `run` | Smoke runner + evidence bundle publication |

## Features

```sh
cargo test -p residiuum-rql-qual
cargo test -p residiuum-rql-qual --features residiuum-embedded
```

## Verify

```sh
bash scripts/verify-rql-q4-harness.sh
```