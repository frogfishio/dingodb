#!/usr/bin/env bash
# Local mirror of .github/workflows/nightly.yml (Stage 7f packaging).
# Run from the repository root: ./scripts/nightly.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export RUSTFLAGS="${RUSTFLAGS:--D warnings}"

echo "== fmt =="
cargo fmt --all -- --check

echo "== clippy (strict) =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== workspace tests =="
cargo test --workspace

echo "== FORMAT_SPEC §13 corpus =="
cargo test -p residuum-format --test section13_corpus -- --nocapture

echo "== OVERVIEW §16 store suite =="
cargo test -p residuum-store --test section16_store -- --nocapture

echo "== Stage 6 store suite =="
cargo test -p residuum-store --test stage6_store -- --nocapture

echo "== Stage 6 bench skeleton =="
cargo test -p residuum-store --test stage6_bench_skeleton -- --nocapture

echo "== DEF-022 full crash matrix =="
RESIDUUM_CRASH_MATRIX_FULL=1 cargo test -p residuum-store --test stage_def_022_crash_matrix -- --nocapture --test-threads=1

echo "== Stage 7 CLI =="
cargo test -p residuum-cli --test cli -- --nocapture

echo "== DEF-091-F fuzz smoke (property bar + optional cargo-fuzz) =="
# Property tests always; cargo-fuzz when nightly+cargo-fuzz installed.
# CI nightly workflow runs the full 30s×N cargo-fuzz list.
RESIDUUM_FUZZ_SECONDS="${RESIDUUM_FUZZ_SECONDS:-5}" bash ./scripts/fuzz-smoke.sh

echo "== DEF-041-N multiproc OS chaos (short soak) =="
cargo test -p residuum-cluster --test stage_def_041n_multiproc -- --nocapture

echo "nightly packaging OK"