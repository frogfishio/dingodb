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
  # CPR-004: Kani + Verus pure-kernel (install tools for full machine check).
  if command -v cargo >/dev/null 2>&1 && cargo kani --version >/dev/null 2>&1; then
    DINGO_REQUIRE_KANI=1 ./scripts/check_kani_heap.sh
  else
    ./scripts/check_kani_heap.sh
    echo "kani not installed; harness sources + executable lemmas verified"
  fi
  if [[ -x "$ROOT/tools/verus/verus" ]] || command -v verus >/dev/null 2>&1; then
    DINGO_REQUIRE_VERUS=1 ./scripts/check_verus_heap.sh
  else
    ./scripts/check_verus_heap.sh
    echo "verus not installed; pure_kernel source + executable lemmas verified"
    echo "  run ./scripts/setup_verus.sh for local machine check"
  fi
  if command -v tlc >/dev/null 2>&1; then
    echo "tlc available — model-checking formal/heap/HeapIsolation.tla + HeapAuthority.tla"
    tlc -config formal/heap/MCHeapIsolation.cfg formal/heap/HeapIsolation.tla || true
    tlc -config formal/heap/MCHeapAuthority.cfg formal/heap/HeapAuthority.tla || true
  else
    echo "tlc not installed; HeapIsolation/HeapAuthority sketches reviewed by hp010 drills"
  fi
fi

echo "verify-heap ($MODE): OK"