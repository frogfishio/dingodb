#!/usr/bin/env bash
# RQL-Q4 structural verify: Q4.1 architecture + Q4.2 datasets + Q4.3 metrics/adapters.
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
need "$ROOT/crates/residiuum-rql-qual/src/metrics.rs"
need "$ROOT/crates/residiuum-rql-qual/src/engine.rs"
need "$ROOT/crates/residiuum-rql-qual/src/run.rs"
need "$ROOT/crates/residiuum-rql-qual/src/shared_work.rs"
need "$ROOT/doc/todo/rql/RQL_Q4_1_HARNESS_ARCHITECTURE.md"
need "$ROOT/doc/todo/rql/RQL_Q4_2_DATASET_CELLS.md"
need "$ROOT/doc/todo/rql/RQL_Q4_3_METRICS_ADAPTERS.md"
need "$ROOT/spec/rql/qualification/harness-v1/evidence-bundle-v1.schema.json"
need "$ROOT/spec/rql/qualification/corpus-v1/corpus-v1.json"

command -v cargo >/dev/null 2>&1 || fail "cargo required"
grep -q 'residiuum-rql-qual' "$ROOT/Cargo.toml" || fail "workspace member missing"

ok "unit tests (Q4.1–Q4.3 structural)"
cargo test -p residiuum-rql-qual

REPORT1="$ROOT/spec/rql/qualification/harness-v1/q4_1_architecture_report.json"
REPORT2="$ROOT/spec/rql/qualification/harness-v1/q4_2_dataset_cells_report.json"
REPORT3="$ROOT/spec/rql/qualification/harness-v1/q4_3_metrics_adapters_report.json"
BUNDLE3="$ROOT/spec/rql/qualification/harness-v1/q4_3_smoke_evidence_bundle.json"
need "$REPORT1"
need "$REPORT2"
need "$REPORT3"
need "$BUNDLE3"

python3 - "$REPORT1" "$REPORT2" "$REPORT3" "$BUNDLE3" <<'PY'
import json, sys

def load(p):
    with open(p, encoding="utf-8") as f:
        return json.load(f)

r1, r2, r3, b = load(sys.argv[1]), load(sys.argv[2]), load(sys.argv[3]), load(sys.argv[4])
if r1.get("format") != "residiuum-rql-q4-1-architecture-report-v1":
    raise SystemExit(f"bad q4.1 format {r1.get('format')!r}")
if r2.get("format") != "residiuum-rql-q4-2-dataset-cells-report-v1":
    raise SystemExit(f"bad q4.2 format {r2.get('format')!r}")
if r3.get("format") != "residiuum-rql-q4-3-metrics-adapters-report-v1":
    raise SystemExit(f"bad q4.3 format {r3.get('format')!r}")
if b.get("format") != "residiuum-rql-qual-evidence-bundle-v1":
    raise SystemExit(f"bad bundle format {b.get('format')!r}")

s2 = r2.get("summary") or {}
s3 = r3.get("summary") or {}
if int(s2.get("smoke_plans") or 0) != 12:
    raise SystemExit(f"q4.2 smoke_plans {s2}")
if int(s3.get("smoke_cells") or 0) != 12:
    raise SystemExit(f"q4.3 smoke_cells {s3}")
if int(s3.get("logical_ready_with_result") or 0) != 12:
    raise SystemExit(f"q4.3 logical ready {s3}")
if s3.get("lane_s_fixture_identity") is not True:
    raise SystemExit(f"q4.3 lane_s identity {s3}")
if not b.get("content_hash"):
    raise SystemExit("bundle missing content_hash")
if len(b.get("cells") or []) != 12:
    raise SystemExit(f"bundle cells {len(b.get('cells') or [])}")

print(
    "verify-rql-q4-harness: report ok "
    f"q4.2 smoke={s2.get('smoke_plans')} "
    f"q4.3 cells={s3.get('smoke_cells')} ready={s3.get('logical_ready_with_result')} "
    f"lane_s={s3.get('lane_s_fixture_identity')} "
    f"bundle_hash={str(b.get('content_hash'))[:12]}…"
)
PY

ok "PASS (Q4.1–Q4.3 labor evidence only — not package accept / not competitive)"
