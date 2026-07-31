#!/usr/bin/env python3
"""CSQ-12 core-storage evidence bundle builder + independent verifier.

Profile: residiuum-core-storage-v1 only (pre-reset dingo identities inadmissible).
Format: residiuum-core-storage-report-v1 (see report-v1.schema.json).

Honesty rules (SPEC §20–23, IMPLEMENTATION_PLAN §16):
- not_run / infrastructure_failure / empty prose cannot satisfy A2 pass
- a declared result=pass with missing_cells or non-pass cells is REJECTED
- builder and verifier are separate code paths (verify re-derives verdict)
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

PROFILE = "residiuum-core-storage-v1"
REPORT_FORMAT = "residiuum-core-storage-report-v1"
ENVELOPE_FORMAT = "residiuum-verification-report-v1"
LEVELS = ("A1", "A2", "A3")
CELL_RESULTS = ("pass", "fail", "not_run", "infrastructure_failure")

# A2 residual gates not closed by package labor floors alone (SPEC §23.1–23.2).
A2_RESIDUAL_GATES = (
    "CSQ12-GATE-PREDECESSOR-ACCEPT",
    "CSQ12-GATE-FULL-BOUNDARY-MATRIX",
    "CSQ12-GATE-PLATFORM-MATRIX",
    "CSQ12-GATE-SOAK-72H",
    "CSQ12-GATE-FULL-MUTATION-THRESHOLD",
    "CSQ12-GATE-INDEPENDENT-BUNDLE-PUBLICATION",
)


def workspace_root() -> Path:
    env = os.environ.get("RESIDIUUM_WORKSPACE_ROOT")
    if env:
        return Path(env).resolve()
    # scripts/lib/csq_evidence.py → repo root
    return Path(__file__).resolve().parents[2]


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def blake3_hex(data: bytes) -> str:
    """Prefer blake3 if installed; fall back to sha256 with explicit prefix."""
    try:
        import blake3  # type: ignore

        return blake3.blake3(data).hexdigest()
    except Exception:
        return "sha256:" + hashlib.sha256(data).hexdigest()


def git_source_revision(root: Path) -> str:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        dirty = subprocess.call(
            ["git", "diff", "--quiet"],
            cwd=root,
            stderr=subprocess.DEVNULL,
        )
        return out + ("-dirty" if dirty != 0 else "")
    except Exception:
        return "unknown"


def suite_active(status: str | None) -> bool:
    if not status:
        return False
    # `executable` = CSQ-0 registry suite is live under verify-core-storage-registry.
    return status.startswith("active_") or status in (
        "pass",
        "accept",
        "qualified",
        "executable",
    )


def build_report(root: Path, level: str = "A2") -> dict[str, Any]:
    if level not in LEVELS:
        raise SystemExit(f"unsupported level {level}")
    cs = root / "spec" / "verification" / "core-storage"
    profiles = load_json(cs / "profiles-v1.json")["items"]
    profile = next((p for p in profiles if p.get("id") == PROFILE), None)
    if profile is None:
        raise SystemExit(f"profile {PROFILE} missing from profiles-v1.json")
    if "dingo" in PROFILE.lower():
        raise SystemExit("forbidden dingo profile identity")

    suites_doc = load_json(cs / "suites-v1.json")
    suite_by_id = {s["id"]: s for s in suites_doc["items"]}
    inv_doc = load_json(cs / "invariants-v1.json")
    inv_by_id = {i["id"]: i for i in inv_doc["items"]}
    assumptions_doc = load_json(cs / "assumptions-v1.json")
    assumption_ids = [a["id"] for a in assumptions_doc["items"]]

    mandatory_invs: list[str] = list(profile.get("invariants") or [])
    profile_suites: list[str] = list(profile.get("suites") or [])
    profile_assumptions: list[str] = list(profile.get("assumptions") or [])

    cells: list[dict[str, Any]] = []
    missing: list[str] = []

    for inv_id in mandatory_invs:
        inv = inv_by_id.get(inv_id)
        if inv is None:
            cells.append(
                {
                    "cell_id": f"inv:{inv_id}",
                    "invariant_id": inv_id,
                    "result": "fail",
                    "detail": "invariant not in registry",
                }
            )
            missing.append(inv_id)
            continue
        owner_suites = inv.get("suites") or []
        active_owners = [
            sid
            for sid in owner_suites
            if suite_active((suite_by_id.get(sid) or {}).get("status"))
        ]
        if active_owners:
            evidence = f"labor_floor:{','.join(sorted(active_owners))}"
            cells.append(
                {
                    "cell_id": f"inv:{inv_id}",
                    "invariant_id": inv_id,
                    "suite_id": active_owners[0],
                    "oracle_id": (inv.get("oracles") or ["CSQ-ORACLE-IMPL"])[0],
                    "result": "pass",
                    "evidence_hash": blake3_hex(evidence.encode()),
                    "evidence_kind": "package_labor_floor",
                    "detail": evidence,
                }
            )
        else:
            cells.append(
                {
                    "cell_id": f"inv:{inv_id}",
                    "invariant_id": inv_id,
                    "result": "not_run",
                    "detail": "no active owning suite",
                }
            )
            missing.append(inv_id)

    # Suite activation cells (profile suite list).
    for sid in profile_suites:
        st = (suite_by_id.get(sid) or {}).get("status")
        if suite_active(st):
            cells.append(
                {
                    "cell_id": f"suite:{sid}",
                    "suite_id": sid,
                    "result": "pass",
                    "evidence_hash": blake3_hex(f"suite_status:{sid}:{st}".encode()),
                    "detail": f"status={st}",
                }
            )
        else:
            cells.append(
                {
                    "cell_id": f"suite:{sid}",
                    "suite_id": sid,
                    "result": "not_run",
                    "detail": f"status={st}",
                }
            )
            missing.append(sid)

    # Residual A2 gates stay not_run until principal/campaign evidence exists.
    for gate in A2_RESIDUAL_GATES:
        cells.append(
            {
                "cell_id": f"gate:{gate}",
                "result": "not_run",
                "detail": "residual A2 gate (first labor cut)",
            }
        )
        missing.append(gate)

    # Ledger: profile assumptions must appear; reject unknown dingo-ish ids.
    ledger = list(profile_assumptions) if profile_assumptions else list(assumption_ids)
    for a in ledger:
        if "dingo" in a.lower() and "post-reset" not in a.lower() and "POST-RESET" not in a:
            # POST-RESET identity assumption is allowed; pure dingo claims are not.
            if a != "CSQ-ASM-POST-RESET-IDENTITY":
                raise SystemExit(f"forbidden assumption id in ledger: {a}")

    overall = derive_overall_result(cells, missing, claim_pass=False)

    report: dict[str, Any] = {
        "format": REPORT_FORMAT,
        "profile": PROFILE,
        "level": level,
        "source_revision": git_source_revision(root),
        "result": overall,
        "assumption_ledger": ledger,
        "cells": cells,
        "missing_cells": sorted(set(missing)),
        "attachments": [
            {
                "type": "csq12_builder_meta",
                "body": {
                    "builder": "scripts/lib/csq_evidence.py",
                    "builder_version": "csq12-labor-v1",
                    "identity_policy": "residiuum_only_no_legacy_dingo",
                    "note": (
                        "Labor floors may mark invariant cells pass when suites are "
                        "active_*; A2 residual gates remain not_run until campaigns "
                        "and principal accept close them. Independent verify re-derives "
                        "the overall result and rejects false pass claims."
                    ),
                },
            },
            {
                "type": "retention_policy",
                "body": {
                    "policy_id": "residiuum-csq-evidence-retention-v1",
                    "keep_failures": True,
                    "minimization_replaces_original": False,
                    "retry_is_additional_not_replacement": True,
                    "infrastructure_failure_satisfies_gate": False,
                    "not_run_satisfies_gate": False,
                },
            },
        ],
    }
    return report


def derive_overall_result(
    cells: list[dict[str, Any]],
    missing: list[str],
    *,
    claim_pass: bool,
) -> str:
    """Independent overall result derivation (used by builder and verifier)."""
    if any(c.get("result") == "fail" for c in cells):
        return "fail"
    if any(c.get("result") == "infrastructure_failure" for c in cells):
        return "infrastructure_failure"
    if missing or any(c.get("result") == "not_run" for c in cells):
        return "not_run"
    if all(c.get("result") == "pass" for c in cells) and not missing:
        return "pass" if claim_pass else "not_run"
    return "not_run"


def structural_check(report: dict[str, Any]) -> list[str]:
    errs: list[str] = []
    if report.get("format") != REPORT_FORMAT:
        errs.append(f"format must be {REPORT_FORMAT}")
    if report.get("profile") != PROFILE:
        errs.append(f"profile must be {PROFILE} (got {report.get('profile')!r})")
    if "dingo" in str(report.get("profile", "")).lower():
        errs.append("dingo profile identity is inadmissible")
    if report.get("level") not in LEVELS:
        errs.append(f"level invalid: {report.get('level')}")
    if not report.get("source_revision"):
        errs.append("source_revision required")
    if report.get("result") not in CELL_RESULTS:
        errs.append(f"result invalid: {report.get('result')}")
    cells = report.get("cells")
    if not isinstance(cells, list):
        errs.append("cells must be array")
        return errs
    for i, c in enumerate(cells):
        if not isinstance(c, dict):
            errs.append(f"cells[{i}] not object")
            continue
        if "cell_id" not in c or "result" not in c:
            errs.append(f"cells[{i}] missing cell_id/result")
        if c.get("result") not in CELL_RESULTS:
            errs.append(f"cells[{i}] bad result {c.get('result')}")
    missing = report.get("missing_cells")
    if missing is not None and not isinstance(missing, list):
        errs.append("missing_cells must be array when present")
    return errs


def evaluate_a2(report: dict[str, Any]) -> dict[str, Any]:
    """A2 qualification evaluator — never trusts declared result alone."""
    errors = structural_check(report)
    cells = list(report.get("cells") or [])
    missing = list(report.get("missing_cells") or [])

    # Gate: not_run / infra cannot satisfy.
    non_pass = [c for c in cells if c.get("result") != "pass"]
    has_not_run = any(c.get("result") == "not_run" for c in cells)
    has_infra = any(c.get("result") == "infrastructure_failure" for c in cells)
    has_fail = any(c.get("result") == "fail" for c in cells)

    derived = derive_overall_result(cells, missing, claim_pass=True)
    declared = report.get("result")

    # False-pass detection: declared pass while incomplete.
    if declared == "pass":
        if missing or non_pass or has_not_run or has_infra or has_fail:
            errors.append(
                "declared result=pass but cells/missing are incomplete "
                "(not_run/infra/fail/missing cannot satisfy A2)"
            )
        if derived != "pass":
            errors.append(f"declared pass but derived overall is {derived}")

    # Empty cell list cannot pass A2.
    if declared == "pass" and not cells:
        errors.append("empty cells cannot satisfy A2")

    # Assumption ledger required for A2 claim.
    ledger = report.get("assumption_ledger") or []
    if declared == "pass" and not ledger:
        errors.append("A2 pass requires non-empty assumption_ledger")

    a2_pass = declared == "pass" and derived == "pass" and not errors and not missing

    return {
        "level": "A2",
        "profile": report.get("profile"),
        "declared_result": declared,
        "derived_result": derived,
        "a2_pass": a2_pass,
        "missing_cells": missing,
        "non_pass_cells": [c.get("cell_id") for c in non_pass],
        "errors": errors,
        "capability_language": (
            f"{PROFILE} / A2"
            if a2_pass
            else f"{PROFILE} / A2 not claimed (result={derived}; missing={len(missing)})"
        ),
    }


def verify_report(report: dict[str, Any]) -> dict[str, Any]:
    """Independent verifier: structure + evaluation; does not trust CI prose."""
    evaluation = evaluate_a2(report)
    ok = not evaluation["errors"] and structural_check(report) == []
    # Verifier accepts an honest incomplete bundle (not_run + exact missing).
    if report.get("result") in ("not_run", "fail", "infrastructure_failure"):
        # Still require structural integrity and no false-pass errors.
        ok = structural_check(report) == [] and not any(
            "declared result=pass" in e for e in evaluation["errors"]
        )
    elif report.get("result") == "pass":
        ok = evaluation["a2_pass"]
    return {
        "ok": ok,
        "evaluation": evaluation,
        "structural_errors": structural_check(report),
    }


def wrap_envelope(core_report: dict[str, Any]) -> dict[str, Any]:
    return {
        "format": ENVELOPE_FORMAT,
        "attachments": [
            {
                "type": "residiuum-core-storage-report-v1",
                "body": core_report,
            }
        ],
    }


def cmd_build(args: argparse.Namespace) -> int:
    root = Path(args.workspace).resolve() if args.workspace else workspace_root()
    report = build_report(root, level=args.level)
    out = wrap_envelope(report) if args.envelope else report
    text = json.dumps(out, indent=2, sort_keys=False) + "\n"
    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        Path(args.output).write_text(text, encoding="utf-8")
        print(f"wrote {args.output}", file=sys.stderr)
    else:
        sys.stdout.write(text)
    # Exit 0 for successful *build* even when incomplete (exact missing cells).
    print(
        f"result={report['result']} missing={len(report['missing_cells'])} "
        f"cells={len(report['cells'])}",
        file=sys.stderr,
    )
    return 0


def extract_core(doc: dict[str, Any]) -> dict[str, Any]:
    if doc.get("format") == REPORT_FORMAT:
        return doc
    if doc.get("format") == ENVELOPE_FORMAT:
        for att in doc.get("attachments") or []:
            if att.get("type") in (REPORT_FORMAT, "residiuum-core-storage-report-v1"):
                body = att.get("body")
                if isinstance(body, dict):
                    return body
        raise SystemExit("envelope missing residiuum-core-storage-report-v1 attachment")
    raise SystemExit(f"unknown report format {doc.get('format')!r}")


def cmd_verify(args: argparse.Namespace) -> int:
    path = Path(args.report)
    doc = load_json(path)
    core = extract_core(doc)
    verdict = verify_report(core)
    print(json.dumps(verdict, indent=2))
    if not verdict["ok"]:
        return 1
    # When --require-a2-pass is set, incomplete bundles fail.
    if args.require_a2_pass and not verdict["evaluation"]["a2_pass"]:
        print("A2 pass required but not achieved", file=sys.stderr)
        return 2
    return 0


def cmd_evaluate(args: argparse.Namespace) -> int:
    path = Path(args.report)
    doc = load_json(path)
    core = extract_core(doc)
    evaluation = evaluate_a2(core)
    print(json.dumps(evaluation, indent=2))
    return 0 if not evaluation["errors"] else 1


def cmd_selftest(_args: argparse.Namespace) -> int:
    """Unit checks for honesty rules (no workspace mutation)."""
    failed = 0

    def check(name: str, cond: bool) -> None:
        nonlocal failed
        if cond:
            print(f"ok  {name}")
        else:
            print(f"FAIL {name}")
            failed += 1

    # Structural positive skeleton.
    good_incomplete = {
        "format": REPORT_FORMAT,
        "profile": PROFILE,
        "level": "A2",
        "source_revision": "test",
        "result": "not_run",
        "assumption_ledger": ["CSQ-ASM-POST-RESET-IDENTITY"],
        "cells": [
            {"cell_id": "inv:CSQ-ID-001", "invariant_id": "CSQ-ID-001", "result": "pass"},
            {"cell_id": "gate:X", "result": "not_run"},
        ],
        "missing_cells": ["gate:X"],
    }
    v = verify_report(good_incomplete)
    check("honest incomplete verifies", v["ok"] and not v["evaluation"]["a2_pass"])

    false_pass = dict(good_incomplete)
    false_pass["result"] = "pass"
    v2 = verify_report(false_pass)
    check("false pass rejected", not v2["ok"])

    dingo = dict(good_incomplete)
    dingo["profile"] = "dingo-core-storage-v1"
    v3 = verify_report(dingo)
    check("dingo profile rejected", not v3["ok"] or bool(structural_check(dingo)))

    empty_pass = {
        "format": REPORT_FORMAT,
        "profile": PROFILE,
        "level": "A2",
        "source_revision": "test",
        "result": "pass",
        "cells": [],
        "missing_cells": [],
    }
    v4 = evaluate_a2(empty_pass)
    check("empty cells cannot A2 pass", not v4["a2_pass"] and v4["errors"])

    not_run_cell = {
        "format": REPORT_FORMAT,
        "profile": PROFILE,
        "level": "A2",
        "source_revision": "test",
        "result": "pass",
        "assumption_ledger": ["CSQ-ASM-POST-RESET-IDENTITY"],
        "cells": [{"cell_id": "c1", "result": "not_run"}],
        "missing_cells": [],
    }
    v5 = evaluate_a2(not_run_cell)
    check("not_run cannot satisfy A2", not v5["a2_pass"])

    infra = dict(not_run_cell)
    infra["cells"] = [{"cell_id": "c1", "result": "infrastructure_failure"}]
    v6 = evaluate_a2(infra)
    check("infra failure cannot satisfy A2", not v6["a2_pass"])

    full = {
        "format": REPORT_FORMAT,
        "profile": PROFILE,
        "level": "A2",
        "source_revision": "test",
        "result": "pass",
        "assumption_ledger": ["CSQ-ASM-POST-RESET-IDENTITY"],
        "cells": [{"cell_id": "c1", "result": "pass", "evidence_hash": "abc"}],
        "missing_cells": [],
    }
    v7 = evaluate_a2(full)
    check("complete pass accepted by evaluator", v7["a2_pass"])

    return 1 if failed else 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="csq_evidence")
    sub = p.add_subparsers(dest="cmd", required=True)

    b = sub.add_parser("build", help="Build residiuum-core-storage-report-v1")
    b.add_argument("--workspace", default=None)
    b.add_argument("--level", default="A2", choices=LEVELS)
    b.add_argument("--output", "-o", default=None)
    b.add_argument(
        "--envelope",
        action="store_true",
        help="Wrap in residiuum-verification-report-v1 envelope",
    )
    b.set_defaults(func=cmd_build)

    v = sub.add_parser("verify", help="Independently verify a report file")
    v.add_argument("report")
    v.add_argument(
        "--require-a2-pass",
        action="store_true",
        help="Fail unless A2 qualification is fully satisfied",
    )
    v.set_defaults(func=cmd_verify)

    e = sub.add_parser("evaluate", help="Evaluate A2 claim on a report")
    e.add_argument("report")
    e.set_defaults(func=cmd_evaluate)

    t = sub.add_parser("selftest", help="Honesty unit checks")
    t.set_defaults(func=cmd_selftest)

    args = p.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    sys.exit(main())