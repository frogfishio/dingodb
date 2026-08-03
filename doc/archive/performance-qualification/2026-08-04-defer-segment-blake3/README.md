# Defer whole-segment BLAKE3 off authoritative SealDone (2026-08-04)

Status: **architectural distinction resolved; paired-median gate still open**.  
Seal Fast Lane remains **`in_review`** (not labor-`done`).

## Question

> Why is whole-segment BLAKE3 still required before authoritative `SealDone`?

## Verdict: **derived, not authoritative**

| Artifact | Role |
|---|---|
| Durable prefix + segment-summary frame | **Authoritative** sealed image |
| Per-frame CRC + body BLAKE3 | **Authoritative** corruption detection |
| Whole-segment BLAKE3 | **Derived** (tier placement + segment catalog) |
| Hydra / Chimera / catalogs | **Derived** (rebuildable) |

The summary wire body (`encode_summary_body`) carries segment ids, sealed length,
and frame count — **not** a content hash. Placement/`segments.cat` digests are
accelerators; salvage must not depend on them (LAWS §6).

Therefore the authoritative lane must not stream-hash the pending prefix.

## Implementation

Hot path `FinalizeSealMeta`:

1. Build summary footer from rotation metadata (`meta_publish_plan`) — **no
   pending read**.
2. Append summary + rename into `segments/`.
3. `SealDone` → `{segment_id, size, summary}` with
   `content_hash = CONTENT_HASH_PENDING` (`[0;32]`).
4. Enrichment (`EnrichDerived`) computes BLAKE3 + Hydra/Chimera; writer applies
   derived tier/catalog digests on `EnrichDone`.

**Not** moved onto the put path. Stream-hash helper retained for diagnostics
only (`plan_from_pending_prefix`).

## Paired-median remeasure (identical recipe)

Enrichment **off**, alternate 6× control (512 MiB) + 6× stream64 (64 MiB),
same machine family, new release binary (`binary.sha256`).

| Cell | Median ack TPS | Min–Max | Multi-rotate | Exact reopen |
|---|---:|---:|---|---|
| Control (512 MiB) | **74 112** | 65 852–80 921 | n/a | 6/6 |
| Meta-publish (64 MiB) | **64 410** | 58 473–66 033 | ≥4 sealed, 6/6 | 6/6 |

| Gate | Value | Result |
|---|---:|---|
| median ratio | **0.869** | **FAIL** (≥ 0.90) |

Prior stream-hash campaign (same bed): median ratio **0.878**
(`../2026-08-04-paired-median-gate/`). Removing auth-lane BLAKE3 did **not**
recover the ~2.2 pp gap; overhead is elsewhere (rotate/fsync/start_active /
seal-worker I/O overlap while puts continue).

## Residual

- Paired 90% gate still open (~13% rotation overhead vs 10% allowance).
- Do **not** reintroduce put-path or write-tail hashing.
- Next: profile non-hash publish costs, or principal waiver — not “make BLAKE3
  authoritative again.”
