#!/usr/bin/env bash
# CSQ-9: concurrency / ownership / limits / resources (DEF-101/096/020).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-9 concurrency/resources suite..."
cargo test -p residiuum-store --features legacy-raw-store --test csq9_concurrency_resources -- --nocapture || ERR=1

echo "Running DEF-101 writer-lock regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_101_writer_lock -- --nocapture || ERR=1

echo "Running DEF-096 sharded writers regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_096_sharded_writers -- --nocapture || ERR=1

echo "Running DEF-020/021 lock coverage regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_020_021_lock_coverage -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-concurrency-resources: FAILED" >&2
  exit 1
fi
echo "verify-csq-concurrency-resources: OK"
echo "  suite: csq9_concurrency_resources + DEF-101/096/020"
