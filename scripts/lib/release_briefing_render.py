#!/usr/bin/env python3
"""Render release-briefing JSON → self-contained HTML.

Input: path to briefing JSON (schema residiuum-release-briefing-v1).
Output: HTML path (stdout or --html).
"""
from __future__ import annotations

import argparse
import html
import json
import sys
from pathlib import Path


def esc(s: object) -> str:
    return html.escape("" if s is None else str(s), quote=True)


def status_class(st: str) -> str:
    st = (st or "unknown").lower()
    if st in ("pass", "ok", "green"):
        return "pass"
    if st in ("fail", "error", "red"):
        return "fail"
    if st in ("not_run", "skipped", "skip"):
        return "skip"
    if st in ("warn", "warning", "partial"):
        return "warn"
    return "unknown"


def render(data: dict) -> str:
    steps = data.get("steps") or []
    artifacts = data.get("artifacts") or []
    meta = data.get("meta") or {}
    overall = data.get("overall_status") or "unknown"
    counts = {"pass": 0, "fail": 0, "skip": 0, "warn": 0, "unknown": 0}
    for s in steps:
        c = status_class(s.get("status", ""))
        counts[c if c in counts else "unknown"] += 1

    rows = []
    for s in steps:
        st = s.get("status") or "unknown"
        sc = status_class(st)
        dur = s.get("duration_ms")
        dur_s = f"{dur / 1000:.1f}s" if isinstance(dur, (int, float)) else "—"
        cmd = s.get("command") or s.get("path") or ""
        detail = s.get("detail") or s.get("message") or ""
        if len(detail) > 400:
            detail = detail[:400] + "…"
        rows.append(
            f"<tr class='{sc}'>"
            f"<td><code>{esc(s.get('id'))}</code></td>"
            f"<td>{esc(s.get('title'))}</td>"
            f"<td><span class='badge {sc}'>{esc(st)}</span></td>"
            f"<td>{esc(dur_s)}</td>"
            f"<td><code class='cmd'>{esc(cmd)}</code></td>"
            f"<td class='detail'>{esc(detail)}</td>"
            f"</tr>"
        )

    art_rows = []
    for a in artifacts:
        art_rows.append(
            f"<tr>"
            f"<td><code>{esc(a.get('id'))}</code></td>"
            f"<td>{esc(a.get('path'))}</td>"
            f"<td><span class='badge {status_class(a.get('status',''))}'>"
            f"{esc(a.get('status'))}</span></td>"
            f"<td>{esc(a.get('summary') or '')}</td>"
            f"</tr>"
        )

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Residiuum release briefing — {esc(meta.get('profile'))}</title>
<style>
  :root {{
    --bg: #0f1419; --card: #1a2332; --text: #e7ecf3; --muted: #8b9bb4;
    --pass: #3dd68c; --fail: #f07178; --skip: #7a8494; --warn: #ffcc66;
    --border: #2a3548; --mono: ui-monospace, SFMono-Regular, Menlo, monospace;
    --sans: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0; padding: 2rem 1.25rem 4rem;
    font-family: var(--sans); background: var(--bg); color: var(--text);
    line-height: 1.45;
  }}
  .wrap {{ max-width: 1100px; margin: 0 auto; }}
  h1 {{ font-size: 1.5rem; margin: 0 0 0.35rem; font-weight: 650; }}
  h2 {{ font-size: 1.1rem; margin: 2rem 0 0.75rem; color: var(--muted); font-weight: 600; }}
  .sub {{ color: var(--muted); font-size: 0.95rem; margin-bottom: 1.25rem; }}
  .cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 0.75rem; }}
  .card {{
    background: var(--card); border: 1px solid var(--border); border-radius: 10px;
    padding: 0.9rem 1rem;
  }}
  .card .n {{ font-size: 1.6rem; font-weight: 700; font-family: var(--mono); }}
  .card .l {{ color: var(--muted); font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.04em; }}
  .card.pass .n {{ color: var(--pass); }}
  .card.fail .n {{ color: var(--fail); }}
  .card.skip .n {{ color: var(--skip); }}
  .card.warn .n {{ color: var(--warn); }}
  .badge {{
    display: inline-block; padding: 0.15rem 0.5rem; border-radius: 999px;
    font-size: 0.75rem; font-weight: 650; text-transform: uppercase; letter-spacing: 0.03em;
  }}
  .badge.pass {{ background: #143d2a; color: var(--pass); }}
  .badge.fail {{ background: #4a1c22; color: var(--fail); }}
  .badge.skip {{ background: #2a2f38; color: var(--skip); }}
  .badge.warn {{ background: #4a3a14; color: var(--warn); }}
  .badge.unknown {{ background: #2a2f38; color: var(--muted); }}
  table {{
    width: 100%; border-collapse: collapse; background: var(--card);
    border: 1px solid var(--border); border-radius: 10px; overflow: hidden;
    font-size: 0.88rem;
  }}
  th, td {{ text-align: left; padding: 0.55rem 0.65rem; vertical-align: top; border-bottom: 1px solid var(--border); }}
  th {{ color: var(--muted); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.04em; }}
  tr:last-child td {{ border-bottom: 0; }}
  code {{ font-family: var(--mono); font-size: 0.82em; }}
  code.cmd {{ color: var(--muted); word-break: break-all; }}
  td.detail {{ color: var(--muted); max-width: 280px; }}
  .note {{
    background: var(--card); border-left: 3px solid var(--warn);
    padding: 0.75rem 1rem; margin: 1rem 0; border-radius: 0 8px 8px 0;
    color: var(--muted); font-size: 0.92rem;
  }}
  .overall {{ margin: 1rem 0 1.5rem; }}
  footer {{ margin-top: 2.5rem; color: var(--skip); font-size: 0.8rem; }}
</style>
</head>
<body>
<div class="wrap">
  <h1>Residiuum release briefing</h1>
  <p class="sub">
    Profile <strong>{esc(meta.get('profile'))}</strong>
    · generated <code>{esc(meta.get('generated_at'))}</code>
    · host <code>{esc(meta.get('host'))}</code>
    · git <code>{esc(meta.get('git_head'))}</code>
  </p>
  <div class="overall">
    Overall: <span class="badge {status_class(overall)}">{esc(overall)}</span>
    · exit policy: fail if any gate step failed; <code>not_run</code> is not pass
  </div>
  <div class="cards">
    <div class="card pass"><div class="n">{counts['pass']}</div><div class="l">Pass</div></div>
    <div class="card fail"><div class="n">{counts['fail']}</div><div class="l">Fail</div></div>
    <div class="card skip"><div class="n">{counts['skip']}</div><div class="l">Not run / skip</div></div>
    <div class="card warn"><div class="n">{counts['warn']}</div><div class="l">Warn</div></div>
  </div>
  <div class="note">
    <strong>Honesty:</strong> this briefing aggregates package gates and existing
    evidence. It is <em>not</em> a claim that Residiuum is “formally verified” or
    release-qualified unless every required package on the scoreboard is
    <code>accept</code> with matching evidence. PQH qualification and full
    <code>quality.sh</code> may be heavier than this profile.
  </div>
  <h2>Steps</h2>
  <table>
    <thead><tr>
      <th>Id</th><th>Title</th><th>Status</th><th>Time</th><th>Command / path</th><th>Detail</th>
    </tr></thead>
    <tbody>
      {''.join(rows) if rows else '<tr><td colspan="6">No steps</td></tr>'}
    </tbody>
  </table>
  <h2>Ingested artifacts</h2>
  <table>
    <thead><tr><th>Id</th><th>Path</th><th>Status</th><th>Summary</th></tr></thead>
    <tbody>
      {''.join(art_rows) if art_rows else '<tr><td colspan="4">No artifacts</td></tr>'}
    </tbody>
  </table>
  <h2>How to re-run</h2>
  <p class="sub">
    <code>bash scripts/release-briefing.sh --profile {esc(meta.get('profile') or 'snapshot')}</code><br/>
    Full PR-local quality: <code>./scripts/quality.sh</code> ·
    CSQ A2: <code>bash scripts/residiuum-verify-core-storage.sh --require-a2-pass</code> ·
    FAS: see <code>formal/HOW_TO_USE.md</code>
  </p>
  <footer>
    schema {esc(data.get('schema'))} · Residiuum release briefing · do not treat missing evidence as pass
  </footer>
</div>
</body>
</html>
"""


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("json_path", type=Path)
    ap.add_argument("--html", type=Path, required=True)
    args = ap.parse_args()
    data = json.loads(args.json_path.read_text())
    args.html.parent.mkdir(parents=True, exist_ok=True)
    args.html.write_text(render(data), encoding="utf-8")
    print(args.html)
    return 0


if __name__ == "__main__":
    sys.exit(main())
