# Formal assurance registries

Authority: `doc/todo/formal-assurance/FORMAL_ASSURANCE_REGISTRY_CONTRACT.md`.

## Wave / package honesty

| State | Meaning |
|-------|---------|
| **Structural catalogue present** | All REGISTRY §12 theorem IDs + §5 assumptions registered as `proposed`/`specified` stubs |
| **`FAS0_CLOSED` absent** | Package **not** accepted; `scripts/check-formal-registry.sh` exits non-zero |
| **Scoreboard FAS-0 accept** | Requires **CSQ-12 = accept** + green check after closed marker + principal review |

Do **not** create `FAS0_CLOSED` until CSQ-12 scoreboard accept and FAS-0-T1/T2 exit are honest.

Do **not** elevate theorem `status` to `machine_proved` / `implementation_connected` without real `result_refs` hashes.

## Files

| Path | Role |
|------|------|
| `theorems-v1.json` | Mandatory catalogue §12 (FND/CON/SEC/ATM/CLU) |
| `assumptions-v1.json` | Mandatory assumption catalogue §5 |
| `tcb-v1.json` | Trusted computing base stubs |
| `toolchain-lock-v1.json` | Tool pins (Verus pin known; others FAS-1) |
| `claims-v1.json` | Public claims (empty until policy allows) |
| `profiles-v1.json` | Formal profile ids |
| `artifact-ownership-v1.json` | Migration map → theorem ownership seed |
| `fixtures/rejected/*` | Negative controls |
| `schemas/*` | Draft 2020-12 schemas (expand in FAS-0-T2) |

## Verify

```bash
bash scripts/check-formal-registry.sh
# expected today: STRUCTURAL_OK + fail package accept (exit 1)
# report: target/formal-assurance/fas0-registry-report.json
```

Migration prose: `doc/wip/status/FAS_MIGRATION_MAP.md`.
