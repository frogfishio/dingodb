#!/usr/bin/env bash
# PQH-1…PQH-5: runner, workload, metrics, envelope, L2 shadow unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1…5: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf PQH-1…5 tests passed"
