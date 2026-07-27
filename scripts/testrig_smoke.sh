#!/usr/bin/env bash
# Fast smoke for the three-prong store testrig (small target, not 1G).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="${DINGO_TESTRIG_BIN:-}"
if [[ -z "$BIN" ]]; then
  cargo build -q -p dingo-testrig
  BIN="$ROOT/target/debug/dingo-testrig"
fi

WORKDIR="${TMPDIR:-/tmp}/dingo-testrig-smoke-$$"
mkdir -p "$WORKDIR"
trap 'rm -rf "$WORKDIR"' EXIT

echo "== dingo-testrig smoke (8 MiB target) =="
"$BIN" run \
  --work "$WORKDIR" \
  --target-bytes 8M \
  --payload-size 2048 \
  --seal-threshold 1M \
  --chaos-hits 8 \
  --chaos-bytes 64 \
  --sample-keys 32 \
  --seed 1

echo "smoke PASS (summary under $WORKDIR before cleanup)"
