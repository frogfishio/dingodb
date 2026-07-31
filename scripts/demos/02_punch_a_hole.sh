#!/usr/bin/env bash
# Demo 2 — “Punch a hole” (Stage 2 / human milestone §8.2)
#
# Create a store, seal data, corrupt a sealed segment mid-file, then show that
# doctor/salvage still report survivors instead of a silent total loss.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

BIN="${RESIDIUUM_BIN:-}"
if [[ -z "$BIN" ]]; then
  cargo build -q -p residiuum-cli --bin residiuum
  BIN="$ROOT/target/debug/residiuum"
fi

WORKDIR="${TMPDIR:-/tmp}/residiuum-demo-punch-$$"
mkdir -p "$WORKDIR"
STORE="$WORKDIR/app.dingo"
trap 'rm -rf "$WORKDIR"' EXIT

echo "== put two documents =="
"$BIN" put "$STORE" users/alice --json '{"name":"Alice","ok":true}'
"$BIN" put "$STORE" users/bob --json '{"name":"Bob","ok":true}'

echo "== seal active segment (via compact / reopen path not required; put durable) =="
# Force a sealed segment by compacting live state when available.
if "$BIN" --help 2>/dev/null | grep -q compact; then
  "$BIN" compact "$STORE" 2>/dev/null || true
fi

# Find a sealed .dingo under segments/ (or active if only one file).
SEG=$(find "$STORE/segments" -name '*.dingo' -type f 2>/dev/null | head -1 || true)
if [[ -z "$SEG" ]]; then
  # Fall back: corrupt active journal if nothing sealed yet.
  SEG=$(find "$STORE" -name '*.dingo' -type f | head -1)
fi
echo "target segment: $SEG"
SIZE=$(wc -c <"$SEG" | tr -d ' ')
if [[ "$SIZE" -lt 128 ]]; then
  echo "segment too small to punch meaningfully (size=$SIZE); skipping corrupt"
  exit 0
fi

echo "== punch a hole (overwrite middle 64 bytes with zeros) =="
# Portable: use dd with seek into the file.
OFF=$((SIZE / 2))
dd if=/dev/zero of="$SEG" bs=1 seek="$OFF" count=64 conv=notrunc status=none

echo "== doctor (read-only; should still run) =="
"$BIN" doctor "$STORE" || true

echo "== get survivors (some keys may still be readable depending on damage) =="
"$BIN" get "$STORE" users/alice || echo "(alice unreadable — expected if its frames were hit)"
"$BIN" get "$STORE" users/bob || echo "(bob unreadable — expected if its frames were hit)"

echo "== salvage to a new path (source never mutated by salvage) =="
OUT="$WORKDIR/recovered.dingo"
"$BIN" salvage "$STORE" --output "$OUT"
"$BIN" list "$OUT" || true
echo "demo complete: workdir was $WORKDIR (cleaned on exit)"
