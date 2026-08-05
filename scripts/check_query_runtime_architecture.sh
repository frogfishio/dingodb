#!/usr/bin/env bash
# Decision 0 / RQL-X2c: forbid additional production semantic query executors.
# Allowlist: query_bytecode_v1/** only for pub fn execute_*.
# Shims query_exec_v1 + rql_full_v1 must re-export only.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_query_runtime_architecture: $*" >&2; exit 1; }

SDK_SRC="$ROOT/crates/residiuum-sdk/src"
[[ -d "$SDK_SRC" ]] || fail "missing $SDK_SRC"
[[ -f "$SDK_SRC/query_bytecode_v1/mod.rs" ]] || fail "missing query_bytecode_v1/mod.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/core_page.rs" ]] || fail "missing core_page.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/full_attach.rs" ]] || fail "missing full_attach.rs"

rg -q 'residiuum-query-bytecode-v1' "$SDK_SRC/query_bytecode_v1/mod.rs" \
  || fail "BYTECODE_PROFILE missing residiuum-query-bytecode-v1"

for shim in query_exec_v1.rs rql_full_v1.rs; do
  if rg -n '^\s*pub\s+fn\s+execute_' "$SDK_SRC/$shim" 2>/dev/null; then
    fail "$shim must not define pub fn execute_* (shim re-exports only)"
  fi
  rg -q 'pub use crate::query_bytecode_v1' "$SDK_SRC/$shim" \
    || fail "$shim must re-export from query_bytecode_v1"
done

ALLOW='query_bytecode_v1/'
hits="$(
  rg -n --glob '*.rs' '^\s*pub\s+fn\s+execute_' "$SDK_SRC" \
    | rg -v "$ALLOW" \
    || true
)"
if [[ -n "$hits" ]]; then
  echo "$hits" >&2
  fail "new pub fn execute_* outside query_bytecode_v1/ (Decision 0)"
fi

rg -q 'pub trait HostCapabilities' "$SDK_SRC/query_bytecode_v1/mod.rs" \
  || fail "HostCapabilities trait missing"

echo "check_query_runtime_architecture: OK"
