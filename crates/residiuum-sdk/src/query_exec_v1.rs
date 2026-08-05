//! Compatibility shim — Decision 0 / RQL-X2b.
//!
//! Core page semantics moved to [`crate::query_bytecode_v1::core_page`].
//! This module re-exports the public surface so existing `query_exec_v1::`
//! paths keep compiling during migration. Prefer
//! [`crate::query_bytecode_v1`] for new code.
//!
//! Residual: delete this shim once callers and tests import bytecode directly;
//! op **118** full-language still refuses on Core wire (F2).

pub use crate::query_bytecode_v1::{
    execute_plan, execute_rql, explain_rql_source, DocScan, EXEC_PROFILE,
};
