#!/usr/bin/env bash
# PQH-0: validate Residiuum performance qualification registries.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PERF="$ROOT/spec/performance"
ERR=0

fail() { printf 'FAIL: %s\n' "$*" >&2; ERR=1; }
need() {
  if [[ ! -f "$1" ]]; then
    fail "missing required file: $1"
  fi
}

for f in \
  profiles-v1.json layers-v1.json stages-v1.json metrics-v1.json \
  verdicts-v1.json validity-v1.json omission-reasons-v1.json matrix-v1.json \
  schemas/manifest-v1.schema.json schemas/result-v1.schema.json \
  schemas/comparison-v1.schema.json \
  fixtures/manifest.accepted.json fixtures/manifest.rejected-legacy-profile.json \
  fixtures/result.accepted.json fixtures/result.rejected-missing-unit.json \
  fixtures/result.rejected-zero-as-unavailable-policy.json \
  README.md
do
  need "$PERF/$f"
done

python3 - "$ROOT" <<'PY'
import json, sys
from pathlib import Path

root = Path(sys.argv[1])
perf = root / "spec/performance"
err = 0
PROFILE = "residiuum-performance-qualification-v1"

def fail(msg):
    global err
    print(f"FAIL: {msg}", file=sys.stderr)
    err = 1

def load(name):
    with (perf / name).open() as f:
        return json.load(f)

def ids(name):
    d = load(name)
    return [i["id"] for i in d.get("items", [])]

# Profile identity
profiles = load("profiles-v1.json")["items"]
if not any(p.get("id") == PROFILE for p in profiles):
    fail(f"profiles must contain {PROFILE}")
_legacy = "din" + "go"
for p in profiles:
    if _legacy in p.get("id", "").lower():
        fail(f"forbidden pre-reset product profile: {p.get('id')}")

# Non-empty closed sets
for name, min_n in [
    ("layers-v1.json", 7),
    ("stages-v1.json", 9),
    ("metrics-v1.json", 20),
    ("verdicts-v1.json", 11),
    ("validity-v1.json", 8),
    ("omission-reasons-v1.json", 4),
]:
    items = load(name)["items"]
    if len(items) < min_n:
        fail(f"{name}: need >= {min_n} items, got {len(items)}")
    seen = set()
    for it in items:
        i = it.get("id")
        if not i:
            fail(f"{name}: item missing id")
        if i in seen:
            fail(f"{name}: duplicate id {i}")
        seen.add(i)

layer_ids = set(ids("layers-v1.json"))
for need in ["L0", "L1", "L2", "L3", "L4", "L5", "L6"]:
    if need not in layer_ids:
        fail(f"layers missing {need}")

stage_ids = set(ids("stages-v1.json"))
for need in ["queue", "validation", "encoding", "chunking", "indexing", "io", "publication", "durability", "residual"]:
    if need not in stage_ids:
        fail(f"stages missing {need}")

verdict_ids = set(ids("verdicts-v1.json"))
if "mixed_or_unknown" not in verdict_ids:
    fail("verdicts must include mixed_or_unknown")

# Metrics: units required; zero_is_not_unavailable policy
for m in load("metrics-v1.json")["items"]:
    mid = m.get("id")
    if not m.get("unit"):
        fail(f"metric {mid} missing unit")
    if m.get("zero_is_not_unavailable") is not True:
        fail(f"metric {mid} must set zero_is_not_unavailable=true")
    if "kind" not in m:
        fail(f"metric {mid} missing kind")

# Profile closes against registries
prof = next(p for p in profiles if p["id"] == PROFILE)
for lid in prof.get("layers", []):
    if lid not in layer_ids:
        fail(f"profile references unknown layer {lid}")
for sid in prof.get("stages", []):
    if sid not in stage_ids:
        fail(f"profile references unknown stage {sid}")
for vid in prof.get("verdicts", []):
    if vid not in verdict_ids:
        fail(f"profile references unknown verdict {vid}")
metric_ids = set(ids("metrics-v1.json"))
for mid in prof.get("metrics_required", []):
    if mid not in metric_ids:
        fail(f"profile required metric unknown: {mid}")

# Matrix cells reference closed axes
matrix = load("matrix-v1.json")
axes = {a["id"]: set(a["values"]) for a in matrix.get("axes", [])}
if "layer" not in axes:
    fail("matrix axes must include layer")
for cell in matrix.get("items", []):
    cid = cell.get("id")
    for axis, allowed in axes.items():
        if axis in cell and cell[axis] not in allowed:
            fail(f"matrix cell {cid}: {axis}={cell[axis]} not in axis values")
    if cell.get("layer") not in layer_ids:
        fail(f"matrix cell {cid}: unknown layer")

# Fixtures: accepted must match profile const; pre-reset profile rejected by policy
acc = load("fixtures/manifest.accepted.json")
if acc.get("profile") != PROFILE:
    fail("accepted manifest must use residiuum profile")
rej = load("fixtures/manifest.rejected-legacy-profile.json")
_legacy = "din" + "go"
if _legacy not in rej.get("profile", "").lower():
    fail("rejected-legacy fixture must use a pre-reset product profile id")

# Policy: unavailable must not be represented as bare zero without reason
res = load("fixtures/result.accepted.json")
for k, v in res.get("metrics", {}).items():
    if v.get("status") == "unavailable":
        if v.get("value") is not None:
            fail(f"accepted result metric {k}: unavailable must have value null")
        if not v.get("unavailable_reason"):
            fail(f"accepted result metric {k}: unavailable needs reason")
    if v.get("status") == "present":
        if "unit" not in v or not v["unit"]:
            fail(f"accepted result metric {k}: present needs unit")
        if v.get("value") is None:
            fail(f"accepted result metric {k}: present needs value")

# Missing unit fixture exists for negative CI
mu = load("fixtures/result.rejected-missing-unit.json")
for k, v in mu.get("metrics", {}).items():
    if v.get("status") == "present" and "unit" not in v:
        break
else:
    fail("missing-unit fixture must have a present metric without unit")

# Lightweight structural schema checks (no jsonschema dependency)
for schema_name, fmt in [
    ("schemas/manifest-v1.schema.json", "residiuum-pqh-manifest-v1"),
    ("schemas/result-v1.schema.json", "residiuum-pqh-result-v1"),
    ("schemas/comparison-v1.schema.json", "residiuum-pqh-comparison-v1"),
]:
    s = load(schema_name)
    if s.get("properties", {}).get("format", {}).get("const") != fmt:
        fail(f"{schema_name}: format const mismatch")
    if s.get("properties", {}).get("profile", {}).get("const") != PROFILE:
        fail(f"{schema_name}: profile must const {PROFILE}")

if err:
    sys.exit(1)
print("OK: performance registry validation passed")
print(f"  profile: {PROFILE}")
print(f"  layers: {len(layer_ids)} stages: {len(stage_ids)} metrics: {len(metric_ids)} verdicts: {len(verdict_ids)}")
print(f"  matrix cells: {len(matrix.get('items', []))}")
PY

echo "Running residiuum-perf registry tests..."
cargo test -p residiuum-perf --lib -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-performance-registry: FAILED" >&2
  exit 1
fi
echo "verify-performance-registry: OK"