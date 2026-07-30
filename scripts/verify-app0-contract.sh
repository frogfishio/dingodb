#!/usr/bin/env bash
# APP-0 — application contract lock checks (CORE plan §13–§14).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "verify-app0-contract: FAIL: $*" >&2; exit 1; }
ok() { echo "verify-app0-contract: $*"; }

required=(
  spec/app/v1/README.md
  spec/app/v1/error_mapping_v1.json
  spec/app/v1/plan_vectors_v1.json
  spec/app/v1/cursor_vectors_v1.json
  spec/app/v1/residuals_v1.json
  spec/heap/rpc-v1/collection_create.request.json
  spec/heap/rpc-v1/collection_create.response.json
  spec/heap/rpc-v1/dql_query.request.json
  spec/heap/rpc-v1/dql_query.response.json
  spec/heap/fixtures/collection_create.accepted.json
  spec/heap/fixtures/collection_create.rejected.json
  spec/heap/fixtures/dql_query.accepted.json
  spec/heap/fixtures/dql_query.rejected.json
  crates/dingo-sdk/src/app_v1.rs
  crates/dingo-sdk/tests/app0_contract_lock.rs
  doc/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md
)

for f in "${required[@]}"; do
  [[ -f "$f" ]] || fail "missing $f"
done
ok "required paths present (${#required[@]})"

command -v python3 >/dev/null 2>&1 || fail "python3 required"

python3 - <<'PY'
import json, sys
from pathlib import Path

def load(p):
    return json.loads(Path(p).read_text(encoding="utf-8"))

# JSON parses
for p in [
    "spec/app/v1/error_mapping_v1.json",
    "spec/app/v1/plan_vectors_v1.json",
    "spec/app/v1/cursor_vectors_v1.json",
    "spec/heap/rpc-v1/collection_create.request.json",
    "spec/heap/rpc-v1/collection_create.response.json",
    "spec/heap/rpc-v1/dql_query.request.json",
    "spec/heap/rpc-v1/dql_query.response.json",
    "spec/heap/fixtures/collection_create.accepted.json",
    "spec/heap/fixtures/collection_create.rejected.json",
    "spec/heap/fixtures/dql_query.accepted.json",
    "spec/heap/fixtures/dql_query.rejected.json",
]:
    load(p)

ops = load("spec/heap/operations-v1.json")
by_id = {o["id"]: o for o in ops["operations"]}
for oid, name in ((106, "collection_create"), (118, "dql_query")):
    o = by_id.get(oid)
    if not o:
        sys.exit(f"missing operation {oid}")
    if o.get("wire_name") != name:
        sys.exit(f"op {oid} wire_name expected {name}, got {o.get('wire_name')}")
    if o.get("status") != "reserved":
        sys.exit(f"op {oid} must remain reserved during APP-0 (got {o.get('status')})")
    if o.get("request_schema") is not None or o.get("response_schema") is not None:
        sys.exit(f"reserved op {oid} must keep null schema pointers (architecture rule)")

em = load("spec/app/v1/error_mapping_v1.json")
if not em.get("required_error_codes"):
    sys.exit("error_mapping missing required_error_codes")
if not em.get("mappings"):
    sys.exit("error_mapping missing mappings")

plans = load("spec/app/v1/plan_vectors_v1.json")
if plans.get("profile") != "dql-plan-v1":
    sys.exit("plan profile must be dql-plan-v1")
if len(plans.get("vectors") or []) < 3:
    sys.exit("need ≥3 plan vectors")

cursors = load("spec/app/v1/cursor_vectors_v1.json")
if cursors.get("profile") != "dingo-cursor-v1":
    sys.exit("cursor profile must be dingo-cursor-v1")
fields = set(cursors.get("fields_required") or [])
for req in ("plan_hash", "mac", "heap_id", "collection_id"):
    if req not in fields:
        sys.exit(f"cursor fields_required missing {req}")

create = load("spec/heap/fixtures/collection_create.accepted.json")
if create.get("op") != 106 or create.get("ok") is not True:
    sys.exit("collection_create.accepted malformed")
dql = load("spec/heap/fixtures/dql_query.accepted.json")
if dql.get("op") != 118 or dql.get("ok") is not True:
    sys.exit("dql_query.accepted malformed")

src = Path("crates/dingo-sdk/src/app_v1.rs").read_text(encoding="utf-8")
for needle in (
    "pub struct HeapClient",
    "pub struct CollectionClient",
    "pub struct QueryPage",
    "dingo-rust-app-v1",
    "dql-app-core-v1",
):
    if needle not in src:
        sys.exit(f"app_v1.rs missing {needle!r}")

print("verify-app0-contract: JSON + registry + surface OK")
PY

ok "APP-0 contract lock checks passed"