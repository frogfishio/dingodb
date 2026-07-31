#!/usr/bin/env bash
# verify-security-process.sh — DEF-063-A process documents present and linked.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "verify-security-process: FAIL: $*" >&2; exit 1; }
ok() { echo "verify-security-process: $*"; }

need_file() {
  [[ -f "$1" ]] || fail "missing $1"
}

need_file "SECURITY.md"
need_file "doc/reference/operations/SUPPORTED_VERSIONS.md"
need_file "doc/wip/security/SECURITY_AUDIT_PACKAGE.md"
need_file "doc/wip/security/THREAT_MODEL.md"
need_file "README.md"

python3 - <<'PY' || exit 1
from pathlib import Path

checks = [
    ("SECURITY.md", [
        "Reporting a vulnerability",
        "Coordinated disclosure",
        "SUPPORTED_VERSIONS",
        "Do not open a public GitHub issue",
    ]),
    ("doc/reference/operations/SUPPORTED_VERSIONS.md", [
        "residiuum-supported-versions-v1",
        "Development tip",
        "WIRE_PROFILE_LABEL",
        "1.0-draft",
    ]),
    ("doc/wip/security/SECURITY_AUDIT_PACKAGE.md", [
        "residiuum-security-audit-package-v1",
        "fuzz-smoke",
        "THREAT_MODEL",
        "Not complete",
    ]),
    ("README.md", [
        "SECURITY.md",
        "SUPPORTED_VERSIONS",
        "SECURITY_AUDIT_PACKAGE",
    ]),
    ("doc/wip/security/THREAT_MODEL.md", [
        "SECURITY.md",
        "SUPPORTED_VERSIONS",
        "SECURITY_AUDIT_PACKAGE",
    ]),
]
for path, needles in checks:
    text = Path(path).read_text(encoding="utf-8")
    missing = [n for n in needles if n not in text]
    if missing:
        raise SystemExit(f"{path} missing markers: {missing}")
print("verify-security-process: document markers OK")
PY

ok "OK"
