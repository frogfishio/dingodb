# How to use the Formal Assurance Spine (FAS)

Audience: implementers and reviewers who need the **big picture** and the
**day-to-day commands** after FAS-0…FAS-4 landed.

Normative depth stays in `doc/todo/formal-assurance/*`. Living package state:
[NEXT_BUILD_STATUS.md](../doc/wip/status/NEXT_BUILD_STATUS.md).

---

## 1. Big picture (what FAS is for)

Residiuum does **not** claim “the whole database is formally verified.”

FAS is a **claim discipline**: every product claim that sounds like safety,
consistency, or security must eventually name:

1. a **theorem ID** (what is claimed),
2. **assumptions / TCB** (what is trusted, not proved),
3. **proof status** (abstract vs machine-checked vs connected to Rust),
4. **Rust entrypoints** (if behavior is claimed for production code),
5. **physical evidence** (CSQ / crash / mutation — when the claim is about the
   real store).

```text
  Product claim  ──►  Registry (FAS-0)     “is this claim allowed / versioned?”
                         │
                         ▼
  Abstract math  ──►  Lean kernel (FAS-2)  “what do State/Observe/Step mean?”
                         │
                         ▼
  Tooling pin    ──►  FAS-1                 “can we re-run the same proofs?”
                         │
                         ▼
  Code link      ──►  Refinement (FAS-3)   “which Rust paths refine the model?”
                         │
                         ▼
  Consistency    ──►  CON family (FAS-4)   “no fabricated values, damage honesty, …”
                         │
                         ▼
  Security …     ──►  FAS-5+               (next) heap noninterference, etc.
                         │
                         ▼
  Release        ──►  FAS-9                public proof bundle (not yet)
```

**Where it sits in the product plan**

| Lane | Role | Relation to M2 |
|------|------|----------------|
| **C0 / CSQ** | Physical store qualification (A2 green) | Required floor |
| **PQH** | Performance measurement honesty | Parallel |
| **APB / HAR** | Application API + Heap journeys | **M1 / M2 critical path** |
| **FAS** | Mathematical claim spine (`P1-TRUST`) | **Does not replace APB**; strengthens claims beside the path |

Use FAS when you **add or change a claim**. Use CSQ/APB when you **ship store or
app behavior**. Both can run; order still comes from `MASTER_DELIVERY_PLAN.md`.

---

## 1b. Overarching picture: is FAS part of CI?

**Short answer:** partly. FAS is a **package gate stack** (run to accept or
re-verify a formal package). CI today runs **some related jobs** and the
**cheap registry gate**; it does **not** yet re-run the full FAS-1…FAS-4 stack
on every PR.

```text
  Every PR (quality job)
    cargo fmt/clippy/test/doc
    verify-delivery-status          ← scoreboard honesty
    check-formal-registry (FAS-0)   ← claim catalogue closed / fail-closed
    …

  Separate PR jobs (already present)
    kani-heap                       ← Heap pure lemmas (FAS precursor / FAS-3 peer)
    verus-heap                      ← Verus pure_kernel (FAS-3 connection evidence)

  Package / implementer / future heavy CI (local scripts; not all PR-default)
    check-formal-toolchain  (FAS-1)  needs pinned Lean/Verus/Kani/TLC
    check-formal-foundation (FAS-2)  needs lake/Lean
    check-formal-refinement (FAS-3)  needs Lean + Verus (+ path checks)
    check-formal-consistency(FAS-4)  needs Lean + CSQ evidence under target/
```

| Layer | Default PR CI? | Why |
|-------|----------------|-----|
| FAS-0 registry | **Yes** (`check-formal-registry.sh` on quality job) | Bash/Python only; fail-closed catalogue |
| Heap Kani / Verus | **Yes** (dedicated jobs) | Precursors wired for Gate H6; feed FAS-3 honesty |
| FAS-1 toolchain smokes | **No** (residual) | Multi-tool install; run on formal changes / accept |
| FAS-2…FAS-4 gates | **No** (residual) | Lean/Verus cost; FAS-4 also expects CSQ reports under `target/` (gitignored) |
| CSQ A2 | **Not full A2 every PR** | Heavy; release / explicit verify |

**Mental model**

- **CI** keeps the tree from shipping **broken claim governance** (FAS-0) and
  from silently dropping **Heap pure proofs** (Kani/Verus jobs).
- **FAS package scripts** are the bar when you **change formal artifacts** or
  claim a package **accept** — same as CSQ’s verify script for A2, not the same
  as `cargo test` on every crate.
- Full “formal spine green on every PR” is **FAS-1 CI residual**, not required
  for M2 product path; product critical path remains PQH → APB/HAR → M2.

When you touch `formal/**` or claim FAS status, run the relevant
`scripts/check-formal-*.sh` locally (or add a heavy job later) before review.

### Pre-release briefing (HTML for humans)

You do **not** need a separate product for this — but you **should** have one
orchestrator that **chains gates honestly** and writes a readable report.
That is:

```bash
bash scripts/release-briefing.sh                  # snapshot: cheap gates + collect
bash scripts/release-briefing.sh --profile formal # + FAS-1…4
bash scripts/release-briefing.sh --profile pre-release  # + CSQ A2
open target/release-briefing/LATEST.html          # or xdg-open
```

| Profile | What it runs | Typical use |
|---------|----------------|-------------|
| `snapshot` | delivery-status, identity, FAS-0; ingests existing FAS/CSQ JSON | quick status before a meeting |
| `formal` | snapshot + FAS-1…4 package scripts | formal spine re-verify |
| `pre-release` | formal + CSQ A2 verify | before tagging / release review |
| (manual) | `./scripts/quality.sh` · `./scripts/nightly.sh` · PQH qual | full CI mirror / soak — **not** auto-chained (too long) |

Output: `target/release-briefing/LATEST.html` + `LATEST.json` (also timestamped
copies). **`not_run` is never treated as pass**; overall fails if any executed
gate fails. This is a **briefing**, not a substitute for scoreboard `accept`.

---

## 2. What each accepted package gives you

| Package | You get | You do **not** get (yet) |
|---------|---------|---------------------------|
| **FAS-0** | Closed theorem/assumption/operation catalogues under `formal/registry/` | Proof that any theorem is true |
| **FAS-1** | Pinned Lean/Verus/Kani/TLC + smoke scripts | Full CI matrix / TLAPS |
| **FAS-2** | One abstract universe: `State`, `Observation`, `Input`, `WellFormed`, `Observe` in Lean | Production store refinement |
| **FAS-3** | Entrypoint census, type map, **one** Rust-connected vertical slice (heap authority binding) | Full put/get forward simulation |
| **FAS-4** | Eight `FAS-CON-*` obligations as Lean theorems + CSQ links + negatives | Full `physically_qualified` consistency profile |

Honest labels matter: FAS-4 profile is **`mvp_abstract_plus_csq_links`**, not
“Residiuum is consistency-verified end-to-end.”

---

## 3. Day-to-day use (operators)

### 3.1 Bootstrap tools (once / when pins change)

```bash
bash scripts/setup-formal-tools.sh --locked
```

### 3.2 Re-run gates (CI-shaped local bar)

```bash
bash scripts/check-formal-registry.sh      # FAS-0  → target/formal-assurance/fas0-registry-report.json
bash scripts/check-formal-toolchain.sh     # FAS-1  → fas1-toolchain-report.json
bash scripts/check-formal-foundation.sh    # FAS-2  → fas2-foundation-report.json
bash scripts/check-formal-refinement.sh    # FAS-3  → fas3-refinement-report.json
bash scripts/check-formal-consistency.sh   # FAS-4  → fas4-consistency-report.json
```

Each script **fails closed** if a mandatory piece is missing or a negative
control would pass wrongly.

### 3.3 Work in Lean (abstract model)

```bash
lake --dir formal/lean build
```

Modules under `formal/lean/Residiuum/`:

| Module | Use when |
|--------|----------|
| `Observation` / `State` / `WellFormed` | Changing abstract vocabulary |
| `Operations` / `Observe` | New abstract op or observation law |
| `Refinement` | New α / vertical slice theorem |
| `Consistency` | Strengthening or adding a `FAS-CON-*` obligation |

**Rule:** if prose and Lean disagree, the profile fails — fix one of them
(`FORMAL_KERNEL_MODEL_CONTRACT.md`).

### 3.4 Work on a Rust-connected claim (refinement)

1. Pick theorem ID from `formal/registry/theorems-v1.json`.
2. Add or extend a bridge under `formal/refinement/bridges/`.
3. Point at **real** production paths (not `examples/`).
4. Name Lean symbols + Verus/Kani if used.
5. Ensure `check-formal-refinement.sh` still fails on rename (negative fixtures).

Template slice already in tree:
`formal/refinement/bridges/FAS-BRIDGE-AUTHORITY-BINDING-001.json`
(heap `authority_binding_holds` ↔ Verus `pure_kernel` ↔ Lean `Refinement`).

### 3.5 Work on consistency claims

1. Theorem ID is one of the eight `FAS-CON-*` in the registry.
2. Lean statement/proof lives in `Residiuum.Consistency`.
3. Connection row in `formal/consistency/theorem-connections-v1.json`
   (Lean symbols, CSQ links, Rust entrypoints, negative control id).
4. Negative mutant in `formal/consistency/negative-controls-v1.json`.
5. Gate: `check-formal-consistency.sh`.

### 3.6 Register a new claim (before marketing language)

1. Add theorem / operation / assumption rows under `formal/registry/`
   (schemas enforce structure).
2. Run `check-formal-registry.sh` — must stay green.
3. Do **not** set `machine_proved` without source/result hashes and real proofs.
4. Prefer status honesty: `proposed` / `specified` / connected later.

### 3.7 Physical evidence (still CSQ’s job)

Consistency **links** to CSQ; it does not replace it:

```bash
bash scripts/residiuum-verify-core-storage.sh --require-a2-pass
# target/csq-evidence/a2-evaluation.json  (a2_pass)
```

FAS-4 connections reference those paths. If A2 is red, you may still have Lean
theorems — you must **not** claim physical qualification.

---

## 4. How implementers should *think* when coding

| Situation | What to do |
|-----------|------------|
| New observation shape (e.g. treat “not found” as absence) | Check FAS-2 Observe + forbidden collapse; amend contract if needed |
| “We formally verified durability” | Need CON durable-ack + FS assumption + CSQ persistence evidence — not a unit test alone |
| Heap isolation claim | FAS-3 slice pattern + FAS-5 family (not started); use existing Verus pure_kernel / pure_proofs honestly |
| Marketing / docs wording | Registry claim linter forbids vague “formally verified database” |
| Unrelated APB feature | Ship APB; only open a FAS change if the feature **introduces a theorem-bearing claim** |

---

## 5. What is still open (so you don’t over-use FAS)

- **FAS-5** security family (heap noninterference bundle as product profile)
- Stronger store **put/get** refinement (FAS-3 residual)
- Full **physically_qualified** consistency profile (FAS-4 residual)
- **FAS-6…8** Atomics/cluster formal (feature-gated)
- **FAS-9** public proof bundle / CLI for customers
- Product critical path remains **PQH principal accept → APB/HAR → M2**

---

## 6. One-screen mental model

```text
  CSQ  =  “the bytes on disk survive and match the store model”
  FAS  =  “we are not allowed to say more than we can name, prove, and link”

  Use CSQ to qualify the kernel.
  Use FAS to govern what we may claim about it (and later about Heap/Atomics).
  Use APB/HAR to make a product people can adopt.
```

Re-run the five `check-formal-*.sh` scripts when you change formal artifacts;
treat green reports under `target/formal-assurance/` as the package bar, not as
a blanket product certificate.