# RQL-Q4.3 — Metrics, engine adapters, evidence publication

Status: **labor complete → board `in_review`** (2026-08-08) · package **not accepted**  
Package: RQL-Q4 · Feature `019fda4c-59bf-7320-a0cb-35f92c50fc45` · Task Q4.3  
Depends: Q4.1 architecture · Q4.2 dataset/cells  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §7.4  
Prior: [RQL_Q4_1_HARNESS_ARCHITECTURE.md](./RQL_Q4_1_HARNESS_ARCHITECTURE.md) ·
[RQL_Q4_2_DATASET_CELLS.md](./RQL_Q4_2_DATASET_CELLS.md)

## 1. Goal

Complete **metrics collection**, wire **engine adapters** for **shared logical work**,
and **publish evidence bundles** (versions, seeds, hashes). Harness is Q5-ready as
**structure** — no competitive claims until principal accepts design and Q3 families
are green for measured cells.

## 2. Ownership

| Module | Role |
|---|---|
| `metrics` | Latency collector, quantiles, assemble §7.4 envelope, key presence |
| `shared_work` | Same `LogicalDataset` content_hash for all engines |
| `engine` | Adapters: logical Ready, Mongo/CBL/server NotConfigured after load, Residiuum embedded feature |
| `run` | Smoke portfolio runner + evidence publish |
| `residiuum_embedded` | Optional product `CollectionClient::rql` path |

## 3. Metrics (§7.4)

Collectors fill:

- result digest + coverage + validity
- queries/s (from mean latency) + p50/p95/p99/max
- RSS best-effort (optional; Linux `/proc`; macOS residual)
- documents examined (path)
- explain plan digest (logical or product plan hash echo)
- lifecycle + cold method
- deferred work drain flags

Physical bytes / amplification / index size remain residual until store probes
(keys present in envelope as `None`).

## 4. Adapters

| Engine | Shared work | Execute |
|---|---|---|
| Logical harness | load | **Ready** pure digests (not product) |
| Residiuum embedded | load | feature `residiuum-embedded` product rql |
| Residiuum server | load | `adapter_not_configured` (op 118 residual) |
| Mongo local | load (hash identity) | `adapter_not_configured` (driver 3.8.0 residual) |
| CBL embedded | load (hash identity) | `adapter_not_configured` (native residual) |

**Law:** stubs never invent result digests. They record `shared_work_hash` for
fixture identity proofs across lanes.

## 5. Evidence publication

| Artefact | Path |
|---|---|
| Smoke evidence bundle | `spec/rql/qualification/harness-v1/q4_3_smoke_evidence_bundle.json` |
| Labor report | `spec/rql/qualification/harness-v1/q4_3_metrics_adapters_report.json` |
| Command | `bash scripts/verify-rql-q4-harness.sh` |

Bundle includes env fingerprint (Q0 pins), 12 smoke cells, content_hash, notes
that CBL/Mongo are not competitive Ready.

## 6. Evidence (labor)

```
cargo test -p residiuum-rql-qual   # 33/33
bash scripts/verify-rql-q4-harness.sh
```

Smoke: 12/12 logical Ready with results; CBL shared_work loaded; lane S fixture
identity true.

## 7. Non-claims

- **Not Gate-1**; **not RQL-Q4 package accept**; **not competitive baseline (Q5)**.
- Logical harness digests ≠ Residiuum product competitiveness.
- Mongo/CBL drivers not shipped; execute refuse is honest.
- Principal still accepts harness design before Q5 campaign.

## 8. Exit checklist (Q4.3)

- [x] Metrics collectors + §7.4 envelope assembly
- [x] Shared logical work across adapters
- [x] Mongo + CBL + server adapters (load + honest refuse)
- [x] Logical smoke execute + digests + metrics
- [x] Evidence bundle publication (hashes, seeds, versions)
- [x] One-command verify floors
- [ ] Principal harness accept (not labor)

## 9. Q4 package residual for principal

Q4.1–Q4.3 labor is on the board as `in_review`. Package `accept` requires principal
review of design + evidence format before Q5 admits competitive runs.
