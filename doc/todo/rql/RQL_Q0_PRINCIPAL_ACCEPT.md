# RQL-Q0 — Principal accept pack (re-issue after amendments)

Status: **Q0.A10 closeout complete · awaiting principal package accept**  
Package: RQL-Q0 Target and profile freeze  
Board Features: first freeze `019fda4b-d981-7980-a283-549a7312f2a9` · amendment `019fdac4-1408-7321-8edc-a09851c9e656`  
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3 exit · §11  
Labor re-issue date: 2026-08-07 (Q0.A9 + **Q0.A10** closeout)  
Tree baseline at re-issue: git `fd3c8b1db8da456c6293220f92c87daaa259dc7c` (A10 content) · VERSION **0.2.2** · accept on clean tip including this pack note · **git_dirty=false**

**This file does not accept Q0.** Only the principal fills §5.  
**Q1 corpus labor must not start until §5 records ACCEPT.**  
**Decision 0 / RQL-C1 are out of scope for this pack** (see [RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md)).

Doc map: [RQL_Q0_DOC_INDEX.md](./RQL_Q0_DOC_INDEX.md).

---

## 0. Principal review history

| Date | Disposition | Note |
|---|---|---|
| 2026-08-07 | First freeze labor complete | Pack v1; §5 blank |
| 2026-08-07 | **Do not accept yet** | Principal review: material holes in qualification foundation |
| 2026-08-07 | Amendment package A1–A8 labor | This re-issue (A9) incorporates required amendments |

---

## 1. What Q0 is (and is not)

| Is | Is not |
|---|---|
| Freeze of Gate-1 competitive **target and profile** before corpus/harness | Gate-1 pass |
| Env/engine pins, Tier A/B/C classes, equivalence defs, lanes, refusals | Implementation of all Tier A blockers |
| Prerequisite for Q1 corpus | Q1 schema or fixtures |
| Reviewable without chat history | Package accept by labor or board `in_review` |

---

## 2. Artefact index (must accept or amend together)

| # | Artefact | Deliverable | Labor status |
|---|---|---|---|
| 1 | [RQL_Q0_ENV_MANIFEST.md](./RQL_Q0_ENV_MANIFEST.md) | Pins + full comparator config | **A1 amended**: Mongo **8.2.12**, driver `mongodb` **3.8.0**, CBL **4.1.0**, WC/RC/pool/TLS/auth frozen |
| 2 | [RQL_Q0_CAPABILITY_MATRIX.md](./RQL_Q0_CAPABILITY_MATRIX.md) | Tier A/B/C class + owner | **A3 amended**: DISTINCT Tier A; IN/string/array/arith/date/COUNT DISTINCT/pipeline rows |
| 3 | [RQL_Q0_RESULT_EQUIVALENCE.md](./RQL_Q0_RESULT_EQUIVALENCE.md) | Equivalent-result laws | **A2 amended**: str/int/mn/agg/arr/cur laws; ban post-hoc exclusion |
| 4 | [RQL_Q0_LANES_EXCLUSIONS.md](./RQL_Q0_LANES_EXCLUSIONS.md) | Lanes + exclusions + refusals | **A4 amended**: **Q2-BLOCK-FULL-WIRE** (Full not lane-S pass) |
| 5 | **This file** | Principal accept pack | **A9 re-issue** |

Related honesty (not Q0 freeze exit criteria):

- [RQL_Q0_DOC_INDEX.md](./RQL_Q0_DOC_INDEX.md) — normative vs process (A8)
- [RQL_LABOR_HOLD.md](./RQL_LABOR_HOLD.md) — Q1 claim policy until §5
- [RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md) / [RQL_D0_RESIDUAL_INVENTORY.md](./RQL_D0_RESIDUAL_INVENTORY.md) — Decision 0 (**A7**: micro-op purity not close blocker)
- Product code: **RQB1 fully removed** (A10; supersedes A5 quarantine claim); dialect **store-scoped durable ids** (A6; not Heap catalog); primary lane E = **`CollectionClient::rql`**

---

## 3. Freeze summary (rapid principal scan)

### 3.1 Engines and lanes

| Lane | Residiuum side | Comparator pin |
|---|---|---|
| **E** Embedded | SDK / store in-process | Couchbase Lite **4.1.0** (C binding primary; core recorded) |
| **S** Local client/server | `residiuum serve` + client (loopback) | MongoDB Community **8.2.12** + Rust driver **3.8.0**; `w:1,j:true`; read `local` |

Residiuum package pin: **0.2.2**, MSRV **1.88.0**, evidence `git_sha` = full HEAD (`git_dirty=false`).
Residiuum writes: **`DurabilityMode::Durable`** ack before next step; product API **`CollectionClient::rql`** (not legacy-flat).
CBL: autocommit single-doc saves; **sync off**; query compile outside timed windows.  
Do not conflate lanes in portfolio scoring.

### 3.2 Capability law

- Every Tier A row has exactly one class; no Tier A `TBD`.
- **Blockers** remain honest (aggregates, computed proj, UNWIND, regex, …).
- **DISTINCT** is Tier A (promoted).
- Full enrich/within/full project: Tier A semantics, but **not lane-S competitive** until Q2-BLOCK-FULL-WIRE.

### 3.3 Equivalence law

- Compare values, keys, multiplicity, order, continuation, coverage, refusal — never row-count alone.
- Pre-declared freezes only; **no post-hoc `type_incomparable`**.
- Default: binary strings; i64; three-way missing/null; no inter-page writes.

### 3.4 Product runtime honesty (adjacent to Q0, not substitute)

- Public execute form: **QVM1 only** (RQB1 not public product encode/execute).
- Decision 0 close test: one product QVM path — **not** pure stack micro-op rewrite (A7).
- DX portable dialect: store-scoped durable host ids (A6); Heap catalog path = `CollectionClient`.

---

## 4. Proposed state moves **after** principal ACCEPT

Labor must not apply these as if already accepted.

| Surface | Proposed move |
|---|---|
| Scoreboard **RQL-Q0** | `active` (amending) → **accept** of freeze only (not Gate-1) |
| Kanban amendment tasks Q0.A* | Principal may advance to `done` after review |
| Kanban Q1.* | Remove `[BLOCKED:Q0]` claim ban; admit Q1.1 |
| Decision 0 Feature | **Unchanged** — still OPEN for principal D0 disposition |
| Process docs (hold) | Eligible to archive per [RQL_Q0_DOC_INDEX.md](./RQL_Q0_DOC_INDEX.md) |

**Explicit non-moves**

- Do not accept RQL-C1 / close Decision 0 via this pack alone.
- Do not start Q5 performance claims.
- Do not score Full Tier-A as lane-S wins until Q2-BLOCK-FULL-WIRE closes.

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

Confirm DISTINCT stays Tier A:   YES | NO
Confirm Q2-BLOCK-FULL-WIRE (Full not lane-S until wire):   YES | NO
Confirm Decision 0 close does not require pure micro-op rewrite:   YES | NO

Scoreboard move authorised as in §4:   YES | NO
Q1 corpus labor admitted after this accept: YES | NO
```

Labor must leave this block blank.

---

## 6. Implementer constraints until §5 is filled

1. **Do not** implement Q1 fixture bulk or claim Q1 package progress as admitted programme work.
2. **Do not** treat board `in_review` on Q0.A* as package accept.
3. **Do not** weaken Tier A by inventing green matrix classes without SPEC + principal.
4. Decision 0 residual docs may proceed; they do not substitute for Q0 accept.
5. See [RQL_LABOR_HOLD.md](./RQL_LABOR_HOLD.md).

---

## 7. Exit (Q0.A9 + Q0.A10 labor)

- [x] Re-issue after A1–A8 amendments (A9)
- [x] A10: RQB1 fully removed from SDK; arch gate forbids
- [x] A10: exact Residiuum/CBL durability + product APIs frozen
- [x] A10: clean tree SHA + `git_dirty=false` required for accept
- [x] All four freeze artefacts + code honesty linked
- [x] Scoreboard / Q1 unlock proposed (not applied as accept)
- [x] §5 left blank for principal
- [ ] Principal §5 filled (human)

---

## One-line verdict

```text
Q0 A10 closeout   = complete for principal re-review
Q0 package        = NOT accepted (principal only) — ready for §5 ACCEPT if review ok
Q1                = BLOCKED until §5 ACCEPT
Decision 0        = OPEN (micro-op purity not D0 close bar)
RQB1              = removed from SDK
```
