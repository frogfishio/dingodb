#!/usr/bin/env bash
# PQH-1 + PQH-2: safe runner + deterministic workload unit tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1/2: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf runner+workload (PQH-1/2) tests passed"
