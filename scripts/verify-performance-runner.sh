#!/usr/bin/env bash
# PQH-1…PQH-9: full residiuum-perf harness unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1…9: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf PQH-1…9 tests passed"