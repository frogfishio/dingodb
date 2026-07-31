# Human-facing demos

Narrative checkpoints from [DELIVERY_PLAN.md](../../DELIVERY_PLAN.md) §8.
Stages **0–9** are complete; these scripts are the live smoke path for the
product thesis (*damage it; find what survived*).

Run from the repo root (or set `RESIDUUM_BIN` to a built `residuum` binary).

| Script | What it proves |
|--------|----------------|
| [`02_punch_a_hole.sh`](02_punch_a_hole.sh) | Corrupt a segment; doctor/salvage still speak |
| [`03_salvage_survives.sh`](03_salvage_survives.sh) | Wipe catalogs; salvage recovers live keys |
| [`07_tier_move.sh`](07_tier_move.sh) | Tier/archive acceptance + [retention runbook](../../doc/RUNBOOK_RETENTION.md) |
| [`08_kill_a_node.sh`](08_kill_a_node.sh) | Multi-hop `serve-cluster` + kill-node survivor |

For a **scale + multi-hit chaos ladder** (1 GiB → 10 GiB → …), use the
non-product harness [`residuum-testrig`](../../crates/residuum-testrig/README.md)
(`scripts/testrig_smoke.sh` for a small smoke).

```sh
chmod +x scripts/demos/*.sh
./scripts/demos/03_salvage_survives.sh
```

Related operator docs: root [README.md](../../README.md),
[`residuum-cli` README](../../crates/residuum-cli/README.md),
[doc/RUNBOOK_RETENTION.md](../../doc/RUNBOOK_RETENTION.md).
