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
ALLOW_FAIL=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile) PROFILE="${2:-}"; shift 2 ;;
    --collect-only) COLLECT_ONLY=1; shift ;;
    --allow-fail)
      # Still write HTML/JSON; exit 0 even if a gate failed (for make dist packaging).
      ALLOW_FAIL=1
      shift
      ;;
    -h|--help)
      sed -n '2,30p' "$0"
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
export PROFILE STAMP generated_at git_head host COLLECT_ONLY ALLOW_FAIL

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

# --- latest published write/read diagnostic metrics (surveys; not product gates) ---
def _rss_mib(b):
    try:
        return round(float(b) / (1024 * 1024), 1)
    except (TypeError, ValueError):
        return None

def _load_peer_cell(rel, label):
    p = root / rel
    if not p.is_file():
        return None
    try:
        d = json.loads(p.read_text())
    except Exception:
        return None
    if not isinstance(d, dict) or "ops_per_sec" not in d:
        return None
    elapsed_ms = d.get("elapsed_ms")
    elapsed_s = round(float(elapsed_ms) / 1000.0, 3) if elapsed_ms is not None else None
    return {
        "id": Path(rel).stem,
        "label": label,
        "path": rel,
        "engine": d.get("engine"),
        "mode": d.get("mode"),
        "ops_per_sec": round(float(d["ops_per_sec"]), 1),
        "logical_mib_s": round(float(d.get("mb_per_sec") or 0), 2),
        "disk_mib_s": round(float(d.get("mb_per_sec_disk") or 0), 2) if d.get("mb_per_sec_disk") is not None else None,
        "keys_written": d.get("keys_written"),
        "payload_size": d.get("payload_size"),
        "peak_rss_bytes": d.get("peak_rss_bytes"),
        "peak_rss_mib": _rss_mib(d.get("peak_rss_bytes")),
        "peak_cpu_pct": d.get("peak_cpu_pct"),
        "elapsed_s": elapsed_s,
        "put_batch_size": d.get("put_batch_size"),
        "residiuum_durability": d.get("residiuum_durability") or None,
        "ok": d.get("ok"),
        "disclosure": d.get("disclosure"),
    }

def _ratio(a, b):
    if a is None or b is None or b == 0:
        return None
    return f"{a / b:.2f}×"

def _parse_phase_bench_txt(rel):
    p = root / rel
    if not p.is_file():
        return None
    wanted = {
        "store_put_memory",
        "store_put_buffered_batch1",
        "store_put_many_buffered",
        "raw_write_all_payload_only",
        "blake3_body_hash_only",
    }
    phases = []
    for line in p.read_text(errors="replace").splitlines():
        parts = line.split()
        if len(parts) < 4:
            continue
        name = parts[0]
        if name not in wanted:
            continue
        try:
            wall_ms = float(parts[1])
            ops = float(parts[2])
            mib = float(parts[3])
        except ValueError:
            continue
        note = " ".join(parts[4:]) if len(parts) > 4 else ""
        # Truncate long mode-a breakdown notes for HTML
        if len(note) > 160:
            note = note[:157] + "…"
        phases.append({
            "name": name,
            "wall_ms": wall_ms,
            "ops_per_sec": round(ops, 1),
            "logical_mib_s": round(mib, 1),
            "note": note,
        })
    if not phases:
        return None
    return {
        "title": "phase-bench after write-through (20k × 8 KiB Scratch micro)",
        "path": rel,
        "phases": phases,
        "band_rule": (
            "Three-band rule: ~10k multi-seal PEER long peer · ~100–160k real-disk 1 GiB batch · "
            "~330k short cook micro only — do not sell micro as media capacity "
            "(see TEST_RESULTS Campaign H park)."
        ),
    }

# Prefer post write-through peer cells (latest fair same-bed); fall back to Campaign F.
peer_specs = [
    ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/post-wt-ra.json",
     "residiuum-A (post write-through)"),
    ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/post-wt-sa.json",
     "sqlite-A (post write-through)"),
    ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/post-wt-rb.json",
     "residiuum-B (post write-through)"),
    ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/post-wt-sb.json",
     "sqlite-B (post write-through)"),
]
peer_cells = []
sources = []
for rel, lab in peer_specs:
    cell = _load_peer_cell(rel, lab)
    if cell:
        peer_cells.append(cell)
        sources.append(rel)

campaign = "post-write-through-20260801"
if not peer_cells:
    campaign = "campaign-f-20260801"
    for rel, lab in [
        ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/residiuum-A.json", "residiuum-A"),
        ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/sqlite-A.json", "sqlite-A"),
        ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/residiuum-B.json", "residiuum-B"),
        ("doc/wip/status/surveys/scratch-sqlite-peer-20260801/sqlite-B.json", "sqlite-B"),
    ]:
        cell = _load_peer_cell(rel, lab)
        if cell:
            peer_cells.append(cell)
            sources.append(rel)

# Also surface Mode-A long peer after write-through if present (extra write cell).
extra = _load_peer_cell(
    "doc/wip/status/surveys/scratch-mode-a-breakdown-20260801/peer-A-after-write-through.json",
    "residiuum Mode-A long peer (after write-through, 64 MiB seal)",
)
if extra:
    # keep in sources; do not mix into A/B ratio table
    sources.append(extra["path"])

by_engine_mode = {}
for c in peer_cells:
    by_engine_mode[(c.get("engine"), c.get("mode"))] = c

ratios = []
for mode, label_a, label_b in [
    ("A_autocommit", "A (autocommit / per-put Buffered)", "Residiuum ≈ SQLite within noise on fair Mode A"),
    ("B_txn_128", "B (txn-128 / put_many 128)", "SQLite amortizes commit; Mode B is not equal semantics"),
]:
    r = by_engine_mode.get(("residiuum", mode))
    s = by_engine_mode.get(("sqlite", mode))
    if not r or not s:
        continue
    ratios.append({
        "mode": label_a,
        "ops_ratio": _ratio(r.get("ops_per_sec"), s.get("ops_per_sec")),
        "logical_mib_ratio": _ratio(r.get("logical_mib_s"), s.get("logical_mib_s")),
        "read": label_b,
    })

micro_rel = "doc/wip/status/surveys/scratch-mode-a-breakdown-20260801/phase-bench-after-write-through.txt"
write_micro = _parse_phase_bench_txt(micro_rel)
if write_micro:
    sources.append(micro_rel)

# ---------------------------------------------------------------------------
# Parallel cook (the real "4-core batched" story) + three-band rule
# ---------------------------------------------------------------------------
def _load_phase_json_cooks(rel):
    """Return list of cook/batch phases from a phase-bench JSON."""
    p = root / rel
    if not p.is_file():
        return []
    try:
        d = json.loads(p.read_text())
    except Exception:
        return []
    out = []
    want = {
        "store_put_buffered_batch1",
        "store_put_many_cook1",
        "store_put_many_cook2",
        "store_put_many_cook4",
        "store_put_many_buffered",
    }
    for ph in d.get("phases") or []:
        name = ph.get("name")
        if name not in want:
            continue
        workers = None
        if name.endswith("cook1"):
            workers = 1
        elif name.endswith("cook2"):
            workers = 2
        elif name.endswith("cook4"):
            workers = 4
        out.append({
            "name": name,
            "ops_per_sec": round(float(ph.get("ops_per_sec") or 0), 1),
            "logical_mib_s": round(float(ph.get("logical_mib_per_sec") or 0), 2),
            "wall_ms": ph.get("wall_ms"),
            "cook_workers": workers,
            "put_batch_size": d.get("batch") or 128,
            "path": rel,
            "note": (ph.get("note") or "")[:200],
        })
    return out

def _load_phase_txt_cooks(rel):
    """Parse cook/batch lines from phase-bench.txt (tmp-real-disk has no JSON)."""
    p = root / rel
    if not p.is_file():
        return []
    want = {
        "store_put_buffered_batch1",
        "store_put_many_cook1",
        "store_put_many_cook2",
        "store_put_many_cook4",
        "store_put_many_buffered",
    }
    out = []
    for line in p.read_text(errors="replace").splitlines():
        parts = line.split()
        if len(parts) < 4:
            continue
        name = parts[0]
        if name not in want:
            continue
        try:
            wall_ms = float(parts[1])
            ops = float(parts[2])
            mib = float(parts[3])
        except ValueError:
            continue
        workers = None
        if name.endswith("cook1"):
            workers = 1
        elif name.endswith("cook2"):
            workers = 2
        elif name.endswith("cook4"):
            workers = 4
        out.append({
            "name": name,
            "ops_per_sec": round(ops, 1),
            "logical_mib_s": round(mib, 2),
            "wall_ms": wall_ms,
            "cook_workers": workers,
            "put_batch_size": 128,
            "path": rel,
            "note": " ".join(parts[4:])[:200],
        })
    return out

scratch_cook_rel = "doc/wip/status/surveys/scratch-parallel-cooker-20260801/phase-bench.json"
tmp_cook_rel = "doc/wip/status/surveys/tmp-real-disk-20260801/phase-bench.txt"
scratch_cooks = _load_phase_json_cooks(scratch_cook_rel)
tmp_cooks = _load_phase_txt_cooks(tmp_cook_rel)
if scratch_cooks:
    sources.append(scratch_cook_rel)
if tmp_cooks:
    sources.append(tmp_cook_rel)

def _pick_cook(rows, name):
    for r in rows:
        if r.get("name") == name:
            return r
    return None

scratch_cook4 = _pick_cook(scratch_cooks, "store_put_many_cook4")
scratch_cook1 = _pick_cook(scratch_cooks, "store_put_many_cook1")
tmp_cook4 = _pick_cook(tmp_cooks, "store_put_many_cook4")
tmp_cook2 = _pick_cook(tmp_cooks, "store_put_many_cook2")
tmp_cook1 = _pick_cook(tmp_cooks, "store_put_many_cook1")

# Three-band rule (Campaign H / PARKED-write-path-wall) — the operator story
# Band 1: multi-seal peer ~10k (from peer Residiuum Mode A if present)
peer_ra = next((c for c in peer_cells if c.get("engine") == "residiuum" and c.get("mode") == "A_autocommit"), None)
peer_rb = next((c for c in peer_cells if c.get("engine") == "residiuum" and c.get("mode") == "B_txn_128"), None)

three_bands = [
    {
        "band": 1,
        "name": "~10k multi-seal / PEER long peer",
        "ops_per_sec": peer_ra.get("ops_per_sec") if peer_ra else 10000,
        "logical_mib_s": peer_ra.get("logical_mib_s") if peer_ra else None,
        "what": "Adoption-floor peer path (Buffered, multi-seal over hundreds of MiB). SQLite Mode A same bed.",
        "cores": "1 producer thread (peer-pump)",
        "batch": peer_ra.get("put_batch_size") if peer_ra else 1,
        "bed": "Scratch PEER-SQL 256 MiB logical",
        "path": peer_ra.get("path") if peer_ra else None,
        "highlight": False,
    },
    {
        "band": 2,
        "name": "~100–160k real-disk 1 GiB batch (cook workers)",
        "ops_per_sec": tmp_cook4.get("ops_per_sec") if tmp_cook4 else None,
        "logical_mib_s": tmp_cook4.get("logical_mib_s") if tmp_cook4 else None,
        "what": (
            "THIS is the ~100k · 4-core batched number: put_many cook_parallelism=4, "
            "batch=128, ~1 GiB logical phase on APFS /tmp. cook2 peaks higher (~158k); "
            "cook4 ~116k (disk competing — cook4 does not win vs cook1 here)."
        ),
        "cores": "cook_parallelism=4 (parallel encode/Blake; ordered install)",
        "batch": 128,
        "bed": "macOS APFS /tmp · 131072 × 8 KiB ≈ 1 GiB/phase",
        "path": tmp_cook_rel if tmp_cook4 else None,
        "highlight": True,
        "cook_ladder": [
            {"workers": 1, "ops": tmp_cook1.get("ops_per_sec") if tmp_cook1 else None},
            {"workers": 2, "ops": tmp_cook2.get("ops_per_sec") if tmp_cook2 else None},
            {"workers": 4, "ops": tmp_cook4.get("ops_per_sec") if tmp_cook4 else None},
        ],
    },
    {
        "band": 3,
        "name": "~330k short Scratch cook micro only",
        "ops_per_sec": scratch_cook4.get("ops_per_sec") if scratch_cook4 else None,
        "logical_mib_s": scratch_cook4.get("logical_mib_s") if scratch_cook4 else None,
        "what": (
            "Short Scratch phase-bench cook4 (~20k ops). Cook scales ~1.8× vs cook1 when disk is slack. "
            "Do NOT sell as media / multi-seal capacity."
        ),
        "cores": "cook_parallelism=4 · Scratch micro",
        "batch": 128,
        "bed": "Scratch · short phase-bench",
        "path": scratch_cook_rel if scratch_cook4 else None,
        "highlight": False,
    },
]

# Parallel-cook detail rows (both beds)
cook_detail = {
    "title": "Parallel cook (put_many · cook_parallelism) — the real 4-core batched path",
    "contract": "doc/wip/status/surveys/PARKED-write-path-wall-20260801.md",
    "beds": [
        {
            "bed_id": "tmp-real-disk-1gib",
            "title": "/tmp real disk · ~1 GiB/phase (band ~100–160k)",
            "path": tmp_cook_rel,
            "rows": tmp_cooks,
            "note": "cook4 ~116k is the measured ~100k-class 4-worker batch number on real APFS /tmp.",
        },
        {
            "bed_id": "scratch-micro",
            "title": "Scratch short micro (band ~330k — do not mix with peer)",
            "path": scratch_cook_rel,
            "rows": scratch_cooks,
            "note": "cook4 ~321–330k; cook1→cook4 ~1.8× when disk is slack.",
        },
    ],
    "notes": [
        "“4 cores batched” = Store::set_cook_parallelism / put_many cook workers — NOT --writer-shards 4.",
        "writer-shards=4 (Axis B multicore survey) is a different experiment and is SLOWER than 1 shard on one store (~9k vs ~10k).",
        "Parallel cook fails if the segment seals mid-batch install (see tmp peer-b-cook4.log).",
    ],
}

# Axis B/C capacity experiments (secondary — NOT the 100k story)
def _load_pump_cell(rel, label, category, axis):
    p = root / rel
    if not p.is_file():
        return None
    try:
        d = json.loads(p.read_text())
    except Exception:
        return None
    if not isinstance(d, dict) or "ops_per_sec" not in d:
        return None
    elapsed_ms = d.get("elapsed_ms")
    elapsed_s = round(float(elapsed_ms) / 1000.0, 3) if elapsed_ms is not None else None
    return {
        "id": Path(rel).stem,
        "label": label,
        "category": category,
        "axis": axis,
        "path": rel,
        "engine": "residiuum",
        "ops_per_sec": round(float(d["ops_per_sec"]), 1),
        "pump_mib_s": round(float(d.get("mb_per_sec") or 0), 2),
        "writer_shards": d.get("writer_shards"),
        "put_batch_size": d.get("put_batch_size"),
        "store_count": d.get("store_count") or 1,
        "concurrency": d.get("concurrency"),
        "writer_model": d.get("writer_model"),
        "pump_mode": d.get("pump_mode"),
        "peak_rss_bytes": d.get("peak_rss_bytes") or d.get("sum_peak_rss_bytes"),
        "peak_rss_mib": _rss_mib(d.get("peak_rss_bytes") or d.get("sum_peak_rss_bytes")),
        "peak_cpu_pct": d.get("peak_cpu_pct") or d.get("sum_peak_cpu_pct_children"),
        "elapsed_s": elapsed_s,
        "target_bytes": d.get("target_bytes") or d.get("per_store_target"),
        "payload_size": d.get("payload_size") or 8192,
        "ok": d.get("ok"),
    }

mc_dir = "doc/wip/status/surveys/scratch-multicore-4-20260801"
multicore_specs = [
    (f"{mc_dir}/b1.json", "1 shard · batch=1",
     "writer-shards=1 · batch=1 (NOT cook workers)", "B"),
    (f"{mc_dir}/b1b.json", "1 shard · batch=128",
     "writer-shards=1 · batch=128 (NOT cook workers)", "B"),
    (f"{mc_dir}/b4.json", "4 shards · batch=1",
     "writer-shards=4 · batch=1 (slower; not the 100k path)", "B"),
    (f"{mc_dir}/b4b.json", "4 shards · batch=128",
     "writer-shards=4 · batch=128 (slower; not the 100k path)", "B"),
    (f"{mc_dir}/c4fair.json", "4 stores · batch=1 aggregate",
     "4 independent processes aggregate (~14.6k)", "C"),
]
multicore_cells = []
for rel, lab, cat, axis in multicore_specs:
    cell = _load_pump_cell(rel, lab, cat, axis)
    if cell:
        multicore_cells.append(cell)
        sources.append(rel)

write_multicore = {
    "title": "Secondary: writer-shards / multi-process capacity (NOT the ~100k cook4 path)",
    "campaign": "scratch-multicore-4-20260801",
    "contract": "doc/wip/status/surveys/scratch-multicore-4-20260801/README.md",
    "cells": multicore_cells,
    "notes": [
        "These are ~8–15k multi-seal/on-disk pump rates — same order as PEER, not the ~100k cook band.",
        "Do not call writer-shards=4 “100k on 4 cores.” That name belongs to cook_parallelism=4 phase-bench.",
    ],
}

# Headline category rows: three bands + peer Mode B + explicit cook4 rows
categories = []
for b in three_bands:
    categories.append({
        "category": b["name"],
        "axis": f"band-{b['band']}",
        "label": b.get("what", "")[:120],
        "ops_per_sec": b.get("ops_per_sec"),
        "rate_mib_s": b.get("logical_mib_s"),
        "rate_kind": "logical MiB/s",
        "cpu_or_shards": b.get("cores"),
        "batch": b.get("batch"),
        "peak_rss_mib": None,
        "path": b.get("path"),
        "comparable_to_sqlite_peer": b["band"] == 1,
        "highlight": b.get("highlight", False),
        "bed": b.get("bed"),
    })
if peer_rb:
    categories.append({
        "category": "Residiuum peer Mode B (1 thread · batch=128)",
        "axis": "peer-sql",
        "label": peer_rb.get("label"),
        "ops_per_sec": peer_rb.get("ops_per_sec"),
        "rate_mib_s": peer_rb.get("logical_mib_s"),
        "rate_kind": "logical MiB/s",
        "cpu_or_shards": "1 thread",
        "batch": peer_rb.get("put_batch_size") or 128,
        "peak_rss_mib": peer_rb.get("peak_rss_mib"),
        "path": peer_rb.get("path"),
        "comparable_to_sqlite_peer": True,
        "highlight": False,
        "bed": "Scratch PEER-SQL",
    })

inventory = {
    "title": "What this briefing actually pulls in (provenance)",
    "items": [
        {
            "group": "PEER-SQL post write-through (vs SQLite)",
            "paths": [c["path"] for c in peer_cells if c.get("path")],
            "why": "Fair same-bed adoption floor (~10k ops/s). Single-threaded peer-pump.",
        },
        {
            "group": "Parallel cook · /tmp 1 GiB (≈100k on 4 cook workers)",
            "paths": [tmp_cook_rel] if tmp_cooks else [],
            "why": "Campaign H real-disk band: cook1/2/4 put_many. cook4 ≈ 116k ops/s.",
        },
        {
            "group": "Parallel cook · Scratch micro (≈330k)",
            "paths": [scratch_cook_rel] if scratch_cooks else [],
            "why": "Short micro only; shows cook scales when disk is slack — not multi-seal capacity.",
        },
        {
            "group": "Mode-A phase-bench after write-through",
            "paths": [micro_rel] if write_micro else [],
            "why": "Buffered put micro phases (encode/append/write split).",
        },
        {
            "group": "writer-shards / multi-process (secondary)",
            "paths": [c["path"] for c in multicore_cells],
            "why": "Capacity experiments at ~8–15k. Not the 100k cook path.",
        },
        {
            "group": "Reads (get path)",
            "paths": [],
            "why": "Nothing published under surveys yet — intentionally blank.",
        },
    ],
}

knobs = {
    "payload": "8192",
    "seed": "20260801",
    "residiuum_durability": "buffered",
    "sqlite_journal": "WAL",
    "sqlite_synchronous": "NORMAL",
    "logical_target": "256 MiB peer · 1 GiB /tmp phase · short Scratch micro",
    "threads": "peer=1 · cook_workers=1/2/4 · writer-shards≠cook",
    "volume_hint": "Scratch + APFS /tmp (tmp-real-disk campaign)",
}
if peer_cells and peer_cells[0].get("payload_size"):
    knobs["payload"] = str(peer_cells[0]["payload_size"])

metrics = {
    "schema": "residiuum-briefing-metrics-v1",
    "disclosure": (
        "Diagnostic only — not a published SLO. Not a scoreboard accept, not PQH product "
        "qualification, and not absolute marketing MiB/s. See BENCHMARK_DISCLOSURE.md. "
        "Three-band rule: ~10k peer multi-seal · ~100–160k /tmp 1 GiB cook · ~330k Scratch micro — do not mix."
    ),
    "narrative": "TEST_RESULTS.md",
    "contract": "doc/wip/status/surveys/README-PEER-SQL.md",
    "parked_wall": "doc/wip/status/surveys/PARKED-write-path-wall-20260801.md",
    "disclosure_doc": "doc/reference/operations/BENCHMARK_DISCLOSURE.md",
    "knobs": knobs,
    "inventory": inventory,
    "three_bands": {
        "title": "Three-band write rates (do not mix beds)",
        "rows": three_bands,
        "notes": [
            "The ~100k on 4 cores number is band 2: /tmp put_many cook_parallelism=4 ≈ 116k ops/s (batch=128, ~1 GiB phase).",
            "writer-shards=4 is a different lever and sits in the ~10k band — not band 2.",
        ],
    },
    "write_categories": {
        "title": "Headline bands + peer Mode B",
        "rows": categories,
        "notes": [
            "Highlight band 2 = real ~100k-class 4-worker batched writes on /tmp.",
            "Only PEER-SQL rows may be compared to SQLite peer cells.",
        ],
    },
    "write_peer": {
        "title": "PEER-SQL peer-pump: Residiuum vs SQLite (same bed) — band ~10k",
        "campaign": campaign,
        "cells": peer_cells,
        "ratios": ratios,
        "notes": [
            "Mode A is the fair general-load peer (per-ack Buffered vs SQLite autocommit).",
            "Mode B is not equal semantics: SQLite BEGIN…COMMIT amortizes durability.",
            "Single-threaded peer-pump — no cook_parallelism here.",
        ],
        "extra_mode_a_long": {
            "ops_per_sec": extra.get("ops_per_sec") if extra else None,
            "logical_mib_s": extra.get("logical_mib_s") if extra else None,
            "peak_rss_mib": extra.get("peak_rss_mib") if extra else None,
            "path": extra.get("path") if extra else None,
        } if extra else None,
    },
    "write_cook": cook_detail,
    "write_multicore": write_multicore,
    "write_micro": write_micro or {},
    "read": {
        # Hot-path get stats: published only in BENCHMARK_DISCLOSURE (testrig monitor
        # open-once PrimaryIndex samples). Aug 2026 Scratch peer/pump surveys are write-only.
        "status": "published_disclosure",
        "title": "Read / get path (hot PrimaryIndex, open-once)",
        "source": "doc/reference/operations/BENCHMARK_DISCLOSURE.md",
        "how_measured": (
            "residiuum-testrig monitor after pump: already-open store, PrimaryIndex "
            "point gets (sample-keys), p50/p95/p99 in µs. Not a full read-throughput "
            "SLO; not Chimera/Hydra hot path (sidecars exist at seal but are not on Store::get)."
        ),
        "tooling": [
            "testrig monitor (gets ok/fail + p50/p95/p99 µs)",
            "residiuum-store example read_latency_breakdown (phase attribution)",
        ],
        "note": (
            "No Aug-2026 Scratch survey JSON contains get stats (peer-pump/phase-bench are write). "
            "Numbers below are the disclosed diagnostic snapshots from BENCHMARK_DISCLOSURE "
            "(2026-07 testrig campaigns). Diagnostic only — not a published SLO."
        ),
        "cells": [
            {
                "label": "10 GiB single-shard · baseline gets",
                "campaign": "10g testrig (DEF-095 locator-first PrimaryIndex)",
                "phase": "baseline",
                "sample_keys": 128,
                "payload_size": 8192,
                "ok": "128/128",
                "p50_us": 18,
                "p95_us": 139,
                "p99_us": 284,
                "path_class": "hot PrimaryIndex",
                "notes": "buffered pump to ~10 GiB; monitor open-once gets",
            },
            {
                "label": "10 GiB single-shard · post-chaos gets",
                "campaign": "10g testrig + 64 offline punches",
                "phase": "post-chaos",
                "sample_keys": 128,
                "payload_size": 8192,
                "ok": "128/128",
                "p50_us": 19,
                "p95_us": 152,
                "p99_us": 279,
                "path_class": "hot PrimaryIndex after damage/salvage",
                "notes": "sampled live keys still complete",
            },
            {
                "label": "10 GiB writer-shards=4 · baseline gets",
                "campaign": "10g Axis B testrig",
                "phase": "baseline",
                "sample_keys": 128,
                "payload_size": 8192,
                "ok": "128/128",
                "p50_us": 24,
                "p95_us": 426,
                "p99_us": 563,
                "path_class": "hot PrimaryIndex",
                "notes": "higher tail than single-shard on same machine class",
            },
            {
                "label": "10 GiB writer-shards=4 · post-chaos gets",
                "campaign": "10g Axis B + chaos",
                "phase": "post-chaos",
                "sample_keys": 128,
                "payload_size": 8192,
                "ok": "128/128",
                "p50_us": 40,
                "p95_us": 495,
                "p99_us": 808,
                "path_class": "hot PrimaryIndex after damage",
                "notes": None,
            },
            {
                "label": "10 GiB × 4 stores (Axis C) · baseline gets",
                "campaign": "multi-store 10g",
                "phase": "baseline",
                "sample_keys": "128/store",
                "payload_size": 8192,
                "ok": "all 4 roots",
                "p50_us": "19–23",
                "p95_us": None,
                "p99_us": "33–59",
                "path_class": "hot PrimaryIndex per store",
                "notes": "p50/p99 ranges across four roots",
            },
            {
                "label": "10 GiB × 4 stores · post-chaos gets",
                "campaign": "multi-store 10g + chaos",
                "phase": "post-chaos",
                "sample_keys": "128/store",
                "payload_size": 8192,
                "ok": "3/4 full; 1 store 127/128 missing (expected)",
                "p50_us": "18–20",
                "p95_us": None,
                "p99_us": None,
                "path_class": "hot PrimaryIndex after damage",
                "notes": "integrity path still speaks",
            },
        ],
        "not_measured": [
            "Sustained get ops/s / MiB/s campaign (no peer-get or read-pump survey under doc/wip/status/surveys).",
            "Cold NVMe full-payload get_payload rates at scale.",
            "Hydra-wired hot get (sidecars at seal; not on Store::get yet).",
            "Chimera full .cmr load is diagnostic only — not the product hot path.",
        ],
    },
    "sources": sources + [
        "doc/reference/operations/BENCHMARK_DISCLOSURE.md",
    ],
}

# Mark inventory read group as disclosure-backed
for it in inventory["items"]:
    if it.get("group", "").startswith("Reads"):
        it["paths"] = ["doc/reference/operations/BENCHMARK_DISCLOSURE.md"]
        it["why"] = (
            "Disclosed testrig monitor get p50/p95/p99 (µs, open-once PrimaryIndex). "
            "No Aug Scratch survey JSON for gets; peer-pump is write-only."
        )

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
    "metrics": metrics,
    "pointers": {
        "quality": "./scripts/quality.sh",
        "nightly": "./scripts/nightly.sh",
        "csq_a2": "bash scripts/residiuum-verify-core-storage.sh --require-a2-pass",
        "fas_howto": "formal/HOW_TO_USE.md",
        "scoreboard": "doc/wip/status/NEXT_BUILD_STATUS.md",
        "master_plan": "MASTER_DELIVERY_PLAN.md",
        "test_results": "TEST_RESULTS.md",
        "benchmark_disclosure": "doc/reference/operations/BENCHMARK_DISCLOSURE.md",
        "peer_sql": "doc/wip/status/surveys/README-PEER-SQL.md",
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
allow_fail = os.environ.get("ALLOW_FAIL", "0") == "1"
if overall == "fail" and not allow_fail:
    raise SystemExit(1)
raise SystemExit(0)
PY