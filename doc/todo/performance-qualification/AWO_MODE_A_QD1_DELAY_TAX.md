# Why Static/Adaptive ≈ 2.5k (not the CPU wall)

Status: **labor answer (self_check) — not package accept**  
Date: 2026-08-03  
Principal: *Static ~2460 / Adaptive ~2470 — is this because we’re hammering the
CPU too much?*

## Verdict

**No.**

~2.5k is a **latency / collection-delay tax** on Mode A QD=1 with
`independent_admit_put+collection`. It is **not** “we burned more CPU than
Residiuum-off.” Off already sits on the CPU wall (~12.5k @ ~68% CPU). Static and
Adaptive keep similar peak CPU while wall-clock stretches ~5×.

**Also:** Static ≠ “we batched successfully” on this table — intent to coalesce,
but QD=1 left nothing to coalesce. See
[STATIC_IS_NOT_BATCHED_ON_FN2.md](STATIC_IS_NOT_BATCHED_ON_FN2.md).

```text
Residiuum-off      ~12 550/s   CPU ~68%   path=put_many                         ← CPU wall
Static / Adaptive  ~2 460/s    CPU ~68–76% path=independent_admit_put+collection ← delay tax
```

## Evidence (FN-2 same bed, same 32 768 keys)

| Cell | ops/s | elapsed | peak CPU | path |
|------|------:|--------:|---------:|------|
| Residiuum-off | 12 553 | **2.6 s** | 68.5% | `put_many` |
| Static | 2 464 | **13.3 s** | 67.8% | `independent_admit_put+collection` |
| Adaptive | 2 467 | **13.3 s** | 76.2% | `independent_admit_put+collection` |

Same CPU band, **~5× longer wall** for the same work → waiting, not “CPU
hammered harder.”

## Mechanism

Mode A + AWO lease uses the PQH batch=1 path: enqueue → collector → wait for
ack (QD=1). Policy `maximum_collection_delay` defaults to **250 µs**. With
outstanding=1 there is **never a pile-up**, so every put pays (most of) that
delay hoping for a second item that never comes:

```text
32 768 × 250 µs  ≈  8.2 s delay floor
+ cook/IO that takes ~2.6 s when off
≈ observed ~13 s  →  ~2.5k ops/s
```

Static ≈ Adaptive here because Adaptive’s `select_plan` never sees multi-item
batches under QD=1 — both modes are “collector + one entry + delay.”

T11’s ~2× thr win needed **outstanding pile-up** (Durable saturated). Mode A
PEER knobs forbid that (QD=1). Different bed, different story.

## Two walls — keep them separate

| Wall | Where | ops/s (FN-2 APFS) | Cause |
|------|-------|------------------:|-------|
| **CPU / cook** | Residiuum-off vs SQLite | ~12.5k vs ~29k | Blake/frame per put (`FAST_DISK_CPU_WALL.md`) |
| **Collection delay** | Static/Adaptive vs off | ~2.5k vs ~12.5k | QD=1 + collector delay (this doc) |

Do **not** diagnose Adaptive 2.5k as “CPU too hot.” Fixing Blake will not turn
2.5k into 12.5k while QD=1 still waits out collection delay. Fixing collection
for no-pile-up (skip delay / natural path when outstanding cannot build) is the
AWO-shaped residual; CPU cook is the off-vs-SQLite residual.

## Non-claims

Not AWO package accept. Not “AWO always slower.” Not a thr floor. Adaptive can
still help under pile-up (T11); Mode A QD=1 is the wrong shape for that win.
