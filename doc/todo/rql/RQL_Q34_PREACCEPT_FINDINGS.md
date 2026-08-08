# Q3/Q4 pre-accept findings (block Q5+)

Status: **F1–F9 labor complete → board `in_review`** (2026-08-08) · **awaiting principal accept**  
Feature: `019fdfaf-e590-7ea3-b39c-671648ecaca5`  
Authority: principal review notes (same day); programme §6–§8  

## Gate

**Q5+ competitive baseline must not be treated as ready** until **all P1** cards on
this feature are labor-complete (`in_review`) **and principal accepts** Q3/Q4 honesty.
P2 preferred before package accept; P3 hygiene before check-in.

**Labor gate (this wave):** F1–F9 are all board `in_review`. No further F-cards remain in
`todo`. Implementer must **not** claim Q5.1+ until principal allows Q5 intake
(Q5.1 objective still carries HOLD language).

Note: Host stage graph forbids `todo → backlog` demotion of existing Q5–Q7 cards.

## Priority queue (claim order) — labor done

| ID | Pri | Board title | Labor result |
|---|---|---|---|
| F1 | P1 | Fix residiuum-embedded adapter compile + feature-gated CI | `e.code().as_str()`; verify enables feature |
| F2 | P1 | Make mandatory Q4 §7.2 cells real runnable variants | multipage cursor, R/W, avg, enrich×3, cond high_band, concurrency |
| F3 | P1 | Align Point selectivity plan query with generator oracle | `sel_bucket_literal` POINT vs HIT |
| F4 | P1 | Fix 0.01% selectivity generator at campaign scale | exact target hits; tests 64/10k/1e6 |
| F5 | P2 | Distinct logical-harness identity in machine evidence | `LogicalHarness` + `execution_kind` |
| F6 | P2 | Stop `\|\|true` metric completeness; explicit residual states | `MetricPresenceState`; competitive fails on residual |
| F7 | P2 | Fix vacuous Q3 inter-page-write + token replay coverage | Available contract; token replay + cross-query |
| F8 | P2 | Verification must not rewrite checked-in evidence | `target/` default; publish scripts; verify no-spec-churn |
| F9 | P3 | Q3.4 rustfmt + restore `verify-rql-q3.sh` executable bit | rustfmt page_concat; scripts git mode 100755 |

## Verification (principal — re-run 2026-08-08)

| Command | Result |
|---|---|
| `bash scripts/verify-rql-q3.sh` | **PASS** — floors: `oracle_ok=101` `matrix_equal=101` `unsupported=5` `adv_dims=16` `page_concat_laws=4`; F8 no-spec-churn |
| `bash scripts/verify-rql-q4-harness.sh` | **PASS** — smoke=12 ready=12 lane_s=true; reports from `target/rql-q4`; F8 no-spec-churn |
| `cargo test -p residiuum-rql-qual --features residiuum-embedded --lib` | **44/44** |

Evidence write policy (F8): default tests/verify write under `target/rql-q{3,4}/` only.
Checked-in `spec/` snapshots: `scripts/publish-rql-q3-evidence.sh` /
`scripts/publish-rql-q4-evidence.sh` or `RESIDIUUM_WRITE_SPEC_EVIDENCE=1`.

## Principal actions (not labor)

1. Review F1–F4 (P1) fixes; accept or reject with notes.
2. Optionally accept F5–F9 (P2/P3) before or with package honesty.
3. On accept of pre-accept honesty: promote board cards `in_review` → `done` (principal only).
4. Then allow Q5 intake (lift HOLD on Q5.1 / Q5.2) if Q3/Q4 package path is clear.

## Non-claims

- Labor `in_review` ≠ package `accept` ≠ Gate-1.
- Not competitive baseline (Q5); not Atomics unlock.
- Decision 0 / RQL-C1 still open.
- Q5.1–Q7.2 remain on `todo` with dependency HOLDs — do not claim as implementer yet.

## Wave 2 (principal)

Mechanical F1–F9 labor is `in_review`. A second principal review found **Q4 §7.2
faithfulness** gaps that still block competitive proving — board feature
`019fe054-1091-7c43-8db0-25394545d377`, pack
[RQL_Q4_FAITHFULNESS_FINDINGS.md](./RQL_Q4_FAITHFULNESS_FINDINGS.md) (F10–F17).
