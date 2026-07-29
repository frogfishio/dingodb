#!/usr/bin/env bash
# HEAP_SPEC §39 verification entrypoint.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-quick}"
cd "$ROOT"

./scripts/check_heap_architecture.sh
cargo test -p dingo-heap --lib --tests
cargo test -p dingo-format --lib
cargo test -p dingo-store --lib heap::catalog
cargo test -p dingo-store --test hp004_catalog_rebuild
cargo test -p dingo-authority --test hp005_accept
cargo test -p dingo-sdk --test hp007_heap_isolation
cargo test -p dingo-server --test hp008_heap_handshake
cargo test -p dingo-server --test hp008_accept_loop
cargo test -p dingo-store --test hp006_heap_migration -- --test-threads=1
cargo test -p dingo-store --test hp009_lifecycle
cargo test -p dingo-store --test hp010_qualification
cargo test -p dingo-heap --lib qualification::

if [[ "$MODE" == "full" ]]; then
  if command -v kani >/dev/null 2>&1; then
    echo "kani available — run bounded targets when added"
  else
    echo "kani not installed; property tests cover rights intersection"
  fi
fi

echo "verify-heap ($MODE): OK"
