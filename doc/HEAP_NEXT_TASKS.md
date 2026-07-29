# Heap program — next tasks

Status: living queue (2026-07-30)  
Audience: humans + agents continuing the HEAP_SPEC / HP-010 program  
Companions: [HEAP_SPEC.md](../HEAP_SPEC.md) (contract + progress),
[spec/heap/qualification/hp010-matrix-v1.json](../spec/heap/qualification/hp010-matrix-v1.json),
[HEAP_COMPLETE_PATH_REVIEW.md](HEAP_COMPLETE_PATH_REVIEW.md),
[PRIME_TIME_PLAN.md](PRIME_TIME_PLAN.md),
[WORK_HORIZON.md](WORK_HORIZON.md)

## North star

Close **single-node** `dingo-heap-v1` **honestly** (`qualified=true` only when the
matrix says so). Do **not** open HP-011/012 (cluster) until HP-010 is Accept or
deliberately deferred with Level-1 product language.

Today: **`qualified=false`**, Level 1 claim only. **H3 Accept.** H0–H2/H4–H5/H6 partial.

## Recommended order (anti-diminishing-returns)

Work top-down. Prefer gate closers over more polish on already-landed cuts.

### Wave A — Product path honesty (CPR-001 / H0–H1)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| A1 | Migrate high-traffic demos/tests to `open_deployment` + `HeapCap` | Unblocks default flip | Named demos + at least core SDK tests run heap-bound |
| A2 | Flip **`legacy-flat-sdk` default off** (keep feature for Stages 3–9) | Closes CPR-001 residual for claim honesty | `--no-default-features` is default profile; matrix H0/H1 open bullets updated |
| A3 | Flip or document **`legacy-raw-store`** default consistently with A2 | HP-003 cutover | Architecture script + dependents green |

### Wave B — §32.4 surface completeness (H1–H2)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| B1 | Activate **find** (116) + minimal filter args | Query is the everyday path | **Done (2026-07-30)** — schemas/fixtures + scan+Filter dispatch + `RemoteHeap::find` + `connect_heap_find_filter` |
| B2 | Activate **history** (117) | Spec + DX parity | **Done (2026-07-30)** — schemas/fixtures + SubjectV2 history + `RemoteHeap::history` + `connect_heap_history` (rights first-cut = Read) |
| B3 | Activate **index_list / index_create / index_drop / index_rebuild** (130–133) | Find acceleration | Same §32.4 bar |
| B4 | Activate **collection_create** (106) (needs rights story) | Provisioning without offline catalog | Ceremony/rights + Accept |
| B5 | Remaining reserved ops only as needed for drills (lifecycle RPC, export, …) | Avoid activation thrash | Matrix-driven only |

### Wave C — Ops residuals (H4–H5)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| C1 | Live filesystem multi-tier media wipe drill | H4 open | Matrix evidence path + Accept test |
| C2 | HSM / provider data-key adapter scaffold or explicit out-of-scope | H4 open | Adapter or matrix note disposing residual |
| C3 | Mixed-heap salvage classification drill | H4 open | Matrixed drill Accept |
| C4 | Broader destructive crash-matrix cells (beyond peer lifecycle) | H5 open | crash_matrix cells + CI subset |

### Wave D — Gate H6 claim (blocks `qualified=true`)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| D1 | Connect **Verus or Kani** to pure decide / isolation Inv in CI | CPR-004 | `VERUS_PROOFS_CONNECTED=true` (or Kani) + green job |
| D2 | Commission **signed external security review** (brief already on file) | CPR-005 | Report under `doc/` + open findings dispositioned |
| D3 | Re-run complete-path review; close CPR-001…006 as honestly possible | H6 exit | [HEAP_COMPLETE_PATH_REVIEW.md](HEAP_COMPLETE_PATH_REVIEW.md) updated |
| D4 | Only then: flip matrix `qualified=true` + `may_advertise_qualified` | Product claim | `verify-heap.sh` enforces true; Level-2 language allowed |

### Wave E — Server product posture (parallel, not H6-blocking alone)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| E1 | Make **qualified HeapKey listener the default** remote profile | HP-008 residual | Token path opt-in / clearly legacy |
| E2 | Expand RPC / authority vector corpus (`spec/heap`) | HP-000 residual | corpus_status + CI |
| E3 | HP-005 residuals: COSE transition corpus, threshold recovery, peer-cred barrier | Authority depth | Accept tests named in matrix |

### Wave F — After single-node qualification

| # | Task | Why |
|---|------|-----|
| F1 | **HP-011** cluster control and placement | Spec §40 |
| F2 | **HP-012** cluster qualification (HC1) | Spec §40 |
| F3 | Prime-time wedge (embedded early-access) per [PRIME_TIME_PLAN.md](PRIME_TIME_PLAN.md) | Product, not only heap gates |

## What *not* to do next

- Primary-index micro-opts / polish of finished Stages 0–9 cuts (see WORK_HORIZON).
- Opening ENR2 or large SDA expansions as the main HEAP program.
- Flipping `qualified=true` without D1–D3.
- Expanding erasure/lifecycle *scaffolds* before H4/H6 honesty.

## Suggested next 3 labor packages (concrete)

1. **B3 — indexes §32.4 (130–133)** or **A1–A2 CPR-001 default flip**.  
2. **C1 — live multi-tier media wipe** (H4).  
3. **D1 — Verus/Kani connection** (H6 engineering gate).

## Machine checks (today)

```bash
bash ./scripts/check_heap_architecture.sh
./scripts/verify-heap.sh quick   # or full before release candidate
cargo test -p dingo-sdk --features dangerous-key-export --test hp007_connect_heap
```

Matrix truth: `spec/heap/qualification/hp010-matrix-v1.json` → `"qualified": false`.