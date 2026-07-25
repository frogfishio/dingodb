# Human-facing demos

Narrative checkpoints from [DELIVERY_PLAN.md](../../DELIVERY_PLAN.md) §8.
Stages **0–9** are complete; these scripts are the live smoke path for the
product thesis (*damage it; find what survived*).

Run from the repo root (or set `DINGO_BIN` to a built `dingo` binary).

| Script | What it proves |
|--------|----------------|
| [`02_punch_a_hole.sh`](02_punch_a_hole.sh) | Corrupt a segment; doctor/salvage still speak |
| [`03_salvage_survives.sh`](03_salvage_survives.sh) | Wipe catalogs; salvage recovers live keys |
| [`07_tier_move.sh`](07_tier_move.sh) | Tier/archive acceptance + [retention runbook](../../doc/RUNBOOK_RETENTION.md) |
| [`08_kill_a_node.sh`](08_kill_a_node.sh) | Multi-hop `serve-cluster` + kill-node survivor |

```sh
chmod +x scripts/demos/*.sh
./scripts/demos/03_salvage_survives.sh
```

Related operator docs: root [README.md](../../README.md),
[`dingo-cli` README](../../crates/dingo-cli/README.md),
[doc/RUNBOOK_RETENTION.md](../../doc/RUNBOOK_RETENTION.md).
