# Q4 §7.2 faithfulness findings (block Q5 competitive)

Status: **board filled** (2026-08-08) · labor not started  
Feature: `019fe054-1091-7c43-8db0-25394545d377`  
Authority: principal review wave 2 (same day); programme §7–§8  
Prior wave: [RQL_Q34_PREACCEPT_FINDINGS.md](./RQL_Q34_PREACCEPT_FINDINGS.md) F1–F9 labor `in_review`

## Gate

**Do not accept Q4 as competitive-ready** and **do not run Q5 baseline** until all
**P1** cards on this feature are labor-complete (`in_review`) and principal
re-accepts harness honesty. Prefer P2 before package accept; P3 hygiene for
campaign cleanliness.

F1–F9 fixed mechanical defects; verifiers PASS as **scaffold only**. This wave
addresses **specification faithfulness** for §7.2 / §7.4.

## Priority queue (claim order)

| ID | Pri | Board title | Primary paths |
|---|---|---|---|
| F10 | P1 | Execute real concurrency (not metadata-only) | `run.rs` ~54; `evidence.rs` scaffold concurrency=1 |
| F11 | P1 | Complete mandatory §7.2 plan variants | `cell_plan.rs` expanded portfolio ~329 |
| F12 | P1 | 1:N enrich must produce multiple matches | `engine.rs` enrich ~423–452 |
| F13 | P1 | Deep cursor must drive multi-page product API | `engine.rs` ~271–305; product adapter |
| F14 | P1 | Evidence model for §7.4 campaign fields | `evidence.rs` bundle; harness schemas; verify script |
| F15 | P2 | explain_plan_digest must not be result digest | `engine.rs` ~494–503 |
| F16 | P2 | Explicit Refused/Unsupported adapter status | `AdapterStatus` + product refusals |
| F17 | P3 | Remove `ringtail-sda-starter.zip` from repo tree | workspace root untracked zip |

## Board task ids

| ID | kanban_task_id |
|---|---|
| F10 | `019fe054-937e-7083-b680-503860cf7766` |
| F11 | `019fe054-95b9-73c1-9da9-db3899198521` |
| F12 | `019fe054-9895-7520-9bab-c008f693af2b` |
| F13 | `019fe054-9bc1-7b73-9df7-8593eaf24b35` |
| F14 | `019fe054-9e58-7811-82e8-011cdd9313d0` |
| F15 | `019fe054-a1ab-7701-83e2-eddbe1fd17eb` |
| F16 | `019fe054-a493-7f11-8849-782829ef928d` |
| F17 | `019fe054-a733-7610-87e9-03cb1e71c40d` |

## What still passes (scaffold)

- Q3 verifier PASS (oracle/differential/adversarial/page-concat)
- Q4 verifier PASS as scaffold (default + residiuum-embedded feature)
- F1–F9 labor residual honesty items (see prior findings pack)

## Non-claims

Board fill only this turn — no defect implementation in this package.
Q5.1+ remains HOLD until P1 F10–F14 labor-complete and principal allows intake.
