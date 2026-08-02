#!/usr/bin/env bash
# AWO package verification orchestrator.
# AWO-0: contract + pure model + formal source checks.
# Later packages extend the ordered steps below without reordering earlier ones.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

section() { printf '\n==> %s\n' "$*"; }

section "1/8 verify-awo-contract.sh"
bash "$ROOT/scripts/verify-awo-contract.sh"

section "2/8 AWO pure model unit tests"
cargo test -p residiuum-store --lib adaptive_write::model

section "3/8 AWO contract integration (golden runner)"
cargo test -p residiuum-store --test awo_contract

section "4/8 formal + verification source checks (AWO-0)"
need() {
  if [[ ! -e "$1" ]]; then
    printf 'FAIL: missing %s\n' "$1" >&2
    exit 1
  fi
}
need "$ROOT/formal/awo/tla/AdaptiveWrite.tla"
need "$ROOT/formal/awo/tla/AdaptiveWrite.cfg"
need "$ROOT/formal/awo/verus/model.rs"
need "$ROOT/formal/awo/README.md"
need "$ROOT/verification/awo/golden/README.md"
need "$ROOT/verification/awo/golden/golden-decisions-v1.json"
# Symlink or file must resolve to the same closed golden set.
python3 - "$ROOT" <<'PY'
import json, sys
from pathlib import Path
root = Path(sys.argv[1])
canon = root / "spec/performance/awo/golden-decisions-v1.json"
link = root / "verification/awo/golden/golden-decisions-v1.json"
if not canon.is_file():
    sys.exit("missing canonical golden-decisions-v1.json")
if not link.exists():
    sys.exit("missing verification/awo/golden/golden-decisions-v1.json")
a = json.loads(canon.read_text())
b = json.loads(link.read_text())
if a != b:
    sys.exit("verification golden copy/symlink diverges from spec/performance/awo")
# TLA skeleton must name closed plan variables (presence check).
tla = (root / "formal/awo/tla/AdaptiveWrite.tla").read_text()
for token in (
    "reqState", "reqLane", "reqTicket", "laneNextAdmit", "laneNextInstall",
    "queueBytes", "queueEntries", "reservation", "persisted", "published",
    "acked", "writerHealth", "AckImpliesPersisted", "NoPublishBeforePersist",
    "Admit", "PersistOk", "Publish", "Complete", "Crash", "Recover",
):
    if token not in tla:
        sys.exit(f"AdaptiveWrite.tla missing required token: {token}")
print("formal/verification source checks OK")
PY

section "5/8 CSQ ack/recovery subset (deferred until AWO-1+)"
printf 'SKIP (AWO-0): CSQ acknowledgement/recovery subset reserved for AWO-1+ verify-awo expansion\n'

section "6/8 heap isolation tests (deferred until AWO-3+)"
printf 'SKIP (AWO-0): heap isolation reserved for product integration packages\n'

section "7/8 server qualified mutation tests (deferred until AWO-3+)"
printf 'SKIP (AWO-0): server AWO RPC reserved for AWO-3+\n'

section "8/8 PQH AWO smoke (deferred; never qualification claim)"
printf 'SKIP (AWO-0): PQH AWO smoke reserved for AWO-6; smoke never marks AWO-G8\n'

# Optional TLC if present — best-effort, non-fatal for AWO-0.
if command -v tlc >/dev/null 2>&1; then
  section "optional TLC AdaptiveWrite skeleton"
  (cd "$ROOT/formal/awo/tla" && tlc -config AdaptiveWrite.cfg AdaptiveWrite.tla) || {
    printf 'WARN: tlc present but AdaptiveWrite check failed (AWO-0 non-fatal)\n' >&2
  }
else
  printf '\n(optional) tlc not on PATH — skeleton not model-checked this run\n'
fi

printf '\nverify-awo: AWO-0 checks OK (contract + model + formal sources)\n'
