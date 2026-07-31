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
| A1 | Migrate high-traffic demos/tests to `open_deployment` + `HeapCap` | Unblocks default flip | **Done (partial)** — heap Accept paths use connect_heap / open_deployment; Stages 3–9 tests opt into `legacy-flat-sdk` |
| A2 | Flip **`legacy-flat-sdk` default off** (keep feature for Stages 3–9) | Closes CPR-001 residual for claim honesty | **Done (2026-07-30)** — `default = []`; stage tests `required-features = ["legacy-flat-sdk"]`; claim + arch check updated |
| A3 | Flip or document **`legacy-raw-store`** default consistently with A2 | HP-003 cutover | **Done (2026-07-30)** — `residiuum-store` `default = []`; Stages 3–9 tests/examples `required-features = ["legacy-raw-store"]`; arch check updated |

### Wave B — §32.4 surface completeness (H1–H2)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| B1 | Activate **find** (116) + minimal filter args | Query is the everyday path | **Done (2026-07-30)** — schemas/fixtures + scan+Filter dispatch + `RemoteHeap::find` + `connect_heap_find_filter` |
| B2 | Activate **history** (117) | Spec + DX parity | **Done (2026-07-30)** — schemas/fixtures + SubjectV2 history + `RemoteHeap::history` + `connect_heap_history` (rights first-cut = Read) |
| B3 | Activate **index_list / index_create / index_drop / index_rebuild** (130–133) | Find acceleration | **Done (2026-07-30)** — schemas/fixtures + heap-scoped SecondaryIndex + dispatch + `RemoteHeap` index APIs + `connect_heap_indexes` |
| B3b | IndexAdmin rights + find acceleration | Close B3 residuals | **Done (2026-07-30)** — ops 131–133 require IndexAdmin; bootstrap cert rights_mask=13; equality `find` uses ready indexes (`connect_heap_find_via_index`) |
| B3c | Index stale maintain after SubjectV2 writes | Index honesty after put/delete | **Done (2026-07-30)** — `HeapStore::mark_indexes_stale` on put/delete; Accept in `connect_heap_find_via_index` |
| B4 | Activate **collection_create** (106) (needs rights story) | Provisioning without offline catalog | Ceremony/rights + Accept |
| B5 | Remaining reserved ops only as needed for drills (lifecycle RPC, export, …) | Avoid activation thrash | Matrix-driven only |

### Wave C — Ops residuals (H4–H5)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| C1 | Live filesystem multi-tier media wipe drill | H4 open | **Done (2026-07-30)** — `destroy_coverage_unit_on_media` + `wipe_heap_object_media`; Accept `live_filesystem_multi_tier_media_wipe` / unavailable root stays retired |
| C2 | HSM / provider data-key adapter scaffold or explicit out-of-scope | H4 open | **Done (2026-07-30)** — scaffold + **live AWS KMS** (`feature aws-kms`, `AwsKmsDataKeyProvider` SigV4 GenerateDataKey); mock Accept; PKCS#11/GCP/Azure still scaffold |
| C3 | Mixed-heap salvage classification drill | H4 open | **Done (2026-07-30)** — `classify_mixed_heap_frame` / `MixedHeapSalvageClass`; Accept `mixed_heap_salvage_classification_drill` |
| C4 | Broader destructive crash-matrix cells (beyond peer lifecycle) | H5 open | crash_matrix cells + CI subset |

### Wave D — Gate H6 claim (blocks `qualified=true`)

| # | Task | Why | Done when |
|---|------|-----|-----------|
| D1 | Connect **Verus or Kani** to pure decide / isolation Inv in CI | CPR-004 | **Done (2026-07-30)** — Kani + **Verus** connected: `VERUS_PROOFS_CONNECTED=true`, `pure_kernel.rs` (8 verified), `scripts/setup_verus.sh` + `check_verus_heap.sh`, CI `verus-heap` / `kani-heap` |
| D2 | Commission **signed external security review** (brief already on file) | CPR-005 | Report under `doc/` + findings dispositioned. **Requires independent third party** — see [HEAP_H6_RESIDUAL_DISPOSITION.md](HEAP_H6_RESIDUAL_DISPOSITION.md). Cannot close by in-tree self-review. |
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

1. **D2 — procure external reviewer** (CPR-005). Not a code package: engage
   independent firm/researcher using [HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md](HEAP_EXTERNAL_SECURITY_REVIEW_BRIEF.md);
   disposition in [HEAP_H6_RESIDUAL_DISPOSITION.md](HEAP_H6_RESIDUAL_DISPOSITION.md).  
2. **E1 — qualified HeapKey listener as default remote profile** (code).  
3. **Live PKCS#11** (or GCP/Azure) connector; AWS KMS live path is feature `aws-kms`.

## Machine checks (today)

```bash
bash ./scripts/check_heap_architecture.sh
./scripts/verify-heap.sh quick   # or full before release candidate
cargo test -p residiuum-sdk --features dangerous-key-export --test hp007_connect_heap
```

Matrix truth: `spec/heap/qualification/hp010-matrix-v1.json` → `"qualified": false`.