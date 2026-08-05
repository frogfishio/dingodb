#!/usr/bin/env bash
# Decision 0 architecture gate (behavioral, not filename theatre).
# Shims forbidden; ISA must drive Core execute; envelope fields private.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_query_runtime_architecture: $*" >&2; exit 1; }

SDK_SRC="$ROOT/crates/residiuum-sdk/src"
MOD="$SDK_SRC/query_bytecode_v1/mod.rs"
[[ -d "$SDK_SRC" ]] || fail "missing $SDK_SRC"
[[ -f "$MOD" ]] || fail "missing query_bytecode_v1/mod.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/core_page.rs" ]] || fail "missing core_page.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/full_attach.rs" ]] || fail "missing full_attach.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/isa.rs" ]] || fail "missing isa.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/kernel.rs" ]] || fail "missing kernel.rs"

# Shims must be gone.
[[ ! -e "$SDK_SRC/query_exec_v1.rs" ]] || fail "query_exec_v1.rs shim must be deleted"
[[ ! -e "$SDK_SRC/rql_full_v1.rs" ]] || fail "rql_full_v1.rs shim must be deleted"

rg -q 'residiuum-query-isa-v1' "$SDK_SRC/query_bytecode_v1/isa.rs" \
  || fail "ISA_PROFILE missing"
rg -q 'residiuum-query-kernel-sda-v1' "$SDK_SRC/query_bytecode_v1/kernel.rs" \
  || fail "KERNEL_PROFILE missing"
rg -q 'residiuum-query-bytecode-v1' "$MOD" \
  || fail "BYTECODE_PROFILE missing"

# RQL-X5: envelope must not expose an independent executable plan.
if rg -n 'pub plan:' "$MOD" | rg -q .; then
  fail "QueryBytecodeV1 must not have pub plan (ISA sole input)"
fi
if rg -n 'pub isa:' "$MOD" | rg -q .; then
  fail "QueryBytecodeV1.isa must be private (use isa_bytes())"
fi
rg -q 'isa: Vec<u8>' "$MOD" || fail "QueryBytecodeV1 must store isa bytes"

# Core execute must decode ISA (not trust a side plan).
rg -n 'fn execute_bytecode' -A 25 "$MOD" | rg -q 'execute_isa_bytes|decode_isa' \
  || fail "execute_bytecode must decode/dispatch via ISA"
rg -n 'fn execute_isa_bytes' -A 40 "$MOD" | rg -q 'decode_isa' \
  || fail "execute_isa_bytes must call decode_isa"
rg -n 'fn execute_isa_bytes' -A 40 "$MOD" | rg -q 'execute_plan' \
  || fail "execute_isa_bytes must call execute_plan on decoded Core"

# Mismatch / ISA-drives-exec test must exist.
rg -q 'execute_bytecode_uses_isa_not_sidecar_plan' "$MOD" \
  || fail "missing ISA-drives-execution mismatch test"

rg -q 'compile_where' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must compile_where"
rg -q 'compile_where' "$SDK_SRC/query_bytecode_v1/full_attach.rs" \
  || fail "full_attach must compile_where"

hits="$(
  rg -n --glob '*.rs' '\.eval\(' "$SDK_SRC/query_bytecode_v1" \
    | rg -v 'kernel\.rs:' \
    | rg -v '^\s*//' \
    || true
)"
if [[ -n "$hits" ]]; then
  echo "$hits" >&2
  fail "Predicate::eval outside kernel tests"
fi

ALLOW='query_bytecode_v1/'
hits="$(
  rg -n --glob '*.rs' '^\s*pub\s+fn\s+execute_' "$SDK_SRC" \
    | rg -v "$ALLOW" \
    || true
)"
if [[ -n "$hits" ]]; then
  echo "$hits" >&2
  fail "new pub fn execute_* outside query_bytecode_v1/"
fi

rg -q 'pub trait HostCapabilities' "$MOD" || fail "HostCapabilities missing"
rg -q 'execute_core_rql' crates/residiuum-server/src/heap_dispatch.rs \
  || fail "op 118 must use execute_core_rql"

# Honesty: full path still bypasses ISA until X5b — document residual.
if ! rg -q 'RQL-X5b' doc/todo/rql/RQL_WHAT_IS_LEFT.md; then
  fail "SoT must name RQL-X5b residual for full-from-ISA"
fi

echo "check_query_runtime_architecture: OK (X5 Core ISA sole input; full residual X5b)"
