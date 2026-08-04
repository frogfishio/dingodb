#!/usr/bin/env bash
# CSE-3 — segment-ID never-reuse crash matrix (must run serially).
# Process-global failpoints race under default parallel test threads.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec cargo test -p residiuum-store --features legacy-raw-store \
  --test cse3_stage2_segment_id_never_reuse -- --test-threads=1 "$@"
