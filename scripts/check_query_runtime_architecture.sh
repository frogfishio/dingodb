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
[[ -f "$SDK_SRC/query_bytecode_v1/isa.rs" ]] || fail "missing isa.rs (RQL-X3)"
[[ -f "$SDK_SRC/query_bytecode_v1/kernel.rs" ]] || fail "missing kernel.rs (RQL-X4)"

# Shims must be gone.
[[ ! -e "$SDK_SRC/query_exec_v1.rs" ]] || fail "query_exec_v1.rs shim must be deleted"
[[ ! -e "$SDK_SRC/rql_full_v1.rs" ]] || fail "rql_full_v1.rs shim must be deleted"
[[ ! -d "$SDK_SRC/query_exec_v1" ]] || fail "query_exec_v1/ must not exist"
[[ ! -d "$SDK_SRC/rql_full_v1" ]] || fail "rql_full_v1/ must not exist"

rg -q 'residiuum-query-isa-v1' "$SDK_SRC/query_bytecode_v1/isa.rs" \
  || fail "ISA_PROFILE missing residiuum-query-isa-v1"
rg -q 'pub fn encode_core_program' "$SDK_SRC/query_bytecode_v1/isa.rs" \
  || fail "encode_core_program missing"
rg -q 'pub fn decode_isa' "$SDK_SRC/query_bytecode_v1/isa.rs" \
  || fail "decode_isa missing"
rg -q 'pub isa:' "$SDK_SRC/query_bytecode_v1/mod.rs" \
  || fail "QueryBytecodeV1.isa field missing"
rg -q 'residiuum-query-kernel-sda-v1' "$SDK_SRC/query_bytecode_v1/kernel.rs" \
  || fail "KERNEL_PROFILE missing"
rg -q 'compile_where' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must compile_where (SDA kernel)"

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
