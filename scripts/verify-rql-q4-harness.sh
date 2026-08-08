#!/usr/bin/env bash
# RQL-Q4 structural verify: Q4.1 architecture + Q4.2 dataset/cells.
# Exit 0 = scaffold labor green. Not package accept / not competitive.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ok() { printf 'verify-rql-q4-harness: %s\n' "$*"; }
fail() { printf 'verify-rql-q4-harness: FAIL: %s\n' "$*" >&2; exit 1; }

need() {
  if [[ ! -f "$1" ]]; then
    fail "missing required file: $1"
  fi
}

need "$ROOT/crates/residiuum-rql-qual/src/lib.rs"
need "$ROOT/crates/residiuum-rql-qual/src/dataset.rs"
need "$ROOT/crates/residiuum-rql-qual/src/generator.rs"
need "$ROOT/crates/residiuum-rql-qual/src/cell_plan.rs"
need "$ROOT/crates/residiuum-rql-qual/src/lifecycle.rs"
need "$ROOT/crates/residiuum-rql-qual/Cargo.toml"
need "$ROOT/doc/todo/rql/RQL_Q4_1_HARNESS_ARCHITECTURE.md"
need "$ROOT/doc/todo/rql/RQL_Q4_2_DATASET_CELLS.md"
need "$ROOT/spec/rql/qualification/harness-v1/evidence-bundle-v1.schema.json"
need "$ROOT/spec/rql/qualification/harness-v1/env-fingerprint-v1.schema.json"
need "$ROOT/spec/rql/qualification/harness-v1/cell-result-v1.schema.json"
need "$ROOT/spec/rql/qualification/corpus-v1/corpus-v1.json"

command -v cargo >/dev/null 2>&1 || fail "cargo required"

grep -q 'residiuum-rql-qual' "$ROOT/Cargo.toml" || fail "workspace member missing"

ok "unit tests (Q4.1+Q4.2 structural)"
cargo test -p residiuum-rql-qual

REPORT1="$ROOT/spec/rql/qualification/harness-v1/q4_1_architecture_report.json"
REPORT2="$ROOT/spec/rql/qualification/harness-v1/q4_2_dataset_cells_report.json"
need "$REPORT1"
need "$REPORT2"

python3 - "$REPORT1" "$REPORT2" <<'PY'
import json, sys

def load(p):
    with open(p, encoding="utf-8") as f:
        return json.load(f)

r1, r2 = load(sys.argv[1]), load(sys.argv[2])
if r1.get("format") != "residiuum-rql-q4-1-architecture-report-v1":
    raise SystemExit(f"bad q4.1 format {r1.get('format')!r}")
if r1.get("mandatory_cells") != 12:
    raise SystemExit(f"q4.1 mandatory_cells {r1.get('mandatory_cells')}")
if len(r1.get("lanes") or []) != 2:
    raise SystemExit("expected 2 lanes")
if len(r1.get("engines") or []) != 4:
    raise SystemExit("expected 4 engines")

if r2.get("format") != "residiuum-rql-q4-2-dataset-cells-report-v1":
    raise SystemExit(f"bad q4.2 format {r2.get('format')!r}")
s = r2.get("summary") or {}
if int(s.get("mandatory_cells") or 0) != 12:
    raise SystemExit(f"q4.2 mandatory_cells {s}")
if int(s.get("smoke_plans") or 0) != 12:
    raise SystemExit(f"q4.2 smoke_plans {s}")
if int(s.get("concurrency_matrix_len") or 0) < 5:
    raise SystemExit(f"q4.2 concurrency matrix {s}")
if int(s.get("selectivity_matrix_len") or 0) != 5:
    raise SystemExit(f"q4.2 selectivity {s}")
if int(s.get("lifecycle_matrix_len") or 0) != 7:
    raise SystemExit(f"q4.2 lifecycle {s}")
if s.get("cold_reopen_claims_device_cold") is not False:
    raise SystemExit("reopen must not claim device cold")

print(
    "verify-rql-q4-harness: report ok "
    f"lanes={r1.get('lanes')} engines={len(r1.get('engines') or [])} "
    f"smoke_plans={s.get('smoke_plans')} "
    f"conc={s.get('concurrency_matrix_len')} "
    f"sel={s.get('selectivity_matrix_len')} "
    f"life={s.get('lifecycle_matrix_len')}"
)
PY

ok "PASS (Q4.1+Q4.2 labor evidence only — not package accept / not competitive)"
