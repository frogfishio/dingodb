#!/usr/bin/env bash
# CSQ-5: crash / filesystem campaign suite.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-5 crash campaign tests..."
cargo test -p residiuum-store --features legacy-raw-store --test csq5_crash_campaign -- --nocapture || ERR=1

# Regression authorities linked to CSQ-5
echo "Running DEF-022 crash matrix CI subset..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_022_crash_matrix -- --nocapture || ERR=1

echo "Running DEF-101 writer lock..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_101_writer_lock -- --nocapture || ERR=1

echo "Running DEF-104 crash recovery contract..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_104_crash_recovery_contract -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-crash-campaign: FAILED" >&2
  exit 1
fi
echo "verify-csq-crash-campaign: OK"
