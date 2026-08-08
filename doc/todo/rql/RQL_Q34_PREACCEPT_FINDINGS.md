# Q3/Q4 pre-accept findings (block Q5+)

Status: **board filled** (2026-08-08) · labor not started  
Feature: `019fdfaf-e590-7ea3-b39c-671648ecaca5`  
Authority: principal review notes (same day); programme §6–§8  

## Gate

**Q5+ competitive baseline must not be treated as ready** until **all P1** cards on
this feature are labor-complete (`in_review`) and principal accepts Q3/Q4 honesty.
P2 preferred before package accept; P3 hygiene before check-in.

Note: Host stage graph forbids `todo → backlog` demotion of existing Q5–Q7 cards.
Those cards remain on the board with hold language in Q5.1 objective where
updated; implementers claim **F1–F9 first** (priority titles).

## Priority queue (claim order)

| ID | Pri | Board title | Primary paths |
|---|---|---|---|
| F1 | P1 | Fix residiuum-embedded adapter compile + feature-gated CI | **labor in_review**: `e.code().as_str()`; verify enables feature |
| F2 | P1 | Make mandatory Q4 §7.2 cells real runnable variants | **labor in_review**: multipage cursor, R/W writes, avg, enrich×3, cond high_band, concurrency executed |
| F3 | P1 | Align Point selectivity plan query with generator oracle | **labor in_review**: `sel_bucket_literal` POINT vs HIT |
| F4 | P1 | Fix 0.01% selectivity generator at campaign scale | **labor in_review**: exact target hits; tests 64/10k/1e6 |
| F5 | P2 | Distinct logical-harness identity in machine evidence | **labor in_review**: `LogicalHarness` + `execution_kind` |
| F6 | P2 | Stop `\|\|true` metric completeness; explicit residual states | **labor in_review**: `MetricPresenceState`; competitive fails on residual |
| F7 | P2 | Fix vacuous Q3 inter-page-write + token replay coverage | **labor in_review**: non-vacuous Available contract; token replay+cross-query |
| F8 | P2 | Verification must not rewrite checked-in evidence nondeterministically | verify scripts + tests |
| F9 | P3 | Q3.4 rustfmt + restore `verify-rql-q3.sh` executable bit | page_concat test; script mode |

## Verification (principal)

- `bash scripts/verify-rql-q3.sh` — green (default path)
- `bash scripts/verify-rql-q4-harness.sh` — default + **feature embedded** (F1 labor)
- `cargo test -p residiuum-rql-qual --features residiuum-embedded --lib` — **35/35** after F1 (was compile fail)

## Non-claims

Board fill only this turn — no defect fixes in this package.
Q3/Q4 labor packages remain `in_review` until principal accept after F1–F4+.