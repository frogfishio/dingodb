# CSE-3 Stage 0 — Materialized recovery bound (2026-08-04)

Status: **labor complete** — analysis / proof only. **No** XOR/RS/code selection.

> Strongest Materialized Chimera format guarantee (**P★**): recover the sealed
> live value set \(V_S\) after total loss of authoritative segment payloads,
> provided the Materialized `.cmr` sidecar remains intact.

## Verdict

Matching P★ requires ≈100% independent redundancy for incompressible data.
**Reduced-overhead Compact equivalence to P★ is information-theoretically
impossible.** Codec selection is deferred until principal chooses keep-P★ /
weaken-claim / hybrid.

## Artifacts

| Path | Role |
|---|---|
| `CSE3_STAGE0_MATERIALIZED_RECOVERY_BOUND.md` | Full Stage 0 write-up |
| `bound.json` | Machine-readable summary |
| Charter | `doc/todo/performance-qualification/CSE3_COMPACT_RECOVERY_CODE.md` |

## Non-claims

No Compact product default; no ETQ-2 resume; no historical-generation Chimera
claim; no multi-media placement proof.
