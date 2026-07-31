#!/usr/bin/env bash
# CSQ-1: dependency firewall for independent oracles.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ERR=0
fail() { echo "FAIL: $*" >&2; ERR=1; }

MODEL_TOML="$ROOT/crates/residiuum-store-model/Cargo.toml"
READER_TOML="$ROOT/tools/core-storage-reference-reader/Cargo.toml"

check_toml() {
  local f="$1"
  if [[ ! -f "$f" ]]; then
    fail "missing $f"
    return
  fi
  # Ban production crates as dependencies (not our own package name).
  if grep -E '^\s*(residiuum-store|residiuum-format|residiuum-examine|residiuum-heap)\s*=' "$f" >/dev/null; then
    fail "oracle Cargo.toml must not depend on production store/format crates: $f"
  fi
  if grep -E 'path\s*=\s*".*residiuum-store"' "$f" | grep -v store-model >/dev/null 2>&1; then
    fail "oracle Cargo.toml path-dep on residiuum-store: $f"
  fi
}

check_toml "$MODEL_TOML"
check_toml "$READER_TOML"

# Source-level import ban (crate names, not package dir names)
if rg -n 'use residiuum_store::|use residiuum_format::|extern crate residiuum_store|extern crate residiuum_format' \
  "$ROOT/crates/residiuum-store-model" --glob '*.rs' >/dev/null 2>&1; then
  fail "residiuum-store-model sources import production store/format"
fi
if rg -n 'use residiuum_store::|use residiuum_format::|extern crate residiuum_store|extern crate residiuum_format' \
  "$ROOT/tools/core-storage-reference-reader" --glob '*.rs' >/dev/null 2>&1; then
  fail "reference-reader sources import production store/format"
fi

if command -v cargo >/dev/null; then
  # Only match exact package name lines from cargo tree
  if cargo tree -p residiuum-store-model --prefix none -e normal 2>/dev/null \
    | awk '{print $1}' | grep -qx 'residiuum-store'; then
    fail "cargo tree: residiuum-store-model pulls residiuum-store"
  fi
  if cargo tree -p residiuum-store-model --prefix none -e normal 2>/dev/null \
    | awk '{print $1}' | grep -qx 'residiuum-format'; then
    fail "cargo tree: residiuum-store-model pulls residiuum-format"
  fi
  if cargo tree -p core-storage-reference-reader --prefix none -e normal 2>/dev/null \
    | awk '{print $1}' | grep -qx 'residiuum-store'; then
    fail "cargo tree: reference-reader pulls residiuum-store"
  fi
  if cargo tree -p core-storage-reference-reader --prefix none -e normal 2>/dev/null \
    | awk '{print $1}' | grep -qx 'residiuum-format'; then
    fail "cargo tree: reference-reader pulls residiuum-format"
  fi
fi

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-oracle-firewall: FAILED" >&2
  exit 1
fi
echo "verify-csq-oracle-firewall: OK"
