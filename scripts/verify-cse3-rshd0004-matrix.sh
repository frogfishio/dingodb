#!/usr/bin/env bash
# CSE-3 — RSHD0004 failpoint matrix (must run serially).
#
# Process-global failpoints race under cargo's default parallel test threads
# and invalidate the evidence. Always pass --test-threads=1.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_rshd0004_matrix -- --test-threads=1 "$@"
