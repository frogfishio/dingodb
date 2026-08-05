//! Compatibility shim — Decision 0 / RQL-X2c.
//!
//! Full-language attach semantics moved to
//! [`crate::query_bytecode_v1::full_attach`]. This module re-exports the public
//! surface so existing `rql_full_v1::` / crate-root paths keep compiling.
//! Prefer [`crate::query_bytecode_v1`] for new code.
//!
//! Residual: delete this shim once callers import bytecode directly; unify
//! op **118** onto the same runtime (still Core-refuse for full-language).

pub use crate::query_bytecode_v1::{
    apply_project_rows, attach_enrich_rows, attach_within_rows, compile_rql_full,
    execute_rql_full, execute_rql_full_with, explain_rql_full, explain_rql_full_on_heap,
    filter_rows, refuse_full_language_on_core_wire, source_uses_rql_full_constructs,
    CompiledRqlFull, EnrichAttachMode, EnrichCardinality, EnrichLoadEvidence, EnrichStepV1,
    FullPipelineStepV1, ProjectItemV1, RqlFullExecuteOptions, RqlFullPage, WithinStepV1,
    DIAG_RQL_ENRICH_CARDINALITY, DIAG_RQL_FULL_RESIDUAL, DIAG_RQL_PROJECTION_CONFLICT,
    DIAG_RQL_PROJECT_TYPE, DIAG_RQL_WITHIN_TYPE, FULL_EXPLAIN_HASH_DOMAIN, MAX_PROJECT_DEPTH,
    MAX_WITHIN_DEPTH, RQL_FULL_PROFILE,
};
