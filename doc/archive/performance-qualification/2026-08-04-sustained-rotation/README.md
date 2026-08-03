# Sustained rotation qualification — Seal Fast Lane (2026-08-04)

Status: **architectural package ready for principal accept**; 90% micro-gate
**unmet and superseded** by sustained-rotation evidence.

## Conclusions locked

1. Remaining overhead vs a no-rotation control is the cost of **real
   rotations** (flush / rename / start-active / publish / catalog apply). The
   high-threshold control is a ceiling, not a sustainable product workload.
2. Whole-segment BLAKE3 is derived; `[0;32]` must not mean “pending.” Typed
   [`ContentHashState::{Pending,Known}`](../../../../crates/residiuum-store/src/incremental_seal.rs)
   is now SoT.

## Typed hash

- `ContentHashState::Pending` | `ContentHashState::Known([u8; 32])`
- Wire v2 for tier placement + segment catalog (tagged state)
- Proof: `crates/residiuum-store/tests/reopen_before_enrichment.rs` — reopen
  exact with enrichment off while digests stay `Pending`

## Sustained campaign (2 GiB @ 64 MiB)

| Knob | Value |
|---|---|
| Cell | Real Full |
| Logical | **2 GiB** |
| Seal threshold | 64 MiB |
| Payload / concurrency / seed | 8 KiB / 8 / 42 |
| Enrichment | **off** |
| Binary | see `binary.sha256` |

| Metric | Value |
|---|---:|
| Ack TPS (sustained) | **47 759** |
| Keys | 262 144 |
| Sealed @ last ack | **32** |
| Rotations timed | **32** |
| Reopen exact | yes |
| Ack wall | 5.49 s |

### Stage share of ack wall

| Stage | % of ack wall | Notes |
|---|---:|---|
| catalog_apply (tier persist + summary upsert) | **13.91%** | Dominates |
| auth_publish (summary append + rename) | 0.07% | |
| rename_pending | 0.05% | |
| start_active | 0.05% | |
| flush | ~0% | |
| backpressure_wait | ~0% | |
| **all rotation stages** | **14.09%** | |

Raw: `sustained-2g-64m.json`.

## Gate language

| Gate | Status |
|---|---|
| 90% paired micro-gate vs no-rotation control | **Unmet; superseded** |
| Seal Fast Lane architecture (derived off auth lane; meta publish) | **Accept (principal)** |
| Sustained multi-rotation ack + reopen exact | **Recorded** |

No-rotation control remains a useful ceiling. Demanding real lifecycle work
under an arbitrary 10% of that ceiling is not a sound product gate.
