#!/usr/bin/env bash
# CSQ-7: damage / salvage / deterministic recovery (DEF-011 linkage).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-7 damage/salvage suite..."
cargo test -p residiuum-store --features legacy-raw-store --test csq7_damage_salvage -- --nocapture || ERR=1

# Permanent salvage / recovery authorities remain linked.
echo "Running DEF-011 salvage regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_011_salvage -- --nocapture || ERR=1

echo "Running stage salvage recovery regression..."
cargo test -p residiuum-store --features legacy-raw-store --test salvage -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-damage-salvage: FAILED" >&2
  exit 1
fi
echo "verify-csq-damage-salvage: OK"
echo "  suite: csq7_damage_salvage + stage_def_011_salvage + salvage"
