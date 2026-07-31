#!/usr/bin/env bash
# PQH-1…PQH-4: runner, workload, metrics, L0/L1 envelope unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1…4: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf runner+workload+metrics+envelope (PQH-1…4) tests passed"
