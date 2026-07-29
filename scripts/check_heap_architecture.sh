#!/usr/bin/env bash
# HP-000 / HP-003 architecture and contract drift checks.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_heap_architecture: $*" >&2; exit 1; }

OPS="$ROOT/spec/heap/operations-v1.json"
CBOR="$ROOT/spec/heap/cbor-v1.json"
VEC="$ROOT/spec/heap/vectors-v1.json"
[[ -f "$OPS" ]] || fail "missing $OPS"
[[ -f "$CBOR" ]] || fail "missing $CBOR"
[[ -f "$VEC" ]] || fail "missing $VEC"

python3 - <<'PY'
import json, sys
from pathlib import Path
root = Path("spec/heap")
ops = json.loads((root/"operations-v1.json").read_text())
ids = [o["id"] for o in ops["operations"]]
if len(ids) != len(set(ids)):
    sys.exit("duplicate operation ids")
names = [o["wire_name"] for o in ops["operations"]]
if len(names) != len(set(names)):
    sys.exit("duplicate wire names")
active = [o["id"] for o in ops["operations"] if o["status"]=="active"]
if active != [1,2,3]:
    sys.exit(f"HP-000 active set must be [1,2,3], got {active}")
for o in ops["operations"]:
    if o["status"]=="active":
        for key in ("request_schema","response_schema"):
            p = root/o[key]
            if not p.is_file():
                sys.exit(f"missing schema {p}")
        name = o["wire_name"]
        for kind in ("accepted","rejected"):
            p = root/"fixtures"/f"{name}.{kind}.json"
            if not p.is_file():
                sys.exit(f"missing fixture {p}")
    else:
        if o.get("request_schema") is not None or o.get("response_schema") is not None:
            sys.exit(f"reserved op {o['id']} must not have schemas yet")
cbor = json.loads((root/"cbor-v1.json").read_text())
for k in ("31","32","33","34","35","36"):
    if k not in cbor["envelope_keys"]:
        sys.exit(f"missing envelope key {k}")
vec = json.loads((root/"vectors-v1.json").read_text())
if len(vec.get("accepted",[])) < 2:
    sys.exit("vectors need bootstrap cert+proof")
if len(vec.get("rejected",[])) < 1:
    sys.exit("vectors need negative corpus")
fv = vec.get("format_vectors")
if not fv or len(fv.get("accepted",[])) < 5:
    sys.exit("vectors need format_vectors (SubjectV2/descriptors/admit)")
if len(fv.get("rejected",[])) < 3:
    sys.exit("format_vectors need reject corpus")
status = vec.get("corpus_status") or {}
if "format_subject_descriptor_admit" not in status:
    sys.exit("corpus_status must record format vector status")
print("spec/heap contract OK")
PY

# No capability Serialize/Deserialize outside dingo-heap (quick grep).
if rg -n 'impl\s+(Serialize|Deserialize).*HeapCap|HeapCap\s*\{' crates --glob '!dingo-heap/**' 2>/dev/null | rg 'Serialize|Deserialize'; then
  fail "HeapCap must not implement Serialize/Deserialize outside kernel"
fi

# Qualified upper layers must not import kernel::PhysicalStore.
if rg -n 'kernel::PhysicalStore|use crate::kernel' crates/dingo-sdk crates/dingo-server crates/dingo-client crates/dingo-cli 2>/dev/null; then
  fail "upper layers must not import PhysicalStore/kernel"
fi

# dingo-heap must not depend on store/sdk/server.
if rg -n 'dingo-store|dingo-sdk|dingo-server|dingo-cluster' crates/dingo-heap/Cargo.toml; then
  fail "dingo-heap dependency firewall violated"
fi

# HP-003: legacy Store export is feature-gated.
if ! rg -n 'cfg\(feature = "legacy-raw-store"\)' crates/dingo-store/src/lib.rs >/dev/null; then
  fail "Store must be gated behind legacy-raw-store feature"
fi
if ! rg -n 'legacy-raw-store' crates/dingo-store/Cargo.toml >/dev/null; then
  fail "dingo-store must declare legacy-raw-store feature"
fi

# Qualified feature surface builds without public raw Store.
cargo check -p dingo-store --no-default-features --quiet \
  || fail "dingo-store --no-default-features must build"

echo "check_heap_architecture: OK"
