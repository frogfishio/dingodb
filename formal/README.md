# Formal assurance tree

| Path | Role |
|------|------|
| `registry/` | FAS-0 claim/toolchain registries (`FAS0_CLOSED` = package accept) |
| `heap/` | Heap TLA+ sketches (security precursors) |
| `lean/` | FAS-1 Lean smoke + FAS-2 kernel home |
| `kani-smoke/` | FAS-1 Kani smoke harness |
| `tla/smoke/` | FAS-1 TLC smoke module |

Bootstrap: `bash scripts/setup-formal-tools.sh --locked`  
Gate: `bash scripts/check-formal-toolchain.sh`
