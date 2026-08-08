#!/usr/bin/env bash
# Explicit publish: rewrite checked-in Q3 evidence snapshots under spec/.
# Default verify/tests do NOT do this (F8).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export RESIDIUUM_WRITE_SPEC_EVIDENCE=1
printf 'publish-rql-q3-evidence: writing target/ + spec/ (RESIDIUUM_WRITE_SPEC_EVIDENCE=1)\n'
cargo test -p residiuum-sdk --test rql_q3_semantic_oracle rql_q3_1_corpus_oracle_suite
cargo test -p residiuum-sdk --test rql_q3_differential_matrix rql_q3_2_corpus_differential_matrix
cargo test -p residiuum-sdk --test rql_q3_adversarial q33_write_adversarial_report
cargo test -p residiuum-sdk --test rql_q3_page_concat q34_write_report
printf 'publish-rql-q3-evidence: done — review git diff under spec/rql/qualification/corpus-v1/\n'
