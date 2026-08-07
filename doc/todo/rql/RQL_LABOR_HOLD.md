# RQL critical-path labor hold (historical — Q0 hold lifted)

Status: **2026-08-07 · Q0 ACCEPT · Q1 admitted · Q2+ still gated**
Authority: principal §5 ACCEPT on [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md) ·
[RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3, §11 ·
[CRITICAL_PATH.md](../../../CRITICAL_PATH.md)
Q0 amendment Feature: `019fdac4-1408-7321-8edc-a09851c9e656` (complete)
Q1 Feature: `019fda4c-11fd-7102-bd55-10a347802144`

---

## Verdict

```text
Q0 package accept        = ACCEPT (freeze only; SHA e1f5c670a99dc54da477c531c83bca4985199a42)
Admitted Gate-1 labor    = RQL-Q1 corpus (claim Q1.1 first)
Q1 corpus implementation = ADMITTED
Decision 0 / RQL-C1      = OPEN / FORBIDDEN (micro-op purity not D0 close bar — Q0.A7)
```

Principal (2026-08-07): **Q0 has been accepted** after A11–A14 closeout. Q1 unblocked.
RQB1 remains **deleted**; architecture gate forbids restoration. Gate-1 is **not** passed.

---

## What is ready for principal (after amendments)

| Pack | Path | Labor |
|---|---|---|
| First freeze (Q0.1–Q0.4) | `RQL_Q0_*.md` | labor cards board `done` — **requires amendment** |
| Accept pack v1 | [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md) | superseded for accept until A1–A8 + A9 re-issue |
| Amendment package | Feature `019fdac4-1408-7321-8edc-a09851c9e656` | A1–A10 landed; claim **A11–A14** closeout from `todo` |
| Decision 0 | D0.1/D0.2 | OPEN; A7 may de-scope micro-op purity as close blocker |

**After Q0.A9:** principal fills re-issued §5 (`ACCEPT` / further amend / `REJECT`).

**Do not confuse:** board `done` on Q0.1–Q0.8 labor cards ≠ package accept.

---

## Admitted amendment todos (claim order)

| Task | Priority | Title |
|---|---|---|
| Q0.A1 | P0 | Current comparator pins + full config freeze |
| Q0.A2 | P0 | Tighten semantic-equivalence laws |
| Q0.A3 | P1 | Expand capability matrix (Mongo/SQL++ surface) |
| Q0.A4 | P1 | Full-over-wire as Q2 server-lane blocker |
| Q0.A5 | P1 | Quarantine/delete public RQB1 SDK surface |
| Q0.A6 | P1 | Durable HeapId/CollectionId on portable path |
| Q0.A7 | P2 | Decision 0 — micro-op purity not a close blocker |
| Q0.A8 | P2 | Doc consolidation + rename conflicting RQL-Q1 |
| Q0.A9 | exit | Re-issue principal accept pack after amendments |

### Closeout wave (ACCEPT_WITH_AMENDMENTS review 2)

| Task | Priority | Title |
|---|---|---|
| Q0.A10 | exit | RQB1 delete + durability freeze + accept SHA (landed) |
| Q0.A11 | P0 | CBL Full Sync for competitive write cells |
| Q0.A12 | P1 | Remove stale live RQB1 documentation |
| Q0.A13 | P2 | Freeze named query defaults in Q0 manifest |
| Q0.A14 | exit | Re-run gates + fill accept pack §5 SHA |

Pull **P0 first** (A1, A2), then P1 docs/code (A3–A6), then P2 (A7–A8), then A9.
A5/A6 may parallel A3/A4.

---

## Board honesty (Q1 cards)

Q1.1–Q1.4 are stage **`todo`** and **claimable** after Q0 ACCEPT:

| Task | Claim policy |
|---|---|
| Q1.1 schema | **Claim next** (scaffolding) |
| Q1.2 Commerce/Messaging | landed → in_review |
| Q1.3 Directory/Telemetry/Project | After Q1.1 |
| Q1.4 floors + comparator | After Q1.2–Q1.3 bulk |

---

## What labor will not do under this hold

- Self-accept Decision 0 or RQL-C1
- Start Q2/Q5/Q7 or performance claims
- Invent parallel roadmap docs that compete with CRITICAL_PATH
- Claim Gate-1 pass from Q0 freeze accept alone

---

## Resume conditions

| Condition | Next admitted labor |
|---|---|
| Principal **ACCEPT** on Q0 pack §5 (done 2026-08-07) | **Q1.1** corpus schema (then Q1.2…) |
| Principal **REJECT** | Follow rejection notes; hold until new freeze |
| Principal Decision 0 disposition only | Does **not** by itself admit Q1 |

---

## Evidence

- Architecture gate (runtime honesty, not Q0 accept):
  `bash scripts/check_query_runtime_architecture.sh`
- Q0 pack: [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md)
- Scoreboard banner: [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md)
- Amendment Feature: `019fdac4-1408-7321-8edc-a09851c9e656`

---

## Exit (hold / schedule labor)

- [x] Citable labor-hold note
- [x] Q1 claim policy restated (titles marked blocked; stage demotion blocked by host)
- [x] Q0 not self-accepted
- [x] Principal feedback scheduled as Q0.A1–Q0.A9 `todo` (Feature `019fdac4-…`)
- [ ] Principal Q0 §5 filled on post-amendment pack (human)
