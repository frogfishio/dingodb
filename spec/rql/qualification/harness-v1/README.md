# RQL qualification harness contracts (v1)

Machine contracts for **RQL-Q4** evidence. Authority:

- `doc/todo/rql/RQL_QUERY_QUALIFICATION_PROGRAM.md` §7
- `doc/todo/rql/RQL_Q4_1_HARNESS_ARCHITECTURE.md`
- Q0 env / lanes / equivalence freezes under `doc/todo/rql/RQL_Q0_*.md`

| Artefact | Role |
|---|---|
| `evidence-bundle-v1.schema.json` | Campaign evidence bundle shape |
| `env-fingerprint-v1.schema.json` | Host + pin fingerprint |
| `cell-result-v1.schema.json` | Per-cell comparative record |
| `q4_1_architecture_report.json` | Labor architecture smoke report (written by tests) |

Implementing crate: `crates/residiuum-rql-qual` (`publish = false`).

**Non-claims:** Schema presence ≠ Gate-1; ≠ competitive campaign.
