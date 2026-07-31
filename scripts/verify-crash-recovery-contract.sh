#!/usr/bin/env bash
# verify-crash-recovery-contract.sh — DEF-104 contract page + executable suite.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "verify-crash-recovery-contract: FAIL: $*" >&2; exit 1; }
ok() { echo "verify-crash-recovery-contract: $*"; }

DOC="doc/reference/operations/CRASH_AND_RECOVERY_CONTRACT.md"
[[ -f "$DOC" ]] || fail "missing $DOC"

python3 - <<'PY' || exit 1
from pathlib import Path
text = Path("doc/reference/operations/CRASH_AND_RECOVERY_CONTRACT.md").read_text(encoding="utf-8")
needles = [
    "dingo-crash-recovery-v1",
    "Durability-mode acknowledgement",
    "Inline and chunked publication",
    "Read outcome decision table",
    "Exact Store and Collection recovery APIs",
    "Key coverage versus body completeness",
    "Historical-version selection",
    "Writer-lock recovery",
    "Authority versus derived",
    "Large and rewrite-heavy",
    "Operator decision tree",
    "Capability limitations",
    "Forbidden",
    "stage_def_104_crash_recovery_contract",
]
missing = [n for n in needles if n not in text]
if missing:
    raise SystemExit(f"missing contract markers: {missing}")
print("verify-crash-recovery-contract: document sections OK")
PY

ok "running stage_def_104 suite"
cargo test -p residiuum-store --features legacy-raw-store --test stage_def_104_crash_recovery_contract -- --quiet

ok "OK"
