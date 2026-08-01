# FAS-0 package accept gate

Status: **structural labor complete; package accept blocked**  
Date: 2026-08-01

## Current signal

```bash
bash scripts/check-formal-registry.sh
# STRUCTURAL_OK + exit 1
# report: target/formal-assurance/fas0-registry-report.json
#   structural_ok: true
#   closed: false
```

## What is done (labor)

| Item | Evidence |
|------|----------|
| REGISTRY §12 theorems (35) | `formal/registry/theorems-v1.json` |
| §5 assumptions (8) | `formal/registry/assumptions-v1.json` |
| Ownership / migration seed | `artifact-ownership-v1.json` + `FAS_MIGRATION_MAP.md` |
| Linter baseline | cycles, elevated status, claim/theorem, forbidden wording |
| Negative fixture self-tests | elevated + circular fixtures must fail linter |
| Schemas | theorems, assumptions, tcb, claims, profiles, ops, negatives, toolchain, ownership, package-report |

## What blocks package accept (honest)

1. **`CSQ-12` scoreboard still `active`, not `accept`** (master plan entry for FAS-0).
2. **`formal/registry/FAS0_CLOSED` must not be invented** until (1) + principal review of catalogue/linter.
3. Only then: `check-formal-registry.sh` exit 0 and scoreboard FAS-0 → `accept` in the **same** change.

## Explicit non-goals until then

- Do **not** start FAS-1 package accept labor as if FAS-0 accepted (toolchain pin work may be *drafted* but cannot claim FAS-1).
- Do **not** elevate any theorem to `machine_proved` / `implementation_connected` without real proof result hashes.
- Do **not** freestyle Lean/kernel (FAS-2) ahead of the graph.

## Recommended principal path

```text
CSQ-12 accept  →  review FAS-0 catalogue  →  write FAS0_CLOSED
  →  check-formal-registry exit 0  →  scoreboard FAS-0 accept
  →  FAS-1-T1 toolchain lock
```

## Board

| Task | Stage |
|------|--------|
| FA0-W0-* | done / in_review |
| FAS-0-T1 / T2 | in_review (structural) |
| FAS-1-T1… | todo — waits FAS-0 accept |
