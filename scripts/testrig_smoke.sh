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

echo "== dingo-testrig smoke (8 MiB target, single shard) =="
"$BIN" run \
  --work "$WORKDIR" \
  --target-bytes 8M \
  --payload-size 2048 \
  --seal-threshold 1M \
  --chaos-hits 8 \
  --chaos-bytes 64 \
  --sample-keys 32 \
  --seed 1

SHARDS_WORKDIR="${TMPDIR:-/tmp}/dingo-testrig-smoke-shards-$$"
mkdir -p "$SHARDS_WORKDIR"
trap 'rm -rf "$WORKDIR" "$SHARDS_WORKDIR"' EXIT

echo "== dingo-testrig smoke (8 MiB target, --writer-shards 4) =="
"$BIN" run \
  --work "$SHARDS_WORKDIR" \
  --target-bytes 8M \
  --payload-size 2048 \
  --seal-threshold 1M \
  --chaos-hits 8 \
  --chaos-bytes 64 \
  --sample-keys 32 \
  --seed 1 \
  --writer-shards 4

echo "smoke PASS (single-shard + 4-shard; summaries cleaned on exit)"
