# Firm numbers — goals before optimize

Status: **principal direction locked** (labor FN-0; not package accept / not SLO)  
Card: `18f0a8f7-4ce4-4982-8f31-222a7fb1bcfd`  
Date: 2026-08-03  

## Why this exists

Without measured numbers we are talking nonsense: optimising what, to what end,
compared to what? Desire to “do better” with no boundaries is not a program.

**Rule:** no AWO/perf tuning claim without filling the tables below. Odometer
first (`ODOMETER_FIRST_COMPLETED_WRITES.md`). Honest max (`PERF_HONEST_MAX_CHARTER.md`).

## 1. What we optimise

| Field | Value |
|-------|--------|
| **Unit** | Completed end-to-end **acked puts/s** (principal “T”) |
| **Shape** | 1:1 presents (not harness `put_many(N)` sold as Adaptive) |
| **Primary bed** | PEER Mode A knobs: Scratch, 8 KiB, QD=1, ~256 MiB logical, Residiuum **Buffered** vs SQLite autocommit |
| **Smart mode** | Residiuum `awo_mode=adaptive` (and Static as explicit-ceiling control) |

## 2. Compared to what (fixed baselines)

| Baseline | Role | Known today |
|----------|------|-------------|
| **SQLite Mode A** | External peer | ≈ **29 200** acked writes/s (FN-2 APFS `/var/tmp`; Scratch history ≈10 000) |
| **Residiuum Mode A, AWO off** | Internal 1:1 | ≈ **12 600** acked writes/s (FN-2 APFS; Scratch history ≈10 000) |
| **Residiuum Mode A, Static** | Explicit batching ceiling on same bed | ≈ **2 460** (FN-2; loses to off under QD=1 collection) |
| **Residiuum Mode A, Adaptive** | Smart mode X | ≈ **2 470** (FN-2; ≈ Static, loses to off) |

Evidence: [FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md). Scratch
not mounted for FN-2 — re-run there for peer-ratio continuity.

## 3. To what end (success criteria — firm, not mystical)

After FN-2 measure:

1. Publish a **four-cell table** (SQLite A / Residiuum off / Static / Adaptive) of
   **acked puts/s** on the Mode A bed — same run recipe, disclosure attached.
2. State in one sentence: Adaptive **beats / matches / loses to** Residiuum-off
   and vs SQLite, with the integers.
3. Only then set an **optimization bound**:
   - If Adaptive ≰ Residiuum-off on that bed → fix residuals (collection /
     `select_plan` / seal) or document wall; do not tune randomly.
   - If Adaptive > Residiuum-off → next bound is Static ceiling and/or SQLite;
     squeeze under `PERF_HONEST_MAX_CHARTER` until wall is named physics/contract.

**Interim target language (not a floor until measured):** close or beat SQLite
Mode A on the same bed under Adaptive **without** weakening Buffered semantics.
If the honest max is still ~10k, that is an acceptable firm outcome.

## 4. Boundaries (what we will not do)

- No vanity beds sold as Mode A (short no-seal micro, Discard, Durable T11 smoke).
- No product default-on before principal + evidence chain.
- No “2× from T11” as Mode A smart X.
- Consistency / reopen / crash honesty unchanged (`PERF_HONEST_MAX_CHARTER`).

## 5. Task sequence (board — pull from `todo` only)

| ID | Card title | Stage intent | Done when |
|----|------------|--------------|-----------|
| **FN-0** | Firm numbers goal freeze | this card → `in_review` | This doc + pre-staged FN-1..3 |
| **FN-1** | Harness: Mode A + AWO modes runnable | labor | peer-pump `--awo-mode` **done** (2026-08-03) |
| **FN-2** | Measure four-cell Mode A odometer | labor | Table filled — [FIRM_NUMBERS_FN2_MODE_A.md](FIRM_NUMBERS_FN2_MODE_A.md); `SMART_MODE_X_MODE_A.md` updated |
| **FN-3** | Freeze optimize bound from FN-2 | `todo` | One-page: Adaptive loses to off on Mode A QD=1; next residual = collection delay / pile-up (draft in FN-2 §5) |

Related backlog (do **not** substitute for FN-2): AWO-Q3 diagnostic, AWO-Q4 sparse
bound, Q2 `select_plan` collector residual — pull after Mode A X exists unless
FN-2 shows Adaptive cannot diverge without them. **FN-2 showed Adaptive cannot
beat Residiuum-off under QD=1 collection** — residual is the delay tax, not
missing measure.

## 6. Non-claims

Not PQH/AWO package accept. Not published SLO until disclosure ladder says so.
