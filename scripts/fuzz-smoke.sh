#!/usr/bin/env bash
# DEF-091-F — continuous/scheduled fuzz policy entrypoint.
#
# Always: PR-safe property/hostile decode tests (stable toolchain).
# When cargo-fuzz + nightly available: short smoke per registered target.
#
# Env:
#   DINGO_FUZZ_SECONDS   seconds per target (default 30; use 5 for quick local)
#   DINGO_FUZZ_SKIP_CARGO_FUZZ=1  property tests only
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SECONDS_PER="${DINGO_FUZZ_SECONDS:-30}"

ok() { echo "fuzz-smoke: $*"; }
fail() { echo "fuzz-smoke: FAIL: $*" >&2; exit 1; }

# --- Property / hostile decode bar (always, PR-safe) ---
ok "dingo-format property suite (DEF-091)"
cargo test -p dingo-format --test stage_def_091_properties --quiet

ok "hostile chunk-manifest length refuses pre-check (DEF-091-F)"
cargo test -p dingo-store --features legacy-raw-store --lib chunk_payload --quiet

ok "SDA no-panic property suite (DEF-091-F)"
cargo test -p dingo-sda --test stage_def_091f_sda_properties --quiet

ok "RPC frame refuse-before-alloc properties (DEF-091-F)"
cargo test -p dingo-client --lib protocol --quiet

# --- cargo-fuzz smoke (scheduled / local with nightly) ---
TARGETS=(
  decode_frame
  cbor_envelope
  scan_forward
  scan_reverse
  heap_ownership
  sda_parse
  rpc_frame
  chunk_manifest
  item_envelope
  backup_manifest
  cursor_token
)

if [[ "${DINGO_FUZZ_SKIP_CARGO_FUZZ:-0}" == "1" ]]; then
  ok "skipping cargo-fuzz (DINGO_FUZZ_SKIP_CARGO_FUZZ=1)"
  ok "OK (property bar only)"
  exit 0
fi

if ! command -v cargo-fuzz >/dev/null 2>&1 && ! cargo fuzz --help >/dev/null 2>&1; then
  ok "cargo-fuzz not installed — property bar still green"
  ok "install: cargo install cargo-fuzz  (needs nightly for run)"
  ok "OK (property bar only)"
  exit 0
fi

if ! rustup run nightly rustc --version >/dev/null 2>&1; then
  ok "nightly toolchain missing — property bar still green"
  ok "OK (property bar only)"
  exit 0
fi

ok "cargo-fuzz smoke (${SECONDS_PER}s × ${#TARGETS[@]} targets)"
cd "$ROOT/fuzz"
for t in "${TARGETS[@]}"; do
  echo "== fuzz $t =="
  cargo +nightly fuzz run "$t" -- -max_total_time="$SECONDS_PER" -timeout=5
done
cd "$ROOT"

ok "OK"
