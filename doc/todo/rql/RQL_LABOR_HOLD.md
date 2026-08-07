# RQL critical-path labor hold (Q0.6)

Status: **2026-08-07 · labor hold active · not package accept**  
Authority: [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md) §5–§6 ·
[RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3, §11 ·
[CRITICAL_PATH.md](../../../CRITICAL_PATH.md)  
Board task: Q0.6 (`019fda97-73ef-76c3-afa1-d3466411b3d0`)

---

## Verdict

```text
Admitted implementer labor on RQL Gate-1 critical path = NONE
Blocker = principal has not filled Q0 accept pack §5
Q1 corpus implementation = FORBIDDEN until Q0 ACCEPT
Decision 0 / RQL-C1 = OPEN / FORBIDDEN (separate; not a substitute for Q0)
```

This is process honesty, not idle. Labor completed all Q0 freeze artefacts and the
principal accept pack. Further corpus/schema work would violate programme law.

---

## What is ready for principal

| Pack | Path | Labor |
|---|---|---|
| Q0 freeze (env/matrix/equiv/lanes) | `RQL_Q0_*.md` | complete → labor cards board `done` |
| Q0 principal accept pack (doc) | [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md) | pack labor board `done`; **§5 still blank** |
| Decision 0 inventory + checklist | [RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md), [RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md) | labor cards board `done`; Decision 0 still OPEN |
| Scoreboard honesty (Q0.7) | [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md) RQL-Q* rows | labor card board `done` |

**Human action required:** fill Q0 accept pack §5 (`ACCEPT` / amend / `REJECT`).

**Do not confuse:** advancing Q0.1–Q0.7 labor cards to board `done` accepted the
implementer work products. It does **not** fill §5 and does **not** admit Q1.

---

## Board honesty (Q1 cards)

Q1.1–Q1.4 remain stage **`todo`** on the Kanban board (host data-plane forbids
`todo` → `backlog` transitions). That **does not** admit claim:

1. Programme and accept pack forbid Q1 implementation until Q0 principal accept.
2. Task titles are prefixed **`[BLOCKED:Q0]`** so implementers do not claim them.
3. After principal ACCEPT, remove the prefix (or re-admit tasks) before labor starts Q1.1.

| Task | Claim policy until Q0 ACCEPT |
|---|---|
| Q1.1 schema | **Do not claim** |
| Q1.2 Commerce/Messaging | **Do not claim** |
| Q1.3 Directory/Telemetry/Project | **Do not claim** |
| Q1.4 floors + comparator | **Do not claim** |

---

## What labor will not do under this hold

- Land Q1 schema/fixtures/corpus as admitted programme progress
- Self-accept Q0, Decision 0, or RQL-C1
- Start Q2/Q5/Q7 or performance claims
- Invent parallel roadmap docs that compete with CRITICAL_PATH

---

## Resume conditions

| Condition | Next admitted labor |
|---|---|
| Principal **ACCEPT** on Q0 pack §5 | Q1.1 corpus schema (then Q1.2…) |
| Principal **ACCEPT_WITH_AMENDMENTS** | Rework named Q0 artefacts; re-issue pack; hold remains until re-accept |
| Principal **REJECT** | Follow rejection notes; hold until new freeze |
| Principal Decision 0 disposition only | Does **not** by itself admit Q1 |

---

## Evidence

- Architecture gate (runtime honesty, not Q0 accept):
  `bash scripts/check_query_runtime_architecture.sh`
- Q0 pack: [RQL_Q0_PRINCIPAL_ACCEPT.md](./RQL_Q0_PRINCIPAL_ACCEPT.md)
- Scoreboard banner: [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md)

---

## Exit (Q0.6 labor)

- [x] Citable labor-hold note
- [x] Q1 claim policy restated (titles marked blocked; stage demotion blocked by host)
- [x] Q0 not self-accepted
- [ ] Principal Q0 §5 filled (human)
