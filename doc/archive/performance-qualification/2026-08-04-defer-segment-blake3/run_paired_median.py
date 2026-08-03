#!/usr/bin/env python3
"""Paired median Seal Fast Lane gate — measurement only.

Alternates high-threshold control (512MiB) and 64MiB stream-hash, enrichment off.
Pass: median(TPS_64)/median(TPS_control) >= 0.90, multi-rotate + exact reopen on 64MiB.
"""

from __future__ import annotations

import json
import os
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
BIN = ROOT / "target/release/residiuum-testrig"
EV = Path(__file__).resolve().parent
RUNS = EV / "runs"
WORK_ROOT = Path("/tmp/residiuum-paired-median")
REPS = 6
RUN_TIMEOUT_S = 120


def run_one(kind: str, seal: str, idx: int) -> dict:
    work = WORK_ROOT / f"{kind}-{idx}"
    out = RUNS / f"{kind}-r{idx}.json"
    err = RUNS / f"{kind}-r{idx}.err"
    if work.exists():
        shutil.rmtree(work)
    cmd = [
        str(BIN),
        "ack-finalize",
        "-w",
        str(work),
        "--cell",
        "real-full",
        "--target-bytes",
        "256M",
        "--payload-size",
        "8192",
        "--concurrency",
        "8",
        "--seed",
        "42",
        "--seal-threshold",
        seal,
        "--min-free",
        "512M",
        "--no-enrichment",
        "--json-out",
    ]
    print(f"=== {kind} r{idx} seal={seal} ===", flush=True)
    t0 = time.time()
    try:
        p = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=RUN_TIMEOUT_S,
            check=False,
        )
        rc = p.returncode
        out.write_text(p.stdout)
        err.write_text(p.stderr)
    except subprocess.TimeoutExpired as e:
        rc = 124
        out.write_text(e.stdout or "")
        err.write_text((e.stderr or "") + f"\nTIMEOUT_{RUN_TIMEOUT_S}s\n")
        if work.exists():
            shutil.rmtree(work, ignore_errors=True)
        raise SystemExit(f"timeout on {kind} r{idx}") from e
    finally:
        if work.exists():
            shutil.rmtree(work, ignore_errors=True)
    wall = time.time() - t0
    if rc != 0 or not out.stat().st_size:
        raise SystemExit(f"fail {kind} r{idx} rc={rc} stderr={err.read_text()[-500:]}")
    d = json.loads(out.read_text())
    print(
        f"  exit={rc} wall={wall:.1f}s ack={d['acknowledged_write_ops_per_sec']:.0f} "
        f"sealed={d['sealed_segments_at_last_ack']} exact={d['reopen_exact']} "
        f"backlog={d['enrichment_backlog_at_last_ack']}",
        flush=True,
    )
    return d


def main() -> int:
    if not BIN.is_file():
        print(f"missing binary: {BIN}", file=sys.stderr)
        return 2
    RUNS.mkdir(parents=True, exist_ok=True)
    if WORK_ROOT.exists():
        shutil.rmtree(WORK_ROOT)
    WORK_ROOT.mkdir(parents=True)

    (EV / "uname.txt").write_text(subprocess.check_output(["uname", "-a"], text=True))
    sha = subprocess.check_output(["shasum", "-a", "256", str(BIN)], text=True)
    (EV / "binary.sha256").write_text(sha)
    (EV / "started-at.txt").write_text(
        subprocess.check_output(["date", "-u"], text=True)
    )
    df = subprocess.check_output(["df", "-h", "/System/Volumes/Data"], text=True)
    (EV / "df-before.txt").write_text(df)
    print(sha.strip(), flush=True)
    print(df, flush=True)

    for i in range(1, REPS + 1):
        run_one("control", "512M", i)
        run_one("stream64", "64M", i)

    (EV / "finished-at.txt").write_text(
        subprocess.check_output(["date", "-u"], text=True)
    )
    (EV / "df-after.txt").write_text(
        subprocess.check_output(["df", "-h", "/System/Volumes/Data"], text=True)
    )

    ctrl: list[float] = []
    s64: list[float] = []
    rows: list[dict] = []
    for kind, bucket in [("control", ctrl), ("stream64", s64)]:
        for i in range(1, REPS + 1):
            d = json.loads((RUNS / f"{kind}-r{i}.json").read_text())
            tps = float(d["acknowledged_write_ops_per_sec"])
            bucket.append(tps)
            rows.append(
                {
                    "kind": kind,
                    "rep": i,
                    "ack_tps": tps,
                    "sealed_segments_at_last_ack": d["sealed_segments_at_last_ack"],
                    "reopen_exact": d["reopen_exact"],
                    "enrichment_backlog_at_last_ack": d["enrichment_backlog_at_last_ack"],
                    "campaign_ops_per_sec": d["campaign_ops_per_sec"],
                    "seal_threshold": d["seal_threshold"],
                }
            )

    def stats(xs: list[float]) -> dict:
        xs_sorted = sorted(xs)
        return {
            "n": len(xs_sorted),
            "values": xs_sorted,
            "min": min(xs_sorted),
            "max": max(xs_sorted),
            "median": statistics.median(xs_sorted),
            "mean": statistics.fmean(xs_sorted),
        }

    c = stats(ctrl)
    s = stats(s64)
    ratio = s["median"] / c["median"] if c["median"] else float("nan")
    rotate_ok = all(
        r["sealed_segments_at_last_ack"] >= 2 for r in rows if r["kind"] == "stream64"
    )
    exact_ok = all(r["reopen_exact"] for r in rows if r["kind"] == "stream64")
    exact_ctrl = all(r["reopen_exact"] for r in rows if r["kind"] == "control")
    pass_ratio = ratio >= 0.90
    summary = {
        "kind": "paired_median_gate",
        "disclosure": (
            "Diagnostic only — paired median Seal Fast Lane gate. "
            "Enrichment disabled. Not a published SLO."
        ),
        "recipe": {
            "cell": "real-full",
            "logical_data_bytes": 256 * 1024 * 1024,
            "payload_size": 8192,
            "concurrency": 8,
            "seed": 42,
            "enrichment_enabled": False,
            "stream64_seal_threshold": 64 * 1024 * 1024,
            "control_seal_threshold": 512 * 1024 * 1024,
            "reps_each": REPS,
            "alternate_order": "control then stream64",
            "binary": str(BIN),
            "binary_sha256": sha.split()[0],
        },
        "control": c,
        "stream64": s,
        "ratio_median_stream64_over_control": ratio,
        "success_floor_ratio": 0.90,
        "gates": {
            "median_ratio_ge_0_90": pass_ratio,
            "stream64_multi_rotate_all_reps": rotate_ok,
            "stream64_reopen_exact_all_reps": exact_ok,
            "control_reopen_exact_all_reps": exact_ctrl,
        },
        "pass": pass_ratio and rotate_ok and exact_ok and exact_ctrl,
        "note": (
            "Frozen 74.7K floor was 0.90*83K (stale). "
            "This package compares medians of contemporary paired cells."
        ),
        "runs": rows,
    }
    (EV / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(
        json.dumps(
            {
                "control_median": round(c["median"]),
                "stream64_median": round(s["median"]),
                "ratio": round(ratio, 4),
                "pass": summary["pass"],
                "gates": summary["gates"],
                "control_values": [round(x) for x in c["values"]],
                "stream64_values": [round(x) for x in s["values"]],
            },
            indent=2,
        ),
        flush=True,
    )
    return 0 if summary["pass"] else 1


if __name__ == "__main__":
    # Avoid buffering surprises under pipe capture.
    sys.stdout.reconfigure(line_buffering=True)
    raise SystemExit(main())
