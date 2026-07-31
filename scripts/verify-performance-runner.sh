#!/usr/bin/env bash
# PQH-1…PQH-8: full residiuum-perf harness unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1…8: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf PQH-1…8 tests passed"
