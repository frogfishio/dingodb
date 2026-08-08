#!/usr/bin/env bash
# RQL-Q3 one-command green: Q3.1 oracle + Q3.2 differential + Q3.3 adversarial.
# Exit 0 = labor evidence only. Does NOT accept the package (principal).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ok() { printf 'verify-rql-q3: %s\n' "$*"; }
fail() { printf 'verify-rql-q3: FAIL: %s\n' "$*" >&2; exit 1; }

need() {
  if [[ ! -f "$1" ]]; then
    fail "missing required file: $1"
  fi
}

need "$ROOT/crates/residiuum-sdk/tests/rql_q3_semantic_oracle.rs"
need "$ROOT/crates/residiuum-sdk/tests/rql_q3_differential_matrix.rs"
need "$ROOT/crates/residiuum-sdk/tests/rql_q3_adversarial.rs"
need "$ROOT/doc/todo/rql/RQL_Q3_1_SEMANTIC_ORACLE.md"
need "$ROOT/doc/todo/rql/RQL_Q3_2_DIFFERENTIAL_MATRIX.md"
need "$ROOT/doc/todo/rql/RQL_Q3_3_ADVERSARIAL_SUITE.md"
need "$ROOT/spec/rql/qualification/corpus-v1/corpus-v1.json"
need "$ROOT/tools/rql_q1/materialise_fixture.py"

command -v cargo >/dev/null 2>&1 || fail "cargo required"
command -v python3 >/dev/null 2>&1 || fail "python3 required"

ok "Q3.1 independent semantic oracle"
cargo test -p residiuum-sdk --test rql_q3_semantic_oracle

ok "Q3.2 differential matrix + metamorphic laws"
cargo test -p residiuum-sdk --test rql_q3_differential_matrix

ok "Q3.3 adversarial + damage + property"
cargo test -p residiuum-sdk --test rql_q3_adversarial

REPORT1="$ROOT/spec/rql/qualification/corpus-v1/q3_1_oracle_report.json"
REPORT2="$ROOT/spec/rql/qualification/corpus-v1/q3_2_differential_report.json"
REPORT3="$ROOT/spec/rql/qualification/corpus-v1/q3_3_adversarial_report.json"
need "$REPORT1"
need "$REPORT2"
need "$REPORT3"

python3 - "$REPORT1" "$REPORT2" "$REPORT3" <<'PY'
import json, sys

def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)

r1, r2, r3 = load(sys.argv[1]), load(sys.argv[2]), load(sys.argv[3])
if r1.get("format") != "residiuum-rql-q3-1-oracle-report-v1":
    raise SystemExit(f"bad q3.1 format {r1.get('format')!r}")
if r2.get("format") != "residiuum-rql-q3-2-differential-report-v1":
    raise SystemExit(f"bad q3.2 format {r2.get('format')!r}")
if r3.get("format") != "residiuum-rql-q3-3-adversarial-report-v1":
    raise SystemExit(f"bad q3.3 format {r3.get('format')!r}")

s1, s2, s3 = r1.get("summary") or {}, r2.get("summary") or {}, r3.get("summary") or {}
if int(s1.get("digest_mismatch") or 0) or int(s1.get("oracle_eval_fail") or 0):
    raise SystemExit(f"q3.1 residual fail: {s1}")
if int(s1.get("oracle_ok") or 0) < 90:
    raise SystemExit(f"q3.1 oracle_ok floor: {s1}")
if int(s2.get("matrix_diverge") or 0) or int(s2.get("errors") or 0) or int(s2.get("reopen_fail") or 0):
    raise SystemExit(f"q3.2 residual fail: {s2}")
if int(s2.get("matrix_equal") or 0) < 90:
    raise SystemExit(f"q3.2 matrix_equal floor: {s2}")
if int(s3.get("false_absence_defects") or 0) or int(s3.get("false_completeness_defects") or 0):
    raise SystemExit(f"q3.3 false absence/completeness: {s3}")
if int(s3.get("unresolved_divergence") or 0):
    raise SystemExit(f"q3.3 unresolved divergence: {s3}")
if int(s3.get("dimensions_covered") or 0) < 10:
    raise SystemExit(f"q3.3 dimensions floor: {s3}")

print(
    "verify-rql-q3: report ok "
    f"oracle_ok={s1.get('oracle_ok')} matrix_equal={s2.get('matrix_equal')} "
    f"unsupported={s2.get('unsupported')} "
    f"adv_dims={s3.get('dimensions_covered')} property_seeds={s3.get('property_seeds')}"
)
PY

ok "PASS (Q3.1+Q3.2+Q3.3 labor evidence only — not package accept)"
