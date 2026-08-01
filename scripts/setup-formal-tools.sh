#!/usr/bin/env bash
# FAS-1: install or validate pinned formal tools from toolchain-lock-v1.json.
#
# Modes:
#   --locked     validate lock + install Verus pin only when missing (default)
#   --propose-update   refuse to mutate lock (review-only stub)
#
# Does not float "latest". FAS-1 package accept requires full pin hashes.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG="$ROOT/formal/registry"
LOCK="$REG/toolchain-lock-v1.json"
MODE="locked"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --locked) MODE=locked; shift ;;
    --propose-update) MODE=propose; shift ;;
    -h|--help)
      echo "Usage: $0 [--locked|--propose-update]"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ ! -f "$REG/FAS0_CLOSED" ]]; then
  echo "setup-formal-tools: FAIL — FAS0_CLOSED absent (complete FAS-0 first)" >&2
  exit 1
fi
if [[ ! -f "$LOCK" ]]; then
  echo "setup-formal-tools: FAIL — missing $LOCK" >&2
  exit 1
fi

if [[ "$MODE" == "propose" ]]; then
  echo "setup-formal-tools: --propose-update is review-only; does not rewrite toolchain-lock-v1.json"
  echo "  draft candidate pins offline and open a principal review"
  exit 0
fi

# Validate Verus pin entry exists
python3 - "$LOCK" <<'PY'
import json, sys
from pathlib import Path
lock = json.loads(Path(sys.argv[1]).read_text())
tools = {t.get("id"): t for t in lock.get("tools") or []}
v = tools.get("FAS-TOOL-VERUS-001")
if not v or v.get("version") != "0.2026.07.27.31579f0":
    print("setup-formal-tools: FAIL — Verus pin FAS-TOOL-VERUS-001 missing or wrong version", file=sys.stderr)
    sys.exit(1)
print("setup-formal-tools: lock lists Verus", v.get("version"))
unpinned = [t["id"] for t in lock.get("tools") or [] if "unpinned" in str(t.get("version", "")).lower()]
if unpinned:
    print("setup-formal-tools: WARN — unpinned tools (FAS-1 residual):", ", ".join(unpinned))
PY

# Ensure Verus binary if possible
if [[ -x "$ROOT/tools/verus/verus" ]] || command -v verus >/dev/null 2>&1; then
  echo "setup-formal-tools: verus binary present"
else
  echo "setup-formal-tools: verus missing — running scripts/setup_verus.sh"
  bash "$ROOT/scripts/setup_verus.sh"
fi

echo "setup-formal-tools: --locked OK (Verus path); remaining tool pins residual for FAS-1 accept"
