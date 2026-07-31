#!/usr/bin/env bash
# M0-3 — program delivery scoreboard checks.
# Parses doc/wip/status/NEXT_BUILD_STATUS.md and fails on dishonest or invalid rows.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

STATUS="${STATUS_FILE:-doc/wip/status/NEXT_BUILD_STATUS.md}"
MATRIX="${MATRIX_FILE:-spec/heap/qualification/hp010-matrix-v1.json}"
fail() { echo "verify-delivery-status: FAIL: $*" >&2; exit 1; }
warn() { echo "verify-delivery-status: WARN: $*" >&2; }
ok() { echo "verify-delivery-status: $*"; }

[[ -f "$STATUS" ]] || fail "missing $STATUS"
[[ -f "$MATRIX" ]] || fail "missing $MATRIX"

# --- matrix honesty ---
if command -v python3 >/dev/null 2>&1; then
  python3 - "$STATUS" "$MATRIX" <<'PY'
import json, re, sys
from pathlib import Path

status_path, matrix_path = Path(sys.argv[1]), Path(sys.argv[2])
text = status_path.read_text(encoding="utf-8")
matrix = json.loads(matrix_path.read_text(encoding="utf-8"))

errors = []
warnings = []

# 1) qualified must be false until product says otherwise
if matrix.get("qualified") is True:
    # Only legal if scoreboard explicitly documents Level-2; still require note
    if "qualified** | **false**" in text or "`qualified` | **false**" in text or "qualified` | **false**" in text:
        errors.append("matrix.qualified is true but scoreboard still claims false")
elif matrix.get("qualified") is not False:
    errors.append(f"matrix.qualified must be boolean, got {matrix.get('qualified')!r}")

if "qualified` | **false**" not in text and "| `qualified` | **false**" not in text and "qualified | **false**" not in text:
    # tolerate table form from NEXT_BUILD_STATUS
    if re.search(r"\|\s*`?qualified`?\s*\|\s*\*\*false\*\*", text) is None:
        errors.append("scoreboard must record Heap qualified=false in verification truth table")

ALLOWED = {"not_started", "ready", "active", "blocked", "accept", "deferred"}

# Parse only the scoreboard table (header includes last_verified + blocked_by).
rows = []
in_scoreboard = False
for line in text.splitlines():
    if line.startswith("| Package | State | last_verified |"):
        in_scoreboard = True
        continue
    if not in_scoreboard:
        continue
    if not line.startswith("|"):
        # end of table
        break
    cells = [c.strip() for c in line.strip().strip("|").split("|")]
    if len(cells) < 2:
        continue
    pkg, state = cells[0], cells[1]
    if pkg in ("Package",) or re.match(r"^[-:]+$", pkg):
        continue
    if state in ("State",) or re.match(r"^[-:]+$", state):
        continue
    if not re.match(r"^[A-Z0-9][A-Z0-9._-]*$", pkg):
        continue
    rows.append((pkg, state, cells))

if not rows:
    errors.append("no package rows parsed from scoreboard")

seen = {}
for pkg, state, cells in rows:
    if pkg in seen:
        errors.append(f"duplicate package id: {pkg}")
    seen[pkg] = state
    if state not in ALLOWED:
        errors.append(f"{pkg}: illegal state {state!r} (allowed {sorted(ALLOWED)})")

# Required packages for M0/M1 lane
required = [
    "M0-1", "M0-2", "M0-3",
    *[f"CSQ-{i}" for i in range(0, 13)],
    *[f"PQH-{i}" for i in range(0, 10)],
    *[f"FAS-{i}" for i in range(0, 10)],
    *[f"APB-{i}" for i in range(0, 13)],
    "HAR-0", "HAR-1", "HAR-2", "HAR-3", "HAR-4", "HAR-5", "HAR-6", "HAR-7",
    "APP-0", "APP-1", "APP-2", "APP-3", "APP-4", "APP-5", "APP-6", "APP-7", "APP-8",
    "DEL-0", "TEL-0", "DST-000",
]
for pkg in required:
    if pkg not in seen:
        errors.append(f"missing required package row: {pkg}")

# Accept rows must not be empty evidence (column index 4 when full table)
for pkg, state, cells in rows:
    if state != "accept":
        continue
    evidence = cells[4] if len(cells) > 4 else ""
    if not evidence or evidence in ("—", "-", "–"):
        errors.append(f"{pkg}: accept requires Evidence link/text")

# Known inventory facts that must not be mis-accepted
if seen.get("HAR-1") == "accept":
    errors.append("HAR-1 must not be accept while collection_create is reserved")
if seen.get("APP-1") == "accept":
    errors.append("APP-1 must not be accept while collection_create is reserved")
if seen.get("HAR-7") == "accept":
    errors.append("HAR-7 must not be accept before M1 journey evidence")

# Matrix gate H3 accept vs product HAR packages: informational only
gates = matrix.get("gates") or {}
h3 = (gates.get("H3") or {}).get("status")
if h3 != "accept":
    warnings.append(f"matrix H3 status is {h3!r}, expected accept after inventory")

# Plan links
for path in (
    "MASTER_DELIVERY_PLAN.md",
    "doc/todo/application-baseline/MUST_ADD.md",
    "doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md",
    "doc/done/programs/M0_1_EVIDENCE_INVENTORY.md",
):
    if not Path(path).is_file():
        errors.append(f"required plan/evidence missing: {path}")

# Stage order: no APP-8 accept without APP-0..7 accept (basic)
if seen.get("APP-8") == "accept":
    for p in [f"APP-{i}" for i in range(0, 8)]:
        if seen.get(p) != "accept":
            errors.append(f"APP-8 accept requires {p} accept")

# Core-storage interlock: existing APP-0/APP-1 review is grandfathered, but
# no later application/Heap feature package may become active/accept before
# core storage A2 qualification.
if seen.get("CSQ-12") != "accept":
    for p in (
        [f"APP-{i}" for i in range(2, 9)]
        + [f"APB-{i}" for i in range(0, 13)]
        + [f"HAR-{i}" for i in range(1, 8)]
    ):
        if seen.get(p) in ("active", "accept"):
            errors.append(f"{p} {seen.get(p)} before CSQ-12 core-storage qualification")

# Application-baseline order and mathematical-stage interlock.
if seen.get("APB-12") == "accept":
    for p in [f"APB-{i}" for i in range(0, 12)]:
        if seen.get(p) != "accept":
            errors.append(f"APB-12 accept requires {p} accept")
if seen.get("APB-12") != "accept":
    for p, st in seen.items():
        if p.startswith(("RRE-", "ATM-", "DDA-", "DOW-")) and st in ("active", "accept"):
            errors.append(f"{p} {st} before APB-12 application-baseline qualification")

# Engine stage honesty: no RRE accept while M0 incomplete
m0_done = all(seen.get(p) == "accept" for p in ("M0-1", "M0-2", "M0-3"))
if not m0_done:
    for p, st in seen.items():
        if p.startswith("RRE-") and st == "accept":
            errors.append(f"{p} accept while M0 incomplete")
        if p.startswith("RRE-") and st == "active":
            errors.append(f"{p} active while M0 incomplete")

for w in warnings:
    print(f"verify-delivery-status: WARN: {w}", file=sys.stderr)

if errors:
    for e in errors:
        print(f"verify-delivery-status: FAIL: {e}", file=sys.stderr)
    sys.exit(1)

print(f"verify-delivery-status: OK ({len(seen)} packages; qualified={matrix.get('qualified')})")
PY
else
  fail "python3 required"
fi
