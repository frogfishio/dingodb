#!/usr/bin/env bash
# Decision 0 / RQL-X2d: one semantic runtime under query_bytecode_v1/.
# Compat shim modules query_exec_v1.rs / rql_full_v1.rs must not exist.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_query_runtime_architecture: $*" >&2; exit 1; }

SDK_SRC="$ROOT/crates/residiuum-sdk/src"
[[ -d "$SDK_SRC" ]] || fail "missing $SDK_SRC"
[[ -f "$SDK_SRC/query_bytecode_v1/mod.rs" ]] || fail "missing query_bytecode_v1/mod.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/core_page.rs" ]] || fail "missing core_page.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/full_attach.rs" ]] || fail "missing full_attach.rs"

# Shims must be gone.
[[ ! -e "$SDK_SRC/query_exec_v1.rs" ]] || fail "query_exec_v1.rs shim must be deleted"
[[ ! -e "$SDK_SRC/rql_full_v1.rs" ]] || fail "rql_full_v1.rs shim must be deleted"
[[ ! -d "$SDK_SRC/query_exec_v1" ]] || fail "query_exec_v1/ must not exist"
[[ ! -d "$SDK_SRC/rql_full_v1" ]] || fail "rql_full_v1/ must not exist"

rg -q 'residiuum-query-bytecode-v1' "$SDK_SRC/query_bytecode_v1/mod.rs" \
  || fail "BYTECODE_PROFILE missing residiuum-query-bytecode-v1"

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

# Op 118 server path must call the shared product entry.
rg -q 'execute_core_rql' crates/residiuum-server/src/heap_dispatch.rs \
  || fail "op 118 dispatch must use execute_core_rql"

echo "check_query_runtime_architecture: OK"
