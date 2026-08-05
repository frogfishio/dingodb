#!/usr/bin/env bash
# Decision 0 architecture gate (behavioral, not filename theatre).
# Shims forbidden; ISA must drive Core + full execute; one post-decode Core dispatch.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_query_runtime_architecture: $*" >&2; exit 1; }

SDK_SRC="$ROOT/crates/residiuum-sdk/src"
MOD="$SDK_SRC/query_bytecode_v1/mod.rs"
FULL="$SDK_SRC/query_bytecode_v1/full_attach.rs"
IR_DOC="$ROOT/doc/todo/rql/QUERY_IR_RESIDUAL.md"
[[ -d "$SDK_SRC" ]] || fail "missing $SDK_SRC"
[[ -f "$MOD" ]] || fail "missing query_bytecode_v1/mod.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/core_page.rs" ]] || fail "missing core_page.rs"
[[ -f "$FULL" ]] || fail "missing full_attach.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/isa.rs" ]] || fail "missing isa.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/kernel.rs" ]] || fail "missing kernel.rs"
[[ -f "$IR_DOC" ]] || fail "missing QUERY_IR_RESIDUAL.md (X5c honesty)"

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
rg -n 'fn execute_isa_bytes' -A 50 "$MOD" | rg -q 'execute_decoded_core' \
  || fail "execute_isa_bytes must call execute_decoded_core"
rg -n 'fn execute_decoded_core' -A 30 "$MOD" | rg -q 'execute_plan' \
  || fail "execute_decoded_core must call execute_plan"

# Mismatch / ISA-drives-exec test must exist.
rg -q 'execute_bytecode_uses_isa_not_sidecar_plan' "$MOD" \
  || fail "missing ISA-drives-execution mismatch test"

# RQL-X5b: full path must encode→decode via ISA entry (not CompiledRqlFull authority).
rg -q 'fn execute_full_isa_with' "$FULL" \
  || fail "missing execute_full_isa_with (full ISA entry)"
rg -n 'fn execute_rql_full_with' -A 35 "$FULL" | rg -q 'encode_full_program' \
  || fail "execute_rql_full_with must encode_full_program"
rg -n 'fn execute_rql_full_with' -A 35 "$FULL" | rg -q 'execute_full_isa_with' \
  || fail "execute_rql_full_with must dispatch execute_full_isa_with"
rg -n 'fn execute_full_isa_with' -A 50 "$FULL" | rg -q 'decode_isa' \
  || fail "execute_full_isa_with must decode_isa"
# RQL-X5c: full shares execute_decoded_core (no Core re-encode bypass).
rg -n 'fn execute_full_isa_with' -A 80 "$FULL" | rg -q 'execute_decoded_core' \
  || fail "execute_full_isa_with must share execute_decoded_core"
if rg -n 'fn execute_full_isa_with' -A 80 "$FULL" | rg -q 'encode_core_program'; then
  fail "execute_full_isa_with must not re-encode Core ISA (use execute_decoded_core)"
fi
rg -q 'execute_full_isa_enrich_within_project_nonempty' \
  crates/residiuum-sdk/tests/rql_full_isa_execute.rs \
  || fail "missing full ISA non-empty E2E test"

rg -q 'compile_where' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must compile_where"
rg -q 'compile_where' "$FULL" \
  || fail "full_attach must compile_where"

# IR residual honesty (X5c).
rg -qi 'Rust IR residual' "$IR_DOC" || fail "IR residual doc must name Rust IR residual"
rg -qi 'RQL-C1 must not be accepted' "$IR_DOC" || fail "IR residual doc must forbid C1"
rg -q 'execute_decoded_core' "$IR_DOC" || fail "IR residual doc must name execute_decoded_core"

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

# Honesty: Decision 0 still open — C1 forbidden; IR residual named.
rg -qi 'Decision 0 OPEN' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must keep Decision 0 OPEN"
rg -qi 'RQL-C1 must not be accepted' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must forbid premature RQL-C1"
rg -q 'QUERY_IR_RESIDUAL' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must point at QUERY_IR_RESIDUAL"

echo "check_query_runtime_architecture: OK (X5+X5b+X5c; Decision 0 OPEN; C1 forbidden)"
