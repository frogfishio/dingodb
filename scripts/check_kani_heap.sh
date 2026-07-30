#!/usr/bin/env bash
# Gate H6 / CPR-004 — Kani harness connection check for pure heap lemmas.
#
# Always runs the executable pure_proofs bundle via cargo test.
# When `cargo kani` is available, also runs the Kani harnesses.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_kani_heap: $*" >&2; exit 1; }

# Honesty: harness symbols exist in pure_proofs source.
PURE="$ROOT/crates/dingo-heap/src/pure_proofs.rs"
[[ -f "$PURE" ]] || fail "missing $PURE"
for h in \
  kani_binding_rejects_foreign_heap \
  kani_generation_grace_window \
  kani_blacklist_hits_certificate_hash \
  kani_non_serving_refuses_admission \
  kani_isolation_model_inv_walk \
  kani_authority_model_inv_walk \
  kani_connected_pure_proof_bundle
do
  rg -n "fn $h" "$PURE" >/dev/null || fail "missing Kani harness $h in pure_proofs.rs"
done

# Scaffold must advertise Kani + Verus connected (both pure-kernel paths landed).
VERUS="$ROOT/verification/heap-verus/src/lib.rs"
rg -n 'KANI_HARNESSES_CONNECTED: bool = true' "$VERUS" >/dev/null \
  || fail "KANI_HARNESSES_CONNECTED must be true"
rg -n 'VERUS_PROOFS_CONNECTED: bool = true' "$VERUS" >/dev/null \
  || fail "VERUS_PROOFS_CONNECTED must be true (pure_kernel connected; see check_verus_heap.sh)"

# Executable lemmas always green in CI (no Kani required for this step).
cargo test -p dingo-heap pure_proof --quiet \
  || fail "dingo-heap pure_proof tests failed"
cargo test -p dingo-store --test hp010_qualification h6_pure_proof_bundle --quiet \
  || fail "hp010 pure proof bundle Accept failed"

if command -v cargo >/dev/null 2>&1 && cargo kani --version >/dev/null 2>&1; then
  echo "check_kani_heap: running cargo kani on dingo-heap pure harnesses"
  # Bounded unwind; lemmas are concrete (no symbolic input).
  cargo kani -p dingo-heap \
    --harness kani_connected_pure_proof_bundle \
    --default-unwind 16 \
    || fail "cargo kani pure proof bundle failed"
else
  if [[ "${DINGO_REQUIRE_KANI:-}" == "1" ]]; then
    fail "cargo kani required (DINGO_REQUIRE_KANI=1) but not installed"
  fi
  echo "check_kani_heap: cargo kani not installed — harness sources + executable lemmas OK"
  echo "  (CI kani-heap job installs kani-verifier and sets DINGO_REQUIRE_KANI=1)"
fi

echo "check_kani_heap: OK"