#!/usr/bin/env bash
# Fast smoke for the three-prong store testrig (small target, not 1G).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BIN="${RESIDUUM_TESTRIG_BIN:-}"
if [[ -z "$BIN" ]]; then
  cargo build -q -p residuum-testrig
  BIN="$ROOT/target/debug/residuum-testrig"
fi

WORKDIR="${TMPDIR:-/tmp}/residuum-testrig-smoke-$$"
mkdir -p "$WORKDIR"
trap 'rm -rf "$WORKDIR"' EXIT

echo "== residuum-testrig smoke (8 MiB target, single shard) =="
"$BIN" run \
  --work "$WORKDIR" \
  --target-bytes 8M \
  --payload-size 2048 \
  --seal-threshold 1M \
  --chaos-hits 8 \
  --chaos-bytes 64 \
  --sample-keys 32 \
  --seed 1

SHARDS_WORKDIR="${TMPDIR:-/tmp}/residuum-testrig-smoke-shards-$$"
mkdir -p "$SHARDS_WORKDIR"
trap 'rm -rf "$WORKDIR" "$SHARDS_WORKDIR"' EXIT

echo "== residuum-testrig smoke (8 MiB target, --writer-shards 4) =="
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

STORES_WORKDIR="${TMPDIR:-/tmp}/residuum-testrig-smoke-stores-$$"
mkdir -p "$STORES_WORKDIR"
trap 'rm -rf "$WORKDIR" "$SHARDS_WORKDIR" "$STORES_WORKDIR"' EXIT

echo "== residuum-testrig smoke (8 MiB target, --stores 2 multi-process Axis C) =="
"$BIN" run \
  --work "$STORES_WORKDIR" \
  --target-bytes 8M \
  --payload-size 2048 \
  --seal-threshold 1M \
  --chaos-hits 4 \
  --chaos-bytes 64 \
  --sample-keys 16 \
  --seed 1 \
  --stores 2

echo "smoke PASS (single-shard + 4-shard + 2-store multi-process; summaries cleaned on exit)"
