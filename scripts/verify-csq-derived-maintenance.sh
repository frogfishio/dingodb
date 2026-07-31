#!/usr/bin/env bash
# CSQ-8: derived state / maintenance / backup / migration (DEF-102/050/051/052/024).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-8 derived/maintenance suite..."
cargo test -p residiuum-store --features legacy-raw-store --test csq8_derived_maintenance -- --nocapture || ERR=1

echo "Running DEF-102 primary-cache diagnostics regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_102_primary_cache_diag -- --nocapture || ERR=1

echo "Running DEF-050 backup regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_050_backup -- --nocapture || ERR=1

echo "Running DEF-051 scrub regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_051_scrub -- --nocapture || ERR=1

echo "Running DEF-052 migrate regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_052_migrate -- --nocapture || ERR=1

echo "Running DEF-024 compaction regression..."
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_024_compaction -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-derived-maintenance: FAILED" >&2
  exit 1
fi
echo "verify-csq-derived-maintenance: OK"
echo "  suite: csq8_derived_maintenance + DEF-102/050/051/052/024"
