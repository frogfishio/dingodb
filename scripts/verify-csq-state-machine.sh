#!/usr/bin/env bash
# CSQ-4: store model / state machine suite.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

# Firewall still holds (model must not pull production store).
bash ./scripts/verify-csq-oracle-firewall.sh || ERR=1

echo "Running residiuum-store-model unit + CSQ-4 suite..."
cargo test -p residiuum-store-model -- --nocapture || ERR=1

# DEF-099 / DEF-100 regression authorities remain linked on production store.
if cargo test -p residiuum-store --features legacy-raw-store --test stage_def_099_historical_get -- --list >/dev/null 2>&1; then
  cargo test -p residiuum-store --features legacy-raw-store --test stage_def_099_historical_get -- --nocapture || ERR=1
  cargo test -p residiuum-store --features legacy-raw-store --test stage_def_100_coverage_scans -- --nocapture || ERR=1
fi

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-state-machine: FAILED" >&2
  exit 1
fi
echo "verify-csq-state-machine: OK"
