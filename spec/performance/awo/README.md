# Adaptive Write Optimiser executable contracts

Profile: `residiuum-adaptive-write-v1`

| File | Role |
|---|---|
| `profile-v1.json` | Closed profile identity and registry membership |
| `states-v1.json` | Request lifecycle states and terminal classification |
| `transitions-v1.json` | Permitted state transitions |
| `decision-reasons-v1.json` | Closed natural/batch/fallback decision reasons |
| `outcomes-v1.json` | Closed completion and overload outcome classes |
| `policy-v1.json` | Exact safe defaults and controller constants |
| `golden-decisions-v1.json` | Executable selector arithmetic vectors |
| `schemas/golden-decisions-v1.schema.json` | Golden-vector JSON Schema |

Verify:

```bash
bash scripts/verify-awo-contract.sh
```

These files close AWO-0 contracts. Product execution remains gated by the
master delivery plan and later AWO packages.

