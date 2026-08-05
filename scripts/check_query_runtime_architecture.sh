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
rg -q 'residiuum-query-vm-v1' "$SDK_SRC/query_bytecode_v1/vm.rs" \
  || fail "VM_PROFILE missing"
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
rg -n 'fn execute_decoded_core' -A 40 "$MOD" | rg -q 'run_vm_core|vm_exec::' \
  || fail "execute_decoded_core must dispatch via Query VM (run_vm_core)"
rg -q 'fn run_vm_core' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing run_vm_core (RQL-VM1)"
rg -q 'fn run_vm_attach' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing run_vm_attach (RQL-VM1)"
rg -q 'fn lower_core' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing lower_core (RQL-VM1)"
rg -q 'fn lower_full' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "missing lower_full (RQL-VM1)"

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
# RQL-X5c + VM1: full shares execute_decoded_core; attach via VM.
rg -n 'fn execute_full_isa_with' -A 100 "$FULL" | rg -q 'execute_decoded_core' \
  || fail "execute_full_isa_with must share execute_decoded_core"
rg -n 'fn execute_full_isa_with' -A 100 "$FULL" | rg -q 'run_vm_attach|lower_full' \
  || fail "execute_full_isa_with must dispatch Full attach via Query VM"
if rg -n 'fn execute_full_isa_with' -A 100 "$FULL" | rg -q 'encode_core_program'; then
  fail "execute_full_isa_with must not re-encode Core ISA (use execute_decoded_core)"
fi
rg -q 'execute_full_isa_enrich_within_project_nonempty' \
  crates/residiuum-sdk/tests/rql_full_isa_execute.rs \
  || fail "missing full ISA non-empty E2E test"

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

# RQL-IR4: attach helpers remain; product Full path dispatches via VM (VM1).
rg -q 'fn run_attach_pipeline' "$SDK_SRC/query_bytecode_v1/ir_attach.rs" \
  || fail "ir_attach must define run_attach_pipeline"
rg -q 'fn run_vm_attach' "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  || fail "vm_exec must define run_vm_attach (VM1 Full dispatch)"
if rg -n 'fn execute_full_isa_with' -A 120 "$FULL" | rg -q 'FullPipelineStepV1::Enrich'; then
  fail "execute_full_isa_with must not inline Enrich pipeline loop (moved to VM/IR)"
fi

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
rg -qi 'run_core_page|CoreFrame' doc/todo/rql/QUERY_VM_V1.md \
  || fail "QUERY_VM_V1 must name CoreFrame / run_core_page (VM2)"
rg -qi 'P1c' doc/todo/rql/QUERY_VM_V1.md \
  || fail "QUERY_VM_V1 must name P1c residual after VM2"
rg -q 'decode_isa_canonical' "$SDK_SRC/query_bytecode_v1/isa.rs" \
  || fail "isa must define decode_isa_canonical (D0R)"
rg -q 'open_collection_bound' "$FULL" \
  || fail "full_attach must bind collections by immutable id (D0R)"
rg -n 'fn execute_isa_bytes' -A 20 "$MOD" | rg -q 'decode_isa_canonical' \
  || fail "execute_isa_bytes must use decode_isa_canonical"
rg -n 'fn execute_full_isa_with' -A 20 "$FULL" | rg -q 'decode_isa_canonical' \
  || fail "execute_full_isa_with must use decode_isa_canonical"
# SoT must not claim NEXT is principal C1 acceptance while VM unfinished.
if rg -n '^NEXT' doc/todo/rql/RQL_WHAT_IS_LEFT.md | rg -qi 'principal.*C1'; then
  fail "SoT must not set NEXT to principal C1 while Query VM unfinished"
fi
rg -q 'struct CoreFrame' "$SDK_SRC/query_bytecode_v1/core_phases.rs" \
  || fail "missing CoreFrame (RQL-VM2)"
rg -q 'fn run_core_page' "$SDK_SRC/query_bytecode_v1/core_phases.rs" \
  || fail "missing run_core_page (RQL-VM2)"
rg -n 'fn run_vm_core' -A 80 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'index_eq' \
  || fail "run_vm_core must call CoreFrame::index_eq (RQL-VM2)"
rg -n 'fn run_vm_core' -A 120 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'project_paths' \
  || fail "run_vm_core must call CoreFrame::project_paths (RQL-VM2)"
rg -n 'fn run_vm_core' -A 120 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'f\.scan\(scan\)' \
  || fail "run_vm_core must call CoreFrame::scan(scan) (RQL-VM3)"
if rg -n 'fn run_vm_core' -A 120 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'execute_plan\('; then
  fail "run_vm_core must not call execute_plan (RQL-VM2 demotion)"
fi
rg -n 'fn execute_plan' -A 25 "$SDK_SRC/query_bytecode_v1/core_page.rs" | rg -q 'CoreFrame' \
  || fail "execute_plan must be thin CoreFrame wrapper (RQL-VM2)"

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
rg -n 'fn run_vm_core' -A 120 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'f\.filter\(scan\)' \
  || fail "run_vm_core must call CoreFrame::filter(scan) (RQL-VM3b)"
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
rg -n 'fn run_vm_attach' -A 160 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'within_stack' \
  || fail "run_vm_attach must maintain within_stack (RQL-VM4)"
rg -n 'fn run_vm_attach' -A 160 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'within_enter|within_leave' \
  || fail "run_vm_attach must call within_enter/leave (RQL-VM4)"
if rg -n 'fn run_vm_attach' -A 160 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" | rg -q 'attach_within_rows'; then
  fail "run_vm_attach must not call attach_within_rows (RQL-VM4 flatten)"
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
# run_core_page demoted to CoreFrame orchestrator (not fused body).
rg -n 'fn run_core_page' -A 35 "$CORE_PHASES" | rg -q 'frame\.scan|CoreFrame::begin|frame\.project_paths' \
  || fail "run_core_page must orchestrate CoreFrame (RQL-VM3)"

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
# (Honest: this is NOT every frontend — dialects sql/json/mongo remain → SDA.)
APP="$SDK_SRC/app_v1.rs"
DISPATCH="$ROOT/crates/residiuum-server/src/heap_dispatch.rs"
DIALECTS="$SDK_SRC/dialects/mod.rs"
[[ -f "$APP" ]] || fail "missing app_v1.rs"
[[ -f "$DISPATCH" ]] || fail "missing heap_dispatch.rs"
[[ -f "$DIALECTS" ]] || fail "missing dialects/mod.rs"

# Shared funnel: source/ISA → execute_decoded_core → run_vm_core.
rg -n 'fn execute_core_rql' -A 25 "$MOD" | rg -q 'execute_bytecode' \
  || fail "execute_core_rql must call execute_bytecode"
rg -n 'fn execute_bytecode' -A 25 "$MOD" | rg -q 'execute_isa_bytes' \
  || fail "execute_bytecode must call execute_isa_bytes"
rg -n 'fn execute_isa_bytes' -A 35 "$MOD" | rg -q 'execute_decoded_core' \
  || fail "execute_isa_bytes must call execute_decoded_core"
rg -n 'fn execute_decoded_core' -A 30 "$MOD" | rg -q 'run_vm_core' \
  || fail "execute_decoded_core must call run_vm_core"

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

# Full RQL → shared Core VM + attach VM (honest: still two loops until VM1R)
rg -n 'fn execute_full_isa_with' -A 50 "$FULL" | rg -q 'execute_decoded_core' \
  || fail "execute_full_isa_with must call execute_decoded_core"
rg -n 'fn execute_full_isa_with' -A 100 "$FULL" | rg -q 'run_vm_attach' \
  || fail "execute_full_isa_with must call run_vm_attach"

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
rg -n 'fn run_vm_attach' -A 80 "$SDK_SRC/query_bytecode_v1/vm_exec.rs" \
  | rg -q 'BTreeMap<CollectionId' \
  || fail "run_vm_attach foreign_cache must be CollectionId-keyed (RQL-R1)"
rg -qi 'R1' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must mark R1"
rg -qi 'QVM1|durable QVM|mandatory' doc/todo/rql/RQL_WHAT_IS_LEFT.md \
  || fail "SoT must name mandatory QVM1 residual"
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
rg -q 'pub\(crate\) fn execute_plan' "$SDK_SRC/query_bytecode_v1/core_page.rs" \
  || fail "execute_plan must be pub(crate) (RQL-P0b)"
rg -q 'pub\(crate\) fn execute_decoded_core' "$MOD" \
  || fail "execute_decoded_core must be pub(crate) (RQL-P0b)"

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

echo "check_query_runtime_architecture: OK (R1 dialect refuse + cache-by-id; VM0–VM4 intermediate; VM1/P1c rejected; QVM1 mandatory; Decision 0 OPEN; C1 forbidden)"
