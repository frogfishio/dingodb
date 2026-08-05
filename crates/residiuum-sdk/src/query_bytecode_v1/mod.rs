//! Query bytecode v1 — product runtime (Decision 0 — **still open**).
//!
//! Profile: **`residiuum-query-bytecode-v1`**
//! ISA: **`residiuum-query-isa-v1`** ([QUERY_ISA_V1.md](../../../../doc/todo/rql/QUERY_ISA_V1.md))
//!
//! **RQL-X5:** [`QueryBytecodeV1`] holds **only** ISA bytes. Execution always
//! `decode_isa` → interpret. An independent Rust `plan` field is forbidden so
//! ISA identity cannot diverge from executed meaning.
//!
//! **RQL-X5c one-dispatch:** after decode, Core page always goes through
//! [`execute_decoded_core`] → [`execute_plan`]. Full path decodes once and
//! reuses that Core entry (no Core re-encode). See
//! [QUERY_IR_RESIDUAL.md](../../../../doc/todo/rql/QUERY_IR_RESIDUAL.md) for
//! what is still a Rust interpreter of decoded structures.
//!
//! Host adapters supply scan/index/get only.
//!
//! Residual (Decision 0 still open): attach helpers remain Rust; orchestration
//! is a named IR phase (`ir_attach`). Core project/order/page are named IR
//! phases (still Rust). **RQL-C1 must not be accepted.**

mod core_page;
mod full_attach;
mod ir_attach;
mod ir_order;
mod ir_page;
mod ir_project;
mod isa;
mod kernel;

pub use core_page::{explain_rql_source, EXEC_PROFILE};
// Crate-private semantic executors (RQL-P0b) — available inside the SDK crate only.
pub(crate) use core_page::{execute_plan, execute_rql, DocScan};
pub use full_attach::{
    compile_rql_full, execute_full_isa_with, execute_rql_full, execute_rql_full_with,
    explain_rql_full, explain_rql_full_on_heap, refuse_full_language_on_core_wire,
    source_uses_rql_full_constructs, CompiledRqlFull, EnrichAttachMode, EnrichCardinality,
    EnrichLoadEvidence, EnrichStepV1, FullPipelineStepV1, ProjectItemV1, RqlFullExecuteOptions,
    RqlFullPage, WithinStepV1, DIAG_RQL_ENRICH_CARDINALITY, DIAG_RQL_FULL_RESIDUAL,
    DIAG_RQL_PROJECTION_CONFLICT, DIAG_RQL_PROJECT_TYPE, DIAG_RQL_WITHIN_TYPE,
    FULL_EXPLAIN_HASH_DOMAIN, MAX_PROJECT_DEPTH, MAX_WITHIN_DEPTH, RQL_FULL_PROFILE,
};
// IR / attach orchestration: crate-private (RQL-P0b) — profiles remain public stamps.
pub use ir_attach::ATTACH_IR_PROFILE;
pub use ir_order::ORDER_IR_PROFILE;
pub use ir_page::PAGE_IR_PROFILE;
pub use ir_project::PROJECT_IR_PROFILE;
pub use isa::{
    decode_isa, decode_isa_canonical, encode_core_program, encode_full_program, isa_hash,
    QueryIsaFullSection, QueryIsaProgram, ISA_MAGIC, ISA_MAX_SECTION_BYTES, ISA_MAX_TOTAL_BYTES,
    ISA_PROFILE, ISA_VERSION,
};
pub use kernel::{compile_where, lower_predicate, CompiledKernelWhere, KERNEL_PROFILE};

use crate::app_v1::{Parameters, QueryBudget, QueryExplanation, QueryPage, QueryRunOptions};
use crate::error::Error;
use crate::plan_v1::{CollectionBindings, RqlPlanV1};
use crate::rql_app_core::{compile_app_core, CompiledAppCore};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Architecture freeze profile id.
pub const BYTECODE_PROFILE: &str = "residiuum-query-bytecode-v1";

/// Host data-access capabilities only (Decision 0 / RQL-X1).
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

/// Compiled query bytecode envelope — **ISA bytes only** (RQL-X5).
///
/// Fields are private so callers cannot pair a Rust plan with unrelated ISA.
#[derive(Debug, Clone)]
pub struct QueryBytecodeV1 {
    profile: String,
    isa: Vec<u8>,
}

impl QueryBytecodeV1 {
    /// Profile label.
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Durable ISA bytes (sole executable identity).
    pub fn isa_bytes(&self) -> &[u8] {
        &self.isa
    }

    /// Domain-separated hash of the ISA bytes.
    pub fn isa_hash(&self) -> [u8; 32] {
        isa_hash(&self.isa)
    }

    /// Decode/validate ISA with canonical re-encode check.
    pub fn decode(&self) -> Result<QueryIsaProgram, Error> {
        decode_isa_canonical(&self.isa)
    }

    /// Lower a validated Application Core plan → ISA envelope.
    pub fn from_core_plan(plan: RqlPlanV1, budget: Option<QueryBudget>) -> Result<Self, Error> {
        let isa = encode_core_program(&plan, budget)?;
        // Refuse to stamp an envelope whose bytes do not round-trip to `plan`.
        let prog = decode_isa(&isa)?;
        if prog.core != plan || prog.budget != budget || prog.full.is_some() {
            return Err(Error::QueryInvalid(
                "isa encode/decode mismatch on Core lower".into(),
            ));
        }
        Ok(Self {
            profile: BYTECODE_PROFILE.to_string(),
            isa,
        })
    }

    /// Lower compiled Application Core artefact.
    pub fn from_compiled_core(compiled: CompiledAppCore) -> Result<Self, Error> {
        Self::from_core_plan(compiled.plan, compiled.budget)
    }

    /// Construct from already-encoded ISA bytes (validates canonical decode).
    pub fn from_isa_bytes(isa: Vec<u8>) -> Result<Self, Error> {
        let prog = decode_isa_canonical(&isa)?;
        if prog.full.is_some() {
            return Err(Error::QueryInvalid(
                "QueryBytecodeV1::from_isa_bytes: Core envelope cannot carry full pipeline"
                    .into(),
            ));
        }
        Ok(Self {
            profile: BYTECODE_PROFILE.to_string(),
            isa,
        })
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
    QueryBytecodeV1::from_compiled_core(compiled)
}

/// Explain via Core compile (plan tree + hash; no row scan).
pub fn explain_core_source(
    source: &str,
    collection_id: CollectionId,
    collection_name: &str,
) -> Result<QueryExplanation, Error> {
    explain_rql_source(source, collection_id, collection_name)
}

/// Execute bytecode against a host — **decodes ISA; never trusts a side plan**.
pub fn execute_bytecode<H: HostCapabilities>(
    host: &mut H,
    bytecode: &QueryBytecodeV1,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
) -> Result<QueryPage, Error> {
    if bytecode.profile() != BYTECODE_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "query bytecode profile mismatch: got {:?}, want {BYTECODE_PROFILE}",
            bytecode.profile()
        )));
    }
    execute_isa_bytes(
        host,
        bytecode.isa_bytes(),
        params,
        options,
        heap_id,
        collection_id,
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

/// Sole Core ISA entry: decode bytes, refuse full section, then
/// [`execute_decoded_core`].
pub fn execute_isa_bytes<H: HostCapabilities>(
    host: &mut H,
    isa_bytes: &[u8],
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
) -> Result<QueryPage, Error> {
    let prog = decode_isa_canonical(isa_bytes)?;
    if prog.full.is_some() {
        return Err(Error::QueryInvalid(
            "execute_isa_bytes: full-language ISA requires execute_full_isa_with".into(),
        ));
    }
    if prog.profile != ISA_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "isa profile mismatch: got {:?}, want {ISA_PROFILE}",
            prog.profile
        )));
    }
    execute_decoded_core(
        host,
        &prog.core,
        prog.budget,
        params,
        options,
        heap_id,
        collection_id,
    )
}

/// Shared Core page after ISA decode (RQL-X5c one-dispatch).
///
/// Crate-private (RQL-P0b): not a public bypass of validated ISA.
/// Both Core and full-language paths use this after `decode_isa`. Meaning of
/// page/order/project/coverage still lives in [`execute_plan`] (Rust interpreter
/// of the decoded plan) — see QUERY_IR_RESIDUAL.md. Not Decision 0 closed.
pub(crate) fn execute_decoded_core<H: HostCapabilities>(
    host: &mut H,
    core: &RqlPlanV1,
    budget: Option<QueryBudget>,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
) -> Result<QueryPage, Error> {
    if core.from.collection_id != collection_id {
        return Err(Error::QueryInvalid(
            "execute_decoded_core: collection_id mismatch".into(),
        ));
    }
    let mut scan = HostDocScan(host);
    execute_plan(
        &mut scan,
        core,
        params,
        options,
        heap_id,
        collection_id,
        budget,
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
    use crate::predicate::{CompareOp, Operand, Path};
    use serde_json::json;

    fn uuidish(seed: u8) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = seed;
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        b
    }

    struct MapHost {
        docs: BTreeMap<String, JsonValue>,
    }

    impl HostCapabilities for MapHost {
        fn list_keys(
            &mut self,
            limit: Option<usize>,
            after_key: Option<&str>,
        ) -> Result<Vec<String>, Error> {
            let mut keys: Vec<String> = self.docs.keys().cloned().collect();
            keys.sort();
            if let Some(a) = after_key {
                keys.retain(|k| k.as_str() > a);
            }
            if let Some(n) = limit {
                keys.truncate(n);
            }
            Ok(keys)
        }

        fn get_json(&mut self, key: &str) -> Result<Option<JsonValue>, Error> {
            Ok(self.docs.get(key).cloned())
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
        assert_eq!(bc.profile(), BYTECODE_PROFILE);
        let prog = bc.decode().expect("decode");
        assert_eq!(prog.core.from.source_name, "items");
    }

    #[test]
    fn execute_empty_host_one_page() {
        let id = CollectionId::from_bytes(uuidish(9)).expect("id");
        let heap = HeapId::from_bytes(uuidish(1)).expect("heap");
        let mut host = MapHost {
            docs: BTreeMap::new(),
        };
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

    #[test]
    fn execute_bytecode_uses_isa_not_sidecar_plan() {
        let id = CollectionId::from_bytes(uuidish(3)).expect("id");
        let heap = HeapId::from_bytes(uuidish(1)).expect("heap");
        let mut docs = BTreeMap::new();
        docs.insert("a".into(), json!({"status": "active"}));
        docs.insert("b".into(), json!({"status": "paused"}));
        let mut host = MapHost { docs };

        let bc = lower_core_source(
            "from items where status = \"active\"",
            id,
            "items",
        )
        .expect("lower");

        // Tamper: replace Core body with a different filter via re-encode.
        let mut prog = bc.decode().expect("decode");
        prog.core.where_pred = crate::predicate::Predicate::Cmp {
            cmp: CompareOp::Eq,
            left: Operand::path(Path::parse_dotted("status").unwrap()),
            right: Operand::literal(json!("paused")),
        };
        let tampered = encode_core_program(&prog.core, prog.budget).expect("encode");
        let bc_tampered = QueryBytecodeV1::from_isa_bytes(tampered).expect("wrap");

        let page = execute_bytecode(
            &mut host,
            &bc_tampered,
            &BTreeMap::new(),
            &QueryRunOptions::default(),
            heap,
            id,
        )
        .expect("exec");
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].key, "b");
        // Original ISA still selects "active".
        let page0 = execute_bytecode(
            &mut host,
            &bc,
            &BTreeMap::new(),
            &QueryRunOptions::default(),
            heap,
            id,
        )
        .expect("exec0");
        assert_eq!(page0.rows.len(), 1);
        assert_eq!(page0.rows[0].key, "a");
    }

    #[test]
    fn execute_decoded_core_is_shared_core_entry() {
        let id = CollectionId::from_bytes(uuidish(12)).expect("id");
        let heap = HeapId::from_bytes(uuidish(1)).expect("heap");
        let mut host = MapHost {
            docs: BTreeMap::from([("k1".into(), json!({"n": 1}))]),
        };
        let bc = lower_core_source("from items page size 8", id, "items").expect("lower");
        let prog = bc.decode().expect("decode");
        let via_decoded = execute_decoded_core(
            &mut host,
            &prog.core,
            prog.budget,
            &BTreeMap::new(),
            &QueryRunOptions::default(),
            heap,
            id,
        )
        .expect("decoded");
        let via_isa = execute_isa_bytes(
            &mut host,
            bc.isa_bytes(),
            &BTreeMap::new(),
            &QueryRunOptions::default(),
            heap,
            id,
        )
        .expect("isa");
        assert_eq!(via_decoded.rows, via_isa.rows);
        assert_eq!(via_decoded.rows.len(), 1);
    }

    #[test]
    fn corrupted_isa_refuses_execute() {
        let id = CollectionId::from_bytes(uuidish(4)).expect("id");
        let heap = HeapId::from_bytes(uuidish(1)).expect("heap");
        let mut host = MapHost {
            docs: BTreeMap::new(),
        };
        let bc = lower_core_source("from items", id, "items").expect("lower");
        let mut bad = bc.isa_bytes().to_vec();
        if bad.len() > 10 {
            bad[10] ^= 0xff;
        }
        let err = execute_isa_bytes(
            &mut host,
            &bad,
            &BTreeMap::new(),
            &QueryRunOptions::default(),
            heap,
            id,
        )
        .expect_err("corrupt");
        let _ = err;
    }
}
