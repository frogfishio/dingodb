#!/usr/bin/env bash
# CSQ-3: freeze check + format exhaustive corpus tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ERR=0
MANIFEST="$ROOT/spec/verification/core-storage/vectors/csq3/canonical-manifest-v1.json"

if [[ "${1:-}" == "--write-manifest" ]] || [[ ! -f "$MANIFEST" ]]; then
  echo "Writing CSQ-3 frozen manifest..."
  mkdir -p "$(dirname "$MANIFEST")"
  CSQ3_WRITE_MANIFEST="$MANIFEST" cargo test -p residiuum-format --test csq3_format_corpus \
    csq3_write_manifest_if_env -- --nocapture --exact
fi

if [[ ! -f "$MANIFEST" ]]; then
  echo "FAIL: missing $MANIFEST (run with --write-manifest)" >&2
  exit 1
fi

# Pre-reset magics must not appear in frozen CSQ-3 vectors (token split for linter).
if python3 - "$ROOT/spec/verification/core-storage/vectors/csq3" <<'PY'
import sys
from pathlib import Path
root = Path(sys.argv[1])
legacy = [("DIN", "GOFRM"), ("DIN", "GOEND")]
bad = False
for p in root.rglob("*"):
    if not p.is_file():
        continue
    try:
        text = p.read_text(errors="replace")
    except Exception:
        continue
    for a, b in legacy:
        if a + b in text:
            print(f"{p}: contains pre-reset magic {a}{b}")
            bad = True
sys.exit(1 if bad else 0)
PY
then
  echo "FAIL: former pre-reset magics in CSQ-3 vectors" >&2
  ERR=1
fi

echo "Running CSQ-3 format corpus tests..."
cargo test -p residiuum-format --test csq3_format_corpus -- --nocapture || ERR=1
cargo test -p residiuum-format --lib csq_corpus -- --nocapture || ERR=1
cargo test -p residiuum-format --test section13_corpus -- --nocapture || ERR=1

if [[ "$ERR" -ne 0 ]]; then
  echo "verify-csq-format-corpus: FAILED" >&2
  exit 1
fi
echo "verify-csq-format-corpus: OK"
echo "  manifest: $MANIFEST"