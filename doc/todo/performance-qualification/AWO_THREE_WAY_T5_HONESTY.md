# Measure AWO three-way — T5 honesty pass

Status: **labor complete (self_check) — not package accept**  
Card: `7da7703b-19cc-43bc-a5b4-5a0db096e920`  
Date: 2026-08-02  
Depends on: T4 disk-safe artifacts  
Source numbers: `artifacts/awo-three-way-t4-disksafe/summary.json`

---

## 0. Host disk budget (principal-stated / re-checked)

| Source | Free on data volume |
|--------|---------------------|
| Principal this turn | **~30 GiB total free** |
| Re-check (`df`) this labor | **~31 GiB avail** on `/System/Volumes/Data` |
| Workspace `target/` | ~5.5 GiB (keep; do not treat as free for campaigns) |
| T4 artifacts (JSON only) | ~1.3 MiB |

**Operational rule under this budget**

| Run class | Allowed on this host? |
|-----------|------------------------|
| Smoke / T3 correctness / T4 disk-safe slice | **Yes** — op-capped; delete work dirs after each mode |
| T2 **diagnostic** freeze (2 GiB/cell + 5 reps × 2 procs) | **No** until free space is much larger or harness gains a disk-budget class |
| Qualification (120s + 512 MiB floors, controlled) | **No** on this host budget |

Diagnostic peak risk (order of magnitude): even `max_cells=1` can approach **multi-GiB to tens of GiB** with multiproc/reps if stores are not deleted between modes — already nearly filled the volume once.

**Reserve:** leave **≥15 GiB free** as hard stop before any non-smoke campaign. With ~30 GiB free, that leaves ~15 GiB working room — enough for smoke only.

---

## 1. What T4 compared (same cell)

| Knob | Value |
|------|--------|
| Class | **smoke** (not diagnostic) |
| Seed | 42 |
| Primary cell | `L4-durable-s16384-c1-o8-43` |
| Payload / durability | 16 KiB · Durable |
| Ops | smoke cap (~24 ack) |
| Modes | disabled · static · adaptive |
| Metric | `throughput_bytes_per_sec_proxy` / `e2e_ns_proxy` |

### Median proxies (n=6 primary runs each)

| Mode | thr proxy MiB/s | e2e proxy ms | validity | reopen |
|------|-----------------|--------------|----------|--------|
| disabled | ~3.53 | ~106 | valid | ok |
| static | ~3.91 | ~96 | valid | ok |
| adaptive | ~3.90 | ~96 | valid | ok |

Relative to disabled (proxy thr): static ≈ **+11%**, adaptive ≈ **+10%**.  
Absolute levels are **tiny-work smoke proxies**, not sustained throughput.

---

## 2. What the numbers mean

| Observation | Honest reading |
|-------------|----------------|
| All three modes **valid + reopen_ok** | Path correctness holds for this micro-cell (aligns with T3). |
| Static/adaptive slightly faster than disabled | Plausible under smoke noise; **not** proof AWO wins product workloads. |
| Static ≈ adaptive | Adaptive controller has **almost no room** to act on 24-op smoke cells. |
| thr labeled **proxy** | Harness field `throughput_bytes_per_sec_proxy` under smoke e2e — not a sustained window. |
| Single cell / 16 KiB / durable | **Not** the T2 freeze set (256/4KiB/8KiB × Buffered/Durable). |

---

## 3. Gaps that unfair or weak comparisons (plain English)

| Gap | Why it weakens “AWO better/worse” claims |
|-----|------------------------------------------|
| **Smoke op cap** | Work ends at ~24 ops; cooking/pipeline/adaptive estimator never see real queues. |
| **Not diagnostic floors** | No 30s / 2 GiB sustained floor — cannot speak to steady-state. |
| **Cold micro-runs** | Lease attach + cooker start dominate relative cost on tiny work. |
| **Disk forced slice** | Full T2 matrix + diagnostic class **blocked** by ~30 GiB free host budget. |
| **Multiproc probe cells** | Campaign still schedules extra multiproc finding cells; primary comparison is one L4 cell. |
| **No p50/p99 latency fields** | Only e2e proxy on this result shape. |
| **Synthetic plan platform** | Lab plan is `synthetic_harness` — not a product baseline platform. |

**Code fairness fix?** **No.** Nothing in the comparison is *invalid* for a smoke path check; the gap is **measurement class + host disk**, not a broken mode flag. Do not “fix” AWO to chase smoke MiB/s.

---

## 4. Claim / non-claim table

| Claim | Status |
|-------|--------|
| Three modes run real_store smoke and reopen cleanly on this host | **Yes** (T3 + T4) |
| Same-seed three-way commands exist and produce artifacts | **Yes** |
| Static/adaptive **product** faster than disabled | **No** |
| Adaptive better than static | **No** |
| T2 diagnostic matrix completed | **No** — disk residual |
| Qualification / G8 / bottleneck | **No** |
| Default-on AWO | **No** |
| Package accept for measure feature or AWO | **No** |

---

## 5. Improved / not improved

| Area | Result |
|------|--------|
| Path readiness for interactive re-run | **Improved** (T1–T4 path works under smoke) |
| Evidence under disk constraint | **Improved** (JSON-only artifacts; cleanup documented) |
| Ability to rank modes for product | **Not improved** — smoke scale + disk block diagnostic |
| Sustained / floor-honest diagnostic numbers | **Not available** on this host budget |

---

## 6. Stop / next under ~30 GiB free

**Allowed next without more disk**

1. **T6** interactive re-run: smoke subset, delete work dirs, show live numbers.  
2. Document-only refinements to runbook.

**Do not attempt on this host until free space is much larger**

- `class=diagnostic` T2 freeze (max_cells=64 or even 1 without careful peak math)  
- Qualification / soak  

**If free rises (e.g. ≥50–80 GiB free):** revisit one-cell diagnostic with delete-after-mode and reserve 15 GiB headroom.

---

## 7. Artifacts

| Path | Role |
|------|------|
| `artifacts/awo-three-way-t4-disksafe/summary.json` | Numeric summary |
| `artifacts/awo-three-way-t4-disksafe/campaigns/*` | Per-mode JSON bundles |
| `AWO_THREE_WAY_T4_DISKSAFE_MEASURE.md` | T4 delivery note |
| This file | Honesty / claim table |
