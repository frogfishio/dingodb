#!/usr/bin/env bash
# AWO-0: validate Adaptive Write Optimiser closed contracts and golden maths.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AWO="$ROOT/spec/performance/awo"

python3 - "$AWO" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
profile_id = "residiuum-adaptive-write-v1"
errors = []

def fail(message):
    errors.append(message)

def load(name):
    path = root / name
    if not path.is_file():
        fail(f"missing {name}")
        return {}
    try:
        with path.open() as handle:
            value = json.load(handle)
    except Exception as exc:
        fail(f"invalid JSON {name}: {exc}")
        return {}
    if name != "README.md" and value.get("profile") != profile_id:
        fail(f"{name}: wrong profile {value.get('profile')!r}")
    return value

required = [
    "profile-v1.json",
    "states-v1.json",
    "transitions-v1.json",
    "decision-reasons-v1.json",
    "outcomes-v1.json",
    "policy-v1.json",
    "golden-decisions-v1.json",
    "schemas/golden-decisions-v1.schema.json",
    "README.md",
]
for name in required:
    if not (root / name).is_file():
        fail(f"missing {name}")

profile = load("profile-v1.json")
states_doc = load("states-v1.json")
transitions_doc = load("transitions-v1.json")
reasons_doc = load("decision-reasons-v1.json")
outcomes_doc = load("outcomes-v1.json")
policy_doc = load("policy-v1.json")
golden_doc = load("golden-decisions-v1.json")
try:
    with (root / "schemas/golden-decisions-v1.schema.json").open() as handle:
        schema_doc = json.load(handle)
    if schema_doc.get("title") != "residiuum-awo-golden-decisions-v1":
        fail("golden decision schema has wrong title")
except Exception as exc:
    fail(f"invalid golden decision schema: {exc}")

def closed_items(doc, name):
    items = doc.get("items")
    if not isinstance(items, list) or not items:
        fail(f"{name}: items must be non-empty")
        return []
    seen = set()
    for item in items:
        item_id = item.get("id")
        if not item_id:
            fail(f"{name}: item missing id")
        elif item_id in seen:
            fail(f"{name}: duplicate id {item_id}")
        seen.add(item_id)
    return items

states = closed_items(states_doc, "states")
transitions = closed_items(transitions_doc, "transitions")
reasons = closed_items(reasons_doc, "decision reasons")
outcomes = closed_items(outcomes_doc, "outcomes")
state_ids = {item.get("id") for item in states}
reason_map = {item.get("id"): item.get("plan") for item in reasons}

for transition in transitions:
    if transition.get("from") not in state_ids:
        fail(f"transition {transition.get('id')}: unknown from state")
    if transition.get("to") not in state_ids:
        fail(f"transition {transition.get('id')}: unknown to state")

outgoing = {transition.get("from") for transition in transitions}
for state in states:
    if state.get("authority") not in {"no", "yes", "unknown"}:
        fail(f"state {state.get('id')}: invalid authority classification")
    if state.get("terminal") is True and state.get("id") in outgoing:
        fail(f"terminal state {state.get('id')} has an outgoing transition")

for required_state in [
    "received", "queued", "cooking", "ready", "persisting", "persisted",
    "published", "acknowledged", "rejected", "failed",
    "uncertain_pending_recovery",
]:
    if required_state not in state_ids:
        fail(f"missing required state {required_state}")

for required_reason in [
    "natural_single_request", "natural_no_positive_gain",
    "natural_insufficient_evidence", "natural_stale_model",
    "natural_deadline", "natural_incompatible", "natural_memory_bound",
    "natural_deadline_mitigation", "natural_tie", "batch_existing_backlog",
    "batch_deadline_mitigation",
]:
    if required_reason not in reason_map:
        fail(f"missing required decision reason {required_reason}")

if set(profile.get("modes", [])) != {"disabled", "static", "adaptive"}:
    fail("profile modes are not closed to disabled/static/adaptive")
for field, expected in {
    "states_registry": "states-v1.json",
    "transitions_registry": "transitions-v1.json",
    "decision_reasons_registry": "decision-reasons-v1.json",
    "outcomes_registry": "outcomes-v1.json",
    "policy_registry": "policy-v1.json",
    "golden_decisions": "golden-decisions-v1.json",
}.items():
    if profile.get(field) != expected:
        fail(f"profile {field} must reference {expected}")

defaults = policy_doc.get("defaults", {})
validation = policy_doc.get("validation", {})
numeric_positive = [
    "queue_entry_limit", "queue_byte_limit", "maximum_batch_entries",
    "maximum_batch_bytes", "maximum_collection_delay_ns",
    "default_completion_deadline_ns", "minimum_active_cookers",
    "maximum_cookers_cap", "pipeline_depth_limit", "decision_margin_ppm",
    "estimator_alpha_denominator", "estimator_deviation_multiplier",
    "estimator_min_samples", "estimator_stale_after_ns",
    "controller_interval_ns", "scale_up_consecutive_intervals",
    "scale_down_consecutive_intervals", "scale_up_dwell_ns",
    "scale_down_dwell_ns",
]
for key in numeric_positive:
    value = defaults.get(key)
    if not isinstance(value, int) or value <= 0:
        fail(f"policy default {key} must be a positive integer")

if defaults.get("queue_entry_limit", 0) < defaults.get("maximum_batch_entries", 0):
    fail("queue_entry_limit must hold a maximum batch")
if defaults.get("queue_byte_limit", 0) < defaults.get("maximum_batch_bytes", 0):
    fail("queue_byte_limit must hold a maximum batch")
if defaults.get("default_completion_deadline_ns", 0) < defaults.get("maximum_collection_delay_ns", 0):
    fail("completion deadline must not precede collection cap")
if not (validation.get("pipeline_depth_min", 0) <= defaults.get("pipeline_depth_limit", 0) <= validation.get("pipeline_depth_max", -1)):
    fail("pipeline depth default outside validation range")
if defaults.get("maximum_collection_delay_ns", 0) > validation.get("maximum_collection_delay_hard_max_ns", -1):
    fail("collection default exceeds hard maximum")

def decide(vector):
    q = vector["queue_count"]
    if q == 1:
        return "natural", "natural_single_request"
    if not vector["compatible"]:
        return "natural", "natural_incompatible"
    if not vector["memory_ok"]:
        return "natural", "natural_memory_bound"
    if not vector["evidence_warm"]:
        return "natural", "natural_insufficient_evidence"
    if vector["evidence_stale"]:
        return "natural", "natural_stale_model"
    service = vector["natural_service_lower_ns"]
    mean_natural = service * (q + 1) // 2
    tail_natural = service * q
    batch = vector["batch_completion_upper_ns"]
    deadline = vector["earliest_deadline_ns"]
    if batch > deadline:
        if tail_natural <= deadline:
            return "natural", "natural_deadline"
        if batch < tail_natural:
            return "batch", "batch_deadline_mitigation"
        return "natural", "natural_deadline_mitigation"

    j_natural = mean_natural + tail_natural
    j_batch = batch * 2
    lhs = j_batch * 1_000_000
    rhs = j_natural * (1_000_000 - vector["decision_margin_ppm"])
    if lhs < rhs:
        return "batch", "batch_existing_backlog"
    if lhs == rhs:
        return "natural", "natural_tie"
    return "natural", "natural_no_positive_gain"

vectors = closed_items(golden_doc, "golden decisions")
if len(vectors) < 8:
    fail("golden decisions require at least 8 vectors")
for vector in vectors:
    required_fields = [
        "queue_count", "natural_service_lower_ns", "batch_completion_upper_ns",
        "decision_margin_ppm", "evidence_warm", "evidence_stale",
        "compatible", "memory_ok", "earliest_deadline_ns", "expected_plan",
        "expected_reason",
    ]
    missing = [field for field in required_fields if field not in vector]
    if missing:
        fail(f"vector {vector.get('id')}: missing {missing}")
        continue
    plan, reason = decide(vector)
    if (plan, reason) != (vector["expected_plan"], vector["expected_reason"]):
        fail(
            f"vector {vector.get('id')}: expected "
            f"{vector['expected_plan']}/{vector['expected_reason']}, got {plan}/{reason}"
        )
    if reason_map.get(vector["expected_reason"]) != vector["expected_plan"]:
        fail(f"vector {vector.get('id')}: reason/plan registry mismatch")

if not outcomes:
    fail("outcome registry empty")

if errors:
    for error in errors:
        print(f"FAIL: {error}", file=sys.stderr)
    raise SystemExit(1)

print(
    f"AWO contract OK: {len(states)} states, {len(transitions)} transitions, "
    f"{len(reasons)} reasons, {len(outcomes)} outcomes, {len(vectors)} golden vectors"
)
PY
