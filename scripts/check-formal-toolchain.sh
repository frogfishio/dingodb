#!/usr/bin/env bash
# FAS-1 toolchain gate.
# Exit 0 only when toolchain-lock is closed with checksummed pins for all required tools.
# Until then: structural lock presence + FAS0_CLOSED + Verus pin check; exit 1 package accept.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG="$ROOT/formal/registry"
LOCK="$REG/toolchain-lock-v1.json"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas1-toolchain-report.json"

python3 - "$ROOT" "$REPORT" <<'PY'
import json, os, subprocess, sys
from pathlib import Path

root = Path(sys.argv[1])
report_path = Path(sys.argv[2])
reg = root / "formal/registry"
lock_path = reg / "toolchain-lock-v1.json"
errs = []
warns = []

if not (reg / "FAS0_CLOSED").is_file():
    errs.append("FAS0_CLOSED absent")
if not lock_path.is_file():
    errs.append("toolchain-lock-v1.json missing")
    lock = {}
else:
    lock = json.loads(lock_path.read_text())

tools = {t.get("id"): t for t in lock.get("tools") or []}
required_ids = [
    "FAS-TOOL-VERUS-001",
    "FAS-TOOL-KANI-001",
    "FAS-TOOL-LEAN4-001",
    "FAS-TOOL-TLC-001",
    "FAS-TOOL-TLAPS-001",
]
for rid in required_ids:
    if rid not in tools:
        errs.append(f"missing tool lock entry {rid}")

verus = tools.get("FAS-TOOL-VERUS-001") or {}
if verus.get("version") != "0.2026.07.27.31579f0":
    errs.append("Verus pin must be 0.2026.07.27.31579f0")

unpinned = [
    tid for tid, t in tools.items()
    if "unpinned" in str(t.get("version", "")).lower()
]
if unpinned:
    warns.append("unpinned tools remain: " + ",".join(unpinned))

# Detect verus binary
verus_bin = None
cand = root / "tools/verus/verus"
if cand.is_file() and os.access(cand, os.X_OK):
    verus_bin = str(cand)
else:
    import shutil
    verus_bin = shutil.which("verus")

verus_ok = False
if verus_bin:
    try:
        subprocess.run([verus_bin, "--version"], check=True, capture_output=True, timeout=30)
        verus_ok = True
    except Exception as e:
        warns.append(f"verus --version failed: {e}")
else:
    warns.append("verus binary not found (run setup-formal-tools.sh / setup_verus.sh)")

closed = bool(lock.get("closed")) and not unpinned and verus_ok and not errs
# FAS-1 package accept requires closed lock with no unpinned tools
package_pass = closed and not errs and not unpinned and verus_ok

report = {
    "schema": "residiuum-formal-package-report-v1",
    "package": "FAS-1",
    "result": "pass" if package_pass else "fail",
    "closed": closed,
    "structural_ok": not errs,
    "verus_ok": verus_ok,
    "unpinned_tools": unpinned,
    "errors": errs,
    "warnings": warns,
    "message": (
        "FAS-1 package accept"
        if package_pass
        else "FAS-1 not accept: pin remaining tools + closed lock (Verus path may already work)"
    ),
}
report_path.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
if package_pass:
    print("check-formal-toolchain: PASS", file=sys.stderr)
    sys.exit(0)
print("check-formal-toolchain: FAIL package accept — see report", file=sys.stderr)
sys.exit(1)
PY
