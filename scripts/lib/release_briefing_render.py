#!/usr/bin/env python3
"""Render release-briefing JSON → self-contained HTML.

Each data row is followed by a full-width explanation row. Copy is written for
humans who are not formal-methods specialists: plain language, how it maps to
store data, and honest limits of each gate.
"""
from __future__ import annotations

import argparse
import html
import json
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Plain-language CON theorems (FAS-4) — connect math claims to store behavior
# ---------------------------------------------------------------------------

CON_THEOREMS_PLAIN: list[dict[str, str]] = [
    {
        "id": "FAS-CON-NO-FABRICATED-VALUE-001",
        "plain": "No invented values",
        "story": (
            "If the system says “here is the complete value of key K,” that answer "
            "must come from a committed, authoritative version with real evidence — "
            "not from a guess, a cache alone, or half-decoded bytes."
        ),
        "on_data": (
            "When you Store::get / get_payload and the API returns a full payload, "
            "that maps to an abstract complete observation with provenance "
            "(committed generation + value + events). A damaged segment must not "
            "be silently turned into a successful read."
        ),
        "math_hook": "Lean: hasAuthoritativeProvenance; Observation.complete only with committed evidence.",
    },
    {
        "id": "FAS-CON-GENERATION-EXACT-001",
        "plain": "Right version only",
        "story": (
            "Each key has a version number (generation). Reconstruction and reads "
            "must use the generation the store actually declared current — not mix "
            "v3’s metadata with v2’s bytes."
        ),
        "on_data": (
            "Historical get / get_payload_version must bind to one generation. "
            "You should never see a hybrid of two versions for one logical item."
        ),
        "math_hook": "Lean: generationExact — at most one current generation per item.",
    },
    {
        "id": "FAS-CON-PUBLICATION-NONHYBRID-001",
        "plain": "Publish is all-or-nothing (old / new / unknown)",
        "story": (
            "After a crash mid-publish, recovery may keep the old state, the new "
            "state, or admit “we don’t know yet” — never a Frankenstein where half "
            "the keys look new and half look old for the same atomic publish."
        ),
        "on_data": (
            "open/create/reopen after power loss: the publication view is old, new, "
            "or unknown — not a mix of both as if commit half-succeeded."
        ),
        "math_hook": "Lean: publicationView ∈ {old, new, unknown}; publication_not_hybrid.",
    },
    {
        "id": "FAS-CON-DURABLE-ACK-001",
        "plain": "Ack means it was made durable (under FS assumptions)",
        "story": (
            "If the API acknowledged a durable write, the model treats that event "
            "as durable and covered. We still assume the filesystem keeps its "
            "sync/persistence contract (named assumption)."
        ),
        "on_data": (
            "After put / put_many with durable mode, a crash + reopen should still "
            "see that write (given the filesystem assumption ledger)."
        ),
        "math_hook": "Lean: durable_ack_put_event; assumption FAS-ASM-FILESYSTEM-DURABILITY-001.",
    },
    {
        "id": "FAS-CON-RECOVERY-IDEMPOTENT-001",
        "plain": "Recover once is enough",
        "story": (
            "Running recovery twice should not keep changing the store. After one "
            "successful recovery pass, a second pass is a no-op (fixed point)."
        ),
        "on_data": (
            "open after crash, then open again: second open should not rewrite "
            "layout into a third different state."
        ),
        "math_hook": "Lean: recovery_idempotent / recovery_fixed_point.",
    },
    {
        "id": "FAS-CON-DERIVED-NONAUTHORITY-001",
        "plain": "Indexes are not the source of truth",
        "story": (
            "Secondary indexes, caches, and prepared-but-not-committed paths can "
            "hint at data, but they must not alone justify a “complete” answer. "
            "Authority is the durable published record."
        ),
        "on_data": (
            "A radix/index hit is not enough if publication is only prepared. "
            "Observe must stay partial/unknown until committed evidence exists."
        ),
        "math_hook": "Lean: prepared publication → partialObs, never complete.",
    },
    {
        "id": "FAS-CON-DAMAGE-HONESTY-001",
        "plain": "Damage is not absence (and not completeness)",
        "story": (
            "If bytes are torn or unscannable, the honest answers are damaged / "
            "unknown / partial — not “key missing” and not “here is a full value.” "
            "That protects survivors and avoids silent data invention."
        ),
        "on_data": (
            "Punch a hole in a segment: salvage may still find other keys "
            "(healthy islands). The damaged region must not become a clean delete "
            "or a fabricated payload."
        ),
        "math_hook": "Lean: forbidden collapse damaged↛complete/absent; toPublicError stays error.",
    },
    {
        "id": "FAS-CON-HEALTHY-ISLAND-001",
        "plain": "Healthy neighbors stay valid",
        "story": (
            "Corruption in one region must not force the whole store to zero. "
            "Unaffected valid frames remain discoverable within scanner limits."
        ),
        "on_data": (
            "After localized damage, other keys that never shared that damage "
            "should still open when salvage/scan reaches them."
        ),
        "math_hook": "Lean: healthy_island_*; Init WF; create_heap doesn’t invent foreign values.",
    },
]

# ---------------------------------------------------------------------------
# Step explanations
# ---------------------------------------------------------------------------

STEP_EXPLAIN: dict[str, dict[str, str]] = {
    "gates": {
        "what": "Collect-only mode: no verification commands were re-executed.",
        "means": "The HTML only shows whatever reports already existed under target/.",
        "pass": "N/A.",
        "fail": "N/A.",
        "not_run": "Expected for --collect-only.",
        "data": "No store data was touched.",
    },
    "delivery-status": {
        "what": "Program scoreboard integrity check (package list & states).",
        "means": (
            "Validates doc/wip/status/NEXT_BUILD_STATUS.md: every package id, "
            "allowed states (accept/active/…), dependency arrows, and evidence "
            "columns. This is the project’s map of “what is done,” not a unit test."
        ),
        "pass": "The scoreboard file is structurally consistent.",
        "fail": "Broken scoreboard — do not trust package-accept claims until fixed.",
        "not_run": "Skipped for this profile.",
        "data": "Does not read your databases; docs/process only.",
    },
    "identity": {
        "what": "Product/protocol name linter (rebrand hard reset).",
        "means": (
            "Ensures live code and specs only use Residiuum identities — no "
            "obsolete pre-reset product names as accepted profiles, magics, or APIs. "
            "Historical rebrand docs and intentional reject-fixtures are allowlisted."
        ),
        "pass": "No forbidden identity tokens in live surfaces.",
        "fail": "A pre-reset name leaked into live product text — see listed paths.",
        "not_run": "Skipped for this profile.",
        "data": "Guards wire/profile names so old-format bytes are not treated as valid Residiuum.",
    },
    "fas0": {
        "what": "FAS-0 — the formal claim catalogue (registry).",
        "means": (
            "Builds the dictionary of theorem IDs, assumptions, operations, and "
            "profiles under formal/registry/. Without this, “we proved X” has no "
            "stable name. It does not prove the store; it names what may later be proved."
        ),
        "pass": "Catalogue closed and structurally valid (FAS0_CLOSED).",
        "fail": "Registry incomplete or linter failed.",
        "not_run": "Unusual for snapshot — FAS-0 is cheap and usually runs.",
        "data": "No user data; pure claim governance files.",
    },
    "fas1": {
        "what": "FAS-1 — same proof tools, pinned versions, tiny smokes.",
        "means": (
            "Can this machine re-run Lean / Verus / Kani / TLC at locked versions? "
            "If tools float to “latest,” last month’s “proof” is unreproducible."
        ),
        "pass": "Pins + smoke proofs green.",
        "fail": "Tool missing, unpinned, or smoke failed.",
        "not_run": "Default snapshot skips heavy installs. Use --profile formal.",
        "data": "No user data; toolchain only.",
    },
    "fas2": {
        "what": "FAS-2 — abstract math model of Residiuum (Lean).",
        "means": (
            "Defines in Lean what State, Observation (complete / absent / damaged / "
            "unknown…), and operations mean. Later theorems share this vocabulary so "
            "“absent” cannot silently mean “we failed to read.”"
        ),
        "pass": "Lean kernel builds; foundation gate pass.",
        "fail": "Lean/build/foundation check failed.",
        "not_run": "Skipped unless profile formal/pre-release.",
        "data": (
            "Abstract only. Real Store::get is not simulated here yet; this fixes "
            "the language for talking about get/put honesty."
        ),
    },
    "fas3": {
        "what": "FAS-3 — connect abstract math to real Rust paths.",
        "means": (
            "Maps census of production entrypoints and requires one end-to-end "
            "slice: abstract statement + Verus pure_kernel + heap decide/pure_proofs "
            "(authority binding). Prevents claiming “connected to production” while "
            "only demo code exists."
        ),
        "pass": "Bridge files + vertical slice check out.",
        "fail": "Missing/renamed entrypoint or failed bridge proof piece.",
        "not_run": "Skipped unless profile formal/pre-release.",
        "data": (
            "Today’s connected slice is heap authority binding (foreign heap id "
            "must not authenticate). Full Store put/get refinement is still residual."
        ),
    },
    "fas4": {
        "what": "FAS-4 — consistency laws (the CON theorem family).",
        "means": (
            "Eight named honesty laws about published data (see “Consistency laws "
            "in plain English” below). Implemented as Lean theorems plus links to "
            "CSQ physical evidence. Status is MVP: math + links, not “every CON "
            "claim is fully physically qualified forever.”"
        ),
        "pass": "Lean CON module + connection catalogues + negatives present and gate green.",
        "fail": "Missing theorem wiring, negatives, or Lean failure.",
        "not_run": "Skipped unless profile formal/pre-release.",
        "data": (
            "Laws describe how put/get/open/salvage should behave under damage and "
            "crash. Physical cells live in CSQ A2; Lean states the obligations."
        ),
    },
    "csq-a2": {
        "what": "CSQ A2 — physical store qualification campaign.",
        "means": (
            "Runs/aggregates the core-storage invariant × operation × failure "
            "evidence for profile residiuum-core-storage-v1. This is the “bytes on "
            "disk” bar that backs many CON stories with real tests."
        ),
        "pass": "a2_pass with no missing A2 cells (per evaluation JSON).",
        "fail": "Missing or failed cells, or verifier rejected the report.",
        "not_run": "Not auto-run in snapshot. Use --profile pre-release or the CSQ script.",
        "data": "Exercises real store create/put/get/crash/salvage paths in the evidence suite.",
    },
    "quality": {
        "what": "Full local CI quality bar (fmt, clippy, tests, CSQ pieces, …).",
        "means": "scripts/quality.sh — PR-local mirror of CI quality. Long; not nested in every briefing.",
        "pass": "quality.sh exited 0 (when you run it).",
        "fail": "A quality check failed.",
        "not_run": "Run ./scripts/quality.sh yourself before a hard release cut.",
        "data": "Runs workspace tests against temp stores, not your production data.",
    },
    "pqh-qual": {
        "what": "Performance qualification campaigns (PQH).",
        "means": (
            "Measures latency/throughput on declared layers with disclosure rules. "
            "Needs controlled hosts and time; principal accept is on the scoreboard."
        ),
        "pass": "Campaign met contract (when run).",
        "fail": "Invalid campaign or policy fail.",
        "not_run": "Expected here. Use residiuum-perf for real perf claims.",
        "data": "Synthetic/workload stores under harness control — not your customer DB.",
    },
}

ARTIFACT_EXPLAIN: dict[str, dict[str, str]] = {
    "fas0": {
        "what": "Saved result file for the FAS-0 registry gate.",
        "means": (
            "JSON written by check-formal-registry.sh. Fields like result=pass and "
            "closed=true mean the theorem/assumption catalogue is structurally accepted — "
            "still not a proof that the database works."
        ),
        "pass": "Registry gate reported pass/closed.",
        "fail": "Registry gate reported fail.",
        "not_run": "File missing — run check-formal-registry.sh.",
        "data": "No user data; formal/registry/* only.",
    },
    "fas1": {
        "what": "Saved result file for the FAS-1 toolchain gate.",
        "means": (
            "JSON from check-formal-toolchain.sh listing tool pins and which smoke "
            "proofs (Lean/Verus/Kani/TLC) succeeded. Reproducibility of the lab."
        ),
        "pass": "All accept-required smokes succeeded under pinned versions.",
        "fail": "A pin or smoke failed.",
        "not_run": "File missing — run check-formal-toolchain.sh.",
        "data": "No user data.",
    },
    "fas2": {
        "what": "Saved result file for the FAS-2 Lean foundation gate.",
        "means": (
            "JSON from check-formal-foundation.sh. Pass means the abstract State/"
            "Observation/operations model type-checks and listed foundation checks "
            "hold — the shared math dictionary for later theorems."
        ),
        "pass": "Lean foundation gate pass.",
        "fail": "Lean or foundation checklist failed.",
        "not_run": "File missing — run check-formal-foundation.sh.",
        "data": "Abstract model only; see CON table for how words map to get/put.",
    },
    "fas3": {
        "what": "Saved result file for the FAS-3 refinement gate.",
        "means": (
            "JSON from check-formal-refinement.sh. Pass means the entrypoint census, "
            "type map, and vertical slice (heap authority binding) still connect "
            "Lean/Verus/Rust without pointing at missing or demo-only code."
        ),
        "pass": "Bridge/slice checks green.",
        "fail": "Broken connection or proof piece.",
        "not_run": "File missing — run check-formal-refinement.sh.",
        "data": (
            "Connected example: certificate heap id must match resident heap "
            "(foreign heap cannot authenticate). Broader Store put/get bridge residual."
        ),
    },
    "fas4": {
        "what": "Saved result file for the FAS-4 consistency gate.",
        "means": (
            "JSON from check-formal-consistency.sh. Pass means the eight CON "
            "obligations are present as Lean theorems, each has a negative control "
            "and CSQ link entry, and the gate script accepted the MVP package. "
            "It is NOT a one-line “database consistency certified” sticker — "
            "scroll to “Consistency laws in plain English” for what each law says "
            "about your keys and segments."
        ),
        "pass": (
            "MVP gate green: abstract CON family + catalogues + links. Physical "
            "strength still rides on CSQ A2 and residual refinement work."
        ),
        "fail": "CON wiring incomplete or Lean/gate failed.",
        "not_run": "File missing — run check-formal-consistency.sh.",
        "data": (
            "Laws constrain how complete/absent/damaged answers relate to durable "
            "segments, generations, and salvage — see the CON table below."
        ),
    },
    "csq-a2": {
        "what": "A2 evaluation summary for core-storage qualification.",
        "means": (
            "Says whether the physical A2 cell matrix passed (a2_pass) and how many "
            "cells are missing. This is the main empirical store bar that CON stories lean on."
        ),
        "pass": "a2_pass true (no missing A2-required cells).",
        "fail": "a2_pass false or evaluation errors.",
        "not_run": "File missing — run residiuum-verify-core-storage.sh --require-a2-pass.",
        "data": "Built from CSQ campaign evidence (create/put/get/crash/salvage cells), not production customer data.",
    },
    "csq-core": {
        "what": "Full core-storage qualification report body.",
        "means": "Structured report the CSQ verifier checks (profile, level, cells, result).",
        "pass": "Report result=pass.",
        "fail": "Report result=fail or invalid.",
        "not_run": "File missing.",
        "data": "Evidence about temporary/qualified stores in the campaign harness.",
    },
    "csq-verify": {
        "what": "Independent verification envelope for the CSQ report.",
        "means": "Second-reader style wrapper so “we built a report” ≠ “it verifies.”",
        "pass": "Envelope indicates successful verification.",
        "fail": "Verification rejected the report.",
        "not_run": "File missing or incomplete.",
        "data": "Same campaign evidence as csq-core.",
    },
}

STATUS_GLOSSARY = (
    "<strong>pass</strong> — the step ran and exited successfully. "
    "<strong>fail</strong> — the step ran and found a problem. "
    "<strong>not_run</strong> — this profile skipped the step; it is <em>not</em> a green check. "
    "<strong>missing</strong> — expected report file not on disk yet."
)

FAS_PRIMER = """
<div class="primer">
  <h3>What is FAS? (read this first)</h3>
  <p>
    <strong>Formal Assurance Spine</strong> is Residiuum’s system for governing
    <em>claims</em> about honesty of storage and authority. It is
    <em>not</em> a user-facing API and does <em>not</em> mean
    “the whole database is formally verified.”
  </p>
  <p><strong>How the pieces chain (connect the dots):</strong></p>
  <ol>
    <li><strong>FAS-0</strong> names every claim (theorem ids, assumptions).</li>
    <li><strong>FAS-1</strong> freezes the tools that check those claims.</li>
    <li><strong>FAS-2</strong> defines the abstract math words (State, Observation…).</li>
    <li><strong>FAS-3</strong> ties some words to real Rust functions (one slice today).</li>
    <li><strong>FAS-4</strong> states consistency laws in that math language (table below).</li>
    <li><strong>CSQ A2</strong> is the physical test campaign on real store paths that
        many of those laws rely on for “this actually happens on disk.”</li>
  </ol>
  <p>
    Product shipping still follows CSQ / PQH / APB → M2. FAS is a parallel trust lane.
    Operator guide: <code>formal/HOW_TO_USE.md</code>.
  </p>
</div>
"""


def esc(s: object) -> str:
    return html.escape("" if s is None else str(s), quote=True)


def status_class(st: str) -> str:
    st = (st or "unknown").lower()
    if st in ("pass", "ok", "green"):
        return "pass"
    if st in ("fail", "error", "red"):
        return "fail"
    if st in ("not_run", "skipped", "skip", "missing"):
        return "skip"
    if st in ("warn", "warning", "partial", "present"):
        return "warn"
    return "unknown"


def explain_for_step(step_id: str) -> dict[str, str]:
    return STEP_EXPLAIN.get(
        step_id or "",
        {
            "what": "Gate or checklist step recorded by the briefing runner.",
            "means": "See the command column and repository docs for this id.",
            "pass": "Command exited 0.",
            "fail": "Command exited non-zero.",
            "not_run": "Skipped for this profile.",
            "data": "",
        },
    )


def explain_for_artifact(art_id: str) -> dict[str, str]:
    return ARTIFACT_EXPLAIN.get(
        art_id or "",
        {
            "what": "Evidence file the briefing tried to ingest from disk.",
            "means": "Usually a JSON report written by a verify or formal script under target/.",
            "pass": "File present and looks successful.",
            "fail": "File present and indicates failure.",
            "not_run": "File missing.",
            "data": "",
        },
    )


def status_meaning(exp: dict[str, str], status: str) -> str:
    st = (status or "").lower()
    if st in ("pass", "ok"):
        return exp.get("pass") or "Succeeded."
    if st in ("fail", "error"):
        return exp.get("fail") or "Failed."
    if st in ("not_run", "skipped", "skip", "missing"):
        return exp.get("not_run") or "Not available for this run."
    if st in ("warn", "partial", "present"):
        return exp.get("means") or "See detail."
    return exp.get("means") or ""


def explain_row(colspan: int, exp: dict[str, str], status: str, *, fas_extra: str = "") -> str:
    parts = [
        f"<div><span class='ek'>What this is.</span> {esc(exp.get('what'))}</div>",
        f"<div><span class='ek'>Why it matters.</span> {esc(exp.get('means'))}</div>",
    ]
    if exp.get("data"):
        parts.append(
            f"<div><span class='ek'>How it relates to data.</span> {esc(exp.get('data'))}</div>"
        )
    parts.append(
        f"<div><span class='ek'>This status ({esc(status)}).</span> "
        f"{esc(status_meaning(exp, status))}</div>"
    )
    body = "<div class='explain-block'>" + "".join(parts)
    if fas_extra:
        body += f"<div class='fas-extra'>{fas_extra}</div>"
    body += "</div>"
    return f"<tr class='explain'><td colspan='{colspan}'>{body}</td></tr>"


def con_table_html() -> str:
    rows = []
    for t in CON_THEOREMS_PLAIN:
        rows.append(
            f"<tr class='data'>"
            f"<td><code>{esc(t['id'])}</code><div class='plain'>{esc(t['plain'])}</div></td>"
            f"<td>{esc(t['story'])}</td>"
            f"<td>{esc(t['on_data'])}</td>"
            f"<td class='detail'>{esc(t['math_hook'])}</td>"
            f"</tr>"
        )
    return f"""
  <h2>Consistency laws in plain English (FAS-4 CON family)</h2>
  <p class="sub">
    These are the eight honesty rules behind FAS-4. You do not need to read Lean
    to understand the intent. “Math hook” is only for authors tracing proofs.
    <strong>MVP honesty:</strong> Lean states the obligations; CSQ A2 supplies much
    of the physical evidence; full end-to-end refinement of every Store path is still incomplete.
  </p>
  <table>
    <thead><tr>
      <th>Law</th><th>In plain English</th><th>On store data (put/get/crash/salvage)</th><th>Math hook</th>
    </tr></thead>
    <tbody>
      {''.join(rows)}
    </tbody>
  </table>
"""


def _fmt_num(v: object, digits: int = 1) -> str:
    if v is None:
        return "—"
    try:
        f = float(v)
    except (TypeError, ValueError):
        return esc(v)
    if digits == 0:
        return f"{f:,.0f}"
    return f"{f:,.{digits}f}"


def _fmt_bytes_mib(v: object) -> str:
    if v is None:
        return "—"
    try:
        b = float(v)
    except (TypeError, ValueError):
        return esc(v)
    return f"{b / (1024 * 1024):,.1f} MiB"


def metrics_section_html(data: dict) -> str:
    """Render latest published diagnostic write/read metrics (not product gates)."""
    m = data.get("metrics") or {}
    if not m:
        return """
  <h2>Latest published read / write metrics</h2>
  <div class="note">
    No metrics block was assembled for this briefing (collector returned empty).
    Re-run <code>bash scripts/release-briefing.sh</code> from a tree that includes
    <code>doc/wip/status/surveys/</code> peer-pump artifacts.
  </div>
"""
    disclosure = m.get("disclosure") or (
        "Diagnostic only — not a published SLO and not a package accept gate."
    )
    knobs = m.get("knobs") or {}
    knobs_bits = []
    for k in (
        "logical_target",
        "payload",
        "seed",
        "residiuum_durability",
        "sqlite_journal",
        "sqlite_synchronous",
        "threads",
        "volume_hint",
    ):
        if knobs.get(k):
            knobs_bits.append(f"<code>{esc(k)}</code>={esc(knobs[k])}")
    knobs_line = " · ".join(knobs_bits) if knobs_bits else "see source paths"

    # --- inventory: what did we pull? ---
    inv = m.get("inventory") or {}
    inv_items = []
    for it in inv.get("items") or []:
        paths = it.get("paths") or []
        path_html = (
            "<ul class='metric-notes'>"
            + "".join(f"<li><code>{esc(p)}</code></li>" for p in paths)
            + "</ul>"
            if paths
            else "<em class='detail'>(none / not published)</em>"
        )
        inv_items.append(
            f"<tr class='data'>"
            f"<td><strong>{esc(it.get('group'))}</strong></td>"
            f"<td>{esc(it.get('why'))}</td>"
            f"<td>{path_html}</td>"
            f"</tr>"
        )
    inventory_html = ""
    if inv_items:
        inventory_html = f"""
  <h3>What data did we pull in?</h3>
  <p class="sub">
    {esc(inv.get('title') or 'Provenance of every metrics cell')}
    · if a path is missing, the briefing is not inventing a number for it.
  </p>
  <table>
    <thead><tr><th>Group</th><th>Why it is here</th><th>Source files</th></tr></thead>
    <tbody>{''.join(inv_items)}</tbody>
  </table>
"""

    # --- three-band headline ---
    bands = m.get("three_bands") or {}
    band_rows: list[str] = []
    for b in bands.get("rows") or []:
        hl = " class='data warn'" if b.get("highlight") else " class='data'"
        ladder = b.get("cook_ladder") or []
        ladder_s = ""
        if ladder:
            bits = [
                f"cook{x.get('workers')}={_fmt_num(x.get('ops'), 0)}"
                for x in ladder
                if x.get("ops") is not None
            ]
            ladder_s = " · ".join(bits)
        band_rows.append(
            f"<tr{hl}>"
            f"<td><strong>Band {esc(b.get('band'))}</strong><div class='plain'>{esc(b.get('name'))}</div></td>"
            f"<td class='num'>{_fmt_num(b.get('ops_per_sec'), 0)}</td>"
            f"<td class='num'>{_fmt_num(b.get('logical_mib_s'), 1)}</td>"
            f"<td>{esc(b.get('cores') or '—')}"
            f"<div class='detail'>{esc(ladder_s)}</div></td>"
            f"<td class='num'>{esc(b.get('batch') if b.get('batch') is not None else '—')}</td>"
            f"<td>{esc(b.get('bed') or '')}<div class='detail'>{esc(b.get('what') or '')}</div></td>"
            f"<td class='detail'><code>{esc(b.get('path') or '')}</code></td>"
            f"</tr>"
        )
    bands_html = ""
    if band_rows:
        bnotes = ""
        if bands.get("notes"):
            bnotes = "<ul class='metric-notes'>" + "".join(
                f"<li>{esc(n)}</li>" for n in bands["notes"]
            ) + "</ul>"
        bands_html = f"""
  <h3>Three-band write rates (primary story)</h3>
  <p class="sub">
    {esc(bands.get('title') or 'Do not mix beds')}
    · highlighted row = the <strong>~100k on 4 cook workers</strong> measurement
    (not writer-shards).
  </p>
  <table>
    <thead><tr>
      <th>Band</th><th>ops/s</th><th>Logical MiB/s</th>
      <th>Cores / workers</th><th>Batch</th><th>Bed + meaning</th><th>Source</th>
    </tr></thead>
    <tbody>{''.join(band_rows)}</tbody>
  </table>
  {bnotes}
"""

    # --- parallel cook detail ---
    cook = m.get("write_cook") or {}
    cook_html_blocks: list[str] = []
    for bed in cook.get("beds") or []:
        rows_html = []
        for r in bed.get("rows") or []:
            hl = " class='data warn'" if r.get("cook_workers") == 4 else " class='data'"
            rows_html.append(
                f"<tr{hl}>"
                f"<td><code>{esc(r.get('name'))}</code></td>"
                f"<td class='num'>{esc(r.get('cook_workers') if r.get('cook_workers') is not None else '—')}</td>"
                f"<td class='num'>{esc(r.get('put_batch_size') if r.get('put_batch_size') is not None else '—')}</td>"
                f"<td class='num'>{_fmt_num(r.get('ops_per_sec'), 0)}</td>"
                f"<td class='num'>{_fmt_num(r.get('logical_mib_s'), 1)}</td>"
                f"<td class='num'>{_fmt_num(r.get('wall_ms'), 1)}</td>"
                f"<td class='detail'>{esc(r.get('note') or '')}</td>"
                f"</tr>"
            )
        if not rows_html:
            continue
        cook_html_blocks.append(f"""
  <h4>{esc(bed.get('title'))}</h4>
  <p class="sub">source <code>{esc(bed.get('path') or '')}</code>
  · {esc(bed.get('note') or '')}</p>
  <table>
    <thead><tr>
      <th>Phase</th><th>Cook workers</th><th>Batch</th>
      <th>ops/s</th><th>Logical MiB/s</th><th>Wall ms</th><th>Note</th>
    </tr></thead>
    <tbody>{''.join(rows_html)}</tbody>
  </table>
""")
    cook_section = ""
    if cook_html_blocks:
        cnotes = ""
        if cook.get("notes"):
            cnotes = "<ul class='metric-notes'>" + "".join(
                f"<li>{esc(n)}</li>" for n in cook["notes"]
            ) + "</ul>"
        cook_section = f"""
  <h3>Parallel cook detail (real 4-core batched)</h3>
  <p class="sub">
    {esc(cook.get('title') or '')}
    · contract <code>{esc(cook.get('contract') or '')}</code>.
  </p>
  {''.join(cook_html_blocks)}
  {cnotes}
"""

    # --- write peer table ---
    peer = m.get("write_peer") or {}
    peer_rows: list[str] = []
    for cell in peer.get("cells") or []:
        peer_rows.append(
            f"<tr class='data'>"
            f"<td>{esc(cell.get('label') or cell.get('id'))}</td>"
            f"<td><code>{esc(cell.get('engine'))}</code></td>"
            f"<td><code>{esc(cell.get('mode'))}</code></td>"
            f"<td class='num'>{_fmt_num(cell.get('ops_per_sec'), 0)}</td>"
            f"<td class='num'>{_fmt_num(cell.get('logical_mib_s'), 1)}</td>"
            f"<td class='num'>{_fmt_num(cell.get('disk_mib_s'), 1)}</td>"
            f"<td class='num'>{_fmt_bytes_mib(cell.get('peak_rss_bytes'))}</td>"
            f"<td class='num'>{_fmt_num(cell.get('elapsed_s'), 2)}s</td>"
            f"<td class='detail'><code>{esc(cell.get('path') or '')}</code></td>"
            f"</tr>"
        )
    ratio_rows: list[str] = []
    for r in peer.get("ratios") or []:
        ratio_rows.append(
            f"<tr class='data'>"
            f"<td><code>{esc(r.get('mode'))}</code></td>"
            f"<td class='num'>{esc(r.get('ops_ratio') or '—')}</td>"
            f"<td class='num'>{esc(r.get('logical_mib_ratio') or '—')}</td>"
            f"<td>{esc(r.get('read') or '')}</td>"
            f"</tr>"
        )

    peer_html = ""
    if peer_rows:
        peer_html = f"""
  <h3>Write — PEER-SQL same-bed (latest campaign)</h3>
  <p class="sub">
    {esc(peer.get('title') or 'Residiuum vs SQLite peer-pump')}
    · campaign <code>{esc(peer.get('campaign') or '')}</code>
    · compare <strong>A vs A</strong> and <strong>B vs B</strong> only.
  </p>
  <table>
    <thead><tr>
      <th>Cell</th><th>Engine</th><th>Mode</th>
      <th>ops/s</th><th>Logical MiB/s</th><th>Disk MiB/s</th>
      <th>Peak RSS</th><th>Wall</th><th>Source</th>
    </tr></thead>
    <tbody>{''.join(peer_rows)}</tbody>
  </table>
"""
        if ratio_rows:
            peer_html += f"""
  <table>
    <thead><tr>
      <th>Mode</th><th>Residiuum/SQLite ops/s</th>
      <th>Logical MiB/s ratio</th><th>How to read</th>
    </tr></thead>
    <tbody>{''.join(ratio_rows)}</tbody>
  </table>
"""
        if peer.get("notes"):
            notes = "".join(f"<li>{esc(n)}</li>" for n in peer["notes"])
            peer_html += f"<ul class='metric-notes'>{notes}</ul>"
    else:
        peer_html = """
  <h3>Write — PEER-SQL same-bed</h3>
  <div class="note">No peer-pump JSON found under
  <code>doc/wip/status/surveys/scratch-sqlite-peer-*/</code>.</div>
"""

    # --- secondary: writer-shards / multi-process (NOT the 100k path) ---
    mc = m.get("write_multicore") or {}
    mc_rows: list[str] = []
    for cell in mc.get("cells") or []:
        cat = cell.get("category") or ""
        mc_rows.append(
            f"<tr class='data'>"
            f"<td><strong>{esc(cat)}</strong><div class='detail'>{esc(cell.get('label') or '')}</div></td>"
            f"<td><code>Axis {esc(cell.get('axis'))}</code></td>"
            f"<td class='num'>{esc(cell.get('writer_shards') if cell.get('writer_shards') is not None else '—')}</td>"
            f"<td class='num'>{esc(cell.get('put_batch_size') if cell.get('put_batch_size') is not None else '—')}</td>"
            f"<td class='num'>{esc(cell.get('store_count') if cell.get('store_count') is not None else '—')}</td>"
            f"<td class='num'>{_fmt_num(cell.get('ops_per_sec'), 0)}</td>"
            f"<td class='num'>{_fmt_num(cell.get('pump_mib_s'), 1)}</td>"
            f"<td class='num'>{_fmt_num(cell.get('peak_cpu_pct'), 1) if cell.get('peak_cpu_pct') is not None else '—'}</td>"
            f"<td class='num'>{_fmt_bytes_mib(cell.get('peak_rss_bytes'))}</td>"
            f"<td class='detail'><code>{esc(cell.get('path') or '')}</code></td>"
            f"</tr>"
        )
    multicore_html = ""
    if mc_rows:
        mc_notes = ""
        if mc.get("notes"):
            mc_notes = "<ul class='metric-notes'>" + "".join(
                f"<li>{esc(n)}</li>" for n in mc["notes"]
            ) + "</ul>"
        multicore_html = f"""
  <h3>Secondary — writer-shards / multi-process (not ~100k cook)</h3>
  <p class="sub">
    {esc(mc.get('title') or '')}
    · campaign <code>{esc(mc.get('campaign') or '')}</code>
    · contract <code>{esc(mc.get('contract') or '')}</code>.
    These sit in the <strong>~10k band</strong>. Do not confuse with cook_parallelism=4.
  </p>
  <table>
    <thead><tr>
      <th>Category</th><th>Axis</th><th>Shards</th><th>Batch</th><th>Stores</th>
      <th>ops/s</th><th>Pump MiB/s</th><th>Peak CPU%</th><th>Peak RSS</th><th>Source</th>
    </tr></thead>
    <tbody>{''.join(mc_rows)}</tbody>
  </table>
  {mc_notes}
"""

    # --- write micro / phase-bench ---
    micro = m.get("write_micro") or {}
    micro_rows: list[str] = []
    for row in micro.get("phases") or []:
        micro_rows.append(
            f"<tr class='data'>"
            f"<td><code>{esc(row.get('name'))}</code></td>"
            f"<td class='num'>{_fmt_num(row.get('ops_per_sec'), 0)}</td>"
            f"<td class='num'>{_fmt_num(row.get('logical_mib_s'), 1)}</td>"
            f"<td class='num'>{_fmt_num(row.get('wall_ms'), 1)}</td>"
            f"<td class='detail'>{esc(row.get('note') or '')}</td>"
            f"</tr>"
        )
    micro_html = ""
    if micro_rows:
        micro_html = f"""
  <h3>Write — short phase-bench (micro, not multi-seal peer)</h3>
  <p class="sub">
    {esc(micro.get('title') or 'phase-bench after write-through')}
    · source <code>{esc(micro.get('path') or '')}</code>
    · do <strong>not</strong> mix these rates with long PEER-SQL multi-seal numbers.
  </p>
  <table>
    <thead><tr>
      <th>Phase</th><th>ops/s</th><th>Logical MiB/s</th><th>Wall ms</th><th>Note</th>
    </tr></thead>
    <tbody>{''.join(micro_rows)}</tbody>
  </table>
"""
        if micro.get("band_rule"):
            micro_html += f"<p class='sub'>{esc(micro['band_rule'])}</p>"

    # --- reads ---
    rd = m.get("read") or {}
    st = (rd.get("status") or "").lower()
    if st in ("not_published", "missing", "absent", ""):
        read_html = f"""
  <h3>Read — get / get_payload path</h3>
  <div class="note">
    <strong>No get/read numbers</strong> in this briefing
    ({esc(rd.get('note') or 'monitor tooling exists; peer-pump is write-only')}).
  </div>
"""
    else:
        read_rows = []
        for cell in rd.get("cells") or []:
            read_rows.append(
                f"<tr class='data'>"
                f"<td><strong>{esc(cell.get('label'))}</strong>"
                f"<div class='detail'>{esc(cell.get('campaign') or '')} · "
                f"{esc(cell.get('phase') or '')}</div></td>"
                f"<td class='num'>{esc(cell.get('sample_keys') if cell.get('sample_keys') is not None else '—')}</td>"
                f"<td>{esc(cell.get('ok') or '—')}</td>"
                f"<td class='num'>{esc(cell.get('p50_us') if cell.get('p50_us') is not None else '—')}</td>"
                f"<td class='num'>{esc(cell.get('p95_us') if cell.get('p95_us') is not None else '—')}</td>"
                f"<td class='num'>{esc(cell.get('p99_us') if cell.get('p99_us') is not None else '—')}</td>"
                f"<td class='detail'>{esc(cell.get('path_class') or '')}"
                f"<div>{esc(cell.get('notes') or '')}</div></td>"
                f"</tr>"
            )
        not_m = rd.get("not_measured") or []
        not_lis = "".join(f"<li>{esc(x)}</li>" for x in not_m)
        tools = rd.get("tooling") or []
        tools_s = " · ".join(f"<code>{esc(t)}</code>" for t in tools)
        read_html = f"""
  <h3>Read — get stats (disclosed)</h3>
  <p class="sub">
    {esc(rd.get('title') or 'Hot PrimaryIndex gets')}
    · source <code>{esc(rd.get('source') or '')}</code>
    · status <code>{esc(rd.get('status') or '')}</code>.
  </p>
  <div class="note">
    <strong>How measured.</strong> {esc(rd.get('how_measured') or '')}
    <div style="margin-top:0.4rem">{esc(rd.get('note') or '')}</div>
    <div style="margin-top:0.35rem">Tooling: {tools_s}</div>
  </div>
  <table>
    <thead><tr>
      <th>Cell</th><th>Samples</th><th>OK</th>
      <th>p50 µs</th><th>p95 µs</th><th>p99 µs</th><th>Path / notes</th>
    </tr></thead>
    <tbody>{''.join(read_rows) if read_rows else '<tr><td colspan="7">No cells</td></tr>'}</tbody>
  </table>
  <p class="sub"><strong>Not measured yet (honest gaps):</strong></p>
  <ul class="metric-notes">{not_lis if not_lis else '<li>(none listed)</li>'}</ul>
"""

    sources = m.get("sources") or []
    src_lis = "".join(f"<li><code>{esc(s)}</code></li>" for s in sources)

    return f"""
  <h2>Latest published read / write metrics</h2>
  <div class="note">
    <strong>Honesty.</strong> {esc(disclosure)}
    Numbers are <em>diagnostic surveys</em> (Scratch / controlled beds), not PQH
    product accept and not “Residiuum is X MB/s” marketing. Always quote durability,
    batch mode, payload size, and volume class next to a rate.
    Narrative twin: <code>{esc(m.get('narrative') or 'TEST_RESULTS.md')}</code>.
    Contract: <code>{esc(m.get('contract') or 'doc/wip/status/surveys/README-PEER-SQL.md')}</code>.
  </div>
  <p class="sub">Fixed knobs (when known): {knobs_line}</p>
  {inventory_html}
  {bands_html}
  {cook_section}
  {peer_html}
  {multicore_html}
  {micro_html}
  {read_html}
  <h3>Metric sources ingested this run</h3>
  <ul class="metric-notes">{src_lis if src_lis else '<li>(none)</li>'}</ul>
"""


def render(data: dict) -> str:
    steps = data.get("steps") or []
    artifacts = data.get("artifacts") or []
    meta = data.get("meta") or {}
    overall = data.get("overall_status") or "unknown"
    counts = {"pass": 0, "fail": 0, "skip": 0, "warn": 0, "unknown": 0}
    for s in steps:
        c = status_class(s.get("status", ""))
        counts[c if c in counts else "unknown"] += 1

    rows: list[str] = []
    for s in steps:
        st = s.get("status") or "unknown"
        sc = status_class(st)
        dur = s.get("duration_ms")
        dur_s = f"{dur / 1000:.1f}s" if isinstance(dur, (int, float)) else "—"
        cmd = s.get("command") or s.get("path") or ""
        detail = s.get("detail") or s.get("message") or ""
        if len(detail) > 400:
            detail = detail[:400] + "…"
        sid = str(s.get("id") or "")
        exp = explain_for_step(sid)
        fas_extra = ""
        if sid.startswith("fas"):
            fas_extra = (
                "<span class='ek'>FAS note.</span> This is claim/math infrastructure, "
                "not an application feature flag. It governs what we may say about "
                "honesty of storage and authority. See primer and CON table."
            )
            if sid == "fas4":
                fas_extra += (
                    " The eight CON laws are expanded in plain English in the section "
                    "<em>Consistency laws in plain English</em> below — that is the "
                    "map from theorem id → what happens to keys and segments."
                )
        rows.append(
            f"<tr class='data {sc}'>"
            f"<td><code>{esc(sid)}</code></td>"
            f"<td>{esc(s.get('title'))}</td>"
            f"<td><span class='badge {sc}'>{esc(st)}</span></td>"
            f"<td>{esc(dur_s)}</td>"
            f"<td><code class='cmd'>{esc(cmd)}</code></td>"
            f"<td class='detail'>{esc(detail)}</td>"
            f"</tr>"
        )
        rows.append(explain_row(6, exp, st, fas_extra=fas_extra))

    art_rows: list[str] = []
    for a in artifacts:
        aid = str(a.get("id") or "")
        st = a.get("status") or "unknown"
        sc = status_class(st)
        exp = explain_for_artifact(aid)
        fas_extra = ""
        if aid.startswith("fas"):
            fas_extra = (
                "<span class='ek'>How to read this JSON.</span> "
                "<code>result=pass</code> means the <em>gate script</em> accepted its "
                "checklist — not that every Residiuum deployment is proven safe. "
                "Open formal/HOW_TO_USE.md for the chain FAS-0→4."
            )
            if aid == "fas4":
                fas_extra += (
                    " For what “consistency” means on keys and crashes, read the "
                    "CON plain-English table on this page."
                )
        art_rows.append(
            f"<tr class='data {sc}'>"
            f"<td><code>{esc(aid)}</code></td>"
            f"<td><code class='cmd'>{esc(a.get('path'))}</code></td>"
            f"<td><span class='badge {sc}'>{esc(st)}</span></td>"
            f"<td>{esc(a.get('summary') or '')}</td>"
            f"</tr>"
        )
        art_rows.append(explain_row(4, exp, st, fas_extra=fas_extra))

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
    --border: #2a3548; --explain: #141c28; --accent: #6cb6ff;
    --mono: ui-monospace, SFMono-Regular, Menlo, monospace;
    --sans: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0; padding: 2rem 1.25rem 4rem;
    font-family: var(--sans); background: var(--bg); color: var(--text);
    line-height: 1.5;
  }}
  .wrap {{ max-width: 1100px; margin: 0 auto; }}
  h1 {{ font-size: 1.5rem; margin: 0 0 0.35rem; font-weight: 650; }}
  h2 {{ font-size: 1.1rem; margin: 2rem 0 0.75rem; color: var(--muted); font-weight: 600; }}
  h3 {{ font-size: 1rem; margin: 1.5rem 0 0.5rem; color: var(--accent); }}
  h4 {{ font-size: 0.95rem; margin: 1rem 0 0.4rem; color: var(--text); font-weight: 600; }}
  .sub {{ color: var(--muted); font-size: 0.95rem; margin-bottom: 1.25rem; }}
  .plain {{ color: var(--accent); font-weight: 600; margin-top: 0.25rem; font-size: 0.9rem; }}
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
    font-size: 0.88rem; margin-bottom: 0.5rem;
  }}
  th, td {{ text-align: left; padding: 0.55rem 0.65rem; vertical-align: top; border-bottom: 1px solid var(--border); }}
  th {{ color: var(--muted); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.04em; }}
  tr.data:hover {{ background: #1f2a3d; }}
  tr.data.warn {{ background: #2a2410; }}
  tr.data.warn:hover {{ background: #3a3218; }}
  tr.explain td {{
    background: var(--explain); border-bottom: 2px solid var(--border);
    padding: 0.75rem 0.9rem 1rem; color: var(--muted); font-size: 0.86rem;
  }}
  .explain-block div {{ margin: 0.28rem 0; }}
  .ek {{ color: var(--accent); font-weight: 600; margin-right: 0.25rem; }}
  .fas-extra {{ margin-top: 0.45rem; padding-top: 0.4rem; border-top: 1px dashed var(--border); }}
  code {{ font-family: var(--mono); font-size: 0.82em; }}
  code.cmd {{ color: #a8b8d0; word-break: break-all; }}
  td.detail {{ color: var(--muted); max-width: 260px; }}
  td.num {{ font-family: var(--mono); font-variant-numeric: tabular-nums; text-align: right; white-space: nowrap; }}
  .note, .primer, .glossary {{
    background: var(--card); border: 1px solid var(--border);
    padding: 0.9rem 1.1rem; margin: 1rem 0; border-radius: 10px;
    color: var(--muted); font-size: 0.92rem;
  }}
  .note {{ border-left: 3px solid var(--warn); }}
  .primer {{ border-left: 3px solid var(--accent); }}
  .primer ol {{ margin: 0.4rem 0 0.6rem 1.2rem; padding: 0; }}
  .primer li {{ margin: 0.25rem 0; }}
  ul.metric-notes {{ color: var(--muted); font-size: 0.9rem; margin: 0.5rem 0 1rem 1.2rem; }}
  ul.metric-notes li {{ margin: 0.25rem 0; }}
  .overall {{ margin: 1rem 0 1.25rem; }}
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
    · exit policy: fail if any executed gate failed; <code>not_run</code> is not pass
  </div>
  <div class="cards">
    <div class="card pass"><div class="n">{counts['pass']}</div><div class="l">Pass</div></div>
    <div class="card fail"><div class="n">{counts['fail']}</div><div class="l">Fail</div></div>
    <div class="card skip"><div class="n">{counts['skip']}</div><div class="l">Not run / skip</div></div>
    <div class="card warn"><div class="n">{counts['warn']}</div><div class="l">Warn</div></div>
  </div>
  <div class="glossary">
    <span class="ek">Status legend.</span> {STATUS_GLOSSARY}
  </div>
  {FAS_PRIMER}
  <div class="note">
    <strong>How to read this document.</strong> Each result row is followed by an
    explanation row: what the check is, why it matters, how it relates to data,
    and what <em>this</em> status means. Formal rows are expanded further in the
    CON plain-English table — theorem ids alone are not enough for non-authors.
    The <em>Latest published read / write metrics</em> section is diagnostic survey
    data (not a gate status): useful context only; never treat missing read numbers as zero.
  </div>
  <h2>Steps (result + explanation)</h2>
  <table>
    <thead><tr>
      <th>Id</th><th>Title</th><th>Status</th><th>Time</th><th>Command / path</th><th>Detail</th>
    </tr></thead>
    <tbody>
      {''.join(rows) if rows else '<tr><td colspan="6">No steps</td></tr>'}
    </tbody>
  </table>
  <h2>Ingested artifacts (result + explanation)</h2>
  <table>
    <thead><tr><th>Id</th><th>Path</th><th>Status</th><th>Summary</th></tr></thead>
    <tbody>
      {''.join(art_rows) if art_rows else '<tr><td colspan="4">No artifacts</td></tr>'}
    </tbody>
  </table>
  {metrics_section_html(data)}
  {con_table_html()}
  <h2>How to re-run</h2>
  <p class="sub">
    <code>bash scripts/release-briefing.sh --profile {esc(meta.get('profile') or 'snapshot')}</code><br/>
    Full PR-local quality: <code>./scripts/quality.sh</code> ·
    CSQ A2: <code>bash scripts/residiuum-verify-core-storage.sh --require-a2-pass</code> ·
    FAS: <code>formal/HOW_TO_USE.md</code> · package: <code>make dist</code>
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