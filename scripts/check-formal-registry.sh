#!/usr/bin/env bash
# FA0 / FAS-0 registry gate.
#
# Wave 0 (FA0-W0-T3): fail-closed scaffold. Exits non-zero until FAS0_CLOSED
# exists and registries pass completeness checks (FAS-0-T1/T2).
#
# Never treat file existence alone as package accept.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG="$ROOT/formal/registry"
REPORT_DIR="$ROOT/target/formal-assurance"
mkdir -p "$REPORT_DIR"
REPORT="$REPORT_DIR/fas0-registry-report.json"

fail_report() {
  local msg="$1"
  python3 - "$REPORT" "$msg" <<'PY'
import json, sys
from pathlib import Path
path, msg = Path(sys.argv[1]), sys.argv[2]
path.write_text(
    json.dumps(
        {
            "schema": "residiuum-formal-package-report-v1",
            "package": "FAS-0",
            "result": "fail",
            "closed": False,
            "wave0_scaffold": True,
            "message": msg,
        },
        indent=2,
    )
    + "\n"
)
print(f"check-formal-registry: FAIL — {msg}", file=sys.stderr)
PY
  exit 1
}

need() {
  [[ -f "$1" ]] || fail_report "missing required file: $1"
}

# Required tree (REGISTRY_CONTRACT §1 + Wave 0 minimal schemas)
for f in \
  theorems-v1.json \
  assumptions-v1.json \
  tcb-v1.json \
  claims-v1.json \
  profiles-v1.json \
  operations-v1.json \
  negative-controls-v1.json \
  toolchain-lock-v1.json \
  schemas/theorems-v1.schema.json \
  schemas/assumptions-v1.schema.json \
  schemas/package-report-v1.schema.json \
  fixtures/rejected/claim-without-theorem.json \
  README.md
do
  need "$REG/$f"
done

# Fail-closed: FAS-0 is not closed until marker is written by FAS-0 exit work.
if [[ ! -f "$REG/FAS0_CLOSED" ]]; then
  fail_report "FAS0_CLOSED absent (Wave 0 scaffold only; not package accept). See formal/registry/README.md and FAS_MIGRATION_MAP.md"
fi

# Post-close validation (FAS-0-T1+): full catalogue + linter live here later.
# For now any FAS0_CLOSED without further checks is still incomplete — refuse.
fail_report "FAS0_CLOSED present but full registry validation not implemented yet (complete FAS-0-T1/T2)"
