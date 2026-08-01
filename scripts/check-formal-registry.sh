#!/usr/bin/env bash
# FAS-0 registry gate (FA0 Wave A).
#
# Always validates catalogue completeness + basic linter rules.
# Package accept (exit 0) requires FAS0_CLOSED marker — do not create that
# marker until CSQ-12 scoreboard accept and FAS-0-T1/T2 exit are honest.
#
# Never treat file existence or theorem count alone as package accept.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG="$ROOT/formal/registry"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas0-registry-report.json"

export REG REPORT ROOT

python3 <<'PY'
import json, os, re, sys
from pathlib import Path

reg = Path(os.environ["REG"])
report_path = Path(os.environ["REPORT"])
errs = []
warns = []

def fail(msg):
    errs.append(msg)

def load(name):
    p = reg / name
    if not p.is_file():
        fail(f"missing required file: {name}")
        return None
    try:
        with p.open() as f:
            return json.load(f)
    except json.JSONDecodeError as e:
        fail(f"invalid JSON {name}: {e}")
        return None

# Required files
required = [
    "theorems-v1.json",
    "assumptions-v1.json",
    "tcb-v1.json",
    "claims-v1.json",
    "profiles-v1.json",
    "operations-v1.json",
    "negative-controls-v1.json",
    "toolchain-lock-v1.json",
    "artifact-ownership-v1.json",
    "schemas/theorems-v1.schema.json",
    "schemas/assumptions-v1.schema.json",
    "schemas/package-report-v1.schema.json",
    "fixtures/rejected/claim-without-theorem.json",
    "README.md",
]
for f in required:
    if not (reg / f).is_file():
        fail(f"missing required file: {f}")

MANDATORY_THEOREMS = [
    # §12.1
    "FAS-FND-OBSERVATION-SEPARATION-001",
    "FAS-FND-FORBIDDEN-COLLAPSE-001",
    "FAS-FND-REFINEMENT-COMPOSITION-001",
    # §12.2
    "FAS-CON-NO-FABRICATED-VALUE-001",
    "FAS-CON-GENERATION-EXACT-001",
    "FAS-CON-PUBLICATION-NONHYBRID-001",
    "FAS-CON-DURABLE-ACK-001",
    "FAS-CON-RECOVERY-IDEMPOTENT-001",
    "FAS-CON-DERIVED-NONAUTHORITY-001",
    "FAS-CON-DAMAGE-HONESTY-001",
    "FAS-CON-HEALTHY-ISLAND-001",
    # §12.3
    "FAS-SEC-HEAP-NONINTERFERENCE-001",
    "FAS-SEC-AUTHORITY-CONFINEMENT-001",
    "FAS-SEC-DELEGATION-MONOTONE-001",
    "FAS-SEC-EPOCH-REVOCATION-001",
    "FAS-SEC-BLACKLIST-SOUND-001",
    "FAS-SEC-MASTER-NONSERVING-001",
    "FAS-SEC-SCOPE-GUARD-001",
    # §12.4
    "FAS-ATM-ALL-OR-NONE-001",
    "FAS-ATM-PREPARE-COMPLETE-001",
    "FAS-ATM-DECISION-UNIQUE-001",
    "FAS-ATM-PREPARED-INVISIBLE-001",
    "FAS-ATM-RETRY-IDEMPOTENT-001",
    "FAS-ATM-INVARIANT-PRESERVATION-001",
    "FAS-ATM-ISOLATION-HISTORY-001",
    "FAS-ATM-RECOVERY-CONVERGENCE-001",
    # §12.5
    "FAS-CLU-QUORUM-INTERSECTION-001",
    "FAS-CLU-TERM-AUTHORITY-001",
    "FAS-CLU-LEADER-FENCING-001",
    "FAS-CLU-AGREEMENT-001",
    "FAS-CLU-ACK-SURVIVAL-001",
    "FAS-CLU-PARTITION-HONESTY-001",
    "FAS-CLU-REPLICA-CONVERGENCE-001",
    "FAS-CLU-HEAP-CONFINEMENT-001",
    "FAS-CLU-MEMBERSHIP-SAFETY-001",
]

MANDATORY_ASSUMPTIONS = [
    "FAS-ASM-CLASSICAL-LOGIC-001",
    "FAS-ASM-CROSS-TOOL-MAP-001",
    "FAS-ASM-RUST-COMPILER-001",
    "FAS-ASM-CRYPTO-PRIMITIVES-001",
    "FAS-ASM-FILESYSTEM-DURABILITY-001",
    "FAS-ASM-ATOMIC-FAIR-RECOVERY-001",
    "FAS-ASM-CLUSTER-EVENTUAL-SYNCHRONY-001",
    "FAS-ASM-CLUSTER-FAILURE-BOUND-001",
]

ALLOWED_STATUS = {
    "proposed", "specified", "model_checked_bounded", "machine_proved",
    "implementation_connected", "physically_qualified", "revoked",
}
ELEVATED = {
    "model_checked_bounded", "machine_proved",
    "implementation_connected", "physically_qualified",
}
FORBIDDEN_CLAIM_WORDS = [
    "formally verified database",
    "fully verified",
    "mathematically proved implementation",
]

th = load("theorems-v1.json") or {}
asm = load("assumptions-v1.json") or {}
claims = load("claims-v1.json") or {}
own = load("artifact-ownership-v1.json") or {}
rej = load("fixtures/rejected/claim-without-theorem.json") or {}

items = th.get("items") or []
ids = [i.get("id") for i in items if isinstance(i, dict)]
id_set = set(ids)

if len(ids) != len(id_set):
    fail("duplicate theorem ids")

for mid in MANDATORY_THEOREMS:
    if mid not in id_set:
        fail(f"missing mandatory theorem id: {mid}")

# Status / result honesty
for i in items:
    if not isinstance(i, dict):
        continue
    tid = i.get("id", "?")
    st = i.get("status")
    if st not in ALLOWED_STATUS:
        fail(f"{tid}: unknown status {st!r}")
    if st in ELEVATED and not i.get("result_refs"):
        fail(f"{tid}: elevated status {st} without result_refs (no existence proxy)")
    for d in i.get("depends_on") or []:
        if d not in id_set:
            fail(f"{tid}: depends_on unknown theorem {d}")

# Cycle detection
graph = {i["id"]: list(i.get("depends_on") or []) for i in items if isinstance(i, dict) and "id" in i}
WHITE, GRAY, BLACK = 0, 1, 2
color = {n: WHITE for n in graph}

def dfs(n):
    color[n] = GRAY
    for m in graph.get(n, []):
        if m not in color:
            continue
        if color[m] == GRAY:
            fail(f"circular theorem dependency involving {n} -> {m}")
            return
        if color[m] == WHITE:
            dfs(m)
    color[n] = BLACK

for n in list(graph):
    if color[n] == WHITE:
        dfs(n)

# Assumptions
asm_items = asm.get("items") or []
asm_ids = {i.get("id") for i in asm_items if isinstance(i, dict)}
for mid in MANDATORY_ASSUMPTIONS:
    if mid not in asm_ids:
        fail(f"missing mandatory assumption id: {mid}")

# Claims: must reference existing theorems; no forbidden wording
for c in claims.get("items") or []:
    if not isinstance(c, dict):
        continue
    cid = c.get("id", "?")
    for blob in (c.get("title", ""), c.get("text", ""), json.dumps(c)):
        low = blob.lower()
        for w in FORBIDDEN_CLAIM_WORDS:
            if w in low:
                fail(f"claim {cid}: forbidden wording {w!r}")
    tids = c.get("theorem_ids") or []
    if not tids:
        fail(f"claim {cid}: claim without theorem_ids")
    for t in tids:
        if t not in id_set:
            fail(f"claim {cid}: unknown theorem {t}")

# Reject fixture must reference missing theorem (negative control shape)
if rej:
    bad_ids = []
    for c in rej.get("items") or []:
        bad_ids.extend(c.get("theorem_ids") or [])
    if not bad_ids:
        fail("rejected claim fixture must list theorem_ids")
    if all(t in id_set for t in bad_ids):
        fail("rejected claim fixture must include at least one non-existent theorem id")

# Ownership map non-empty for must-keep paths
own_items = own.get("items") or []
if len(own_items) < 4:
    fail("artifact-ownership-v1.json too sparse (expected migration map seed)")
must_paths = [
    "formal/heap/HeapAuthority.tla",
    "formal/heap/HeapIsolation.tla",
    "verification/heap-verus/verus/pure_kernel.rs",
]
own_paths = {i.get("path") for i in own_items if isinstance(i, dict)}
for mp in must_paths:
    if mp not in own_paths:
        fail(f"ownership map missing must-keep path: {mp}")

closed_marker = (reg / "FAS0_CLOSED").is_file()
structural_ok = len(errs) == 0

# Package accept gate
if not closed_marker:
    fail(
        "FAS0_CLOSED absent — catalogue may be structurally complete but "
        "package accept is blocked until CSQ-12 accept + FAS-0-T1/T2 principal exit "
        "(do not invent FAS0_CLOSED early)"
    )
else:
    # Marker present: still require structural_ok; further release checks later
    warns.append("FAS0_CLOSED present — ensure CSQ-12 scoreboard accept before scoreboard FAS-0 accept")

package_pass = structural_ok and closed_marker and not any(
    e.startswith("FAS0_CLOSED") for e in errs
)
# If only failure is closed marker, structural_ok was true before that fail —
# recompute: structural means everything except closed-marker messages
struct_errs = [e for e in errs if not e.startswith("FAS0_CLOSED")]
structural_ok = len(struct_errs) == 0
package_pass = structural_ok and closed_marker

report = {
    "schema": "residiuum-formal-package-report-v1",
    "package": "FAS-0",
    "result": "pass" if package_pass else "fail",
    "closed": closed_marker,
    "structural_ok": structural_ok,
    "wave0_scaffold": not closed_marker,
    "theorem_count": len(ids),
    "mandatory_theorem_count": len(MANDATORY_THEOREMS),
    "assumption_count": len(asm_ids),
    "errors": errs,
    "warnings": warns,
    "message": (
        "FAS-0 package accept"
        if package_pass
        else (
            "structural catalogue OK; package accept blocked (no FAS0_CLOSED / CSQ-12 gate)"
            if structural_ok
            else "registry validation failed"
        )
    ),
}
report_path.write_text(json.dumps(report, indent=2) + "\n")

if package_pass:
    print("check-formal-registry: PASS — FAS-0 package accept conditions met")
    sys.exit(0)

if structural_ok:
    print(
        "check-formal-registry: STRUCTURAL_OK — full §12 catalogue + linter baseline; "
        "FAIL package accept (FAS0_CLOSED / CSQ-12 gate)",
        file=sys.stderr,
    )
else:
    print("check-formal-registry: FAIL — structural errors:", file=sys.stderr)
    for e in struct_errs:
        print(f"  - {e}", file=sys.stderr)
for e in errs:
    if e.startswith("FAS0_CLOSED"):
        print(f"  - {e}", file=sys.stderr)
sys.exit(1)
PY
