# FAS-3 refinement bridge

Authority: `FORMAL_KERNEL_MODEL_CONTRACT.md` §10 / §13, `FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md` §7.

| Artifact | Role |
|----------|------|
| `entrypoint-census-v1.json` | Production entrypoints + connection class |
| `type-map-v1.json` | Concrete→abstract map + unsafe/FFI notes |
| `bridges/*.json` | Vertical slices (Lean + Verus + Rust + CSQ) |
| `negative/*` | Gate must fail on rename / demo-as-connection |

Gate:

```bash
bash scripts/check-formal-refinement.sh
# → target/formal-assurance/fas3-refinement-report.json
```

Connection classes (do not inflate):

1. `abstract_theorem_only`
2. `independent_executable_agreement`
3. `bounded_concrete`
4. `rust_connected_refinement`

MVP slice: **FAS-BRIDGE-AUTHORITY-BINDING-001** (heap authority binding).
