#!/usr/bin/env bash
# FAS-4 consistency theorem family gate.
# Authority: FORMAL_ASSURANCE_IMPLEMENTATION_PLAN.md §8 + REGISTRY §12.2.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas4-consistency-report.json"
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

CON_THEOREMS = [
    "FAS-CON-NO-FABRICATED-VALUE-001",
    "FAS-CON-GENERATION-EXACT-001",
    "FAS-CON-PUBLICATION-NONHYBRID-001",
    "FAS-CON-DURABLE-ACK-001",
    "FAS-CON-RECOVERY-IDEMPOTENT-001",
    "FAS-CON-DERIVED-NONAUTHORITY-001",
    "FAS-CON-DAMAGE-HONESTY-001",
    "FAS-CON-HEALTHY-ISLAND-001",
]

# --- Preconditions FAS-3 ---
fas3 = root / "target/formal-assurance/fas3-refinement-report.json"
if not fas3.is_file():
    errs.append("fas3-refinement-report.json missing")
else:
    j = json.loads(fas3.read_text())
    if j.get("result") != "pass":
        errs.append("FAS-3 not pass")
    else:
        checks["fas3_precondition"] = "ok"

# CSQ evidence (applicable packages)
csq_ok = True
for rel in [
    "target/csq-evidence/a2-evaluation.json",
    "target/csq-evidence/residiuum-core-storage-report-v1.json",
]:
    p = root / rel
    if not p.is_file():
        errs.append(f"CSQ evidence missing: {rel}")
        csq_ok = False
    else:
        checks[f"csq:{p.name}"] = "ok"
if csq_ok:
    # prefer a2_pass when present
    a2 = root / "target/csq-evidence/a2-evaluation.json"
    try:
        a2j = json.loads(a2.read_text())
        if a2j.get("a2_pass") is False:
            warns.append("a2_pass is false — physical qualification residual")
        elif a2j.get("a2_pass") is True:
            checks["csq_a2_pass"] = True
    except Exception as e:
        warns.append(f"a2-evaluation parse: {e}")

cons = root / "formal/consistency"
for rel in [
    "theorem-connections-v1.json",
    "negative-controls-v1.json",
    "README.md",
]:
    if not (cons / rel).is_file():
        errs.append(f"missing consistency artifact: {rel}")
    else:
        checks[f"artifact:{rel}"] = "ok"

lean_path = root / "formal/lean/Residiuum/Consistency.lean"
if not lean_path.is_file():
    errs.append("Residiuum/Consistency.lean missing")
else:
    lean_src = lean_path.read_text()
    checks["lean_consistency_module"] = "ok"
    if re.search(r"\bsorry\b", lean_src.split("--")[0] if False else lean_src):
        # check line by line excluding comments
        for i, line in enumerate(lean_src.splitlines(), 1):
            if re.search(r"\bsorry\b", line.split("--")[0]):
                errs.append(f"sorry in Consistency.lean:{i}")
    else:
        checks["no_sorry"] = "ok"

# Connections: all 8 CON theorems
conn_path = cons / "theorem-connections-v1.json"
neg_path = cons / "negative-controls-v1.json"
if conn_path.is_file() and lean_path.is_file():
    conn = json.loads(conn_path.read_text())
    items = {it["theorem_id"]: it for it in conn.get("items") or []}
    lean_src = lean_path.read_text()
    for tid in CON_THEOREMS:
        if tid not in items:
            errs.append(f"connection missing for {tid}")
            continue
        it = items[tid]
        for sym in it.get("lean_symbols") or []:
            short = sym.split(".")[-1]
            if short not in lean_src and sym not in lean_src:
                errs.append(f"{tid}: lean symbol missing: {sym}")
            else:
                checks[f"lean:{short}"] = "ok"
        if not it.get("negative_control"):
            errs.append(f"{tid}: no negative_control")
        if not it.get("csq_links"):
            errs.append(f"{tid}: no csq_links")
        # entrypoint paths (symbol after colon optional)
        for ep in it.get("rust_entrypoints") or []:
            path = ep.split(":")[0]
            sym = ep.split(":")[1] if ":" in ep else None
            fp = root / path
            if not fp.is_file():
                errs.append(f"{tid}: rust path missing {path}")
            elif sym and sym not in fp.read_text(errors="replace"):
                errs.append(f"{tid}: symbol {sym} missing in {path}")
    checks["con_theorem_count"] = len(items)
    if conn.get("filesystem_assumption") != "FAS-ASM-FILESYSTEM-DURABILITY-001":
        warns.append("filesystem assumption not named")
    else:
        checks["filesystem_assumption"] = "ok"
    if "honest_residual" not in conn:
        warns.append("no honest_residual on profile")
    else:
        checks["honest_residual_documented"] = True

# Negatives: one per CON theorem
if neg_path.is_file():
    neg = json.loads(neg_path.read_text())
    nitems = neg.get("items") or []
    by_th = {n["theorem_id"] for n in nitems}
    for tid in CON_THEOREMS:
        if tid not in by_th:
            errs.append(f"negative control missing for {tid}")
    checks["negative_control_count"] = len(nitems)
    lean_src = lean_path.read_text() if lean_path.is_file() else ""
    for n in nitems:
        rej = n.get("rejected_by_lean", "")
        short = rej.split(".")[-1] if rej else ""
        if short and short not in lean_src:
            errs.append(f"negative {n.get('id')}: rejected_by_lean missing {rej}")

# Registry catalogue still lists all CON
th_reg = root / "formal/registry/theorems-v1.json"
if th_reg.is_file():
    reg = json.loads(th_reg.read_text())
    ids = {it["id"] for it in reg.get("items") or []}
    for tid in CON_THEOREMS:
        if tid not in ids:
            errs.append(f"registry missing {tid}")
    checks["registry_con_catalogue"] = "ok"

# lake build
r = run(["lake", "--dir", "formal/lean", "build"], timeout=300)
out = r.stdout + r.stderr
checks["lake_build_exit"] = r.returncode
if r.returncode != 0 or "Build completed successfully" not in out:
    errs.append("lake build failed")
    checks["lake_build_tail"] = out[-800:]
else:
    checks["lake_build"] = "ok"

# Profile honesty: MVP not full physical profile
profile_claim = "mvp_abstract_plus_csq_links"
if conn_path.is_file():
    profile_claim = json.loads(conn_path.read_text()).get("profile_status", profile_claim)

result = "pass" if not errs else "fail"
report = {
    "schema": "residiuum-formal-package-report-v1",
    "package": "FAS-4",
    "result": result,
    "closed": result == "pass",
    "structural_ok": result == "pass",
    "profile": "residiuum-formal-consistency-v1",
    "profile_status": profile_claim,
    "con_theorems": CON_THEOREMS,
    "checks": checks,
    "errors": errs,
    "warnings": warns,
    "message": (
        "FAS-4 consistency MVP accept (abstract Lean + CSQ links; not full physical profile)"
        if result == "pass"
        else "FAS-4 consistency incomplete"
    ),
}
report_path.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
sys.exit(0 if result == "pass" else 1)
PY
