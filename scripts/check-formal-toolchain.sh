#!/usr/bin/env bash
# FAS-1 toolchain gate + smokes for accept-required tools.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG="$ROOT/formal/registry"
LOCK="$REG/toolchain-lock-v1.json"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas1-toolchain-report.json"
export PATH="${HOME}/.elan/bin:${HOME}/.cargo/bin:${PATH}"
export ROOT REPORT LOCK REG

python3 <<'PY'
import hashlib, json, os, re, subprocess, sys
from pathlib import Path

root = Path(os.environ["ROOT"])
reg = Path(os.environ["REG"])
lock_path = Path(os.environ["LOCK"])
report_path = Path(os.environ["REPORT"])
errs, warns, smokes = [], [], {}

def run(cmd, timeout=600):
    return subprocess.run(
        cmd, cwd=str(root), capture_output=True, text=True, timeout=timeout
    )

if not (reg / "FAS0_CLOSED").is_file():
    errs.append("FAS0_CLOSED absent")
if not lock_path.is_file():
    errs.append("toolchain-lock missing")
    lock = {}
else:
    lock = json.loads(lock_path.read_text())

tools = {t["id"]: t for t in lock.get("tools") or []}
required = [t for t in lock.get("tools") or [] if t.get("accept_required", True)]

# No floating versions
for t in lock.get("tools") or []:
    ver = str(t.get("version", ""))
    if "unpinned" in ver.lower() or ver.lower() == "latest":
        if t.get("accept_required", True):
            errs.append(f"{t.get('id')}: floating/unpinned version {ver!r}")

# Pin checks
if (tools.get("FAS-TOOL-VERUS-001") or {}).get("version") != "0.2026.07.27.31579f0":
    errs.append("Verus pin mismatch")
if (tools.get("FAS-TOOL-KANI-001") or {}).get("version") != "0.67.0":
    errs.append("Kani pin must be 0.67.0")
if "4.32.2" not in str((tools.get("FAS-TOOL-LEAN4-001") or {}).get("version", "")):
    errs.append("Lean pin must be v4.32.2")

# TLC jar hash
jar = root / "tools/formal/tla2tools.jar"
expect = "936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88"
if jar.is_file():
    h = hashlib.sha256(jar.read_bytes()).hexdigest()
    if h != expect:
        errs.append(f"tla2tools.jar hash mismatch {h}")
    else:
        smokes["tlc_jar_hash"] = "ok"
else:
    errs.append("tools/formal/tla2tools.jar missing")

# Locate verus
verus = root / "tools/verus/verus"
verus_bin = str(verus) if verus.is_file() else None
if not verus_bin:
    import shutil
    verus_bin = shutil.which("verus")

# Smokes (accept-required)
def smoke_verus():
    if not verus_bin:
        return False, "verus binary missing"
    r = run([verus_bin, str(root / "verification/heap-verus/verus/pure_kernel.rs")], timeout=300)
    ok = r.returncode == 0 and "verified" in (r.stdout + r.stderr).lower()
    return ok, (r.stdout + r.stderr)[-400:]

def smoke_kani():
    r = run(
        ["cargo", "kani", "--manifest-path", "formal/kani-smoke/Cargo.toml",
         "--harness", "fas1_kani_smoke"],
        timeout=600,
    )
    out = r.stdout + r.stderr
    # Strip ANSI so colorized "SUCCESSFUL" still matches
    plain = re.sub(r"\x1b\[[0-9;]*m", "", out)
    ok = r.returncode == 0 and "SUCCESSFUL" in plain and "0 of 1 failed" in plain
    return ok, plain[-400:]

def smoke_lean():
    r = run(["lake", "--dir", "formal/lean", "build"], timeout=300)
    out = r.stdout + r.stderr
    ok = r.returncode == 0 and "Build completed successfully" in out
    return ok, out[-400:]

def smoke_tlc():
    r = run(
        ["java", "-cp", "tools/formal/tla2tools.jar", "tlc2.TLC",
         "-config", "formal/tla/smoke/MCFAS1Smoke.cfg",
         "formal/tla/smoke/FAS1Smoke.tla"],
        timeout=120,
    )
    out = r.stdout + r.stderr
    ok = r.returncode == 0 and "No error has been found" in out
    return ok, out[-400:]

smoke_fns = {
    "verus": smoke_verus,
    "kani": smoke_kani,
    "lean4": smoke_lean,
    "tlc": smoke_tlc,
}

for t in required:
    if t.get("deferred"):
        continue
    name = t.get("tool")
    if name not in smoke_fns:
        warns.append(f"no smoke for {name}")
        continue
    try:
        ok, detail = smoke_fns[name]()
    except Exception as e:
        ok, detail = False, str(e)
    smokes[name] = {"ok": ok, "detail_tail": detail}
    if not ok:
        errs.append(f"smoke failed for {name}")

# Negative control: wrong kani pin must not be silent — version check above
# Negative control: corrupt TLC hash already fails

closed = bool(lock.get("closed"))
all_smokes = all(
    smokes.get(t.get("tool"), {}).get("ok")
    for t in required
    if not t.get("deferred") and t.get("tool") in smoke_fns
)
package_pass = closed and not errs and all_smokes

report = {
    "schema": "residiuum-formal-package-report-v1",
    "package": "FAS-1",
    "result": "pass" if package_pass else "fail",
    "closed": closed,
    "structural_ok": not any("missing" in e or "pin" in e.lower() for e in errs) or not errs,
    "smokes": smokes,
    "errors": errs,
    "warnings": warns,
    "message": "FAS-1 package accept" if package_pass else "FAS-1 validation failed",
}
# refine structural_ok
report["structural_ok"] = len([e for e in errs if "smoke failed" not in e]) == 0

report_path.write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
if package_pass:
    print("check-formal-toolchain: PASS", file=sys.stderr)
    sys.exit(0)
print("check-formal-toolchain: FAIL", file=sys.stderr)
for e in errs:
    print(f"  - {e}", file=sys.stderr)
sys.exit(1)
PY