#!/usr/bin/env bash
# CSQ-12: evidence-bundle builder + independent verifier ownership.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0

echo "Running CSQ-12 evidence selftest + build/verify..."
bash ./scripts/residiuum-verify-core-storage.sh \
  --profile residiuum-core-storage-v1 \
  --level A2 || ERR=1

# Explicit false-pass rejection (independent of selftest).
python3 - <<'PY' || ERR=1
import json, sys, tempfile
from pathlib import Path
sys.path.insert(0, str(Path("scripts/lib").resolve()))
from csq_evidence import verify_report, REPORT_FORMAT, PROFILE

false_pass = {
    "format": REPORT_FORMAT,
    "profile": PROFILE,
    "level": "A2",
    "source_revision": "test",
    "result": "pass",
    "cells": [{"cell_id": "x", "result": "not_run"}],
    "missing_cells": ["x"],
}
v = verify_report(false_pass)
assert not v["ok"], "false pass must be rejected"
print("ok  false-pass rejection (inline)")
PY

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-evidence-bundle: FAILED" >&2
  exit 1
fi
echo "verify-csq-evidence-bundle: OK"
echo "  command: scripts/residiuum-verify-core-storage.sh --profile residiuum-core-storage-v1 --level A2"
