#!/usr/bin/env bash
# PQH-1…PQH-10: full residiuum-perf harness unit tests (+ optional store-driver).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1…10: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "PQH-10 store-driver: cargo test -p residiuum-perf --features store-driver --lib"
cargo test -p residiuum-perf --features store-driver --lib --quiet
echo "OK: residiuum-perf PQH-1…10 tests passed (default + store-driver)"