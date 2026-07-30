#!/usr/bin/env bash
# DEF-090 — local mirror of the required CI quality bar (.github/workflows/ci.yml).
# Run from the repository root: ./scripts/quality.sh
#
# Optional env:
#   DINGO_QUALITY_SKIP_DENY=1   skip cargo-deny when the binary is not installed
#   DINGO_QUALITY_SKIP_DOC=1    skip cargo doc
#   DINGO_RELEASE_ALLOW_DIRTY=1 pass through to release_content.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export RUSTFLAGS="${RUSTFLAGS:--D warnings}"
export RUSTDOCFLAGS="${RUSTDOCFLAGS:--D warnings}"

echo "== delivery scoreboard (M0-3) =="
bash ./scripts/verify-delivery-status.sh

echo "== APP-0 application contract lock =="
bash ./scripts/verify-app0-contract.sh

echo "== crash-and-recovery contract (DEF-104) =="
bash ./scripts/verify-crash-recovery-contract.sh

echo "== fuzz property bar (DEF-091-F, no cargo-fuzz required) =="
DINGO_FUZZ_SKIP_CARGO_FUZZ=1 bash ./scripts/fuzz-smoke.sh

echo "== fmt =="
cargo fmt --all -- --check

echo "== clippy (strict) =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== build all targets =="
cargo build --workspace --all-targets

echo "== test =="
cargo test --workspace

if [[ "${DINGO_QUALITY_SKIP_DOC:-0}" != "1" ]]; then
  echo "== doc =="
  cargo doc --workspace --no-deps --document-private-items
fi

echo "== release content (DEF-003) =="
./scripts/release_content.sh

if [[ "${DINGO_QUALITY_SKIP_DENY:-0}" == "1" ]]; then
  echo "== cargo-deny skipped (DINGO_QUALITY_SKIP_DENY=1) =="
elif command -v cargo-deny >/dev/null 2>&1; then
  echo "== cargo-deny =="
  cargo deny check --all-features
else
  echo "warning: cargo-deny not installed; install with:" >&2
  echo "  cargo install --locked cargo-deny" >&2
  echo "or set DINGO_QUALITY_SKIP_DENY=1 for a local dry-run." >&2
  exit 1
fi

echo "== DEF-091 property tests (dingo-format) =="
cargo test -p dingo-format --test stage_def_091_properties

echo "quality bar OK (DEF-090); DEF-091 properties exercised"