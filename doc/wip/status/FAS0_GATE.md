# FAS-0 package accept gate

Status: **accept (2026-08-01)**  
Package exit: `bash scripts/check-formal-registry.sh` → exit 0.

## Closed conditions met

| Gate | Evidence |
|------|----------|
| CSQ-12 accept | Scoreboard `CSQ-12 = accept`; A2 `a2_pass=true` via `residiuum-verify-core-storage.sh` |
| Registry closed | `formal/registry/FAS0_CLOSED` present |
| Check script | exit 0; `structural_ok=true`, `closed=true`, 35 theorems, 8 assumptions |
| Report | `target/formal-assurance/fas0-registry-report.json` |

## Still true (honesty)

- No theorem is claimed `machine_proved` / `implementation_connected` without proof result hashes.
- ATM/CLU catalogue entries remain `proposed` until feature waves.
- FAS-1 toolchain pins (Lean/Kani/TLC hashes) remain residual for FAS-1-T1.

## Next

```text
FAS-1-T1 toolchain lock → FAS-2 Lean kernel → FAS-3 refinement → FAS-4 consistency
```
