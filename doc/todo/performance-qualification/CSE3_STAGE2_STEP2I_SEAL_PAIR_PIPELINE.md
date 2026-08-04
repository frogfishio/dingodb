# CSE-3 Stage 2i — Protected Seal-Pair Pipeline

Status: **labor complete → in_review** (2026-08-04)  
Default product remains **Materialized**.

## Problem

Stage 2h showed lifecycle capacity (~6 seal-pairs/s) exceeds writer demand
(~2.8 seg/s at 23K TPS / 8 KiB), but auth finalize + Shadow finalize + catalog
were **serialized onto the foreground writer** (~167 ms/rotation).

## Design

1. **Detach** authoritative active + Shadow staging as one immutable pair
   (`prepare_async_publish` — encode only, no sync/rename).
2. **Start** the next authoritative/Shadow pair immediately.
3. **Finalize** on the protection worker: auth pending→sealed → Shadow
   sync→rename→dir sync → `note_durable` / coverage publish.
4. Advance **`protected_frontier` only after both sides are durable**.
5. Bound `inflight_seals` / `max_pending_seals` — backpressure when the worker
   lags; never claim P★ for in-flight pairs.
6. **Crash recovery** (`protected_pair::recover_protected_pairs`): pending+tmp,
   sealed+tmp, sealed+verified `.rsh` without frontier, orphan tmp cleanup.
7. Catalog / EnrichDerived stay off the writer critical path (apply on
   `ProtectedPairDone`).

## Correctness fix (false P★)

`SealFailed` must **not** call full `finalize_seal` enrichment: Materialized
Chimera dual-write published an RSHD0003 mirror and falsely advanced P★ after
Shadow publish failpoints. SealFailed now uses **`finalize_seal_authoritative`
only**. Enrichment also skips mirror publish when dual-stream already owns
`.rsh` / staging.

## Evidence

| Gate | Result |
|------|--------|
| RSHD0004 matrix (`--test-threads=1`) | **16/16 PASS** |
| Segment-ID never-reuse matrix | **8/8 PASS** |
| `protected_pair` crash unit tests | **3/3 PASS** |
| No false P★ on finalize failpoints | **PASS** (sync/rename/dir_sync/summary) |
| Step 9 product campaign 256 MiB release | **ack=life≈28.7K** TPS; gates_pass; pub≈137 seg/s |
| Step 9 product campaign **2 GiB** (2i activate path) | **ack=life≈30.9K** — **candidate only** (not product SoT) |
| Stage 2k fresh-default product figure | **~21–23K** life=ack — see [`CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md`](./CSE3_STAGE2_STEP2K_DEFAULT_FLIP.md) |

## Commands

```text
cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_rshd0004_matrix -- --test-threads=1

cargo test -p residiuum-store --features legacy-raw-store --lib protected_pair

cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_segment_id_never_reuse -- --test-threads=1

CSE3_STEP9_TARGET_BYTES=268435456 CSE3_STEP9_WORK=/tmp/cse3-step9-256m \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step9_product_campaign step9_product_campaign -- --nocapture
```

Step 9 at 2 GiB (lifecycle≈ack / backlog slope):

```text
CSE3_STEP9_TARGET_BYTES=2147483648 CSE3_STEP9_WORK=/tmp/cse3-step9-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step9_product_campaign step9_product_campaign -- --nocapture
```

## Residual

- Universal CompactShadow default still **not** flipped — see Stage 2j flip
  package ([`CSE3_STAGE2_STEP2J_FLIP_PACKAGE.md`](./CSE3_STAGE2_STEP2J_FLIP_PACKAGE.md)).
- Shard sidecar (`*.rsh.dual.shard`) lands with prepare (Stage 2j).
