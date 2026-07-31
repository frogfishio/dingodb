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
for required in (1, 2, 3):
    if required not in active:
        sys.exit(f"active set must include process op {required}, got {active}")
if active != sorted(active) or len(active) != len(set(active)):
    sys.exit(f"active set must be sorted unique, got {active}")
# §32.4 data cuts: open/list + get/put/delete + list_keys/scan/find/history + indexes.
for required in (105, 110, 111, 112, 114, 115, 116, 117, 120, 121, 122, 130, 131, 132, 133):
    if required not in active:
        sys.exit(f"§32.4 data cut requires active op {required}, got {active}")
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

# No capability Serialize/Deserialize outside residiuum-heap (quick grep).
if rg -n 'impl\s+(Serialize|Deserialize).*HeapCap|HeapCap\s*\{' crates --glob '!residiuum-heap/**' 2>/dev/null | rg 'Serialize|Deserialize'; then
  fail "HeapCap must not implement Serialize/Deserialize outside kernel"
fi

# Qualified upper layers must not import kernel::PhysicalStore.
if rg -n 'kernel::PhysicalStore|use crate::kernel' crates/residiuum-sdk crates/residiuum-server crates/residiuum-client crates/residiuum-cli 2>/dev/null; then
  fail "upper layers must not import PhysicalStore/kernel"
fi

# residiuum-heap must not depend on store/sdk/server.
if rg -n 'residiuum-store|residiuum-sdk|residiuum-server|residiuum-cluster' crates/residiuum-heap/Cargo.toml; then
  fail "residiuum-heap dependency firewall violated"
fi

# HP-003 / A3: legacy Store export is feature-gated; package default is façades-only.
if ! rg -n 'cfg\(feature = "legacy-raw-store"\)' crates/residiuum-store/src/lib.rs >/dev/null; then
  fail "Store must be gated behind legacy-raw-store feature"
fi
if ! rg -n 'legacy-raw-store' crates/residiuum-store/Cargo.toml >/dev/null; then
  fail "residiuum-store must declare legacy-raw-store feature"
fi
if ! rg -n 'default = \[\]' crates/residiuum-store/Cargo.toml >/dev/null; then
  fail "residiuum-store default features must be empty (A3 façades-only default)"
fi

# Qualified data service must never depend on residiuum-authority or enable
# authority-provisioning (HP-005 firewall).
if rg -n 'residiuum-authority|authority-provisioning' crates/residiuum-server/Cargo.toml; then
  fail "residiuum-server must not depend on residiuum-authority or authority-provisioning"
fi
if rg -n 'residiuum-authority' crates/residiuum-sdk/Cargo.toml crates/residiuum-client/Cargo.toml 2>/dev/null; then
  fail "sdk/client must not depend on residiuum-authority"
fi
# residiuum-authority must exist and be AGPL.
if [[ ! -f crates/residiuum-authority/Cargo.toml ]]; then
  fail "residiuum-authority crate missing (HP-005)"
fi
if ! rg -n 'AGPL-3.0-or-later' crates/residiuum-authority/Cargo.toml >/dev/null; then
  fail "residiuum-authority must be AGPL-3.0-or-later"
fi

# Qualified feature surface builds without public raw Store.
cargo check -p residiuum-store --quiet \
  || fail "residiuum-store default (façades-only) must build"
cargo check -p residiuum-store --features legacy-raw-store --quiet \
  || fail "residiuum-store --features legacy-raw-store must build"
# CPR-001: package default is heap-only; legacy flat is opt-in.
if ! rg -n 'legacy-flat-sdk' crates/residiuum-sdk/Cargo.toml >/dev/null; then
  fail "residiuum-sdk must declare legacy-flat-sdk feature (CPR-001)"
fi
if ! rg -n 'default = \[\]' crates/residiuum-sdk/Cargo.toml >/dev/null; then
  fail "residiuum-sdk default features must be empty (CPR-001 heap-only default)"
fi
if ! rg -n 'legacy_flat_sdk_enabled|FLAT_COLLECTION_SURFACE_LABEL' crates/residiuum-sdk/src/claim.rs >/dev/null; then
  fail "residiuum-sdk claim honesty surface missing (CPR-001)"
fi
cargo check -p residiuum-sdk --quiet \
  || fail "residiuum-sdk default (heap-only) must build"
cargo check -p residiuum-sdk --features legacy-flat-sdk --quiet \
  || fail "residiuum-sdk --features legacy-flat-sdk must build"
# Data-service check still builds without linking authority.
cargo check -p residiuum-server --quiet \
  || fail "residiuum-server must build without residiuum-authority"

# HP-008: qualified hot path must not reference the authority store.
for f in heap_auth.rs heap_dispatch.rs heap_registry.rs heap_session.rs heap_audit.rs; do
  p="crates/residiuum-server/src/$f"
  [[ -f "$p" ]] || fail "missing HP-008 module $p"
  if rg -n 'residiuum_authority|MasterAuthorityStore|authority-provisioning' "$p"; then
    fail "$f must not touch authority store (HP-008 hot path)"
  fi
done

echo "check_heap_architecture: OK"