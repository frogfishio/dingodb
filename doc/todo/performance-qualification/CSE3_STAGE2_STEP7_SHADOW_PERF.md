# CSE-3 Stage 2 step 7 — Recovery Shadow performance qualification

Status: **active** (2026-08-04) — harness-only candidate; **no product flip**.  
Depends: Step 6 principal-accepted
([`CSE3_STAGE2_STEP6_CSE_MATRIX.md`](./CSE3_STAGE2_STEP6_CSE_MATRIX.md)).

## Candidate configuration (harness only)

```text
Authoritative segments
+ Compact Chimera
+ Recovery Shadow
− Materialized Chimera
```

Enabled only inside `recovery_shadow::qualify` /
`tests/cse3_stage2_step7_shadow_perf.rs`. Product seal remains Materialized
until step **8**.

## Candidate Shadow path (RSHD0003)

Shadow publication is a **canonical segment image mirror** (physical buffered
copy of sealed `.residiuum` bytes into envelope + image + commitment). No value
decode/re-encode on the Shadow critical path (`encode_ns=0`). Compact decode
remains a separate amp/harness path and is **excluded** from the ≥7 seg/s rate.

## Measured per Shadow

| Stage | Field |
|---|---|
| Source read/decode | `source_read_decode_ns` (Compact-only under RSHD0003) |
| Record encryption | `encrypt_ns` (unused for mirror; 0) |
| Record encoding | `encode_ns` (0 for RSHD0003 mirror) |
| Sequential write | `sequential_write_ns` (physical copy + hash) |
| File sync | `file_sync_ns` (`sync_all`) |
| Rename | `rename_ns` |
| Directory sync | `dir_sync_ns` |
| Frontier publication | `frontier_publish_ns` |
| Bytes / CPU / wall | `bytes_*`, `cpu_ns`, `wall_ns` |

## Acceptance gates

| Gate | Bound |
|---|---|
| Shadow publication | ≥7 segments/sec |
| Backlog slope (post warm-up) | ≤0 |
| `protected_frontier` | follows sealed frontier without gaps |
| Complete-lifecycle TPS | close to acknowledgement TPS (≥80%) |
| Shadow amplification | ≈100% + bounded framing (≤130% live payload) |
| Compact Chimera | ≤5% of auth segment bytes |
| Recovery | succeeds after deleting auth segments + Compact |
| Verified `.rsh` | every claimed protected segment |

Workload floor: **2 GiB logical / 64 MiB seal / 8 KiB payload** (ETQ cell).

## Recipe

```bash
CSE3_STEP7_TARGET_BYTES=2147483648 \
CSE3_STEP7_WORK=/tmp/cse3-step7-2g \
cargo test -p residiuum-store --features legacy-raw-store --release \
  --test cse3_stage2_step7_shadow_perf step7_smoke_candidate_harness -- --nocapture
```

Smoke default target is 8 MiB (CI-friendly). Full campaign asserts `gates.pass`.

## Non-claims

- Does **not** switch product sealing off Materialized.
- Does **not** introduce parallel Shadow workers.
- Step **8** remains blocked until this campaign passes performance **and**
  P★ recovery in the **same** run.

## Evidence

- RSHD0002 baseline archive: [`…/2026-08-04-cse3-stage2-step7-shadow-perf/`](../../archive/performance-qualification/2026-08-04-cse3-stage2-step7-shadow-perf/)
- RSHD0003 mirror archive: [`…/2026-08-04-cse3-stage2-step7-rshd0003-mirror/`](../../archive/performance-qualification/2026-08-04-cse3-stage2-step7-rshd0003-mirror/)

**2026-08-04 RSHD0002:** 2 GiB FAIL 3.69 seg/s (best quiet 256 MiB 6.33).

**2026-08-04 RSHD0003:** encode stage removed. Quiet 256 MiB **9.96 seg/s PASS**.
Best 2 GiB/64 MiB **6.99 seg/s** (FAIL ≥7 under post-seal copy contention).

**2026-08-04 RSHD0004 dual-stream (experimental):** write-time paired append;
commitment over ordered frame hashes. Sustained 2 GiB/64 MiB ×3 median
**55.57 seg/s PASS** (min 37.69); recovery/amp/frontier PASS; lifecycle≈ack;
ack ~27–28K (no bad foreground regression). Archive
[`…/2026-08-04-cse3-stage2-step7-dual-stream/`](../../archive/performance-qualification/2026-08-04-cse3-stage2-step7-dual-stream/).
Harness: `CSE3_STEP7_DUAL_STREAM=1`. **No product flip.** Step 8 still needs
principal accept.

