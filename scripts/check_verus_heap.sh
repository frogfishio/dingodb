#!/usr/bin/env bash
# Gate H6 / CPR-004 — Verus pure-kernel proofs for heap isolation obligations.
#
# Always checks that the Verus source artifacts exist and that executable
# pure_proofs lemmas hold. When `verus` is available (local tools/verus or PATH),
# runs machine-checked verification of verification/heap-verus/verus/pure_kernel.rs.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() { echo "check_verus_heap: $*" >&2; exit 1; }

PURE_RS="$ROOT/verification/heap-verus/verus/pure_kernel.rs"
SCAFFOLD="$ROOT/verification/heap-verus/src/lib.rs"
[[ -f "$PURE_RS" ]] || fail "missing $PURE_RS"
[[ -f "$SCAFFOLD" ]] || fail "missing $SCAFFOLD"

# Harness / lemma names must stay in the Verus file.
for h in \
  authority_binding_holds \
  generation_accepted \
  certificate_blacklisted \
  authority_admission_ok \
  lemma_binding_rejects_foreign_heap \
  lemma_generation_grace_window \
  lemma_blacklist_hits \
  lemma_non_serving_refuses_admission \
  lemma_isolation_foreign_unit \
  lemma_connected_pure_proof_bundle
do
  rg -n "$h" "$PURE_RS" >/dev/null || fail "missing Verus symbol $h in pure_kernel.rs"
done

# Executable stand-ins always green.
cargo test -p dingo-heap pure_proof --quiet \
  || fail "dingo-heap pure_proof tests failed"

find_verus() {
  if [[ -n "${DINGO_VERUS_BIN:-}" && -x "${DINGO_VERUS_BIN}" ]]; then
    echo "$DINGO_VERUS_BIN"
    return 0
  fi
  if [[ -x "$ROOT/tools/verus/verus" ]]; then
    echo "$ROOT/tools/verus/verus"
    return 0
  fi
  if command -v verus >/dev/null 2>&1; then
    command -v verus
    return 0
  fi
  return 1
}

if VERUS_BIN=$(find_verus); then
  echo "check_verus_heap: verifying with $VERUS_BIN"
  # Clear macOS quarantine noise if present (local download).
  if [[ "$(uname -s)" == "Darwin" ]]; then
    xattr -dr com.apple.quarantine "$(dirname "$VERUS_BIN")" 2>/dev/null || true
  fi
  out=$("$VERUS_BIN" "$PURE_RS" 2>&1) || {
    echo "$out" >&2
    fail "verus verification failed"
  }
  echo "$out" | tail -5
  echo "$out" | rg -q 'verified' || fail "verus output missing verification summary"
  # Honesty flag must match successful machine check.
  rg -n 'VERUS_PROOFS_CONNECTED: bool = true' "$SCAFFOLD" >/dev/null \
    || fail "VERUS_PROOFS_CONNECTED must be true when verus proofs are connected"
  echo "check_verus_heap: machine-checked pure_kernel OK"
else
  if [[ "${DINGO_REQUIRE_VERUS:-}" == "1" ]]; then
    fail "verus required (DINGO_REQUIRE_VERUS=1) but not found (set DINGO_VERUS_BIN or tools/verus/verus)"
  fi
  echo "check_verus_heap: verus not installed — pure_kernel source + executable lemmas OK"
  echo "  install: see scripts/setup_verus.sh ; CI job verus-heap sets DINGO_REQUIRE_VERUS=1"
fi

echo "check_verus_heap: OK"
