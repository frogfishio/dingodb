#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 - "$ROOT" <<'PY'
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
contract = json.loads((root / "spec/app/driver-v1/contract-v1.json").read_text())
inventory = json.loads((root / "spec/app/driver-v1/current-runtime-inventory-v1.json").read_text())
status = json.loads((root / "spec/app/driver-v1/implementation-status-v1.json").read_text())

assert contract["format"] == "residiuum-async-driver-contract-v1"
assert len(contract["terminal_outcomes"]) == 6
assert len(contract["terminal_outcomes"]) == len(set(contract["terminal_outcomes"]))
assert contract["defaults"]["remote"]["max_connections"] == 10
assert contract["defaults"]["remote"]["max_waiters"] == 1024
assert contract["defaults"]["embedded"]["queue"] == 1024
assert contract["defaults"]["cursor_prefetch_pages"] == 1
assert set(contract["required_wire_features"]) == {
    "request-deadline-v1",
    "cancel-request-v1",
    "operation-outcome-v1",
    "complete-receipts-v2",
}

assert inventory["format"] == "residiuum-async-driver-current-runtime-inventory-v1"
assert inventory["product_ready"] is False
assert inventory["open_residuals"]
assert status["format"] == "residiuum-async-driver-implementation-status-v1"
assert status["claims"]["bounded_embedded_scheduler"] is True
assert status["claims"]["remote_pool"] is False
assert status["claims"]["streamed_rql"] is False
assert status["residuals"]

candidate_paths = [
    root / "crates/residiuum-client/src/driver_contract.rs",
    root / "crates/residiuum-sdk/src/driver.rs",
]
for path in candidate_paths:
    if not path.exists():
        continue
    source = path.read_text()
    if "Arc<Mutex<RemoteHeap>>" in source:
        raise SystemExit(f"forbidden legacy session mutex in new driver path: {path}")

print("verify-driver-drv0: registries and architecture guard PASS")
PY

cargo test -p residiuum-client --lib
cargo test -p residiuum-sdk --lib driver::tests
cargo test -p residiuum-sdk --test driver_embedded
printf 'verify-driver-drv0: PASS (embedded candidate; remote/query residuals explicit)\n'
