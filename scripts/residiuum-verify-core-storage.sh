#!/usr/bin/env bash
# CSQ-12 stand-in for:
#   residiuum verify --profile residiuum-core-storage-v1 --level A2
#
# Builds a residiuum-core-storage-report-v1 evidence bundle (or exact missing
# cells) and independently verifies it. Does not claim A2 pass while residual
# gates remain open.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROFILE="residiuum-core-storage-v1"
LEVEL="A2"
OUT_DIR="${RESIDIUUM_CSQ_EVIDENCE_DIR:-$ROOT/target/csq-evidence}"
REQUIRE_A2_PASS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="${2:?}"
      shift 2
      ;;
    --level)
      LEVEL="${2:?}"
      shift 2
      ;;
    --output-dir)
      OUT_DIR="${2:?}"
      shift 2
      ;;
    --require-a2-pass)
      REQUIRE_A2_PASS=1
      shift
      ;;
    -h|--help)
      cat <<EOF
Usage: $0 [--profile residiuum-core-storage-v1] [--level A2]
          [--output-dir DIR] [--require-a2-pass]

One clean command for CSQ-12: build evidence bundle + independent verify.
EOF
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

if [[ "$PROFILE" != "residiuum-core-storage-v1" ]]; then
  echo "error: only profile residiuum-core-storage-v1 is admitted (got $PROFILE)" >&2
  exit 2
fi
if [[ "$PROFILE" == *dingo* ]]; then
  echo "error: dingo profile identity is inadmissible" >&2
  exit 2
fi

mkdir -p "$OUT_DIR"
REPORT="$OUT_DIR/residiuum-core-storage-report-v1.json"
ENVELOPE="$OUT_DIR/residiuum-verification-report-v1.json"

export RESIDIUUM_WORKSPACE_ROOT="$ROOT"
PY="$ROOT/scripts/lib/csq_evidence.py"

echo "== CSQ-12 evidence selftest =="
python3 "$PY" selftest

echo "== build core-storage report (level=$LEVEL) =="
python3 "$PY" build --level "$LEVEL" --output "$REPORT"
python3 "$PY" build --level "$LEVEL" --envelope --output "$ENVELOPE"

echo "== independent verify =="
VERIFY_ARGS=("$REPORT")
if [[ "$REQUIRE_A2_PASS" -eq 1 ]]; then
  VERIFY_ARGS+=(--require-a2-pass)
fi
python3 "$PY" verify "${VERIFY_ARGS[@]}"
python3 "$PY" evaluate "$REPORT" | tee "$OUT_DIR/a2-evaluation.json"

echo "residiuum-verify-core-storage: OK"
echo "  profile: $PROFILE"
echo "  level: $LEVEL"
echo "  report: $REPORT"
echo "  envelope: $ENVELOPE"
echo "  note: A2 pass is not claimed until residual gates close; missing cells are exact."
