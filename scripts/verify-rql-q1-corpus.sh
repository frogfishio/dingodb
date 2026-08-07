#!/usr/bin/env bash
# RQL-Q1: validate practical query corpus schema, live document, floors, and fixture self-tests.
# Exit 0 = structural integrity + floors when enforce_floors=true. Does not accept the package.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CORPUS="$ROOT/spec/rql/qualification/corpus-v1"
ERR=0

fail() { printf 'verify-rql-q1-corpus: FAIL: %s\n' "$*" >&2; ERR=1; }
ok() { printf 'verify-rql-q1-corpus: %s\n' "$*"; }

need() {
  if [[ ! -f "$1" ]]; then
    fail "missing required file: $1"
  fi
}

for f in \
  README.md \
  corpus-v1.json \
  corpus-v1.schema.json \
  corpus-case-v1.schema.json \
  fixtures/case.accepted.min.json \
  fixtures/case.rejected.incomplete.json
do
  need "$CORPUS/$f"
done

need "$ROOT/doc/todo/rql/RQL_Q1_CORPUS.md"

command -v python3 >/dev/null 2>&1 || { fail "python3 required"; exit 1; }

python3 - "$ROOT" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
corpus_dir = root / "spec/rql/qualification/corpus-v1"
err = 0

def fail(msg: str) -> None:
    global err
    print(f"verify-rql-q1-corpus: FAIL: {msg}", file=sys.stderr)
    err = 1

def load(path: Path):
    with path.open(encoding="utf-8") as f:
        return json.load(f)

FORMAT = "residiuum-rql-q1-corpus-v1"
PROFILE = "rql-gate1-practical-corpus-v1"
EQ = "rql-q0-result-equivalence-v1"
VERSION_RE = __import__("re").compile(r"^rql-q1-corpus-v\d+\.\d+\.\d+$")
CASE_ID_RE = __import__("re").compile(r"^[a-z][a-z0-9_.-]*$")

DOMAINS = {
    "commerce",
    "messaging",
    "directory",
    "telemetry",
    "project_management",
}
TIERS = {"A", "B", "C"}
FAMILY_TAGS = {
    "selection_key_eq_range_compound",
    "predicate_missing_null_type_nested_array",
    "projection_computed_conditional",
    "order_topk_cursor",
    "enrichment_cardinality",
    "group_aggregate",
    "budget_coverage_damage_refusal",
}
FLOOR_DEFAULTS = {
    "selection_key_eq_range_compound": 20,
    "predicate_missing_null_type_nested_array": 20,
    "projection_computed_conditional": 15,
    "order_topk_cursor": 15,
    "enrichment_cardinality": 15,
    "group_aggregate": 15,
    "budget_coverage_damage_refusal": 10,
}
SELECTIVITY = {
    "point",
    "very_low",
    "low",
    "medium",
    "high",
    "broad",
    "not_applicable",
}
CARDINALITY = {"low", "medium", "high", "not_applicable"}
ORDER = {
    "unordered_multiset",
    "total_order",
    "partial_order_with_tie_break",
    "refusal_unordered",
}
MULTIPLICITY = {"set", "bag", "singleton", "empty_ok"}
EXPECTED_KINDS = {"literal_result", "oracle_rule", "stable_refusal", "deferred_q2"}
EXCL_KINDS = {
    "none",
    "deliberate_exclusion",
    "stable_refusal",
    "lane_local_only",
    "predeclared_native_diff",
}
RQL_STATUS = {"source", "pending", "stable_refusal", "deliberate_exclusion"}
MONGO_STATUS = {
    "pipeline",
    "find",
    "pending",
    "stable_refusal",
    "deliberate_exclusion",
    "lane_local_only",
}
CBL_STATUS = {
    "sqlpp",
    "query_builder",
    "pending",
    "stable_refusal",
    "deliberate_exclusion",
    "lane_local_only",
}

# --- schema files parse + identity ---
doc_schema = load(corpus_dir / "corpus-v1.schema.json")
case_schema = load(corpus_dir / "corpus-case-v1.schema.json")
if doc_schema.get("title") != "residiuum-rql-q1-corpus-v1":
    fail("corpus-v1.schema.json title mismatch")
if case_schema.get("title") != "residiuum-rql-q1-corpus-case-v1":
    fail("corpus-case-v1.schema.json title mismatch")
case_required = set(case_schema.get("required") or [])
needed_case_fields = {
    "case_id",
    "tier",
    "domain",
    "family_tags",
    "plain_english_intent",
    "fixture",
    "expected",
    "ordering_and_multiplicity",
    "implementations",
    "indexes",
    "selectivity_class",
    "cardinality_class",
    "variants",
    "exclusion_or_refusal",
}
missing_req = needed_case_fields - case_required
if missing_req:
    fail(f"case schema missing required fields: {sorted(missing_req)}")

def validate_case(case: dict, label: str) -> list[str]:
    """Return list of structural errors (empty = ok). Lightweight; no jsonschema dep."""
    errors: list[str] = []

    def need_key(obj, key, path):
        if key not in obj:
            errors.append(f"{label}: missing {path}.{key}" if path else f"{label}: missing {key}")
            return False
        return True

    for k in needed_case_fields:
        need_key(case, k, "")

    if "case_id" in case:
        cid = case["case_id"]
        if not isinstance(cid, str) or not CASE_ID_RE.match(cid):
            errors.append(f"{label}: invalid case_id {cid!r}")

    if case.get("tier") not in TIERS:
        errors.append(f"{label}: tier must be A|B|C")
    if case.get("domain") not in DOMAINS:
        errors.append(f"{label}: domain not in required set")

    tags = case.get("family_tags")
    if not isinstance(tags, list) or len(tags) < 1:
        errors.append(f"{label}: family_tags must be non-empty list")
    else:
        if len(tags) != len(set(tags)):
            errors.append(f"{label}: family_tags must be unique")
        for t in tags:
            if t not in FAMILY_TAGS:
                errors.append(f"{label}: unknown family_tag {t!r}")

    intent = case.get("plain_english_intent")
    if not isinstance(intent, str) or len(intent) < 8:
        errors.append(f"{label}: plain_english_intent too short")

    fixture = case.get("fixture")
    if not isinstance(fixture, dict):
        errors.append(f"{label}: fixture must be object")
    else:
        if not fixture.get("generator_id"):
            errors.append(f"{label}: fixture.generator_id required")
        if "seed" not in fixture or not isinstance(fixture["seed"], int) or fixture["seed"] < 0:
            errors.append(f"{label}: fixture.seed must be int >= 0")

    expected = case.get("expected")
    if not isinstance(expected, dict) or expected.get("kind") not in EXPECTED_KINDS:
        errors.append(f"{label}: expected.kind invalid")
    else:
        kind = expected["kind"]
        if kind == "literal_result" and "literal" not in expected:
            errors.append(f"{label}: literal_result requires expected.literal")
        if kind == "oracle_rule" and not (
            expected.get("oracle_rule_id") or expected.get("oracle_rule_text")
        ):
            errors.append(f"{label}: oracle_rule requires id or text")
        if kind == "stable_refusal" and not expected.get("oracle_rule_text") and not expected.get(
            "oracle_rule_id"
        ):
            # refusal detail may also live on implementations; allow exclusion_or_refusal
            pass

    om = case.get("ordering_and_multiplicity")
    if not isinstance(om, dict):
        errors.append(f"{label}: ordering_and_multiplicity required")
    else:
        if om.get("order") not in ORDER:
            errors.append(f"{label}: ordering_and_multiplicity.order invalid")
        if om.get("multiplicity") not in MULTIPLICITY:
            errors.append(f"{label}: ordering_and_multiplicity.multiplicity invalid")

    impl = case.get("implementations")
    if not isinstance(impl, dict):
        errors.append(f"{label}: implementations required")
    else:
        for eng in ("rql", "mongo", "cbl"):
            if eng not in impl or not isinstance(impl[eng], dict):
                errors.append(f"{label}: implementations.{eng} required")
                continue
            st = impl[eng].get("status")
            if eng == "rql" and st not in RQL_STATUS:
                errors.append(f"{label}: implementations.rql.status invalid")
            if eng == "mongo" and st not in MONGO_STATUS:
                errors.append(f"{label}: implementations.mongo.status invalid")
            if eng == "cbl" and st not in CBL_STATUS:
                errors.append(f"{label}: implementations.cbl.status invalid")
            if eng == "rql" and st == "source" and not impl[eng].get("source"):
                errors.append(f"{label}: rql status=source requires source text")
            if eng == "mongo" and st == "pipeline" and "pipeline" not in impl[eng]:
                errors.append(f"{label}: mongo status=pipeline requires pipeline")
            if eng == "mongo" and st == "find" and "find" not in impl[eng]:
                errors.append(f"{label}: mongo status=find requires find")
            if eng == "cbl" and st == "sqlpp" and not impl[eng].get("sqlpp"):
                errors.append(f"{label}: cbl status=sqlpp requires sqlpp")

    indexes = case.get("indexes")
    if not isinstance(indexes, dict) or "required" not in indexes or "optional" not in indexes:
        errors.append(f"{label}: indexes.required and indexes.optional required")
    elif not isinstance(indexes["required"], list) or not isinstance(indexes["optional"], list):
        errors.append(f"{label}: indexes.required/optional must be arrays")

    if case.get("selectivity_class") not in SELECTIVITY:
        errors.append(f"{label}: selectivity_class invalid")
    if case.get("cardinality_class") not in CARDINALITY:
        errors.append(f"{label}: cardinality_class invalid")

    variants = case.get("variants")
    if not isinstance(variants, dict):
        errors.append(f"{label}: variants required")
    else:
        for vk in ("missing_null_type", "cursor_page"):
            v = variants.get(vk)
            if not isinstance(v, dict) or "applies" not in v:
                errors.append(f"{label}: variants.{vk}.applies required")
            elif not isinstance(v["applies"], bool):
                errors.append(f"{label}: variants.{vk}.applies must be bool")

    excl = case.get("exclusion_or_refusal")
    if not isinstance(excl, dict) or excl.get("kind") not in EXCL_KINDS:
        errors.append(f"{label}: exclusion_or_refusal.kind invalid")
    elif excl["kind"] != "none" and not excl.get("code") and not excl.get("reason"):
        errors.append(f"{label}: exclusion_or_refusal needs code or reason when kind != none")

    return errors


def assert_case_ok(case: dict, label: str) -> None:
    errs = validate_case(case, label)
    for e in errs:
        fail(e)


def assert_case_fails(case: dict, label: str) -> None:
    errs = validate_case(case, label)
    if not errs:
        fail(f"{label}: expected incomplete case to fail validation")


# --- live corpus ---
doc = load(corpus_dir / "corpus-v1.json")
if doc.get("format") != FORMAT:
    fail(f"corpus format want {FORMAT}")
if doc.get("profile") != PROFILE:
    fail(f"corpus profile want {PROFILE}")
if doc.get("equivalence_profile") != EQ:
    fail(f"equivalence_profile want {EQ}")
cv = doc.get("corpus_version")
if not isinstance(cv, str) or not VERSION_RE.match(cv):
    fail(f"invalid corpus_version {cv!r}")

auth = doc.get("authority") or {}
if auth.get("package") != "RQL-Q1":
    fail("authority.package must be RQL-Q1")
if auth.get("section") != "4":
    fail("authority.section must be 4")

apol = doc.get("amendment_policy") or {}
if apol.get("requires_principal_review") is not True:
    fail("amendment_policy.requires_principal_review must be true")
if apol.get("process_doc") != "doc/todo/rql/RQL_Q1_CORPUS.md":
    fail("amendment_policy.process_doc path mismatch")
if not isinstance(apol.get("semver_rules"), str) or len(apol["semver_rules"]) < 8:
    fail("amendment_policy.semver_rules too short")

fp = doc.get("floor_policy") or {}
if fp.get("measure") != "count_cases_with_family_tag":
    fail("floor_policy.measure mismatch")
floors = fp.get("floors") or {}
for k, n in FLOOR_DEFAULTS.items():
    if floors.get(k) != n:
        fail(f"floor_policy.floors.{k} must be {n}, got {floors.get(k)!r}")
enforce = fp.get("enforce_floors")
if not isinstance(enforce, bool):
    fail("floor_policy.enforce_floors must be bool")

cases = doc.get("cases")
if not isinstance(cases, list):
    fail("cases must be an array")
else:
    seen = set()
    counts = {t: 0 for t in FAMILY_TAGS}
    for i, case in enumerate(cases):
        label = f"cases[{i}]"
        if not isinstance(case, dict):
            fail(f"{label}: not an object")
            continue
        assert_case_ok(case, label)
        cid = case.get("case_id")
        if cid in seen:
            fail(f"duplicate case_id {cid}")
        seen.add(cid)
        for t in case.get("family_tags") or []:
            if t in counts:
                counts[t] += 1

    print("verify-rql-q1-corpus: floor counts (live corpus):")
    for t in sorted(FLOOR_DEFAULTS):
        c = counts[t]
        need = FLOOR_DEFAULTS[t]
        flag = "OK" if c >= need else "below"
        print(f"  {t}: {c}/{need} ({flag})")

    if enforce:
        for t, need in FLOOR_DEFAULTS.items():
            if counts[t] < need:
                fail(f"floor not met for {t}: {counts[t]} < {need}")
        print("verify-rql-q1-corpus: enforce_floors=true (floors required)")
    else:
        print("verify-rql-q1-corpus: enforce_floors=false (scaffold/domain bulk OK)")

    # Comparator honesty: predeclared_native_diff / deliberate exclusion must not
    # present Mongo/CBL as competitive find/pipeline/sqlpp/query_builder forms.
    COMPETITIVE = {
        "mongo": {"find", "pipeline"},
        "cbl": {"sqlpp", "query_builder"},
    }
    tier_counts = {"A": 0, "B": 0, "C": 0}
    for i, case in enumerate(cases or []):
        if not isinstance(case, dict):
            continue
        tier = case.get("tier")
        if tier in tier_counts:
            tier_counts[tier] += 1
        excl = case.get("exclusion_or_refusal") or {}
        kind = excl.get("kind")
        if kind not in ("predeclared_native_diff", "deliberate_exclusion", "stable_refusal"):
            continue
        # stable_refusal on expected may still document competitor offset forms
        # for awareness — only enforce demotion for native_diff + deliberate_exclusion
        if kind not in ("predeclared_native_diff", "deliberate_exclusion"):
            continue
        # Tier B cases may keep competitor forms as "what they would do" while RQL excludes;
        # only require demotion when exclusion kind is predeclared_native_diff (no equivalence).
        if kind != "predeclared_native_diff":
            continue
        cid = case.get("case_id") or f"cases[{i}]"
        impl = case.get("implementations") or {}
        for eng, competitive in COMPETITIVE.items():
            st = (impl.get(eng) or {}).get("status")
            if st in competitive:
                fail(
                    f"{cid}: exclusion_or_refusal.kind=predeclared_native_diff but "
                    f"implementations.{eng}.status={st!r} is competitive; use "
                    f"lane_local_only / deliberate_exclusion / stable_refusal"
                )
    print(
        "verify-rql-q1-corpus: tier counts: "
        f"A={tier_counts['A']} B={tier_counts['B']} C={tier_counts['C']}"
    )

alog = doc.get("amendment_log")
if not isinstance(alog, list) or not alog:
    fail("amendment_log must be non-empty")
else:
    last = alog[-1]
    if last.get("corpus_version") != cv:
        fail("amendment_log last entry corpus_version must match document")

# --- fixtures ---
accepted = load(corpus_dir / "fixtures/case.accepted.min.json")
assert_case_ok(accepted, "fixtures/case.accepted.min.json")
rejected = load(corpus_dir / "fixtures/case.rejected.incomplete.json")
assert_case_fails(rejected, "fixtures/case.rejected.incomplete.json")

# sanity: accepted would fail if we strip fields
broken = dict(accepted)
del broken["implementations"]
assert_case_fails(broken, "synthetic-missing-implementations")

if err:
    sys.exit(1)
print("verify-rql-q1-corpus: OK")
sys.exit(0)
PY

if [[ "$ERR" -ne 0 ]]; then
  exit 1
fi
