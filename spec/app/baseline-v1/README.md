# APB-0 — Application baseline contract (`residiuum-application-baseline-v1`)

Status: **frozen** (APB-0 scoreboard **accept** 2026-08-01)  
Package: `APB-0`  
Authority: [MUST_ADD.md §4](../../../doc/todo/application-baseline/MUST_ADD.md) ·
[MASTER_DELIVERY_PLAN.md](../../../MASTER_DELIVERY_PLAN.md) §7 / §0.8

This directory is the **application baseline contract freeze**. It **amends**
APP-0; it does **not** invent a second competing API. Implementers of
APB-1…APB-12 **must not invent** public types, operation names, or error codes
outside these artifacts.

**Verify:**

```bash
bash scripts/verify-app-baseline-contract.sh --require-frozen
```

**Honesty:** freeze + script green = APB-0 package accept (contract only). It
does **not** mean `residiuum-application-baseline-v1` product qualification
(that is APB-12) and does not activate reserved wire ops.

## 1. Why APB-0 first (correctness)

| Question | Answer |
|---|---|
| Why not jump to query/atomics code? | Without a frozen contract, every “gotcha” becomes an ad-hoc semantic choice. |
| Why not only APP-0? | APP-0 locked a **partial** implementer surface; APB-0 must register **every** baseline operation, total outcome projections, and local/remote parity. |
| Entry deps | `CSQ-12 = accept` (met). Reconcile APP-0/APP-1 evidence (do not discard). |
| After APB-0 | Query spine APP-4 → APP-5 → APB-7; HAR identity as required. |

## 2. Target inventory (MUST_ADD §4 deliverables)

| Deliverable | Target path under `baseline-v1/` | Status now |
|---|---|---|
| Operation registry | `operations-v1.json` | **frozen** — 47 app ops × APB-1…12; wire links; reserved honest |
| Outcome catalogue | `outcomes-v1.json` | **frozen** — total lower→public; APP-0 error_mapping total |
| Public projections | `projections-v1.json` | **frozen** — PAR-001…012 local/remote parity |
| Capabilities schema | `capabilities-v1.schema.json` | **frozen** — discovery shape |
| Protocol / type freeze | `types-v1.json` | **frozen** — 10 cross-cut types + DEF-099/100 |
| Canonical fixtures | `fixtures/` | capabilities accepted/rejected + outcomes completeness |
| Verify script | `scripts/verify-app-baseline-contract.sh` | **exit 0** with `--require-frozen` |

## 3. Gap inventory — APP-0 / APP-1 → APB-0

### 3.1 Keep and amend (do not rewrite)

| Existing artifact | Role for APB-0 |
|---|---|
| [`spec/app/v1/`](../v1/README.md) | APP-0 lock: error mapping, plan/cursor vectors, residuals |
| `crates/residiuum-sdk/src/app_v1.rs` | Public Rust types that compile |
| `crates/residiuum-sdk/tests/app0_contract_lock.rs` | Contract lock tests |
| `spec/heap/operations-v1.json` | Wire op IDs (e.g. 106 active, 118 reserved) |
| `spec/heap/rpc-v1/collection_create.*`, `rql_query.*` | Staged request/response schemas |
| `spec/heap/fixtures/collection_create.*`, `rql_query.*` | Accepted/rejected goldens |
| APP-1 evidence | Op 106 active + embedded create + remote method path |
| APP-4 precursor | `predicate` + `plan_v1` + plan vectors (query spine later) |

### 3.2 Must close under APB-0 (gaps)

| Gap ID | Gap | Notes |
|---|---|---|
| APB0-G1 | **Full app operation registry** for APB-1…APB-12 surfaces | Not only 106/118; every public app op + stable id/name |
| APB0-G2 | **outcomes-v1.json** — total map lower-layer → public outcomes | Every reachable store/heap condition has a public projection |
| APB0-G3 | **projections-v1.json** — local vs remote parity rules | Same semantic result; no unexplained remote-only paths |
| APB0-G4 | **capabilities-v1.schema.json** | Discovery shape for APB-3 |
| APB0-G5 | Freeze **version, receipt, operation-id, coverage, read-view, change-checkpoint, job, continuation** | Cross-cut types used by APB-2…APB-10 |
| APB0-G6 | Bind **DEF-099 / DEF-100** recovery + coverage-aware scan types into app contract | Surface, not reimplement |
| APB0-G7 | **Legacy SDK deprecation / compatibility** rules | No silent dual APIs |
| APB0-G8 | **Compile fixtures** for complete public Rust surface | Expand beyond `app0_contract_lock` |
| APB0-G9 | No unexplained **reserved** ops that APB will need | e.g. 118 remains reserved until APP-7/APB-7 but must be registered honestly |
| APB0-G10 | **Cross-Heap composition impossible by construction** | Contract language + types |

### 3.3 Named APP residuals that APB-0 must not paper over

From [`residuals_v1.json`](../v1/residuals_v1.json):

| Residual | Handling in APB-0 |
|---|---|
| APP0-R3 owner sign-off | Human; does not block drafting APB-0 artifacts |
| APP0-R2 cursor MAC placeholders | Document as APP-6/APB-6; freeze field binding only |
| APP0-R4 op 118 reserved | Keep reserved until query package; register intent in baseline ops |
| APP1-R3 HeapClient façade | APB-1 deliverable after APB-0 |

## 4. Freeze rules

1. **Spec before behavior** — no APB implementer invents a public type.
2. **Amend APP-0** — prefer extending mappings over forking a second façade.
3. **Reserved ≠ absent** — reserved ops stay registered with null schemas until their package.
4. **No product claim** — `residiuum-application-baseline-v1` verifies only at APB-12; APB-0 freezes the contract only.
5. **Scoreboard honesty** — update `NEXT_BUILD_STATUS` APB-0 in the same change as state moves.

## 5. Full sequence (APB-0 → query → atomics)

**Normative map:**  
[doc/todo/application-baseline/APB_QUERY_ATOMICS_SEQUENCE.md](../../../doc/todo/application-baseline/APB_QUERY_ATOMICS_SEQUENCE.md)

| Phase | What |
|---|---|
| **A** | APB-0 T1…T6 contract freeze (this directory) |
| **B** | Query spine APP-4 → APP-5 → APB-7 (+ APB-1/6, HAR identity) |
| **C** | RRE-0 / ATM-0 pure risk discovery (no product claim) |
| **D** | Finish M1 HAR/APB |
| **E** | M3/M4 product RRE + Atomics |

### APB-0 labor after T1

| Order | Work | Exit of slice |
|---:|---|---|
| T2–T5 | ops / outcomes / projections / caps+types | **frozen** |
| T6 | Fixtures + verify + scoreboard accept | **done** 2026-08-01 |

## 6. Verify

```bash
bash scripts/verify-app0-contract.sh
bash scripts/verify-app-baseline-contract.sh --require-frozen
cargo test -p residiuum-sdk --test app0_contract_lock
```

Next: Phase B query spine (APP-4 → APP-5 → APB-7) per sequence map.