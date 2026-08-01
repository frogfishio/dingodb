#!/usr/bin/env bash
# FAS-1: install or validate pinned formal tools from toolchain-lock-v1.json.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REG="$ROOT/formal/registry"
LOCK="$REG/toolchain-lock-v1.json"
MODE="locked"
export PATH="${HOME}/.elan/bin:${HOME}/.cargo/bin:${PATH}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --locked) MODE=locked; shift ;;
    --propose-update) MODE=propose; shift ;;
    -h|--help) echo "Usage: $0 [--locked|--propose-update]"; exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

[[ -f "$REG/FAS0_CLOSED" ]] || { echo "setup-formal-tools: FAIL — FAS0_CLOSED absent" >&2; exit 1; }
[[ -f "$LOCK" ]] || { echo "setup-formal-tools: FAIL — missing lock" >&2; exit 1; }

if [[ "$MODE" == "propose" ]]; then
  echo "setup-formal-tools: --propose-update is review-only; does not rewrite lock"
  exit 0
fi

python3 - "$LOCK" <<'PY'
import json, sys
from pathlib import Path
lock = json.loads(Path(sys.argv[1]).read_text())
if not lock.get("closed"):
    print("setup-formal-tools: WARN — lock closed=false")
tools = {t["id"]: t for t in lock.get("tools") or []}
v = tools.get("FAS-TOOL-VERUS-001")
assert v and v.get("version") == "0.2026.07.27.31579f0", "bad Verus pin"
print("setup-formal-tools: Verus pin", v["version"])
for tid in ("FAS-TOOL-KANI-001", "FAS-TOOL-LEAN4-001", "FAS-TOOL-TLC-001"):
    t = tools.get(tid)
    assert t and "unpinned" not in str(t.get("version", "")).lower(), f"unpinned {tid}"
    print("setup-formal-tools: pin", tid, t.get("version"))
PY

# Verus
if [[ ! -x "$ROOT/tools/verus/verus" ]] && ! command -v verus >/dev/null 2>&1; then
  bash "$ROOT/scripts/setup_verus.sh"
else
  echo "setup-formal-tools: verus present"
fi

# TLC jar
mkdir -p "$ROOT/tools/formal"
JAR="$ROOT/tools/formal/tla2tools.jar"
EXPECT_SHA="936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88"
if [[ -f "$JAR" ]]; then
  GOT=$(shasum -a 256 "$JAR" | awk '{print $1}')
  if [[ "$GOT" != "$EXPECT_SHA" ]]; then
    echo "setup-formal-tools: tla2tools.jar hash mismatch; re-fetching"
    rm -f "$JAR"
  fi
fi
if [[ ! -f "$JAR" ]]; then
  curl -fsSL -o "$JAR" \
    "https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar"
  GOT=$(shasum -a 256 "$JAR" | awk '{print $1}')
  [[ "$GOT" == "$EXPECT_SHA" ]] || { echo "hash fail $GOT" >&2; exit 1; }
fi
echo "setup-formal-tools: TLC jar OK ($EXPECT_SHA)"

# Kani
if ! cargo kani --version 2>/dev/null | grep -q '0.67.0'; then
  echo "setup-formal-tools: installing kani-verifier 0.67.0"
  cargo install --locked kani-verifier --version 0.67.0
  cargo kani setup
fi
echo "setup-formal-tools: $(cargo kani --version 2>&1 | head -1)"

# Lean via elan
if ! lean --version 2>/dev/null | grep -q '4.32.2'; then
  if [[ ! -x "${HOME}/.elan/bin/elan" ]]; then
    curl -fsSL https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -o /tmp/elan-init.sh
    sh /tmp/elan-init.sh -y --default-toolchain leanprover/lean4:v4.32.2
  else
    "${HOME}/.elan/bin/elan" toolchain install leanprover/lean4:v4.32.2
    "${HOME}/.elan/bin/elan" default leanprover/lean4:v4.32.2
  fi
fi
export PATH="${HOME}/.elan/bin:${PATH}"
echo "setup-formal-tools: $(lean --version | head -1)"

echo "setup-formal-tools: --locked OK"
