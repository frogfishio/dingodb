#!/usr/bin/env bash
# Demo 3 — “Database that survives” (Stage 3–4 / human milestone §8.3)
#
# Put data, wipe derived catalogs/indexes, salvage to a new path, and show
# live values still readable. Source store is never mutated by salvage.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

BIN="${RESIDUUM_BIN:-}"
if [[ -z "$BIN" ]]; then
  cargo build -q -p residuum-cli --bin residuum
  BIN="$ROOT/target/debug/residuum"
fi

WORKDIR="${TMPDIR:-/tmp}/residuum-demo-salvage-$$"
mkdir -p "$WORKDIR"
SRC="$WORKDIR/src.dingo"
DST="$WORKDIR/dst.dingo"
trap 'rm -rf "$WORKDIR"' EXIT

echo "== seed store =="
"$BIN" put "$SRC" users/alice --json '{"name":"Alice"}'
"$BIN" put "$SRC" users/bob --json '{"name":"Bob"}'
"$BIN" put "$SRC" notes/1 --json '{"body":"keep me"}'

echo "== wipe derived state (catalogs/indexes) =="
for name in catalogs indexes snapshots; do
  if [[ -d "$SRC/$name" ]]; then
    rm -rf "$SRC/$name"
    echo "removed $name/"
  fi
done

echo "== salvage (source immutable) =="
"$BIN" salvage "$SRC" --output "$DST"

echo "== source still readable =="
"$BIN" get "$SRC" users/alice
echo "== destination recovered =="
"$BIN" get "$DST" users/bob
"$BIN" get "$DST" notes/1
echo "demo complete"
