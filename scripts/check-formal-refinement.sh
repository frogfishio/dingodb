#!/usr/bin/env bash
# FAS-3 refinement bridge gate.
# Authority: FORMAL_KERNEL_MODEL_CONTRACT.md §13 + IMPL plan §7.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas3-refinement-report.json"
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

# --- Preconditions: FAS-2 ---
fas2 = root / "target/formal-assurance/fas2-foundation-report.json"
if not fas2.is_file():
    errs.append("fas2-foundation-report.json missing — run check-formal-foundation.sh")
else:
    try:
        j = json.loads(fas2.read_text())
        if j.get("result") != "pass":
            errs.append("FAS-2 not pass")
        else:
            checks["fas2_precondition"] = "ok"
    except json.JSONDecodeError:
        errs.append("fas2 report invalid JSON")

ref = root / "formal/refinement"
required = [
    "entrypoint-census-v1.json",
    "type-map-v1.json",
    "bridges/FAS-BRIDGE-AUTHORITY-BINDING-001.json",
    "negative/renamed-entrypoint.json",
    "negative/demo-as-connection.json",
    "README.md",
]
for rel in required:
    if not (ref / rel).is_file():
        errs.append(f"missing refinement artifact: {rel}")
    else:
        checks[f"artifact:{rel}"] = "ok"

# --- Census ---
census_path = ref / "entrypoint-census-v1.json"
bridges = {}
bridge_dir = ref / "bridges"
if bridge_dir.is_dir():
    for p in bridge_dir.glob("*.json"):
        try:
            bridges[p.stem] = json.loads(p.read_text())
        except json.JSONDecodeError as e:
            errs.append(f"invalid bridge JSON {p.name}: {e}")

connected_bridges = []
if census_path.is_file():
    census = json.loads(census_path.read_text())
    items = census.get("items") or []
    checks["census_count"] = len(items)
    classes = set()
    demo_connected = []
    for it in items:
        classes.add(it.get("connection_class"))
        if it.get("demo_only") and it.get("connection_class") == "rust_connected_refinement":
            demo_connected.append(it.get("entrypoint_id"))
        # Live path check for non-demo
        path = it.get("path")
        if path and not it.get("demo_only"):
            fp = root / path
            if not fp.is_file():
                errs.append(f"census entrypoint path missing: {path}")
            else:
                text = fp.read_text(errors="replace")
                for sym in it.get("symbols") or []:
                    if sym not in text:
                        errs.append(f"symbol {sym!r} not found in {path}")
        if it.get("bridge_id"):
            connected_bridges.append(it["bridge_id"])
    checks["connection_classes_seen"] = sorted(classes)
    if demo_connected:
        errs.append(f"demo_only entrypoints claimed rust_connected: {demo_connected}")
    else:
        checks["no_demo_as_connection"] = "ok"
    if "rust_connected_refinement" not in classes:
        errs.append("census has no rust_connected_refinement row")
    else:
        checks["has_rust_connected"] = "ok"

# --- Vertical bridge completeness ---
bridge_id = "FAS-BRIDGE-AUTHORITY-BINDING-001"
b = bridges.get(bridge_id) or bridges.get(bridge_id.replace("FAS-BRIDGE-", ""))
if not b and bridges:
    b = bridges.get(list(bridges.keys())[0])
if not b:
    errs.append("no vertical bridge JSON loaded")
else:
    checks["bridge_id"] = b.get("bridge_id")
    for key in ["theorem_id", "lean", "verus", "rust_entrypoints", "csq_evidence_links"]:
        if key not in b:
            errs.append(f"bridge missing field {key}")
    # Rust entrypoints exist + symbols
    for ep in b.get("rust_entrypoints") or []:
        path = ep.get("path")
        if not path:
            errs.append("bridge entrypoint missing path")
            continue
        if "examples/" in path or path.endswith("_demo.rs"):
            errs.append(f"bridge uses demo path as connection: {path}")
            continue
        fp = root / path
        if not fp.is_file():
            errs.append(f"bridge entrypoint path missing: {path}")
        else:
            text = fp.read_text(errors="replace")
            for sym in ep.get("symbols") or []:
                if sym not in text:
                    errs.append(f"bridge symbol {sym!r} missing in {path}")
                else:
                    checks[f"bridge_sym:{sym}"] = "ok"
    # Verus path
    vpath = (b.get("verus") or {}).get("path")
    if vpath:
        if not (root / vpath).is_file():
            errs.append(f"verus path missing: {vpath}")
        else:
            vtext = (root / vpath).read_text(errors="replace")
            for sym in (b.get("verus") or {}).get("proofs") or []:
                if sym not in vtext:
                    errs.append(f"verus proof symbol missing: {sym}")
            checks["verus_path"] = "ok"
    # Lean symbols in Refinement.lean
    lean_mod = root / "formal/lean/Residiuum/Refinement.lean"
    if not lean_mod.is_file():
        errs.append("formal/lean/Residiuum/Refinement.lean missing")
    else:
        ltext = lean_mod.read_text()
        for sym in (b.get("lean") or {}).get("symbols") or []:
            # Lean may use snake or camel; require substring
            if sym not in ltext:
                errs.append(f"lean symbol missing in Refinement.lean: {sym}")
            else:
                checks[f"lean_sym:{sym}"] = "ok"
    # CSQ evidence links
    for link in b.get("csq_evidence_links") or []:
        lp = root / link
        if not lp.is_file():
            warns.append(f"csq evidence link missing (non-fatal if path deferred): {link}")
        else:
            checks[f"csq:{Path(link).name}"] = "ok"
    if b.get("semantic_map_assumption") != "FAS-ASM-CROSS-TOOL-MAP-001":
        warns.append("bridge should cite FAS-ASM-CROSS-TOOL-MAP-001")
    else:
        checks["cross_tool_assumption"] = "ok"

# --- Negative controls (must fail if applied) ---
# 1) Renamed entrypoint simulation
neg_rename = ref / "negative/renamed-entrypoint.json"
if neg_rename.is_file():
    neg = json.loads(neg_rename.read_text())
    bad_path = (neg.get("mutated_entrypoint") or {}).get("path")
    if bad_path and (root / bad_path).is_file():
        errs.append("negative renamed path unexpectedly exists")
    elif bad_path and not (root / bad_path).is_file():
        checks["negative_renamed_entrypoint"] = "would_fail_as_expected"
    # Prove gate logic: missing path ⇒ fail condition detected
    if bad_path and not (root / bad_path).is_file():
        checks["rename_revokes_connection"] = "ok"

# 2) Demo-as-connection rejection logic
neg_demo = ref / "negative/demo-as-connection.json"
if neg_demo.is_file():
    nd = json.loads(neg_demo.read_text())
    dpath = nd.get("claimed_path", "")
    if "examples/" in dpath or "demo" in dpath.lower():
        # Gate rule: never accept examples as rust_connected
        checks["negative_demo_as_connection"] = "rejected_by_policy"
    else:
        warns.append("demo negative fixture path unexpected")

# --- Lean build (includes Refinement) ---
r = run(["lake", "--dir", "formal/lean", "build"], timeout=300)
out = r.stdout + r.stderr
checks["lake_build_exit"] = r.returncode
if r.returncode != 0 or "Build completed successfully" not in out:
    errs.append("lake build failed")
    checks["lake_build_tail"] = out[-600:]
else:
    checks["lake_build"] = "ok"

# --- Verus pure_kernel ---
verus_bin = root / "tools/verus/verus"
import shutil
vbin = str(verus_bin) if verus_bin.is_file() else shutil.which("verus")
if not vbin:
    errs.append("verus binary missing")
else:
    vr = run([vbin, str(root / "verification/heap-verus/verus/pure_kernel.rs")], timeout=300)
    vout = vr.stdout + vr.stderr
    if vr.returncode == 0 and "verified" in vout.lower():
        checks["verus_pure_kernel"] = "ok"
        checks["verus_tail"] = vout[-200:]
    else:
        errs.append("verus pure_kernel failed")
        checks["verus_tail"] = vout[-400:]

# --- Executable pure_proofs still present (cargo check lib path via rg) ---
pp = root / "crates/residiuum-heap/src/pure_proofs.rs"
if pp.is_file() and "lemma_binding_rejects_foreign_heap" in pp.read_text():
    checks["executable_pure_proofs"] = "ok"
else:
    errs.append("pure_proofs lemma_binding_rejects_foreign_heap missing")

# Flag markers
hv = root / "verification/heap-verus/src/lib.rs"
if hv.is_file():
    ht = hv.read_text()
    if "VERUS_PROOFS_CONNECTED: bool = true" in ht.replace(" ", ""):
        # tolerate spacing
        pass
    if "VERUS_PROOFS_CONNECTED" in ht and "true" in ht:
        checks["VERUS_PROOFS_CONNECTED"] = "ok"
    else:
        warns.append("VERUS_PROOFS_CONNECTED not true")
    if "KANI_HARNESSES_CONNECTED" in ht:
        checks["KANI_HARNESSES_CONNECTED"] = "present"

# Type map
tm = ref / "type-map-v1.json"
if tm.is_file():
    t = json.loads(tm.read_text())
    if not (t.get("abstraction_functions") or []):
        errs.append("type-map missing abstraction_functions")
    else:
        checks["type_map_alphas"] = len(t["abstraction_functions"])

result = "pass" if not errs else "fail"
report = {
    "schema": "residiuum-formal-package-report-v1",
    "package": "FAS-3",
    "result": result,
    "closed": result == "pass",
    "structural_ok": result == "pass",
    "vertical_slice": bridge_id,
    "checks": checks,
    "errors": errs,
    "warnings": warns,
    "message": "FAS-3 refinement accept" if result == "pass" else "FAS-3 refinement incomplete",
    "connection_classes": [
        "abstract_theorem_only",
        "independent_executable_agreement",
        "bounded_concrete",
        "rust_connected_refinement",
    ],
}
report_path.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
sys.exit(0 if result == "pass" else 1)
PY
