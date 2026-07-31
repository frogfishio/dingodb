#!/usr/bin/env bash
# PQH-1…PQH-6: runner, workload, metrics, envelope, shadow, L3 pipeline tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1…6: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf PQH-1…6 tests passed"
