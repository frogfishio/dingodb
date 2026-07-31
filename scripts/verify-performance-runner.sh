#!/usr/bin/env bash
# PQH-1: safe runner unit tests (path guard / marker / preflight / cancel).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "PQH-1: cargo test -p residiuum-perf --lib"
cargo test -p residiuum-perf --lib --quiet
echo "OK: residiuum-perf runner (PQH-1) tests passed"
