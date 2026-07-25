#!/usr/bin/env bash
# Local mirror of .github/workflows/nightly.yml (Stage 7f packaging).
# Run from the repository root: ./scripts/nightly.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export RUSTFLAGS="${RUSTFLAGS:--D warnings}"

echo "== fmt =="
cargo fmt --all -- --check

echo "== workspace tests =="
cargo test --workspace

echo "== FORMAT_SPEC §13 corpus =="
cargo test -p dingo-format --test section13_corpus -- --nocapture

echo "== OVERVIEW §16 store suite =="
cargo test -p dingo-store --test section16_store -- --nocapture

echo "== Stage 6 store suite =="
cargo test -p dingo-store --test stage6_store -- --nocapture

echo "== Stage 6 bench skeleton =="
cargo test -p dingo-store --test stage6_bench_skeleton -- --nocapture

echo "== DEF-022 full crash matrix =="
DINGO_CRASH_MATRIX_FULL=1 cargo test -p dingo-store --test stage_def_022_crash_matrix -- --nocapture --test-threads=1

echo "== Stage 7 CLI =="
cargo test -p dingo-cli --test cli -- --nocapture

echo "nightly packaging OK"
