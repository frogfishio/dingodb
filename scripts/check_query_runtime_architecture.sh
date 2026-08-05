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
[[ -f "$SDK_SRC/query_bytecode_v1/ir_project.rs" ]] || fail "missing ir_project.rs (RQL-IR1)"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_order.rs" ]] || fail "missing ir_order.rs (RQL-IR2)"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_page.rs" ]] || fail "missing ir_page.rs (RQL-IR3)"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_attach.rs" ]] || fail "missing ir_attach.rs (RQL-IR4)"
[[ -f "$IR_DOC" ]] || fail "missing QUERY_IR_RESIDUAL.md (X5c honesty)"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_PROJECT_V1.md" ]] || fail "missing QUERY_IR_PROJECT_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_ORDER_V1.md" ]] || fail "missing QUERY_IR_ORDER_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_PAGE_V1.md" ]] || fail "missing QUERY_IR_PAGE_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_ATTACH_V1.md" ]] || fail "missing QUERY_IR_ATTACH_V1.md"

# Shims must be gone.
[[ ! -e "$SDK_SRC/query_exec_v1.rs" ]] || fail "query_exec_v1.rs shim must be deleted"
[[ ! -e "$SDK_SRC/rql_full_v1.rs" ]] || fail "rql_full_v1.rs shim must be deleted"

rg -q 'residiuum-query-isa-v1' "$SDK_SRC/query_bytecode_v1/isa.rs" \
  || fail "ISA_PROFILE missing"
rg -q 'residiuum-query-kernel-sda-v1' "$SDK_SRC/query_bytecode_v1/kernel.rs" \
  || fail "KERNEL_PROFILE missing"
rg -q 'residiuum-query-ir-project-v1' "$SDK_SRC/query_bytecode_v1/ir_project.rs" \
  || fail "PROJECT_IR_PROFILE missing"
rg -q 'residiuum-query-ir-order-v1' "$SDK_SRC/query_bytecode_v1/ir_order.rs" \
  || fail "ORDER_IR_PROFILE missing"
rg -q 'residiuum-query-ir-page-v1' "$SDK_SRC/query_bytecode_v1/ir_page.rs" \
  || fail "PAGE_IR_PROFILE missing"
rg -q 'residiuum-query-ir-attach-v1' "$SDK_SRC/query_bytecode_v1/ir_attach.rs" \
  || fail "ATTACH_IR_PROFILE missing"
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

# RQL-IR1: Core path-project must use named IR module (not private project_doc).
rg -q 'apply_project_paths' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must call apply_project_paths"
if rg -n 'fn project_doc' "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core_page must not keep private project_doc (moved to ir_project)"
fi
rg -q 'fn apply_project_paths' "$SDK_SRC/query_bytecode_v1/ir_project.rs" \
  || fail "ir_project must define apply_project_paths"

# RQL-IR2: Core order/sort-tuple must use named IR module.
rg -q 'ir_order::compare_rows' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must call ir_order::compare_rows"
rg -q 'ir_order::build_sort_tuple' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must call ir_order::build_sort_tuple"
if rg -n 'fn compare_rows' "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core_page must not keep private compare_rows (moved to ir_order)"
fi
rg -q 'fn compare_rows' "$SDK_SRC/query_bytecode_v1/ir_order.rs" \
  || fail "ir_order must define compare_rows"

# RQL-IR3: Core page/coverage must use named IR module.
rg -q 'ir_page::resolve_page_size' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must call ir_page::resolve_page_size"
rg -q 'ir_page::finish_coverage' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must call ir_page::finish_coverage"
rg -q 'ir_page::mint_page_cursor' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "core_page must call ir_page::mint_page_cursor"
if rg -n 'fn mint_page_cursor' "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core_page must not keep private mint_page_cursor (moved to ir_page)"
fi
rg -q 'fn finish_coverage' "$SDK_SRC/query_bytecode_v1/ir_page.rs" \
  || fail "ir_page must define finish_coverage"

# RQL-IR4: full attach orchestration must use named IR module.
rg -n 'fn execute_full_isa_with' -A 100 "$FULL" | rg -q 'CompiledAttachIr' \
  || fail "execute_full_isa_with must use CompiledAttachIr"
rg -q 'fn run_attach_pipeline' "$SDK_SRC/query_bytecode_v1/ir_attach.rs" \
  || fail "ir_attach must define run_attach_pipeline"
if rg -n 'fn execute_full_isa_with' -A 120 "$FULL" | rg -q 'FullPipelineStepV1::Enrich'; then
  fail "execute_full_isa_with must not inline Enrich pipeline loop (moved to ir_attach)"
fi

# IR residual honesty.
rg -qi 'Rust IR residual' "$IR_DOC" || fail "IR residual doc must name Rust IR residual"
rg -qi 'RQL-C1 must not be accepted' "$IR_DOC" || fail "IR residual doc must forbid C1"
rg -q 'ir_project' "$IR_DOC" || fail "IR residual doc must name ir_project (IR1)"
rg -q 'ir_order' "$IR_DOC" || fail "IR residual doc must name ir_order (IR2)"
rg -q 'ir_page' "$IR_DOC" || fail "IR residual doc must name ir_page (IR3)"
rg -q 'ir_attach' "$IR_DOC" || fail "IR residual doc must name ir_attach (IR4)"
rg -q 'RQL-C1' doc/todo/rql/RQL_WHAT_IS_LEFT.md || fail "SoT must name RQL-C1 residual"

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

echo "check_query_runtime_architecture: OK (X5+X5b+X5c+IR1+IR2+IR3+IR4; Decision 0 OPEN; C1 forbidden)"
