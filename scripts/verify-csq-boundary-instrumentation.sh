#!/usr/bin/env bash
# CSQ-2: static boundary-to-source verification + harness census.
#
# Exit criteria enforced here:
# - every source failpoint::hit / consume_short_write name is registered
# - every registered failpoint boundary has a source site
# - every crash_matrix failpoint name has a source site
# - every boundary has an approved harness (injectable or external)
# - failure combinations are scheduled or rejected with a reason
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ERR=0
fail() { echo "FAIL: $*" >&2; ERR=1; }

python3 - "$ROOT" <<'PY'
import json, re, sys
from pathlib import Path

root = Path(sys.argv[1])
cs = root / "spec/verification/core-storage"
src = root / "crates/residiuum-store/src"
err = 0

def fail(msg):
    global err
    print(f"FAIL: {msg}", file=sys.stderr)
    err = 1

# --- source failpoint names ---
hits = set()
for p in src.rglob("*.rs"):
    text = p.read_text(encoding="utf-8")
    for m in re.finditer(r'failpoint::(?:hit|consume_short_write)\("([^"]+)"\)', text):
        hits.add(m.group(1))

if not hits:
    fail("no failpoint::hit sites found under residiuum-store/src")

bnd = json.loads((cs / "boundaries-v1.json").read_text())
items = bnd["items"]
reg_fps = {i["failpoint"] for i in items if i.get("failpoint")}
fp_rows = [i for i in items if i.get("failpoint")]

missing_reg = sorted(hits - reg_fps)
if missing_reg:
    fail(f"source failpoints not in boundaries-v1.json: {missing_reg[:20]}")

missing_src = sorted(reg_fps - hits)
if missing_src:
    fail(f"boundary failpoints without source site: {missing_src[:20]}")

# --- crash matrix names ---
cm = json.loads((root / "crates/residiuum-store/crash_matrix.v1.json").read_text())
cm_names = {fp["name"] for op in cm["operations"] for fp in op.get("failpoints", [])}
cm_missing = sorted(cm_names - hits)
if cm_missing:
    fail(f"crash_matrix failpoints without source site: {cm_missing[:20]}")

# --- harness approval ---
approved = {
    "in_process_failpoint",
    "operation_matrix_proxy",
    "child_process_abort",
    "process_crash_controller",
    "filesystem_image",
    "external_fs_image",
    "external_harness_csq5",
    "suite_owned",
    "suite_owned_logical",
}
for i in items:
    h = i.get("harness")
    if not h:
        fail(f"boundary {i.get('id')} missing harness")
        continue
    if h not in approved:
        fail(f"boundary {i.get('id')} unapproved harness {h!r}")
    # dishonest: claim pure in-process without a named failpoint
    if h == "in_process_failpoint" and not i.get("failpoint") and i.get("kind") != "logical":
        fail(f"boundary {i.get('id')}: in_process_failpoint requires named failpoint")

# named failpoint rows must use in_process
for i in fp_rows:
    if i.get("harness") != "in_process_failpoint":
        fail(f"failpoint boundary {i.get('id')} must use harness in_process_failpoint")

# --- failure combinations feasibility ---
fc = json.loads((cs / "failure-combinations-v1.json").read_text())
for c in fc["items"]:
    feas = c.get("feasibility")
    if feas not in ("scheduled", "rejected", "infeasible"):
        fail(f"combination {c.get('id')} bad feasibility {feas!r}")
    if feas in ("rejected", "infeasible") and not (c.get("rejection_reason") or "").strip():
        fail(f"combination {c.get('id')} {feas} missing rejection_reason")
    if feas == "scheduled" and not c.get("executable_owner"):
        fail(f"combination {c.get('id')} scheduled without executable_owner")
    if len(c.get("failures") or []) < 2:
        fail(f"combination {c.get('id')} needs ≥2 failures")

if err:
    sys.exit(1)
print("OK: CSQ-2 boundary instrumentation verification passed")
print(f"  source failpoints: {len(hits)}")
print(f"  registered failpoints: {len(reg_fps)}")
print(f"  boundaries: {len(items)}")
print(f"  crash matrix cells: {len(cm_names)}")
print(f"  failure combinations: {len(fc['items'])}")
sys.exit(0)
PY

if command -v cargo >/dev/null 2>&1; then
  echo "Running CSQ-2 Rust instrumentation tests..."
  (cd "$ROOT" && cargo test -p residiuum-store --features legacy-raw-store --test csq2_instrumentation -- --nocapture) || ERR=1
fi

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-boundary-instrumentation: FAILED" >&2
  exit 1
fi
echo "verify-csq-boundary-instrumentation: OK"