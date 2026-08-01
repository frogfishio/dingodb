# FAS-4 consistency family

Authority: `FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md` §8, REGISTRY §12.2.

| Artifact | Role |
|----------|------|
| `theorem-connections-v1.json` | CON theorem → Lean + CSQ + Rust entrypoints |
| `negative-controls-v1.json` | One live negative mutant per CON theorem |
| Lean `Residiuum.Consistency` | Abstract machine-checked obligations |

Gate:

```bash
bash scripts/check-formal-consistency.sh
# → target/formal-assurance/fas4-consistency-report.json
```

**Honesty:** `residiuum-formal-consistency-v1` profile is **MVP** — abstract Lean
+ CSQ evidence links — not full `physically_qualified` for every CON claim.
Filesystem assumption: `FAS-ASM-FILESYSTEM-DURABILITY-001`.
