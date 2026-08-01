#!/usr/bin/env bash
# Pre-release / human briefing: run a tiered set of gates, ingest known
# JSON reports, emit easy-to-read HTML + machine JSON under
# target/release-briefing/.
#
# Usage (repo root):
#   bash scripts/release-briefing.sh
#   bash scripts/release-briefing.sh --profile formal
#   bash scripts/release-briefing.sh --profile pre-release
#   bash scripts/release-briefing.sh --profile snapshot --collect-only
#
# Profiles:
#   snapshot     default — cheap gates + collect existing evidence (fast)
#   formal       snapshot + FAS-0…FAS-4 package scripts (needs tools for full pass)
#   pre-release  formal + CSQ A2 verify + identity (still not full quality.sh)
#   quality-hint does not run quality.sh (too long for a briefing default);
#                prints pointer — use ./scripts/quality.sh separately
#
# Honesty: not_run ≠ pass. Fail if any executed gate fails.
# See formal/HOW_TO_USE.md and MASTER_DELIVERY_PLAN.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="${HOME}/.elan/bin:${HOME}/.cargo/bin:${PATH}"

PROFILE="snapshot"
COLLECT_ONLY=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile) PROFILE="${2:-}"; shift 2 ;;
    --collect-only) COLLECT_ONLY=1; shift ;;
    -h|--help)
      sed -n '2,25p' "$0"
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
done

case "$PROFILE" in
  snapshot|formal|pre-release|quality-hint) ;;
  *)
    echo "unknown profile: $PROFILE (snapshot|formal|pre-release|quality-hint)" >&2
    exit 2
    ;;
esac

OUT_DIR="$ROOT/target/release-briefing"
mkdir -p "$OUT_DIR"
STAMP="$(date -u +"%Y%m%dT%H%M%SZ")"
RUN_LOG="$OUT_DIR/run-${STAMP}.jsonl"
JSON_OUT="$OUT_DIR/briefing-${STAMP}.json"
HTML_OUT="$OUT_DIR/briefing-${STAMP}.html"
LATEST_JSON="$OUT_DIR/LATEST.json"
LATEST_HTML="$OUT_DIR/LATEST.html"
: >"$RUN_LOG"

git_head="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
host="$(hostname 2>/dev/null || echo unknown)"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

log_step() {
  # id title status duration_ms command detail
  python3 - "$1" "$2" "$3" "$4" "$5" "$6" <<'PY' >>"$RUN_LOG"
import json, sys
id_, title, status, dur, cmd, detail = sys.argv[1:7]
try:
    dur_i = int(dur)
except ValueError:
    dur_i = None
print(json.dumps({
    "id": id_,
    "title": title,
    "status": status,
    "duration_ms": dur_i,
    "command": cmd,
    "detail": detail[:2000],
}, ensure_ascii=False))
PY
}

run_gate() {
  local id="$1" title="$2" cmd="$3"
  local start end dur rc=0
  local outf
  outf="$(mktemp)"
  echo "== $id: $title =="
  start=$(python3 -c 'import time; print(int(time.time()*1000))')
  set +e
  bash -c "$cmd" >"$outf" 2>&1
  rc=$?
  set -e
  end=$(python3 -c 'import time; print(int(time.time()*1000))')
  dur=$((end - start))
  local tail
  tail="$(tail -c 800 "$outf" | tr '\n' ' ' | tr -d '\r')"
  if [[ $rc -eq 0 ]]; then
    log_step "$id" "$title" "pass" "$dur" "$cmd" "$tail"
    echo "   PASS (${dur}ms)"
  else
    log_step "$id" "$title" "fail" "$dur" "$cmd" "exit=$rc $tail"
    echo "   FAIL exit=$rc (${dur}ms)" >&2
  fi
  rm -f "$outf"
  return 0  # keep going; overall status computed later
}

skip_step() {
  log_step "$1" "$2" "not_run" "0" "${3:-}" "${4:-skipped by profile/collect-only}"
  echo "== $1: $2 [not_run] =="
}

# --- execute profile ---
if [[ "$COLLECT_ONLY" -eq 1 ]]; then
  skip_step "gates" "All gates" "" "collect-only mode"
else
  run_gate "delivery-status" "Program delivery scoreboard (M0-3)" \
    "bash ./scripts/verify-delivery-status.sh"
  run_gate "identity" "Protocol identity reset" \
    "node ./scripts/check-residiuum-identity.mjs"
  run_gate "fas0" "FAS-0 formal registry" \
    "bash ./scripts/check-formal-registry.sh"

  if [[ "$PROFILE" == "formal" || "$PROFILE" == "pre-release" ]]; then
    run_gate "fas1" "FAS-1 formal toolchain" \
      "bash ./scripts/check-formal-toolchain.sh"
    run_gate "fas2" "FAS-2 foundation kernel" \
      "bash ./scripts/check-formal-foundation.sh"
    run_gate "fas3" "FAS-3 refinement bridge" \
      "bash ./scripts/check-formal-refinement.sh"
    run_gate "fas4" "FAS-4 consistency family" \
      "bash ./scripts/check-formal-consistency.sh"
  else
    skip_step "fas1" "FAS-1 formal toolchain" "bash ./scripts/check-formal-toolchain.sh" "profile=$PROFILE"
    skip_step "fas2" "FAS-2 foundation kernel" "bash ./scripts/check-formal-foundation.sh" "profile=$PROFILE"
    skip_step "fas3" "FAS-3 refinement bridge" "bash ./scripts/check-formal-refinement.sh" "profile=$PROFILE"
    skip_step "fas4" "FAS-4 consistency family" "bash ./scripts/check-formal-consistency.sh" "profile=$PROFILE"
  fi

  if [[ "$PROFILE" == "pre-release" ]]; then
    run_gate "csq-a2" "CSQ core-storage A2 verify" \
      "bash ./scripts/residiuum-verify-core-storage.sh --require-a2-pass"
  else
    skip_step "csq-a2" "CSQ core-storage A2 verify" \
      "bash ./scripts/residiuum-verify-core-storage.sh --require-a2-pass" "profile=$PROFILE"
  fi

  if [[ "$PROFILE" == "quality-hint" ]]; then
    skip_step "quality" "Full quality.sh (DEF-090)" "./scripts/quality.sh" \
      "Not auto-run: long. Run manually before release: ./scripts/quality.sh"
  else
    skip_step "quality" "Full quality.sh (DEF-090)" "./scripts/quality.sh" \
      "Use profile quality-hint reminder or run ./scripts/quality.sh separately"
  fi

  skip_step "pqh-qual" "PQH controlled qualification campaign" \
    "residiuum-perf / PQH CLI qualification" \
    "Not auto-run: host/time bound. Principal accept is scoreboard-driven."
fi

# --- assemble briefing JSON + HTML ---
export ROOT RUN_LOG JSON_OUT HTML_OUT LATEST_JSON LATEST_HTML
export PROFILE STAMP generated_at git_head host COLLECT_ONLY

python3 <<'PY'
import json, os, re
from pathlib import Path

root = Path(os.environ["ROOT"])
run_log = Path(os.environ["RUN_LOG"])
profile = os.environ["PROFILE"]
steps = []
if run_log.is_file() and run_log.stat().st_size:
    for line in run_log.read_text().splitlines():
        line = line.strip()
        if line:
            steps.append(json.loads(line))

# Ingest known artifacts
candidates = [
    ("fas0", "target/formal-assurance/fas0-registry-report.json"),
    ("fas1", "target/formal-assurance/fas1-toolchain-report.json"),
    ("fas2", "target/formal-assurance/fas2-foundation-report.json"),
    ("fas3", "target/formal-assurance/fas3-refinement-report.json"),
    ("fas4", "target/formal-assurance/fas4-consistency-report.json"),
    ("csq-a2", "target/csq-evidence/a2-evaluation.json"),
    ("csq-core", "target/csq-evidence/residiuum-core-storage-report-v1.json"),
    ("csq-verify", "target/csq-evidence/residiuum-verification-report-v1.json"),
]

def summarize(path: Path, data: dict) -> str:
    if "a2_pass" in data:
        return f"a2_pass={data.get('a2_pass')} missing={data.get('missing', data.get('missing_count', '?'))}"
    if data.get("schema", "").startswith("residiuum-formal-package"):
        return f"result={data.get('result')} closed={data.get('closed')} msg={data.get('message', '')[:80]}"
    if "result" in data:
        return f"result={data.get('result')}"
    keys = list(data.keys())[:6]
    return "keys=" + ",".join(keys)

artifacts = []
for aid, rel in candidates:
    p = root / rel
    if not p.is_file():
        artifacts.append({
            "id": aid, "path": rel, "status": "missing",
            "summary": "file not present (run corresponding gate or CSQ verify)",
        })
        continue
    try:
        data = json.loads(p.read_text())
        # derive status
        st = "present"
        if data.get("result") == "pass" or data.get("a2_pass") is True:
            st = "pass"
        elif data.get("result") == "fail" or data.get("a2_pass") is False:
            st = "fail"
        artifacts.append({
            "id": aid, "path": rel, "status": st,
            "summary": summarize(p, data),
        })
    except Exception as e:
        artifacts.append({
            "id": aid, "path": rel, "status": "warn",
            "summary": f"unreadable: {e}",
        })

fail_n = sum(1 for s in steps if (s.get("status") or "").lower() == "fail")
pass_n = sum(1 for s in steps if (s.get("status") or "").lower() == "pass")
if fail_n:
    overall = "fail"
elif pass_n == 0 and not steps:
    overall = "not_run"
elif any((s.get("status") or "").lower() == "pass" for s in steps) and fail_n == 0:
    # snapshot can be pass with skips
    overall = "pass"
else:
    overall = "partial"

briefing = {
    "schema": "residiuum-release-briefing-v1",
    "overall_status": overall,
    "meta": {
        "profile": profile,
        "generated_at": os.environ["generated_at"],
        "git_head": os.environ["git_head"],
        "host": os.environ["host"],
        "stamp": os.environ["STAMP"],
        "collect_only": os.environ.get("COLLECT_ONLY") == "1",
    },
    "steps": steps,
    "artifacts": artifacts,
    "pointers": {
        "quality": "./scripts/quality.sh",
        "nightly": "./scripts/nightly.sh",
        "csq_a2": "bash scripts/residiuum-verify-core-storage.sh --require-a2-pass",
        "fas_howto": "formal/HOW_TO_USE.md",
        "scoreboard": "doc/wip/status/NEXT_BUILD_STATUS.md",
        "master_plan": "MASTER_DELIVERY_PLAN.md",
    },
}

json_out = Path(os.environ["JSON_OUT"])
html_out = Path(os.environ["HTML_OUT"])
json_out.write_text(json.dumps(briefing, indent=2) + "\n")
Path(os.environ["LATEST_JSON"]).write_text(json.dumps(briefing, indent=2) + "\n")

import subprocess
subprocess.check_call([
    "python3", str(root / "scripts/lib/release_briefing_render.py"),
    str(json_out), "--html", str(html_out),
])
import shutil
shutil.copyfile(html_out, os.environ["LATEST_HTML"])

print(json.dumps({
    "overall_status": overall,
    "json": str(json_out),
    "html": str(html_out),
    "latest_html": os.environ["LATEST_HTML"],
    "steps": len(steps),
    "fail": fail_n,
    "pass": pass_n,
}, indent=2))
raise SystemExit(1 if overall == "fail" else 0)
PY
