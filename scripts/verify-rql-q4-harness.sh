#!/usr/bin/env bash
# RQL-Q4.1 structural verify: harness crate + schemas + architecture report.
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
need "$ROOT/crates/residiuum-rql-qual/Cargo.toml"
need "$ROOT/doc/todo/rql/RQL_Q4_1_HARNESS_ARCHITECTURE.md"
need "$ROOT/spec/rql/qualification/harness-v1/evidence-bundle-v1.schema.json"
need "$ROOT/spec/rql/qualification/harness-v1/env-fingerprint-v1.schema.json"
need "$ROOT/spec/rql/qualification/harness-v1/cell-result-v1.schema.json"
need "$ROOT/spec/rql/qualification/corpus-v1/corpus-v1.json"

command -v cargo >/dev/null 2>&1 || fail "cargo required"

# Workspace member present
grep -q 'residiuum-rql-qual' "$ROOT/Cargo.toml" || fail "workspace member missing"

ok "unit tests (structural)"
cargo test -p residiuum-rql-qual

REPORT="$ROOT/spec/rql/qualification/harness-v1/q4_1_architecture_report.json"
need "$REPORT"

python3 - "$REPORT" <<'PY'
import json, sys
path = sys.argv[1]
with open(path, encoding="utf-8") as f:
    r = json.load(f)
if r.get("format") != "residiuum-rql-q4-1-architecture-report-v1":
    raise SystemExit(f"bad format {r.get('format')!r}")
if r.get("mandatory_cells") != 12:
    raise SystemExit(f"mandatory_cells {r.get('mandatory_cells')}")
if len(r.get("lanes") or []) != 2:
    raise SystemExit("expected 2 lanes")
if len(r.get("engines") or []) != 4:
    raise SystemExit("expected 4 engines")
if r.get("crate") != "residiuum-rql-qual":
    raise SystemExit("crate field")
print(
    f"verify-rql-q4-harness: report ok lanes={r['lanes']} "
    f"engines={len(r['engines'])} cells={r['mandatory_cells']}"
)
PY

ok "PASS (Q4.1 scaffold labor only — not package accept / not competitive)"
