#!/usr/bin/env bash
# CSQ-6: chunk / large-value qualification (DEF-098 + DEF-103 linkage).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-6 chunk/large-value suite..."
cargo test -p residiuum-store --features legacy-raw-store --test csq6_chunk_large_value -- --nocapture || ERR=1

# Permanent regression authorities remain linked.
echo "Running DEF-098 chunk generation regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_098_chunk_generation -- --nocapture || ERR=1

echo "Running DEF-103 large-value policy regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_103_large_value_policy -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-chunk-large-value: FAILED" >&2
  exit 1
fi
echo "verify-csq-chunk-large-value: OK"
echo "  suite: csq6_chunk_large_value + DEF-098 + DEF-103"
