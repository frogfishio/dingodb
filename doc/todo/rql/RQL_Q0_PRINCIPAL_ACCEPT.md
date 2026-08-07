# RQL-Q0 — Principal accept pack (re-issue after amendments)

Status: **Q0 package ACCEPT (principal) · freeze accepted · Q1 admitted**
Package: RQL-Q0 Target and profile freeze
Board Features: first freeze `019fda4b-d981-7980-a283-549a7312f2a9` · amendment `019fdac4-1408-7321-8edc-a09851c9e656`
Authority: [RQL_QUERY_QUALIFICATION_PROGRAM.md](./RQL_QUERY_QUALIFICATION_PROGRAM.md) §3 exit · §11
Labor re-issue date: 2026-08-07 (Q0.A9–A14 closeout wave)
Tree baseline: **accepted clean tip** `e1f5c670a99dc54da477c531c83bca4985199a42` (`git_dirty=false`) · parent A10 `e764d218af38f72058e68813a567ae25cd259331` · VERSION **0.2.2**

**§5 records principal ACCEPT (2026-08-07).** This is freeze accept only — **not** Gate-1.
**Q1 corpus labor is admitted** (claim Q1.1 next). Decision 0 / RQL-C1 unchanged (OPEN / forbidden).
**Decision 0 / RQL-C1 are out of scope for this pack** (see [RQL_D0_CLOSE_READINESS.md](./RQL_D0_CLOSE_READINESS.md)).

Doc map: [RQL_Q0_DOC_INDEX.md](./RQL_Q0_DOC_INDEX.md).

---

## 0. Principal review history

| Date | Disposition | Note |
|---|---|---|
| 2026-08-07 | First freeze labor complete | Pack v1; §5 blank |
| 2026-08-07 | **Do not accept yet** | Principal review: material holes in qualification foundation |
| 2026-08-07 | Amendment package A1–A8 labor | This re-issue (A9) incorporates required amendments |
| 2026-08-07 | **ACCEPT_WITH_AMENDMENTS** | Principal: A1–A10 substance ok; hold for CBL Full Sync + RQB1 doc cleanup + named query defaults |
| 2026-08-07 | **ACCEPT** | Principal: Q0 package accepted after A11–A14 closeout; clean tip `e1f5c670…`; Q1 admitted |

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
| 1 | [RQL_Q0_ENV_MANIFEST.md](./RQL_Q0_ENV_MANIFEST.md) | Pins + full comparator config | **A1**: Mongo **8.2.12**, driver `mongodb` **3.8.0**, CBL **4.1.0**, WC/RC/pool/TLS/auth. **A11**: CBL `fullSync=true` + fingerprint `cbl_full_sync=true`; `native_default_non_equivalent` excluded from competitive aggregates |
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
CBL: autocommit single-doc saves; **replication/Sync Gateway off**; **Full Sync on** (`fullSync=true`, fingerprint `cbl_full_sync=true`); query compile outside timed windows.
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

**Applied 2026-08-07** after principal ACCEPT (Q0.A15 labor recording).

| Surface | Proposed move |
|---|---|
| Scoreboard **RQL-Q0** | **accept** of freeze only (not Gate-1) — applied |
| Kanban amendment tasks Q0.A* | Principal may advance to `done` after review |
| Kanban Q1.* | Remove `[BLOCKED:Q0]` claim ban; admit Q1.1 — applied |
| Decision 0 Feature | **Unchanged** — still OPEN for principal D0 disposition |
| Process docs (hold) | Eligible to archive per [RQL_Q0_DOC_INDEX.md](./RQL_Q0_DOC_INDEX.md) |

**Explicit non-moves**

- Do not accept RQL-C1 / close Decision 0 via this pack alone.
- Do not start Q5 performance claims.
- Do not score Full Tier-A as lane-S wins until Q2-BLOCK-FULL-WIRE closes.

### 4.1 Labor tip evidence (A14 — **not** package accept)

| Item | Value |
|---|---|
| Pre-closeout reviewed tip (A10) | `e764d218af38f72058e68813a567ae25cd259331` |
| A11 commit (CBL Full Sync docs) | `98342415d1275d399de0d2fd29b7220cf8b5aad5` |
| A12–A14 content | RQB1 live-doc cleanup + named query defaults + this evidence — **commit before §5** if still dirty |
| Architecture gate | `scripts/check_query_runtime_architecture.sh` → **OK** |
| Delivery-status | `scripts/verify-delivery-status.sh` → **OK (114 packages)** |
| QVM unit tests | `cargo test -p residiuum-sdk --lib query_bytecode_v1` → **56/56** |
| `git diff --check` | **clean** |

Principal: fill §5 on the **clean** post-closeout SHA (after any remaining A12–A14 commit). Parent/content baseline approach remains valid from A10 tip above.

---

## 5. Principal decision block (human only)

```text
Q0 package accept:     ACCEPT
Date / principal:      2026-08-07 / principal (package award: Q0 has been accepted)
Git SHA reviewed:      e1f5c670a99dc54da477c531c83bca4985199a42

Amendments required (if any):
  (none — A11–A14 closeout complete)

Confirm aggregates + computed/conditional projection stay Tier A blockers
(SPEC amend in Q2), not demoted to Tier C:   YES

Confirm DISTINCT stays Tier A:   YES
Confirm Q2-BLOCK-FULL-WIRE (Full not lane-S until wire):   YES
Confirm Decision 0 close does not require pure micro-op rewrite:   YES

Scoreboard move authorised as in §4:   YES
Q1 corpus labor admitted after this accept: YES
```

Filled by principal direction 2026-08-07. Freeze accept only — not Gate-1 / not Decision 0 close.

---

## 6. Implementer constraints until §5 is filled

**Superseded:** §5 is ACCEPT. Constraints now:

1. **Q1 corpus labor is admitted** — claim Q1.1 next (schema/scaffolding first).
2. **Do not** claim Gate-1, Q5 performance, or RQL-C1 / Decision 0 close via this pack.
3. **Do not** weaken Tier A by inventing green matrix classes without SPEC + principal.
4. Decision 0 residual docs may proceed independently; still OPEN.
5. See [RQL_LABOR_HOLD.md](./RQL_LABOR_HOLD.md) (hold lifted for Q1; Q2+ still gated).

---

## 7. Exit (Q0.A9 + Q0.A10 labor)

- [x] Re-issue after A1–A8 amendments (A9)
- [x] A10: RQB1 fully removed from SDK; arch gate forbids
- [x] A10: exact Residiuum/CBL durability + product APIs frozen
- [x] A10: clean tree SHA + `git_dirty=false` required for accept
- [x] All four freeze artefacts + code honesty linked
- [x] Scoreboard / Q1 unlock proposed (not applied as accept)
- [x] §5 left blank for principal
- [x] Principal §5 filled (ACCEPT 2026-08-07 · SHA e1f5c670…)

### Q0.A11 labor (ACCEPT_WITH_AMENDMENTS wave)

- [x] A11: CBL Full Sync required for competitive cells; fingerprint `cbl_full_sync`
- [x] A12: stale live RQB1 documentation cleanup (QVM-only truth; QUERY_ISA_V1 retired)
- [x] A13: named query defaults freeze (`Available` / `Complete` / page size 64)
- [x] A14: gates re-run green; labor tip evidence in §4.1; §5 decision left blank for principal
- [x] A15: record principal ACCEPT + unlock Q1 (this recording)

---

## One-line verdict

```text
Q0 A11–A14 closeout = complete
Q0 package          = ACCEPT (freeze only) · SHA e1f5c670a99dc54da477c531c83bca4985199a42
Q1                = ADMITTED (claim Q1.1)
Decision 0        = OPEN (micro-op purity not D0 close bar)
RQB1              = removed from SDK
```
