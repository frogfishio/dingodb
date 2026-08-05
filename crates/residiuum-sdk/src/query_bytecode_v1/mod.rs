//! Query bytecode v1 — single product runtime (RQL-X2 / X2b / X2c).
//!
//! Profile: **`residiuum-query-bytecode-v1`**
//! Normative: [QUERY_BYTECODE_V1.md](../../../../doc/todo/rql/QUERY_BYTECODE_V1.md)
//!
//! **Decision 0:** this module tree is the only legal **product** semantic runtime.
//! Host adapters supply scan/index/get only.
//!
//! - [`core_page`] — Application Core page semantics (former `query_exec_v1`)
//! - [`full_attach`] — enrich / within / project (former `rql_full_v1`)
//!
//! Compatibility shims: [`crate::query_exec_v1`], [`crate::rql_full_v1`].

mod core_page;
mod full_attach;

pub use core_page::{
    execute_plan, execute_rql, explain_rql_source, DocScan, EXEC_PROFILE,
};
pub use full_attach::{
    apply_project_rows, attach_enrich_rows, attach_within_rows, compile_rql_full,
    execute_rql_full, execute_rql_full_with, explain_rql_full, explain_rql_full_on_heap,
    filter_rows, refuse_full_language_on_core_wire, source_uses_rql_full_constructs,
    CompiledRqlFull, EnrichAttachMode, EnrichCardinality, EnrichLoadEvidence, EnrichStepV1,
    FullPipelineStepV1, ProjectItemV1, RqlFullExecuteOptions, RqlFullPage, WithinStepV1,
    DIAG_RQL_ENRICH_CARDINALITY, DIAG_RQL_FULL_RESIDUAL, DIAG_RQL_PROJECTION_CONFLICT,
    DIAG_RQL_PROJECT_TYPE, DIAG_RQL_WITHIN_TYPE, FULL_EXPLAIN_HASH_DOMAIN, MAX_PROJECT_DEPTH,
    MAX_WITHIN_DEPTH, RQL_FULL_PROFILE,
};

use crate::app_v1::{Parameters, QueryBudget, QueryExplanation, QueryPage, QueryRunOptions};
use crate::error::Error;
use crate::plan_v1::{CollectionBindings, RqlPlanV1};
use crate::rql_app_core::{compile_app_core, CompiledAppCore};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Architecture freeze profile id ([QUERY_BYTECODE_V1.md](../../../../doc/todo/rql/QUERY_BYTECODE_V1.md)).
pub const BYTECODE_PROFILE: &str = "residiuum-query-bytecode-v1";

/// Host data-access capabilities only (Decision 0 / RQL-X1).
///
/// Must not evaluate predicates, enrich, project, order, page meaning, or
/// missing/null/coverage. Index lookup returns **candidates** only.
pub trait HostCapabilities {
    /// Deterministic key stream.
    fn list_keys(
        &mut self,
        limit: Option<usize>,
        after_key: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// Document get (`None` = absent).
    fn get_json(&mut self, key: &str) -> Result<Option<JsonValue>, Error>;

    /// Optional equality-index candidate keys (not a semantic filter).
    fn lookup_index_keys(
        &mut self,
        _equalities: &[(String, JsonValue)],
    ) -> Result<Option<Vec<String>>, Error> {
        Ok(None)
    }
}

/// Compiled query bytecode envelope (logical plan carrier for this cut).
///
/// Binary ISA encoding remains residual; the envelope + single runtime own
/// Core page semantics via [`core_page`].
#[derive(Debug, Clone)]
pub struct QueryBytecodeV1 {
    /// Profile label stamped on the envelope.
    pub profile: String,
    /// Application Core logical plan (canonical).
    pub plan: RqlPlanV1,
    /// Merged source budget from compile (if any).
    pub budget: Option<QueryBudget>,
}

impl QueryBytecodeV1 {
    /// Lower a validated Application Core plan into the bytecode envelope.
    pub fn from_core_plan(plan: RqlPlanV1, budget: Option<QueryBudget>) -> Self {
        Self {
            profile: BYTECODE_PROFILE.to_string(),
            plan,
            budget,
        }
    }

    /// Lower compiled Application Core artefact.
    pub fn from_compiled_core(compiled: CompiledAppCore) -> Self {
        Self::from_core_plan(compiled.plan, compiled.budget)
    }
}

/// Compile Application Core RQL source → bytecode envelope.
pub fn lower_core_source(
    source: &str,
    collection_id: CollectionId,
    collection_name: &str,
) -> Result<QueryBytecodeV1, Error> {
    let mut bindings = CollectionBindings {
        by_name: BTreeMap::new(),
    };
    bindings.bind(collection_name, collection_id);
    let mut compiled = compile_app_core(source, &bindings)?;
    if compiled.plan.from.collection_id != collection_id {
        compiled.plan.from.collection_id = collection_id;
    }
    Ok(QueryBytecodeV1::from_compiled_core(compiled))
}

/// Explain via Core compile (plan tree + hash; no row scan).
pub fn explain_core_source(
    source: &str,
    collection_id: CollectionId,
    collection_name: &str,
) -> Result<QueryExplanation, Error> {
    explain_rql_source(source, collection_id, collection_name)
}

/// Execute bytecode against a host (product Core path).
pub fn execute_bytecode<H: HostCapabilities>(
    host: &mut H,
    bytecode: &QueryBytecodeV1,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
) -> Result<QueryPage, Error> {
    if bytecode.profile != BYTECODE_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "query bytecode profile mismatch: got {:?}, want {BYTECODE_PROFILE}",
            bytecode.profile
        )));
    }
    let mut scan = HostDocScan(host);
    execute_plan(
        &mut scan,
        &bytecode.plan,
        params,
        options,
        heap_id,
        collection_id,
        bytecode.budget,
    )
}

/// Product entry: Core RQL source → lower → execute on host.
pub fn execute_core_rql<H: HostCapabilities>(
    host: &mut H,
    source: &str,
    parameters: &Parameters,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    collection_name: &str,
) -> Result<QueryPage, Error> {
    if options.explain {
        return Err(Error::QueryInvalid(
            "use explain_rql for explain; rql executes rows".into(),
        ));
    }
    let bytecode = lower_core_source(source, collection_id, collection_name)?;
    execute_bytecode(
        host,
        &bytecode,
        &parameters.values,
        options,
        heap_id,
        collection_id,
    )
}

/// Bridge: [`HostCapabilities`] → [`DocScan`] for [`core_page`].
struct HostDocScan<'a, H: HostCapabilities>(&'a mut H);

impl<H: HostCapabilities> DocScan for HostDocScan<'_, H> {
    fn list_keys(
        &mut self,
        limit: Option<usize>,
        after_key: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        self.0.list_keys(limit, after_key)
    }

    fn get_json(&mut self, key: &str) -> Result<Option<JsonValue>, Error> {
        self.0.get_json(key)
    }

    fn try_equality_index_keys(
        &mut self,
        equalities: &[(String, JsonValue)],
    ) -> Result<Option<Vec<String>>, Error> {
        self.0.lookup_index_keys(equalities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuidish(seed: u8) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = seed;
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        b
    }

    struct EmptyHost;

    impl HostCapabilities for EmptyHost {
        fn list_keys(
            &mut self,
            _limit: Option<usize>,
            _after_key: Option<&str>,
        ) -> Result<Vec<String>, Error> {
            Ok(Vec::new())
        }

        fn get_json(&mut self, _key: &str) -> Result<Option<JsonValue>, Error> {
            Ok(None)
        }
    }

    #[test]
    fn bytecode_profile_constant() {
        assert_eq!(BYTECODE_PROFILE, "residiuum-query-bytecode-v1");
    }

    #[test]
    fn lower_core_stamps_profile() {
        let id = CollectionId::from_bytes(uuidish(7)).expect("id");
        let bc = lower_core_source("from items", id, "items").expect("lower");
        assert_eq!(bc.profile, BYTECODE_PROFILE);
        assert_eq!(bc.plan.from.source_name, "items");
    }

    #[test]
    fn execute_empty_host_one_page() {
        let id = CollectionId::from_bytes(uuidish(9)).expect("id");
        let heap = HeapId::from_bytes(uuidish(1)).expect("heap");
        let mut host = EmptyHost;
        let page = execute_core_rql(
            &mut host,
            "from items",
            &Parameters::default(),
            &QueryRunOptions::default(),
            heap,
            id,
            "items",
        )
        .expect("exec");
        assert!(page.rows.is_empty());
    }

    #[test]
    fn core_page_owned_by_bytecode_module() {
        assert_eq!(EXEC_PROFILE, "residiuum-app-core-exec-v1");
    }
}
