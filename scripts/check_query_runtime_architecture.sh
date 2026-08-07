#!/usr/bin/env bash
# Decision 0 architecture gate (behavioral, not filename theatre).
# Shims forbidden; QVM1 drives Core + full execute; RQB1/isa.rs forbidden; one run_vm.
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
[[ -f "$SDK_SRC/query_bytecode_v1/kernel.rs" ]] || fail "missing kernel.rs"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_project.rs" ]] || fail "missing ir_project.rs (RQL-IR1)"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_order.rs" ]] || fail "missing ir_order.rs (RQL-IR2)"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_page.rs" ]] || fail "missing ir_page.rs (RQL-IR3)"
[[ -f "$SDK_SRC/query_bytecode_v1/ir_attach.rs" ]] || fail "missing ir_attach.rs (RQL-IR4)"
[[ -f "$SDK_SRC/query_bytecode_v1/vm.rs" ]] || fail "missing vm.rs (RQL-VM0)"
[[ -f "$IR_DOC" ]] || fail "missing QUERY_IR_RESIDUAL.md (X5c honesty)"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_PROJECT_V1.md" ]] || fail "missing QUERY_IR_PROJECT_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_ORDER_V1.md" ]] || fail "missing QUERY_IR_ORDER_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_PAGE_V1.md" ]] || fail "missing QUERY_IR_PAGE_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_IR_ATTACH_V1.md" ]] || fail "missing QUERY_IR_ATTACH_V1.md"
[[ -f "$ROOT/doc/todo/rql/QUERY_VM_V1.md" ]] || fail "missing QUERY_VM_V1.md"

# Shims must be gone.
[[ ! -e "$SDK_SRC/query_exec_v1.rs" ]] || fail "query_exec_v1.rs shim must be deleted"
[[ ! -e "$SDK_SRC/rql_full_v1.rs" ]] || fail "rql_full_v1.rs shim must be deleted"

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
rg -q 'residiuum-query-vm-v1' "$SDK_SRC/query_bytecode_v1/vm.rs" \
  || fail "VM_PROFILE missing"
rg -q 'residiuum-query-bytecode-v1' "$MOD" \
  || fail "BYTECODE_PROFILE missing"

# Public envelope is QVM1 (sole stored executable identity).
if rg -n 'pub plan:' "$MOD" | rg -q .; then
  fail "QueryBytecodeV1 must not have pub plan (QVM sole input)"
fi
if rg -n 'pub qvm:' "$MOD" | rg -q .; then
  fail "QueryBytecodeV1.qvm must be private (use qvm_bytes())"
fi
rg -q 'qvm: Vec<u8>' "$MOD" || fail "QueryBytecodeV1 must store qvm bytes"
rg -q 'fn qvm_bytes' "$MOD" || fail "QueryBytecodeV1 must expose qvm_bytes()"

# Core execute must decode QVM (not trust a side plan).
rg -n 'fn execute_bytecode' -A 25 "$MOD" | rg -q 'execute_qvm_bytes|decode_qvm|qvm_bytes'   || fail "execute_bytecode must decode/dispatch via QVM"
rg -n 'fn execute_qvm_bytes' -A 30 "$MOD" | rg -q 'decode_qvm'   || fail "execute_qvm_bytes must call decode_qvm"
rg -n 'fn execute_qvm_bytes' -A 30 "$MOD" | rg -q 'run_vm'   || fail "execute_qvm_bytes must call run_vm"
rg -n 'fn execute_decoded_core' -A 40 "$MOD" | rg -q 'run_vm\b|vm_exec::'   || fail "execute_decoded_core must dispatch via Query VM (run_vm)"
rg -q 'fn run_vm\b' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing run_vm (RQL-VM1R)"
if rg -q 'fn run_vm_core\b' "$SDK_SRC/query_bytecode_v1/vm_exec.rs"; then
  fail "run_vm_core must be deleted (RQL-VM1R unified run_vm)"
fi
if rg -q 'fn run_vm_attach\b' "$SDK_SRC/query_bytecode_v1/vm_exec.rs"; then
  fail "run_vm_attach must be deleted (RQL-VM1R unified run_vm)"
fi
rg -q 'fn lower_core' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing lower_core (RQL-VM1)"
rg -q 'fn lower_full' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing lower_full (RQL-VM1)"

# Mismatch / ISA-drives-exec test must exist.
rg -q 'execute_bytecode_uses_isa_not_sidecar_plan' "$MOD" \
  || fail "missing ISA-drives-execution mismatch test"

# Full product path: compile → lower_full → encode_qvm → execute_full_qvm_with.
rg -q 'fn execute_full_qvm_with' "$FULL" \
  || fail "missing execute_full_qvm_with (Full QVM entry)"
rg -n 'fn execute_rql_full_with' -A 40 "$FULL" | rg -q 'encode_qvm' \
  || fail "execute_rql_full_with must encode_qvm (not RQB1 product path)"
rg -n 'fn execute_rql_full_with' -A 40 "$FULL" | rg -q 'execute_full_qvm_with' \
  || fail "execute_rql_full_with must dispatch execute_full_qvm_with"
if rg -n 'fn execute_rql_full_with' -A 40 "$FULL" | rg -q 'encode_full_program'; then
  fail "execute_rql_full_with must not encode_full_program (RQB1 demoted)"
fi
rg -n 'fn execute_full_qvm_with' -A 50 "$FULL" | rg -q 'decode_qvm' \
  || fail "execute_full_qvm_with must decode_qvm"
rg -n 'fn execute_full_qvm_with' -A 50 "$FULL" | rg -q 'run_vm\b' \
  || fail "execute_full_qvm_with must call run_vm"
# Q0.A10: RQB1 fully removed from SDK (no import/encode/execute path).
LIB_RS="$SDK_SRC/lib.rs"
if [[ -f "$SDK_SRC/query_bytecode_v1/isa.rs" ]]; then
  fail "isa.rs must be deleted (RQB1 retired Q0.A10)"
fi
# Allow historical mention only in comments that say removed — re-check hard symbols:
if rg -q 'fn from_isa_bytes|fn execute_isa_bytes|fn execute_full_isa_with|fn decode_isa|fn encode_core_program|fn encode_full_program' \
  "$SDK_SRC/query_bytecode_v1"; then
  fail "RQB1 functions must not exist under query_bytecode_v1 (Q0.A10)"
fi
if rg -n 'pub use query_bytecode_v1::' -A 25 "$LIB_RS" | rg -q 'decode_isa|encode_core|execute_isa|from_isa'; then
  fail "lib.rs must not re-export RQB1 (Q0.A10)"
fi
# Q0.A12: live normative rql docs must not claim RQB1 remains a supported product path.
# QUERY_ISA_V1.md is retired historical (banner + body); evidence/ logs are historical.
RQL_DOC_DIR="$ROOT/doc/todo/rql"
if [[ -d "$RQL_DOC_DIR" ]]; then
  while IFS= read -r -d '' md; do
    base="$(basename "$md")"
    case "$base" in
      QUERY_ISA_V1.md) continue ;;
    esac
    if rg -q 'Legacy RQB1 may lower|from_isa_bytes accepts|execute_isa_bytes \(RQB1\)|execute_full_isa_with \(RQB1\)|Legacy AST carrier — lowers into QVM|RQB1 crate-private quarantine' "$md"; then
      fail "live normative doc claims RQB1 still supported: ${md#$ROOT/} (Q0.A12)"
    fi
  done < <(find "$RQL_DOC_DIR" -maxdepth 1 -type f -name '*.md' -print0)
fi
rg -q 'execute_full_qvm_enrich_within_project_nonempty' \
  crates/residiuum-sdk/tests/rql_full_isa_execute.rs \
  || fail "missing full QVM non-empty E2E test"

CORE_PHASES="$SDK_SRC/query_bytecode_v1/core_phases.rs"
[[ -f "$CORE_PHASES" ]] || fail "missing core_phases.rs (RQL-VM2)"
rg -q 'compile_where' "$CORE_PHASES" \
  || fail "core_phases must compile_where"
rg -q 'compile_where' "$FULL" \
  || fail "full_attach must compile_where"

# RQL-IR1: Core path-project must use named IR module (not private project_doc).
rg -q 'apply_project_paths' "$CORE_PHASES" \
  || fail "core_phases must call apply_project_paths"
if rg -n 'fn project_doc' "$CORE_PHASES" "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core must not keep private project_doc (moved to ir_project)"
fi
rg -q 'fn apply_project_paths' "$SDK_SRC/query_bytecode_v1/ir_project.rs" \
  || fail "ir_project must define apply_project_paths"

# RQL-IR2: Core order/sort-tuple must use named IR module.
rg -q 'ir_order::compare_rows' "$CORE_PHASES" \
  || fail "core_phases must call ir_order::compare_rows"
rg -q 'ir_order::build_sort_tuple' "$CORE_PHASES" \
  || fail "core_phases must call ir_order::build_sort_tuple"
if rg -n 'fn compare_rows' "$CORE_PHASES" "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core must not keep private compare_rows (moved to ir_order)"
fi
rg -q 'fn compare_rows' "$SDK_SRC/query_bytecode_v1/ir_order.rs" \
  || fail "ir_order must define compare_rows"

# RQL-IR3: Core page/coverage must use named IR module.
rg -q 'ir_page::resolve_page_size' "$CORE_PHASES" \
  || fail "core_phases must call ir_page::resolve_page_size"
rg -q 'ir_page::finish_coverage' "$CORE_PHASES" \
  || fail "core_phases must call ir_page::finish_coverage"
rg -q 'ir_page::mint_page_cursor' "$CORE_PHASES" \
  || fail "core_phases must call ir_page::mint_page_cursor"
if rg -n 'fn mint_page_cursor' "$CORE_PHASES" "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core must not keep private mint_page_cursor (moved to ir_page)"
fi
rg -q 'fn finish_coverage' "$SDK_SRC/query_bytecode_v1/ir_page.rs" \
  || fail "ir_page must define finish_coverage"

# RQL-IR4: one executor — run_attach_pipeline deleted; product Full uses run_vm.
if rg -q 'fn run_attach_pipeline' "$SDK_SRC/query_bytecode_v1"; then
  fail "run_attach_pipeline must remain deleted (one executor)"
fi
rg -q 'fn run_vm\b' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "vm_exec must define run_vm (VM1R)"
# Filter is sole where authority (IndexEq has no predicate).
if rg -n 'enum VmImm' -A 40 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'IndexEq \{[^}]*where_pred'; then
  fail "IndexEq imm must not carry where_pred (Filter sole authority)"
fi
rg -n 'fn IndexEq' -A 5 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" 2>/dev/null || true
# program_hash derived from complete QVM, not trusted wire plan_hash field.
if rg -n 'struct VmPool' -A 20 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'plan_hash'; then
  fail "VmPool must not carry plan_hash (use program_hash on VmProgram from qvm_hash)"
fi
rg -q 'program_hash' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "VmProgram must carry program_hash (cursor identity)"
rg -n 'fn decode_qvm' -A 25 "$SDK_SRC/query_bytecode_v1/qvm.rs" | rg -q 'non-canonical|encode_qvm' \
  || fail "decode_qvm must enforce canonical re-encode"

# IR residual honesty.
rg -qi 'Rust IR residual' "$IR_DOC" || fail "IR residual doc must name Rust IR residual"
rg -qi 'RQL-C1 must not be accepted' "$IR_DOC" || fail "IR residual doc must forbid C1"
rg -q 'ir_project' "$IR_DOC" || fail "IR residual doc must name ir_project (IR1)"
rg -q 'ir_order' "$IR_DOC" || fail "IR residual doc must name ir_order (IR2)"
rg -q 'ir_page' "$IR_DOC" || fail "IR residual doc must name ir_page (IR3)"
rg -q 'ir_attach' "$IR_DOC" || fail "IR residual doc must name ir_attach (IR4)"
rg -q 'vm_exec' "$IR_DOC" || fail "IR residual doc must name vm_exec (VM1)"
rg -q 'RQL-C1' doc/todo/rql/RQL_WHAT_IS_LEFT.md || fail "SoT must name RQL-C1 residual"
rg -q 'QUERY_VM_V1' doc/todo/rql/RQL_WHAT_IS_LEFT.md || fail "SoT must point at QUERY_VM_V1"
rg -qi 'Query VM' doc/todo/rql/RQL_WHAT_IS_LEFT.md || fail "SoT must keep Query VM programme visible"
[[ -f "$ROOT/doc/todo/rql/QUERY_VM_V1.md" ]] || fail "missing QUERY_VM_V1.md charter"
rg -q 'OpCode' "$SDK_SRC/query_bytecode_v1/vm.rs" || fail "vm.rs must define OpCode (VM0)"
rg -q 'Scan' doc/todo/rql/QUERY_VM_V1.md || fail "QUERY_VM_V1 must name Scan"
rg -q 'Enrich' doc/todo/rql/QUERY_VM_V1.md || fail "QUERY_VM_V1 must name Enrich"
rg -q 'Within' doc/todo/rql/QUERY_VM_V1.md || fail "QUERY_VM_V1 must name Within"
# VM1 docs must keep Decision 0 / C1 forbidden (negative claims only).
rg -qi 'RQL-C1 must not be accepted' doc/todo/rql/QUERY_VM_V1.md \
  || fail "QUERY_VM_V1 must forbid RQL-C1"
rg -qi 'does \*\*not\*\* close Decision 0|does not close Decision 0|Decision 0 OPEN|Decision 0 remains' doc/todo/rql/QUERY_VM_V1.md \
  || fail "QUERY_VM_V1 must keep Decision 0 unclosed"
if rg -qi 'Decision 0 is closed|Decision 0 closed\.|RQL-C1 (is )?accepted\.|C1 accepted' doc/todo/rql/QUERY_VM_V1.md; then
  fail "QUERY_VM_V1 must not affirmatively close Decision 0 or accept C1"
fi
rg -qi 'CoreFrame' doc/todo/rql/QUERY_VM_V1.md   || fail "QUERY_VM_V1 must name CoreFrame (VM2)"
rg -qi 'P1c' doc/todo/rql/QUERY_VM_V1.md \
  || fail "QUERY_VM_V1 must name P1c residual after VM2"
rg -q 'open_collection_bound' "$FULL" \
  || fail "full_attach must bind collections by immutable id (D0R)"
# SoT must not claim NEXT is principal C1 acceptance while VM unfinished.
if rg -n '^NEXT' doc/todo/rql/RQL_WHAT_IS_LEFT.md | rg -qi 'principal.*C1'; then
  fail "SoT must not set NEXT to principal C1 while Query VM unfinished"
fi
rg -q 'struct CoreFrame' "$SDK_SRC/query_bytecode_v1/core_phases.rs"   || fail "missing CoreFrame (RQL-VM2)"
# RQL-DEL1: obsolete fused orchestrators must stay deleted.
if rg -q 'fn run_core_page' "$SDK_SRC/query_bytecode_v1"; then
  fail "run_core_page must remain deleted (RQL-DEL1)"
fi
if rg -q 'fn execute_plan' "$SDK_SRC/query_bytecode_v1"; then
  fail "execute_plan must remain deleted (RQL-DEL1)"
fi
# Typed QVM operands (no RqlPlanV1 on VmPool).
if rg -n 'struct VmPool' -A 15 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'RqlPlanV1'; then
  fail "VmPool must not embed RqlPlanV1 (typed QVM operands)"
fi
if rg -n 'enum VmImm' -A 50 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'VmImm::Core|imm: VmImm::Core|/// Core pipeline op'; then
  fail "VmImm::Core must be removed (typed operands)"
fi
rg -q 'fn verify_vm_program' "$SDK_SRC/query_bytecode_v1/vm_exec.rs"   || fail "missing verify_vm_program (QVM verifier)"
rg -q 'QVM_MAX_OPS' "$SDK_SRC/query_bytecode_v1/qvm.rs"   || fail "qvm must bound op_count (QVM_MAX_OPS)"
rg -n 'fn run_vm\b' -A 120 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'index_eq' \
  || fail "run_vm must call CoreFrame::index_eq (RQL-VM2)"
rg -n 'fn run_vm\b' -A 200 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'project_paths' \
  || fail "run_vm must call CoreFrame::project_paths (RQL-VM2)"
rg -n 'fn run_vm\b' -A 200 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'f\.scan\(' \
  || fail "run_vm must call CoreFrame::scan (RQL-VM3)"
if rg -n 'fn run_vm\b' -A 200 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'execute_plan\('; then
  fail "run_vm must not call execute_plan (RQL-VM2 demotion)"
fi
# execute_plan deleted (RQL-DEL1) — checked above.

# RQL-VM3: opcode-owned materialize bodies (not gates + fused run_core_page).
rg -n 'pub fn scan' -A 25 "$CORE_PHASES" | rg -q 'list_keys|scan_key_stream|scan_index|scan_full' \
  || fail "CoreFrame::scan must load host keys/docs (RQL-VM3)"
rg -n 'pub fn filter' -A 25 "$CORE_PHASES" | rg -q 'eval_doc|PendingKeys' \
  || fail "CoreFrame::filter must apply where (RQL-VM3)"
# RQL-VM3b: Scan must not apply where; Filter takes DocScan; PendingKeys handoff.
rg -q 'enum PendingKeys' "$CORE_PHASES" \
  || fail "missing PendingKeys (RQL-VM3b)"
if rg -n 'pub fn scan' -A 40 "$CORE_PHASES" | rg -q 'where_k\.eval_doc|filtered_during_scan'; then
  fail "CoreFrame::scan must not apply where (RQL-VM3b)"
fi
rg -n 'pub fn filter' -A 5 "$CORE_PHASES" | rg -q 'scan: &mut S' \
  || fail "CoreFrame::filter must take DocScan (RQL-VM3b)"
rg -n 'fn run_vm\b' -A 200 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'f\.filter\(' \
  || fail "run_vm must call CoreFrame::filter (RQL-VM3b)"
rg -qi 'VM3b labor closed' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark VM3b labor closed"
if rg -qi 'filtered_during_scan' doc/todo/rql/RQL_WHAT_IS_LEFT.md; then
  fail "SoT must not keep filtered_during_scan residual after VM3b"
fi
# RQL-VM4: nested Within body expands onto flat opcode stream (shell Within imm).
rg -q 'fn emit_attach_pipeline' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing emit_attach_pipeline (RQL-VM4)"
rg -q 'fn within_enter' "$SDK_SRC/query_bytecode_v1/full_attach.rs" \
  || fail "missing within_enter (RQL-VM4)"
rg -q 'fn within_leave' "$SDK_SRC/query_bytecode_v1/full_attach.rs" \
  || fail "missing within_leave (RQL-VM4)"
rg -n 'fn run_vm\b' -A 280 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'within_stack' \
  || fail "run_vm must maintain within_stack (RQL-VM4)"
rg -n 'fn run_vm\b' -A 280 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'within_enter|within_leave' \
  || fail "run_vm must call within_enter/leave (RQL-VM4)"
if rg -n 'fn run_vm\b' -A 280 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'attach_within_rows'; then
  fail "run_vm must not call attach_within_rows (RQL-VM4 flatten)"
fi
rg -qi 'VM4 labor closed|Within flatten|R1' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark VM4/R1 progress"
if rg -qi 'nested Within on imm|nested Within remains on immediates' doc/todo/rql/RQL_WHAT_IS_LEFT.md; then
  fail "SoT must not keep nested-Within-on-imm residual after VM4"
fi
rg -n 'pub fn order' -A 20 "$CORE_PHASES" | rg -q 'compare_rows' \
  || fail "CoreFrame::order must call compare_rows (RQL-VM3)"
rg -n 'pub fn page' -A 30 "$CORE_PHASES" | rg -q 'retain_after_sort_tuple|truncate' \
  || fail "CoreFrame::page must page/resume (RQL-VM3)"
rg -n 'pub fn project_paths' -A 40 "$CORE_PHASES" | rg -q 'apply_project_paths' \
  || fail "CoreFrame::project_paths must project (RQL-VM3)"
# Fused orchestrators deleted (RQL-DEL1).
if rg -q 'fn run_core_page' "$CORE_PHASES"; then
  fail "run_core_page must not exist in core_phases (RQL-DEL1)"
fi

rg -qi 'VM2–VM4 accepted as intermediate|VM2 labor closed' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must acknowledge VM2–VM4 intermediate labor"
rg -qi 'VM3 labor closed|VM2–VM4 accepted' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must acknowledge VM3 intermediate labor"
# Principal rejected P1c convergence claim (RQL-R1 honesty).
rg -qi 'P1c rejected' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark P1c rejected (principal)"
if rg -qi 'P1c labor closed' doc/todo/rql/RQL_WHAT_IS_LEFT.md; then
  fail "SoT must not claim P1c labor closed after principal reject"
fi

# Partial funnel: Application Core / builder / view / Full / op 118 → Query VM path.
# Dialects sql/json/mongo → portable → QVM (RQL-DQ1).
APP="$SDK_SRC/app_v1.rs"
DISPATCH="$ROOT/crates/residiuum-server/src/heap_dispatch.rs"
DIALECTS="$SDK_SRC/dialects/mod.rs"
[[ -f "$APP" ]] || fail "missing app_v1.rs"
[[ -f "$DISPATCH" ]] || fail "missing heap_dispatch.rs"
[[ -f "$DIALECTS" ]] || fail "missing dialects/mod.rs"

# Shared funnel: source → QVM envelope → execute_qvm_bytes → run_vm (Core);
# Full → lower_full → run_vm (same machine; RQL-VM1R).
rg -n 'fn execute_core_rql' -A 25 "$MOD" | rg -q 'execute_bytecode' \
  || fail "execute_core_rql must call execute_bytecode"
rg -n 'fn execute_bytecode' -A 25 "$MOD" | rg -q 'execute_qvm_bytes' \
  || fail "execute_bytecode must call execute_qvm_bytes"
rg -n 'fn execute_qvm_bytes' -A 30 "$MOD" | rg -q 'run_vm\b' \
  || fail "execute_qvm_bytes must call run_vm"
rg -n 'fn execute_decoded_core' -A 35 "$MOD" | rg -q 'run_vm\b' \
  || fail "execute_decoded_core must call run_vm"
# SDK CollectionClient::rql (embedded) → execute_core_rql
rg -q 'execute_core_rql' "$APP" \
  || fail "app_v1 must call execute_core_rql"
# First CollectionClient::rql body must use execute_core_rql (before ViewBound).
rg -n 'impl CollectionClient' -A 900 "$APP" | rg -n 'pub fn rql' -A 40 | head -n 45 | rg -q 'execute_core_rql' \
  || fail "CollectionClient::rql must call execute_core_rql"

# Builder CollectionQuery::run → execute_bytecode
rg -n 'impl<.*> CollectionQuery' -A 200 "$APP" | rg -n 'pub fn run' -A 40 | head -n 45 | rg -q 'execute_bytecode' \
  || fail "CollectionQuery::run must call execute_bytecode"

# ViewBoundQuery::run → execute_bytecode
rg -n 'impl<.*> ViewBoundQuery' -A 120 "$APP" | rg -n 'pub fn run' -A 40 | head -n 45 | rg -q 'execute_bytecode' \
  || fail "ViewBoundQuery::run must call execute_bytecode"

# ViewBoundCollection::rql → CollectionClient::rql (inner)
rg -n 'impl<.*> ViewBoundCollection' -A 80 "$APP" | rg -n 'pub fn rql' -A 20 | head -n 25 | rg -q 'self\.inner\.rql' \
  || fail "ViewBoundCollection::rql must delegate to CollectionClient::rql"

# Full RQL → one run_vm (RQL-VM1R)
rg -qi 'VM1R labor closed|one run_vm' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark VM1R labor closed / one run_vm"

# Op 118 server → execute_core_rql (same Core funnel)
rg -q 'execute_core_rql' "$DISPATCH" \
  || fail "op 118 heap_dispatch must call execute_core_rql"

# No alternate Core source executor that bypasses the VM.
if rg -n 'fn execute_rql\b' "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q .; then
  fail "core_page::execute_rql must be deleted (VM bypass)"
fi
# Product SDK surfaces must not call execute_plan directly.
if rg -n 'execute_plan\(' "$APP" | rg -q .; then
  fail "app_v1 must not call execute_plan"
fi
if rg -n 'execute_plan\(' "$DISPATCH" | rg -q .; then
  fail "heap_dispatch must not call execute_plan"
fi

# RQL-R1: dialect id `rql` must refuse SDA compile (no parallel RQL→SDA).
rg -n 'Self::Rql =>' -A 8 "$DIALECTS" | rg -q 'no longer compiles to SDA|RQL-R1' \
  || fail "BuiltinDialect::Rql must refuse SDA compile (RQL-R1)"
if rg -n 'Self::Rql =>' -A 3 "$DIALECTS" | rg -q 'rql::compile_rql'; then
  fail "BuiltinDialect::Rql must not call rql::compile_rql (RQL-R1)"
fi
# Legacy compiler may exist only under cfg(test).
rg -U -q '#\[cfg\(test\)\]\s*\nmod rql' "$DIALECTS" \
  || fail "legacy dialects/rql must be cfg(test) only (RQL-R1)"
# No second unconditional `mod rql` without cfg(test) on the previous line.
uncond="$(
  awk '
    /^#\[cfg\(test\)\]/ { test_next=1; next }
    /^mod rql/ {
      if (!test_next) { print NR ": unconditional mod rql"; exit 1 }
      test_next=0
      next
    }
    { test_next=0 }
  ' "$DIALECTS" || true
)"
if [[ -n "$uncond" ]]; then
  echo "$uncond" >&2
  fail "dialects/mod.rs must not unconditionally mod rql (RQL-R1)"
fi
rg -q 'find_dialect' "$SDK_SRC/collection.rs" \
  || fail "must name find_dialect surface for gate inventory"
rg -q 'pub fn sda\b|fn sda_with' "$SDK_SRC/collection.rs" \
  || fail "must name Collection::sda raw-SDA surface for gate inventory"
# Foreign cache keyed by CollectionId (not using_name).
rg -n 'fn run_vm\b' -A 80 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  | rg -q 'BTreeMap<CollectionId' \
  || fail "run_vm foreign_cache must be CollectionId-keyed (RQL-R1)"
rg -qi 'R1' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark R1"
# RQL-QVM1: durable QVM bytecode is executable authority (not optional).
rg -q 'fn encode_qvm' "$SDK_SRC/query_bytecode_v1/qvm.rs" \
  || fail "missing encode_qvm (RQL-QVM1)"
rg -q 'fn decode_qvm' "$SDK_SRC/query_bytecode_v1/qvm.rs" \
  || fail "missing decode_qvm (RQL-QVM1)"
rg -q 'fn materialize_qvm' "$SDK_SRC/query_bytecode_v1/qvm.rs" \
  || fail "missing materialize_qvm (RQL-QVM1)"
rg -q 'b"QVM1"' "$SDK_SRC/query_bytecode_v1/qvm.rs" \
  || fail "QVM magic must be QVM1"
rg -q 'struct VmPool' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing VmPool (RQL-QVM1)"
# VmProgram must not keep pipeline/project semantic sidecar fields.
if rg -n 'pub struct VmProgram' -A 20 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  | rg -q 'pipeline:|project:'; then
  fail "VmProgram must not carry pipeline/project sidecars (RQL-QVM1)"
fi
if rg -n 'pub struct VmProgram' -A 20 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  | rg -q 'pub core:'; then
  fail "VmProgram must not carry core sidecar field (RQL-QVM1); use VmPool"
fi
rg -n 'fn execute_decoded_core' -A 40 "$MOD" | rg -q 'materialize_qvm|encode_qvm|run_vm' \
  || fail "execute_decoded_core must use QVM path (RQL-QVM1)"
rg -n 'fn execute_full_qvm_with' -A 50 "$FULL" | rg -q 'run_vm\b' \
  || fail "execute_full_qvm_with must call run_vm"
# Public QVM API is byte-oriented (no public VmProgram).
if rg -n 'pub fn encode_qvm' "$SDK_SRC/query_bytecode_v1/qvm.rs" | rg -q .; then
  fail "encode_qvm must be crate-private (VmProgram is not public)"
fi
if rg -n 'pub fn decode_qvm' "$SDK_SRC/query_bytecode_v1/qvm.rs" | rg -q .; then
  fail "decode_qvm must be crate-private (VmProgram is not public)"
fi
rg -q 'pub fn validate_qvm' "$SDK_SRC/query_bytecode_v1/qvm.rs" \
  || fail "missing public validate_qvm byte API"
rg -q 'pub fn qvm_hash' "$SDK_SRC/query_bytecode_v1/qvm.rs" \
  || fail "missing public qvm_hash"
rg -qi 'QVM1 labor closed' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark QVM1 labor closed"
rg -qi 'VM1 rejected|rejected.*VM1|VM1 / P1c' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark VM1 rejected"

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
# RQL-P1b: HostCapabilities ops take CollectionId.
rg -n 'pub trait HostCapabilities' -A 25 "$MOD" | rg -q 'collection_id: CollectionId' \
  || fail "HostCapabilities must be collection-qualified (RQL-P1b)"
rg -q 'open_collection_by_id' "$SDK_SRC/app_v1.rs" \
  || fail "HeapClient must offer open_collection_by_id (RQL-P1b)"
rg -q 'impl crate::query_bytecode_v1::HostCapabilities for HeapClient' "$SDK_SRC/app_v1.rs" \
  || fail "HeapClient must implement HostCapabilities (RQL-P1b)"
rg -n 'fn load_foreign_docs_for_root_enrich' -A 8 "$FULL" | rg -q 'HostCapabilities' \
  || fail "load_foreign_docs must take HostCapabilities (RQL-P1b)"
# RQL-P0b: crate root must not re-export non-ISA semantic executors.
LIB="$SDK_SRC/lib.rs"
for forbid in execute_plan execute_decoded_core attach_enrich_rows attach_within_rows \
  apply_project_paths apply_project_rows filter_rows DocScan; do
  if rg -n "pub use query_bytecode_v1::" -A 30 "$LIB" | rg -q "\b${forbid}\b"; then
    fail "lib.rs must not publicly re-export ${forbid} (RQL-P0b)"
  fi
done
if rg -q 'fn execute_plan' "$SDK_SRC/query_bytecode_v1"; then
  fail "execute_plan must remain deleted (RQL-P0b/DEL1)"
fi
rg -q 'pub\(crate\) fn execute_decoded_core' "$MOD" \
  || fail "execute_decoded_core must be pub(crate) (RQL-P0b)"

# Public QVM identity + SDA escape closure.
rg -n 'fn from_core_plan' -A 15 "$MOD" | rg -q 'encode_qvm|from_core_plan_force_scan' \
  || fail "from_core_plan must encode QVM"
rg -n 'fn isa_hash' -A 5 "$MOD" | rg -q 'qvm_hash' \
  || fail "QueryBytecodeV1::isa_hash must hash QVM bytes"
# compile_json_value signature must be portable (not CompiledSda).
rg -n 'pub fn compile_json_value' "$DIALECTS" | rg -q 'CompiledPortable' \
  || fail "compile_json_value must return CompiledPortable"
if rg -n 'pub fn compile_json_value' "$DIALECTS" | rg -q 'CompiledSda'; then
  fail "compile_json_value must not return raw SDA"
fi
rg -n 'fn compile\(&' -A 2 "$DIALECTS" | rg -q 'CompiledPortable' \
  || fail "QueryDialect::compile must return CompiledPortable"

rg -q 'execute_core_rql' crates/residiuum-server/src/heap_dispatch.rs \
  || fail "op 118 must use execute_core_rql"

# Honesty: Decision 0 still open — C1 forbidden; IR residual named.
rg -qi 'Decision 0 OPEN' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must keep Decision 0 OPEN"
rg -qi 'RQL-C1 must not be accepted' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must forbid premature RQL-C1"
rg -q 'QUERY_IR_RESIDUAL' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must point at QUERY_IR_RESIDUAL"
rg -qi 'P1b labor closed' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark P1b labor closed"

echo "check_query_runtime_architecture: OK (R1+QVM1+WIRE1+typed ops+DEL1; VM0–VM4 intermediate; prior VM1/P1c rejected; Decision 0 OPEN; C1 forbidden)"
