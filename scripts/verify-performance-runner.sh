#!/usr/bin/env bash
# PQH-1 + PQH-2 + PQH-3: runner, workload, metrics kernel unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1/2/3: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf runner+workload+metrics (PQH-1/2/3) tests passed"
