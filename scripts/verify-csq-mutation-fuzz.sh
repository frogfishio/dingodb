#!/usr/bin/env bash
# CSQ-10: mutation kill catalog + fuzz property bar ownership (DEF-091-F).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-10 mutant-kill suite..."
cargo test -p residiuum-store --features legacy-raw-store --test csq10_mutation_fuzz -- --nocapture || ERR=1

echo "Running DEF-091-F fuzz property bar (cargo-fuzz optional)..."
# PR-safe property bar; skip long cargo-fuzz unless explicitly enabled.
RESIDIUUM_FUZZ_SKIP_CARGO_FUZZ="${RESIDIUUM_FUZZ_SKIP_CARGO_FUZZ:-1}" \
  bash ./scripts/fuzz-smoke.sh || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-mutation-fuzz: FAILED" >&2
  exit 1
fi
echo "verify-csq-mutation-fuzz: OK"
echo "  suite: csq10_mutation_fuzz + fuzz-smoke property bar"
