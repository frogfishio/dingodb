#!/usr/bin/env bash
# RQL-Q3.1: independent semantic oracle suite (test-only; no product path).
# Exit 0 = hand units + corpus oracle_ok floor green. Does not accept the package.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ok() { printf 'verify-rql-q3-oracle: %s\n' "$*"; }
fail() { printf 'verify-rql-q3-oracle: FAIL: %s\n' "$*" >&2; exit 1; }

need() {
  if [[ ! -f "$1" ]]; then
    fail "missing required file: $1"
  fi
}

need "$ROOT/crates/residiuum-sdk/tests/rql_q3_semantic_oracle.rs"
need "$ROOT/doc/todo/rql/RQL_Q3_1_SEMANTIC_ORACLE.md"
need "$ROOT/spec/rql/qualification/corpus-v1/corpus-v1.json"
need "$ROOT/tools/rql_q1/materialise_fixture.py"

command -v cargo >/dev/null 2>&1 || fail "cargo required"
command -v python3 >/dev/null 2>&1 || fail "python3 required"

ok "running cargo test -p residiuum-sdk --test rql_q3_semantic_oracle"
cargo test -p residiuum-sdk --test rql_q3_semantic_oracle

REPORT="$ROOT/spec/rql/qualification/corpus-v1/q3_1_oracle_report.json"
need "$REPORT"

python3 - "$REPORT" <<'PY'
import json, sys
path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    doc = json.load(f)
if doc.get("format") != "residiuum-rql-q3-1-oracle-report-v1":
    print(f"verify-rql-q3-oracle: FAIL: bad format {doc.get('format')!r}", file=sys.stderr)
    sys.exit(1)
if doc.get("oracle_profile") != "residiuum-rql-q3-semantic-oracle-v1":
    print(f"verify-rql-q3-oracle: FAIL: bad oracle_profile", file=sys.stderr)
    sys.exit(1)
b = doc.get("boundary") or {}
if b.get("product_callable") is not False:
    print("verify-rql-q3-oracle: FAIL: product_callable must be false", file=sys.stderr)
    sys.exit(1)
if b.get("uses_index_selection") is not False:
    print("verify-rql-q3-oracle: FAIL: uses_index_selection must be false", file=sys.stderr)
    sys.exit(1)
if b.get("uses_execute_rql_full") is not False:
    print("verify-rql-q3-oracle: FAIL: uses_execute_rql_full must be false", file=sys.stderr)
    sys.exit(1)
s = doc.get("summary") or {}
ok = int(s.get("oracle_ok") or 0)
mm = int(s.get("digest_mismatch") or 0)
ef = int(s.get("oracle_eval_fail") or 0)
ff = int(s.get("oracle_fixture_fail") or 0)
if mm != 0 or ef != 0 or ff != 0:
    print(f"verify-rql-q3-oracle: FAIL: mismatch={mm} eval_fail={ef} fixture_fail={ff}", file=sys.stderr)
    sys.exit(1)
if ok < 90:
    print(f"verify-rql-q3-oracle: FAIL: oracle_ok floor 90, got {ok}", file=sys.stderr)
    sys.exit(1)
print(f"verify-rql-q3-oracle: report ok oracle_ok={ok} unsupported={s.get('oracle_unsupported')}")
PY

ok "PASS (Q3.1 labor evidence only — not package accept)"
