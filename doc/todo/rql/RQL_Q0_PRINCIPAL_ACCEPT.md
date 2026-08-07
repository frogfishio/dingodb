# RQL-Q0 — Principal accept pack

Status: **labor complete · awaiting principal package accept**  
Package: RQL-Q0 Target and profile freeze  
Board: Feature `019fda4b-d981-7980-a283-549a7312f2a9` · task Q0.5  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3 exit · §11  
Labor date: 2026-08-07  
Tree baseline at pack write: git `55841993ef9f531c8c28bc5ca978f91a563aff3c` · VERSION **0.2.2**

**This file does not accept Q0.** Only the principal fills §5.  
**Q1 corpus labor must not start until §5 records accept.**  
**Decision 0 / RQL-C1 are out of scope for this pack** (see [RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md)).

---

## 1. What Q0 is (and is not)

| Is | Is not |
|---|---|
| Freeze of Gate-1 competitive **target and profile** before corpus/harness | Gate-1 pass |
| Env/engine pins, Tier A/B/C classes, equivalence defs, lanes, refusals | Implementation of missing Tier A semantics |
| Prerequisite for Q1 corpus | Q1 schema or fixtures |
| Reviewable without chat history | Package accept by labor or board `in_review` |

Plan shape was accepted 2026-08-07 (Features/tasks). **Package exit for Q0 is separate.**

---

## 2. Artefact index (must accept or amend together)

| # | Artefact | Deliverable | Labor status |
|---|---|---|---|
| 1 | [RQL_Q0_ENV_MANIFEST.md](./RQL_Q0_ENV_MANIFEST.md) | Version-pinned env + engines + fingerprint | labor complete |
| 2 | [RQL_Q0_CAPABILITY_MATRIX.md](./RQL_Q0_CAPABILITY_MATRIX.md) | Tier A/B/C class + owner + residual | labor complete |
| 3 | [RQL_Q0_RESULT_EQUIVALENCE.md](./RQL_Q0_RESULT_EQUIVALENCE.md) | Per-family equivalent-result defs | labor complete |
| 4 | [RQL_Q0_LANES_EXCLUSIONS.md](./RQL_Q0_LANES_EXCLUSIONS.md) | Lanes E/S + exclusions + refusal codes | labor complete |
| 5 | **This file** | Principal accept pack + scoreboard propose | labor complete |

Related (not Q0 exit criteria):

- [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) — strategy (plan accepted)
- [RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md) / [RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md) — Decision 0 honesty (OPEN)
- [RQL0_GAP_LEDGER.md](./RQL0_GAP_LEDGER.md) — construct gap ledger (not freeze profile)

---

## 3. Freeze summary (for rapid principal scan)

### 3.1 Engines and lanes

| Lane | Residiuum side | Comparator pin |
|---|---|---|
| **E** Embedded | SDK / store in-process | Couchbase Lite **3.2.1** embedded |
| **S** Local client/server | `residiuum serve` + client (loopback) | MongoDB Community **8.0.4** localhost |

Residiuum package pin: **0.2.2**, MSRV **1.88.0**, evidence `git_sha` = full HEAD.  
Do not conflate lanes in portfolio scoring (see lanes file).

### 3.2 Capability law

- Every Tier A row has exactly one class: `exact` | `document-native-equivalent` | `deliberate-exclusion` | `blocker`.
- No Tier A semantic is `TBD`.
- **Blockers** (honest, expected): e.g. aggregates, computed/conditional projection — still **Tier A** for Gate-1; SPEC amend + Q2 implement; not silent demotion to Tier C without principal.
- Impl state (`implemented` / `partial` / `absent`) is honesty only — not Gate-1 green.

### 3.3 Equivalence law

- Compare values, keys, multiplicity, order, continuation, coverage/validity, refusal — never row-count alone.
- Document-native expression differences allowed only when corpus marks them; results/coverage must match.

### 3.4 Exclusion / refusal law

- Tier C and deliberate exclusions require stable refusal codes (not empty complete pages).
- Full language on Core/op 118 remains refuse-on-wire (not Q0 demotion of Full).

---

## 4. Proposed state moves **after** principal accept

Labor must not apply these as if already accepted.

| Surface | Proposed move |
|---|---|
| Programme §11 scoreboard row **RQL-Q0 Target freeze** | `todo` / labor `in_review` → **principal-accepted freeze** (wording at principal choice: `accepted` / `frozen`) |
| [NEXT_BUILD_STATUS.md](../../wip/status/NEXT_BUILD_STATUS.md) | Add or update an RQL-Q0 row: state reflects **accept** of freeze only — not Gate-1, not RQL-C1 |
| Kanban Q0.1–Q0.5 tasks | Principal may advance to `done` after review |
| Kanban Q1.* tasks | Remain blocked for **implementation** until Q0 accept; after accept, Q1.1 labor is admitted |
| Decision 0 Feature | **Unchanged** — still OPEN; blocks Q2 one-runtime *exit claim* |

**Explicit non-moves**

- Do not accept RQL-C1 / close Decision 0 via this pack.
- Do not mark APB/APP packages accept.
- Do not start Q5 performance claims.

---

## 5. Principal decision block (human only)

```text
Q0 package accept:     ACCEPT | ACCEPT_WITH_AMENDMENTS | REJECT
Date / principal:      _______________________________________________
Git SHA reviewed:      _______________________________________________

Amendments required (if any):
  _______________________________________________________________
  _______________________________________________________________

Confirm aggregates + computed/conditional projection stay Tier A blockers
(SPEC amend in Q2), not demoted to Tier C:   YES | NO (explain)
  _______________________________________________________________

Scoreboard move authorised as in §4:   YES | NO
Q1 corpus labor admitted after this accept: YES | NO
```

Labor must leave this block blank.

---

## 6. Implementer constraints until §5 is filled

1. **Do not** implement Q1 fixture bulk or claim Q1 package progress as admitted programme work.
2. **Do not** treat board `in_review` on Q0.1–Q0.5 as package accept.
3. **Do not** weaken Tier A by inventing green matrix classes without SPEC + principal.
4. Decision 0 residual docs may proceed in parallel; they do not substitute for Q0 accept.
5. See [RQL_LABOR_HOLD.md](./RQL_LABOR_HOLD.md) for the active hold and Q1 `[BLOCKED:Q0]` claim policy.

If principal rejects or amends: rework named artefacts; re-issue this pack; keep Q1 blocked.

---

## 7. Exit (Q0.5 labor)

- [x] Single citable principal accept pack
- [x] All four freeze artefacts linked
- [x] Scoreboard / programme move proposed (not applied as accept)
- [x] Q1 start forbidden until principal accept restated
- [ ] Principal §5 filled (human)

---

## One-line verdict

```text
Q0 labor pack = complete for principal review
Q0 package    = NOT accepted (principal only)
Q1            = BLOCKED until §5 ACCEPT
Decision 0    = OPEN (separate pack)
Labor hold    = RQL_LABOR_HOLD.md (Q0.6)
```
