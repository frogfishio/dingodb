#!/usr/bin/env bash
# FAS-2 abstract semantic kernel gate.
# Authority: FORMAL_KERNEL_MODEL_CONTRACT.md §12 + IMPL plan §6.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas2-foundation-report.json"
export PATH="${HOME}/.elan/bin:${HOME}/.cargo/bin:${PATH}"
export ROOT REPORT

python3 <<'PY'
import json, os, re, subprocess, sys
from pathlib import Path

root = Path(os.environ["ROOT"])
report_path = Path(os.environ["REPORT"])
errs, warns, checks = [], [], {}

def run(cmd, timeout=300):
    return subprocess.run(
        cmd, cwd=str(root), capture_output=True, text=True, timeout=timeout
    )

# Preconditions
if not (root / "formal/registry/FAS0_CLOSED").is_file():
    errs.append("FAS0_CLOSED absent — FAS-2 requires FAS-0 closed")
fas1 = root / "target/formal-assurance/fas1-toolchain-report.json"
if fas1.is_file():
    try:
        j = json.loads(fas1.read_text())
        if j.get("result") != "pass":
            warns.append("fas1-toolchain-report result != pass")
        else:
            checks["fas1_precondition"] = "ok"
    except json.JSONDecodeError:
        warns.append("fas1-toolchain-report unreadable")
else:
    warns.append("fas1-toolchain-report missing (run check-formal-toolchain.sh first)")

# Required Lean modules
lean_root = root / "formal/lean"
required_modules = [
    "Residiuum/Identity.lean",
    "Residiuum/Observation.lean",
    "Residiuum/State.lean",
    "Residiuum/WellFormed.lean",
    "Residiuum/Operations.lean",
    "Residiuum/Observe.lean",
    "Residiuum/Vectors.lean",
    "Residiuum/Foundation.lean",
    "Residiuum.lean",
]
missing = [m for m in required_modules if not (lean_root / m).is_file()]
if missing:
    errs.append(f"missing lean modules: {missing}")
else:
    checks["lean_modules"] = "ok"

# Required symbols / theorems (source grep — type-check is the real gate)
required_patterns = {
    "init_well_formed": r"theorem\s+init_well_formed",
    "Observation": r"inductive\s+Observation",
    "WellFormed": r"def\s+WellFormed",
    "ForbiddenCollapse": r"inductive\s+ForbiddenCollapse",
    "id_projection_no_forbidden_collapse": r"theorem\s+id_projection_no_forbidden_collapse",
    "Observe": r"def\s+Observe\b",
    "foundationOperationIds": r"def\s+foundationOperationIds",
    "input_has_operation_id": r"theorem\s+input_has_operation_id",
    "create_heap_preserves_wf": r"theorem\s+create_heap_preserves_wf",
    "fas2_foundation_ok": r"theorem\s+fas2_foundation_ok",
}
src = ""
for m in required_modules:
    p = lean_root / m
    if p.is_file():
        src += p.read_text() + "\n"
for name, pat in required_patterns.items():
    if re.search(pat, src):
        checks[f"symbol_{name}"] = "ok"
    else:
        errs.append(f"missing required symbol/theorem: {name}")

# Observation constructor separation (all seven kinds present)
for kind in ["complete", "absentProved", "partialObs", "damaged", "unknown",
             "unauthorized", "unavailable"]:
    if kind not in src and f"| {kind}" not in src:
        # also accept space-separated inductive forms
        if not re.search(rf"\|\s*{kind}\b", src):
            errs.append(f"observation constructor missing: {kind}")
checks["observation_constructors"] = "ok" if not any(
    "observation constructor missing" in e for e in errs
) else "fail"

# Forbidden collapse pairs registered
for pair in ["partial_absent", "partial_complete", "damaged_absent",
             "damaged_complete", "unknown_absent", "unknown_complete",
             "unauthorized_absent", "unavailable_absent"]:
    if pair not in src:
        errs.append(f"forbidden collapse case missing: {pair}")

# Operations registry filled (not empty stub)
ops_path = root / "formal/registry/operations-v1.json"
if ops_path.is_file():
    try:
        ops = json.loads(ops_path.read_text())
        items = ops.get("items") or []
        if len(items) < 5:
            errs.append(f"operations-v1.json too thin ({len(items)} items); FAS-2 expects foundation ops")
        else:
            checks["operations_registry"] = f"{len(items)}_ops"
            ids = {it.get("operation_id") or it.get("id") for it in items}
            for need in ["create_heap", "put", "get", "create_collection", "delete"]:
                if need not in ids and not any(need in str(it) for it in items):
                    warns.append(f"operations registry may lack {need}")
    except json.JSONDecodeError as e:
        errs.append(f"operations-v1.json invalid: {e}")
else:
    errs.append("operations-v1.json missing")

# lake build (type-check + prove)
r = run(["lake", "--dir", "formal/lean", "build"], timeout=300)
out = r.stdout + r.stderr
checks["lake_build_exit"] = r.returncode
if r.returncode != 0 or "Build completed successfully" not in out:
    errs.append("lake build failed")
    checks["lake_build_tail"] = out[-800:]
else:
    checks["lake_build"] = "ok"
    checks["lake_build_tail"] = out[-200:]

# Fail if sorry in foundation sources (except comments)
sorry_hits = []
for m in required_modules:
    p = lean_root / m
    if not p.is_file():
        continue
    for i, line in enumerate(p.read_text().splitlines(), 1):
        stripped = line.split("--")[0]
        if re.search(r"\bsorry\b", stripped):
            sorry_hits.append(f"{m}:{i}")
if sorry_hits:
    errs.append(f"sorry in foundation sources: {sorry_hits}")
else:
    checks["no_sorry"] = "ok"

result = "pass" if not errs else "fail"
report = {
    "schema": "residiuum-formal-package-report-v1",
    "package": "FAS-2",
    "result": result,
    "closed": result == "pass",
    "structural_ok": result == "pass",
    "checks": checks,
    "errors": errs,
    "warnings": warns,
    "message": "FAS-2 foundation accept" if result == "pass" else "FAS-2 foundation incomplete",
    "lean_modules": required_modules,
}
report_path.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
sys.exit(0 if result == "pass" else 1)
PY
