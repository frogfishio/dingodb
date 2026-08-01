# Formal assurance tree

Operator entry: **[HOW_TO_USE.md](./HOW_TO_USE.md)** — big picture + day-to-day commands.

Normative specs: [doc/todo/formal-assurance/](../doc/todo/formal-assurance/).  
Living package states: [NEXT_BUILD_STATUS.md](../doc/wip/status/NEXT_BUILD_STATUS.md).

| Path | Role |
|------|------|
| `registry/` | FAS-0 claim/toolchain registries (`FAS0_CLOSED` = package accept) |
| `heap/` | Heap TLA+ sketches (security precursors) |
| `lean/` | FAS-1 smoke + FAS-2 abstract State/Observation kernel + FAS-3/4 modules |
| `kani-smoke/` | FAS-1 Kani smoke harness |
| `tla/smoke/` | FAS-1 TLC smoke module |
| `refinement/` | FAS-3 entrypoint census, type map, vertical bridges |
| `consistency/` | FAS-4 CON theorem connections + negatives |

### Bootstrap and gates

```bash
bash scripts/setup-formal-tools.sh --locked
bash scripts/check-formal-registry.sh      # FAS-0 → target/formal-assurance/fas0-registry-report.json
bash scripts/check-formal-toolchain.sh     # FAS-1
bash scripts/check-formal-foundation.sh    # FAS-2
bash scripts/check-formal-refinement.sh    # FAS-3
bash scripts/check-formal-consistency.sh   # FAS-4
```

### What FAS is (one line)

**Named theorems + assumptions + tools + (optional) Rust connection + CSQ links** —
not a blanket “formally verified database” claim.

### CI today

| In default PR quality job | Separate CI jobs | Local / package accept only |
|---------------------------|------------------|-----------------------------|
| `check-formal-registry.sh` (FAS-0) | `kani-heap`, `verus-heap` | FAS-1…FAS-4 full gates |

Details: [HOW_TO_USE.md](./HOW_TO_USE.md) §1b.

### Pre-release briefing (HTML)

```bash
bash scripts/release-briefing.sh --profile pre-release
# → target/release-briefing/LATEST.html
```

Chains selected gates, ingests FAS/CSQ JSON, writes human HTML + machine JSON.
Does **not** replace `./scripts/quality.sh` or PQH qualification campaigns.