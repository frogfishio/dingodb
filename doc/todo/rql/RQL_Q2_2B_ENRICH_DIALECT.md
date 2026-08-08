# RQL-Q2.2b — `pkg_enrich_corpus_dialect`

Status: **labor complete** (2026-08-08)  
Authority: Q2.1 audit rank 2; RQL_SPEC enrich grammar

## Delivered

1. **Corpus v0.4.1** — 15 enrich RQL sources rewritten to product
   `enrich … using … matching … expect …` form.
2. **Compiler** — still accepts corpus `from/on/card` dialect (normalize paths).
3. **Match semantics** — `_key` / `$key` resolve to the store document key
   (fixtures strip body `_key` after put).
4. **Audit harness** — companion generators seed missing collections named in
   the source (same seed/params).
5. **Re-audit** — execute_ok **122**/147 (was 107); `pkg_enrich_corpus_dialect` **0** residual.

## Evidence

- `spec/rql/qualification/corpus-v1/corpus-v1.json` v0.4.1
- `crates/residiuum-sdk/src/query_bytecode_v1/full_attach.rs` (dialect + `_key`)
- `crates/residiuum-sdk/tests/rql_q2_capability_audit.rs` (companion fixtures)

## Non-claims

- Not Q2 package accept; not Q3 oracle correctness.
- Residual enrich shape gaps (e.g. within-unread) may remain under
  `pkg_enrich_semantics`.
