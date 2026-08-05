#!/usr/bin/env bash
# Decision 0 / RQL-X2: forbid additional production semantic query executors.
# Allowlist: query_bytecode_v1 (product entry) + frozen migration modules.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_query_runtime_architecture: $*" >&2; exit 1; }

SDK_SRC="$ROOT/crates/residiuum-sdk/src"
[[ -d "$SDK_SRC" ]] || fail "missing $SDK_SRC"
[[ -f "$SDK_SRC/query_bytecode_v1.rs" ]] || fail "missing query_bytecode_v1.rs (product runtime)"

# Product entry must export the frozen profile id.
rg -q 'residiuum-query-bytecode-v1' "$SDK_SRC/query_bytecode_v1.rs" \
  || fail "BYTECODE_PROFILE missing residiuum-query-bytecode-v1"

# New pub execute_* semantic entrypoints outside the allowlist are forbidden.
ALLOW='query_bytecode_v1\.rs|query_exec_v1\.rs|rql_full_v1\.rs'
hits="$(
  rg -n --glob '*.rs' '^\s*pub\s+fn\s+execute_' "$SDK_SRC" \
    | rg -v "$ALLOW" \
    || true
)"
if [[ -n "$hits" ]]; then
  echo "$hits" >&2
  fail "new pub fn execute_* outside allowlist (Decision 0: one semantic runtime)"
fi

# HostCapabilities trait must remain the host boundary name.
rg -q 'pub trait HostCapabilities' "$SDK_SRC/query_bytecode_v1.rs" \
  || fail "HostCapabilities trait missing"

echo "check_query_runtime_architecture: OK"
