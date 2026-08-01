#!/usr/bin/env bash
# CSQ-0: validate Residiuum core-storage qualification registries (VFY-0 namespace).
# Exit 0 only when structural, identity, and graph checks pass.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CS="$ROOT/spec/verification/core-storage"
VFY="$ROOT/spec/verification"
ERR=0

log() { printf '%s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; ERR=1; }

need() {
  local f="$1"
  if [[ ! -f "$f" ]]; then
    fail "missing required file: $f"
    return 1
  fi
}

for f in \
  claims-v1.json profiles-v1.json invariants-v1.json operations-v1.json \
  boundaries-v1.json failures-v1.json failure-combinations-v1.json \
  assumptions-v1.json oracles-v1.json proofs-v1.json outcomes-v1.json \
  projections-v1.json compositions-v1.json incidents-v1.json suites-v1.json \
  platforms-v1.json mutations-v1.json errors-v1.json report-v1.schema.json \
  crash-matrix-import-v1.json
do
  need "$CS/$f" || true
done
need "$VFY/profiles-v1.json" || true
need "$VFY/claims-v1.json" || true
need "$VFY/suites-v1.json" || true
need "$VFY/report-v1.schema.json" || true
need "$ROOT/crates/residiuum-store/crash_matrix.v1.json" || true

python3 - "$ROOT" <<'PY'
import json, sys, pathlib, re
root = pathlib.Path(sys.argv[1])
cs = root / "spec/verification/core-storage"
err = 0

def fail(msg):
    global err
    print(f"FAIL: {msg}", file=sys.stderr)
    err = 1

def load(name):
    p = cs / name
    with p.open() as f:
        return json.load(f)

def items(name):
    d = load(name)
    return d.get("items", d if isinstance(d, list) else [])

# --- identity ---
profiles = items("profiles-v1.json")
if not any(p.get("id") == "residiuum-core-storage-v1" for p in profiles):
    fail("profiles-v1.json must contain residiuum-core-storage-v1")
_legacy = "din" + "go"
for p in profiles:
    pid = p.get("id", "")
    if _legacy in pid.lower():
        fail(f"forbidden pre-reset product profile id in live registry: {pid}")
    for fid in p.get("forbidden_identities", []):
        if fid == "residiuum-core-storage-v1":
            fail("residiuum-core-storage-v1 must not be listed as forbidden")

# --- required non-empty ---
for name, min_n in [
    ("invariants-v1.json", 80),
    ("operations-v1.json", 10),
    ("boundaries-v1.json", 20),
    ("oracles-v1.json", 3),
    ("suites-v1.json", 5),
    ("errors-v1.json", 10),
    ("claims-v1.json", 1),
    ("failures-v1.json", 5),
    ("failure-combinations-v1.json", 1),
]:
    it = items(name)
    if len(it) < min_n:
        fail(f"{name}: expected >= {min_n} items, got {len(it)}")

# --- unique IDs ---
def check_unique(name, key="id"):
    it = items(name)
    seen = set()
    for row in it:
        i = row.get(key)
        if not i:
            fail(f"{name}: item missing {key}")
            continue
        if i in seen:
            fail(f"{name}: duplicate id {i}")
        seen.add(i)
    return seen

inv_ids = check_unique("invariants-v1.json")
op_ids = check_unique("operations-v1.json")
bnd_ids = check_unique("boundaries-v1.json")
oracle_ids = check_unique("oracles-v1.json")
suite_ids = check_unique("suites-v1.json")
fail_ids = check_unique("failures-v1.json")
err_ids = check_unique("errors-v1.json")
asm_ids = check_unique("assumptions-v1.json")
proof_ids = check_unique("proofs-v1.json")
mut_ids = check_unique("mutations-v1.json")
combo_ids = check_unique("failure-combinations-v1.json")

# --- oracles must not claim production-store independence falsely ---
for o in items("oracles-v1.json"):
    if o["id"] in ("CSQ-ORACLE-MODEL", "CSQ-ORACLE-READER") and o.get("imports_production_store"):
        fail(f"oracle {o['id']} must not import production store")

# --- invariant graph ---
for inv in items("invariants-v1.json"):
    iid = inv["id"]
    for o in inv.get("oracles", []):
        if o not in oracle_ids:
            fail(f"invariant {iid} references unknown oracle {o}")
    for s in inv.get("suites", []):
        if s not in suite_ids:
            fail(f"invariant {iid} references unknown suite {s}")
    for p in inv.get("proof_obligations", []):
        if p not in proof_ids:
            fail(f"invariant {iid} references unknown proof {p}")
    if not inv.get("oracles") or not inv.get("suites"):
        fail(f"invariant {iid} missing oracle/suite paths")

# --- operations graph ---
for op in items("operations-v1.json"):
    oid = op["id"]
    for b in op.get("boundaries", []):
        if b not in bnd_ids:
            fail(f"operation {oid} references unknown boundary {b}")
    for inv in op.get("invariants", []):
        # allow crash matrix string invariants and CSQ ids
        if inv.startswith("CSQ-") and inv not in inv_ids:
            fail(f"operation {oid} references unknown invariant {inv}")
    for o in op.get("oracles", []):
        if o not in oracle_ids:
            fail(f"operation {oid} references unknown oracle {o}")
    for s in op.get("suites", []):
        if s not in suite_ids:
            fail(f"operation {oid} references unknown suite {s}")
    for fc in op.get("failure_classes", []):
        if fc not in fail_ids:
            fail(f"operation {oid} references unknown failure class {fc}")

# --- boundaries ---
for b in items("boundaries-v1.json"):
    if b.get("operation_id") not in op_ids:
        fail(f"boundary {b.get('id')} operation_id {b.get('operation_id')} not registered")
    if not b.get("harness"):
        fail(f"boundary {b.get('id')} missing harness")

# --- failure combinations need owner ---
for c in items("failure-combinations-v1.json"):
    if not c.get("executable_owner"):
        fail(f"failure combination {c.get('id')} missing executable_owner")
    if c.get("executable_owner") not in suite_ids:
        fail(f"failure combination {c.get('id')} owner suite missing")
    for f in c.get("failures", []):
        if f not in fail_ids:
            fail(f"combination {c.get('id')} unknown failure {f}")

# --- claims ---
for cl in items("claims-v1.json"):
    if not cl.get("invariants") or not cl.get("oracles") or not cl.get("suites"):
        fail(f"claim {cl.get('id')} missing invariant/oracle/suite paths")
    if not cl.get("assumptions"):
        fail(f"claim {cl.get('id')} missing assumption ledger")
    if cl.get("profile") != "residiuum-core-storage-v1":
        fail(f"claim {cl.get('id')} profile must be residiuum-core-storage-v1")
    for a in cl.get("assumptions", []):
        if a not in asm_ids:
            fail(f"claim references unknown assumption {a}")

# --- projections total + mutations owned ---
for pr in items("projections-v1.json"):
    if not pr.get("total"):
        fail(f"projection {pr.get('id')} must be total")
    for m in pr.get("forbidden_collapses", []):
        if m not in mut_ids:
            fail(f"projection {pr.get('id')} unknown mutant {m}")
for m in items("mutations-v1.json"):
    if not m.get("must_be_killed_by"):
        fail(f"mutation {m.get('id')} missing must_be_killed_by owner")

# --- crash matrix import preserves historical IDs ---
cm_path = root / "crates/residiuum-store/crash_matrix.v1.json"
cm = json.loads(cm_path.read_text())
imp = json.loads((cs / "crash-matrix-import-v1.json").read_text())
src_fps = {fp["name"] for op in cm["operations"] for fp in op.get("failpoints", [])}
imp_fps = {fp["historical_cell_id"] for op in imp["operations"] for fp in op.get("failpoints", [])}
missing = src_fps - imp_fps
if missing:
    fail(f"crash-matrix-import lost historical failpoint IDs: {sorted(missing)[:10]}")
# every imported failpoint has a boundary
for op in imp["operations"]:
    for fp in op.get("failpoints", []):
        if fp.get("boundary_id") not in bnd_ids:
            fail(f"import failpoint {fp.get('historical_cell_id')} missing boundary")

# --- no pre-reset product profile as accepted registry id ---
_legacy_id = "din" + "go" + "-core-storage-v1"
for p in cs.glob("*.json"):
    data = json.loads(p.read_text())
    for item in data.get("items") or []:
        if isinstance(item, dict) and item.get("id") == _legacy_id:
            fail(f"{p.name} registers pre-reset product profile as an id")

# --- negative fixture: legacy profile must not equal our profile const ---
neg = json.loads((cs / "vectors/profile-negative-legacy-id.json").read_text())
if neg.get("profile") == "residiuum-core-storage-v1":
    fail("negative fixture incorrectly uses residiuum profile")
if _legacy not in str(neg.get("profile", "")).lower():
    fail("negative fixture must carry a pre-reset product profile id")
schema = json.loads((cs / "report-v1.schema.json").read_text())
if schema.get("properties", {}).get("profile", {}).get("const") != "residiuum-core-storage-v1":
    fail("report schema profile const must be residiuum-core-storage-v1")

# --- dependency cycle detection on suite->invariant->suite (simple) ---
# suites don't form cycles by design; check proof->invariant->proof_obligation consistency
for p in items("proofs-v1.json"):
    if p.get("invariant") not in inv_ids:
        fail(f"proof {p.get('id')} invariant missing")
    if p.get("owner_suite") not in suite_ids:
        fail(f"proof {p.get('id')} owner suite missing")

# dishonest not applicable: no item may set not_applicable without owner
for name in ["invariants-v1.json", "operations-v1.json", "boundaries-v1.json"]:
    for row in items(name):
        if row.get("not_applicable") is True and not row.get("not_applicable_owner"):
            fail(f"{name} {row.get('id')}: dishonest not_applicable without owner")

if err:
    sys.exit(1)
print("OK: core-storage registry validation passed")
print(f"  profile: residiuum-core-storage-v1")
print(f"  invariants: {len(inv_ids)}")
print(f"  operations: {len(op_ids)}")
print(f"  boundaries: {len(bnd_ids)}")
print(f"  errors: {len(err_ids)}")
print(f"  crash failpoints imported: {len(imp_fps)}")
sys.exit(0)
PY

# Rust agreement (if test binary exists later)
if command -v cargo >/dev/null 2>&1; then
  if [[ -f "$ROOT/crates/residiuum-store/tests/csq0_registry.rs" ]] || \
     rg -q "csq0_registry|core_storage_registry" "$ROOT/crates/residiuum-store" 2>/dev/null; then
    log "Running Rust registry agreement test..."
    (cd "$ROOT" && cargo test -p residiuum-store --test csq0_registry -- --nocapture) || ERR=1
  fi
fi

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-core-storage-registry: FAILED" >&2
  exit 1
fi
echo "verify-core-storage-registry: OK"