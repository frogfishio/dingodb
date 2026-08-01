#!/usr/bin/env bash
# APB-0 — application baseline contract freeze checks (MUST_ADD §4).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "verify-app-baseline-contract: FAIL: $*" >&2; exit 1; }
ok() { echo "verify-app-baseline-contract: $*"; }

required=(
  spec/app/baseline-v1/README.md
  spec/app/baseline-v1/operations-v1.json
  spec/app/baseline-v1/outcomes-v1.json
  spec/app/baseline-v1/projections-v1.json
  spec/app/baseline-v1/capabilities-v1.schema.json
  spec/app/baseline-v1/types-v1.json
  spec/app/baseline-v1/fixtures/capabilities.accepted.json
  spec/app/baseline-v1/fixtures/capabilities.rejected.json
  spec/app/baseline-v1/fixtures/outcomes.completeness.json
  spec/app/v1/error_mapping_v1.json
  spec/heap/operations-v1.json
  doc/todo/application-baseline/MUST_ADD.md
  doc/todo/application-baseline/APB_QUERY_ATOMICS_SEQUENCE.md
)

for f in "${required[@]}"; do
  [[ -f "$f" ]] || fail "missing $f"
done
ok "required paths present (${#required[@]})"

command -v python3 >/dev/null 2>&1 || fail "python3 required"

python3 - <<'PY'
import json
import sys
from pathlib import Path

def load(p):
    return json.loads(Path(p).read_text(encoding="utf-8"))

# --- load core docs ---
ops_doc = load("spec/app/baseline-v1/operations-v1.json")
out_doc = load("spec/app/baseline-v1/outcomes-v1.json")
proj_doc = load("spec/app/baseline-v1/projections-v1.json")
cap_schema = load("spec/app/baseline-v1/capabilities-v1.schema.json")
types_doc = load("spec/app/baseline-v1/types-v1.json")
em = load("spec/app/v1/error_mapping_v1.json")
heap = load("spec/heap/operations-v1.json")
acc = load("spec/app/baseline-v1/fixtures/capabilities.accepted.json")
rej = load("spec/app/baseline-v1/fixtures/capabilities.rejected.json")
compl = load("spec/app/baseline-v1/fixtures/outcomes.completeness.json")

# --- profile / freeze status ---
for doc, path in [
    (ops_doc, "operations"),
    (out_doc, "outcomes"),
    (proj_doc, "projections"),
    (types_doc, "types"),
]:
    if doc.get("profile") != "residiuum-application-baseline-v1":
        sys.exit(f"{path}: bad profile {doc.get('profile')!r}")
    if doc.get("package") != "APB-0":
        sys.exit(f"{path}: package must be APB-0")
    if doc.get("status") not in ("frozen", "draft"):
        sys.exit(f"{path}: unexpected status {doc.get('status')!r}")

if cap_schema.get("additionalProperties") is not False:
    sys.exit("capabilities schema must set additionalProperties false")
for req in ("profile", "heap_id", "semantic_profiles", "limits", "operations_advertised", "backends"):
    if req not in cap_schema.get("required", []):
        sys.exit(f"capabilities schema missing required {req}")

# --- operations: APB-1..12 coverage, unique ids, wire resolve ---
ops = ops_doc["operations"]
if len(ops) < 40:
    sys.exit(f"operations: expected >=40 app ops, got {len(ops)}")
app_ids = [o["app_id"] for o in ops]
if len(app_ids) != len(set(app_ids)):
    sys.exit("operations: duplicate app_id")
pkgs = {o["must_add_package"] for o in ops}
need = {f"APB-{i}" for i in range(1, 13)}
if pkgs != need:
    sys.exit(f"operations: package coverage {sorted(pkgs)} != {sorted(need)}")

heap_by_id = {o["id"]: o for o in heap["operations"]}
for o in ops:
    for w in o.get("wire", []):
        hid = w["id"]
        if hid not in heap_by_id:
            sys.exit(f"operations: unknown wire id {hid} on {o['app_id']}")
        if w.get("wire_name") != heap_by_id[hid].get("wire_name"):
            sys.exit(f"operations: wire_name mismatch for {hid}")
        if w.get("wire_status") != heap_by_id[hid].get("status"):
            sys.exit(f"operations: wire_status mismatch for {hid}")

# honesty: rql remains reserved product
rql = next(o for o in ops if o["app_id"] == "apb.collection.rql")
if not rql["wire"] or rql["wire"][0]["id"] != 118:
    sys.exit("rql must bind wire 118")
if rql["wire"][0]["wire_status"] != "reserved":
    sys.exit("rql wire 118 must be reserved until APP-7/APB-7")
if "forbidden" not in rql.get("product_claim", ""):
    sys.exit("rql product_claim must forbid product until active")

# --- outcomes: total ErrorCode set + projections ---
codes = set(out_doc["public_error_codes"])
req_codes = set(em["required_error_codes"])
if codes != req_codes:
    sys.exit(f"outcomes public_error_codes mismatch required: {codes ^ req_codes}")

mapped = set()
for m in em["mappings"]:
    c = m["code"]
    if isinstance(c, list):
        mapped.update(c)
    else:
        mapped.add(c)
if req_codes - mapped:
    sys.exit(f"error_mapping missing codes: {req_codes - mapped}")

kinds = {k["kind"] for k in out_doc["public_success_kinds"]}
lids = []
for p in out_doc["projections"]:
    lids.append(p["lower_id"])
    pub = p["public"]
    if "kind" in pub:
        if pub["kind"] not in kinds:
            sys.exit(f"unknown success kind {pub['kind']}")
    else:
        c = pub["code"]
        cs = set(c) if isinstance(c, list) else {c}
        if not cs <= codes:
            sys.exit(f"unknown error code(s) {cs - codes}")
if len(lids) != len(set(lids)):
    sys.exit("outcomes: duplicate lower_id")
if set(compl["required_lower_ids"]) != set(lids):
    sys.exit("outcomes.completeness fixture out of sync with projections")
if set(compl["required_error_codes"]) != codes:
    sys.exit("outcomes.completeness error codes out of sync")

# bulk kinds present
for bk in ("bulk_committed", "bulk_rejected", "bulk_uncertain", "bulk_not_attempted"):
    if bk not in kinds:
        sys.exit(f"missing bulk kind {bk}")

# --- projections parity ---
if len(proj_doc["rules"]) < 10:
    sys.exit("projections: expected >=10 parity rules")
proj_ops = proj_doc["operations"]
if {o["app_id"] for o in proj_ops} != set(app_ids):
    sys.exit("projections ops set != operations app_ids")
rule_ids = [r["rule_id"] for r in proj_doc["rules"]]
if len(rule_ids) != len(set(rule_ids)):
    sys.exit("projections: duplicate rule_id")
if "PAR-001" not in rule_ids or "PAR-005" not in rule_ids:
    sys.exit("projections: missing core PAR rules")

# --- types freeze ---
need_types = {
    "ApiVersion", "OperationId", "Receipt", "Coverage", "ReadView",
    "ContinuationCursor", "ChangeCheckpoint", "JobHandle", "HistoryRef", "RecoveryHandle",
}
if not need_types <= set(types_doc["types"]):
    sys.exit(f"types missing {need_types - set(types_doc['types'])}")
for d in ("DEF-099", "DEF-100"):
    if d not in types_doc.get("def_bindings", {}):
        sys.exit(f"types missing def_bindings {d}")

# --- capabilities fixtures (manual schema-ish checks; no jsonschema dep) ---
def check_cap_shape(doc, *, accept: bool):
    if accept:
        if doc.get("profile") != "residiuum-application-baseline-v1":
            sys.exit("accepted capabilities: bad profile")
        if not doc.get("heap_id"):
            sys.exit("accepted capabilities: empty heap_id")
        if not doc.get("backends"):
            sys.exit("accepted capabilities: empty backends")
        for b in doc["backends"]:
            if b not in ("embedded", "remote"):
                sys.exit(f"accepted capabilities: bad backend {b}")
        lim = doc.get("limits") or {}
        for k in ("max_page_items", "max_json_depth", "max_bulk_items"):
            if not isinstance(lim.get(k), int) or lim[k] < 1:
                sys.exit(f"accepted capabilities: bad limit {k}")
        for aid in doc.get("operations_advertised", []):
            if aid not in set(app_ids):
                sys.exit(f"accepted capabilities: unknown app_id {aid}")
    else:
        # rejected must fail at least one gate
        bad = (
            not doc.get("heap_id")
            or not doc.get("backends")
            or any(a not in set(app_ids) for a in doc.get("operations_advertised", []))
            or (doc.get("limits") or {}).get("max_page_items", 0) < 1
        )
        if not bad:
            sys.exit("rejected capabilities fixture unexpectedly valid")

check_cap_shape(acc, accept=True)
check_cap_shape(rej, accept=False)

# extraProperties not allowed on schema; accepted fixture keys ⊆ schema properties ∪ notes already in schema
schema_props = set(cap_schema["properties"])
extra = set(acc.keys()) - schema_props
if extra:
    sys.exit(f"accepted capabilities has undeclared keys {extra}")

print("operations", len(ops))
print("outcomes_projections", len(out_doc["projections"]))
print("parity_rules", len(proj_doc["rules"]))
print("types", len(types_doc["types"]))
print("error_mappings", len(em["mappings"]))
PY

ok "JSON structure + cross-links + fixtures OK"

# Freeze gate: docs must be frozen for accept path when FREEZE=1 (set by us after checks)
# Default: allow draft during development; --require-frozen for scoreboard accept.
if [[ "${1:-}" == "--require-frozen" ]]; then
  python3 - <<'PY'
import json, sys
from pathlib import Path
for p in [
    "spec/app/baseline-v1/operations-v1.json",
    "spec/app/baseline-v1/outcomes-v1.json",
    "spec/app/baseline-v1/projections-v1.json",
    "spec/app/baseline-v1/types-v1.json",
]:
    st = json.loads(Path(p).read_text())["status"]
    if st != "frozen":
        sys.exit(f"{p} status must be frozen for accept (got {st})")
print("all contract docs frozen")
PY
  ok "require-frozen satisfied"
fi

ok "APP baseline contract checks passed"
