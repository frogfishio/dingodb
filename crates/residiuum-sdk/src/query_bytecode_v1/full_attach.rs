//! Full RQL attach / enrich / within / project — owned by [`crate::query_bytecode_v1`].
//!
//! Ported from the former standalone `rql_full_v1` façade (Decision 0 / RQL-X2c).
//! Product entry remains via [`crate::query_bytecode_v1`] re-exports.
//!
//! Normative: [RQL_SPEC.md](../../../../doc/wip/query/RQL_SPEC.md) enrich / within.
//! Application Core (`rql-app-core-v1`) still **rejects** `enrich` / `within` /
//! `at rank` at the Core compile surface; full-language runs through this module.
//!
//! Current surface:
//! - ordered root pipeline: `enrich` / `within` / post-attach `where` interleaved
//! - nested `within` depth (bounded by [`MAX_WITHIN_DEPTH`])
//! - nested `where` inside `within` (ordered filter on carrier elements)
//! - root `where` after enrich/within filters the Core page rows (page-then-attach honesty)
//! - nested post-pipeline `project { … }` (leaf / rename / nested product + bag map)
//! - execute `exactly_one` / `optional` / `many` attach via foreign scan oracle
//! - **RQL-I1:** root `enrich` may use equality-index pushdown on the foreign
//!   match field (`lookup_index_keys`); fall back to full scan when no usable
//!   index. Nested `within` enrich still scan-loads (residual).
//! - **RQL-X4b:** attach filters / candidate `where` evaluate via
//!   [`super::kernel`] (same SDA substrate as Core `where`)
//! - **RQL-X5b:** execute only after ISA encode→decode; pipeline/project from
//!   decoded full section (not a raw [`CompiledRqlFull`] authority bypass)
//! - refuse `at rank` / access policies (DDA residual)
//!
//! Not package accept. Not a claim that full RQL-v1 is product-ready.
//! Decision 0 remains open (page/order/project/coverage still Rust interpreters).

use crate::app_v1::{HeapClient, Parameters, QueryExplanation, QueryPage, QueryRunOptions};
use crate::error::Error;
use crate::plan_v1::{CollectionBindings, PLAN_HASH_DOMAIN};
use crate::predicate::{resolve_path, Path, Predicate, Resolve};
use crate::rql_app_core::{
    compile_app_core, CompiledAppCore, DIAG_RQL_FEATURE_UNAVAILABLE,
};
use residiuum_heap::CollectionId;
use serde_json::{Map, Value as JsonValue};
use std::collections::BTreeMap;

use super::isa::{decode_isa, encode_core_program, encode_full_program, ISA_PROFILE};
use super::execute_isa_bytes;

/// Full-language compile profile (Phase 3 kickoff).
pub const RQL_FULL_PROFILE: &str = "rql-full-v1";

/// Domain separator for [`CompiledRqlFull::explain_hash`].
pub const FULL_EXPLAIN_HASH_DOMAIN: &str = "residiuum:rql-full-v1:explain-v1";

/// Diagnostic when an enrich cardinality cannot be satisfied.
pub const DIAG_RQL_ENRICH_CARDINALITY: &str = "rql_enrich_cardinality";

/// Diagnostic when `within` path is absent, Null, or not a sequence/bag.
pub const DIAG_RQL_WITHIN_TYPE: &str = "rql_within_type";

/// Diagnostic when a full-language construct is still residual.
pub const DIAG_RQL_FULL_RESIDUAL: &str = "rql_full_residual";

/// Diagnostic when nested project hits a non-projectable value.
pub const DIAG_RQL_PROJECT_TYPE: &str = "rql_project_type";

/// Diagnostic when project output names collide.
pub const DIAG_RQL_PROJECTION_CONFLICT: &str = "rql_projection_conflict";

/// Host bound on nested `within` depth (root `within` is depth 1).
pub const MAX_WITHIN_DEPTH: usize = 8;

/// Host bound on nested `project { }` depth.
pub const MAX_PROJECT_DEPTH: usize = 8;

/// How foreign docs were loaded for one enrich step (RQL-I1 honesty).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrichAttachMode {
    /// Complete `list_keys`+`get` of the foreign collection.
    Scan,
    /// Equality index on the foreign match field + `get` of hit keys.
    EqualityIndex,
}

impl EnrichAttachMode {
    /// Stable label for evidence / explain.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scan => "scan_list_keys_get",
            Self::EqualityIndex => "equality_index",
        }
    }
}

/// Evidence for one enrich foreign-load decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrichLoadEvidence {
    /// Foreign collection name (`using`).
    pub using: String,
    /// Attach output field.
    pub output: String,
    /// Load mode actually used.
    pub mode: EnrichAttachMode,
}

/// Options for [`execute_rql_full_with`].
#[derive(Debug, Clone, Default)]
pub struct RqlFullExecuteOptions {
    /// Application Core page options (continuation, budgets, …).
    pub query: QueryRunOptions,
    /// When true, never use enrich equality-index pushdown (differential oracle).
    pub force_enrich_scan: bool,
}

/// Enrichment cardinality (RQL_SPEC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrichCardinality {
    /// Exactly one matching foreign document.
    ExactlyOne,
    /// Zero or one match.
    Optional,
    /// Zero or more matches (JSON array / bag).
    Many,
}

impl EnrichCardinality {
    /// Spec spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactlyOne => "exactly_one",
            Self::Optional => "optional",
            Self::Many => "many",
        }
    }
}

/// One compiled enrich step (single-level).
#[derive(Debug, Clone, PartialEq)]
pub struct EnrichStepV1 {
    /// Field name attached onto the current row (root or within element).
    pub output: String,
    /// Bound foreign collection name (diagnostics).
    pub using_name: String,
    /// Bound foreign collection id.
    pub using_id: CollectionId,
    /// Path on the **current** document (left side of matching).
    pub left: Path,
    /// Path on the **foreign** document (right side of matching).
    pub right: Path,
    /// Optional filter evaluated against each **foreign** candidate.
    pub candidate_where: Option<Predicate>,
    /// Cardinality expectation.
    pub expect: EnrichCardinality,
}

/// One step in an ordered enrich/within pipeline (root or nested).
#[derive(Debug, Clone, PartialEq)]
pub enum FullPipelineStepV1 {
    /// Attach foreign docs onto the current row.
    Enrich(EnrichStepV1),
    /// Expand a carrier bag and run nested steps per element.
    Within(WithinStepV1),
    /// Keep current rows where the predicate is true (nested `where` inside `within`).
    Filter(Predicate),
}

/// One compiled `within` step (nested enrich and/or nested `within`).
#[derive(Debug, Clone, PartialEq)]
pub struct WithinStepV1 {
    /// Carrier path on the current row (must resolve to a JSON array).
    pub carrier: Path,
    /// Optional element alias (`as item`); stripped from nested left/carrier paths.
    pub element_alias: Option<String>,
    /// Nested pipeline steps applied in order to each carrier element.
    pub steps: Vec<FullPipelineStepV1>,
}

/// One compiled nested-project item (`project { … }`).
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectItemV1 {
    /// Leaf copy: output name ← source path (`id` or `region: address.region`).
    Leaf {
        /// Output field name.
        output: String,
        /// Source path on the current artefact.
        source: Path,
    },
    /// Nested block: `customer { name }` or bag map `items { sku }`.
    Nested {
        /// Output field name and source field on the current artefact.
        output: String,
        /// Nested projection items.
        fields: Vec<ProjectItemV1>,
    },
}

/// Compiled full-language query (Core base + ordered enrich/within pipeline).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledRqlFull {
    /// Profile label.
    pub profile: &'static str,
    /// Application Core plan for the base `from`/`where`/`project`/`order`/…
    /// (enrich/within/brace-project stripped before Core compile).
    pub base: CompiledAppCore,
    /// Ordered root pipeline (`enrich` / `within` interleaved).
    pub pipeline: Vec<FullPipelineStepV1>,
    /// Optional nested `project { … }` applied after the pipeline.
    pub project: Option<Vec<ProjectItemV1>>,
    /// Source text with enrich/within/brace-project clauses removed (Core surface).
    pub base_source: String,
}

impl CompiledRqlFull {
    /// Root enrich steps in pipeline order (skips `within` / filter).
    pub fn root_enrich(&self) -> Vec<&EnrichStepV1> {
        self.pipeline
            .iter()
            .filter_map(|s| match s {
                FullPipelineStepV1::Enrich(e) => Some(e),
                FullPipelineStepV1::Within(_) | FullPipelineStepV1::Filter(_) => None,
            })
            .collect()
    }

    /// First root `within` step, if any.
    pub fn first_within(&self) -> Option<&WithinStepV1> {
        self.pipeline.iter().find_map(|s| match s {
            FullPipelineStepV1::Within(w) => Some(w),
            FullPipelineStepV1::Enrich(_) | FullPipelineStepV1::Filter(_) => None,
        })
    }

    /// Structured explain tree (base Core plan + pipeline + project). No rows.
    pub fn to_explain_tree(&self) -> JsonValue {
        let mut root = BTreeMap::new();
        root.insert(
            "profile".into(),
            JsonValue::String(RQL_FULL_PROFILE.into()),
        );
        root.insert(
            "base".into(),
            self.base.plan.to_canonical_json(),
        );
        root.insert(
            "base_plan_hash".into(),
            JsonValue::String(bytes_to_hex(&self.base.plan.plan_hash())),
        );
        root.insert(
            "base_source".into(),
            JsonValue::String(self.base_source.clone()),
        );
        root.insert(
            "pipeline".into(),
            JsonValue::Array(
                self.pipeline
                    .iter()
                    .map(pipeline_step_to_json)
                    .collect(),
            ),
        );
        match &self.project {
            Some(items) => root.insert(
                "project".into(),
                JsonValue::Array(items.iter().map(project_item_to_json).collect()),
            ),
            None => root.insert("project".into(), JsonValue::Null),
        };
        root.insert(
            "attach_oracle".into(),
            JsonValue::String("scan_or_equality_index".into()),
        );
        root.insert(
            "wire".into(),
            JsonValue::String("local_facade_only".into()),
        );
        btree_to_json_obj(root)
    }

    /// Domain-separated BLAKE3 hash over [`Self::to_explain_tree`].
    pub fn explain_hash(&self) -> [u8; 32] {
        let body = serde_json::to_vec(&self.to_explain_tree()).expect("explain tree json");
        let mut h = blake3::Hasher::new();
        h.update(FULL_EXPLAIN_HASH_DOMAIN.as_bytes());
        h.update(&[0u8]);
        // Tie explain hash to the Core plan hash domain so Core drift invalidates.
        h.update(PLAN_HASH_DOMAIN.as_bytes());
        h.update(&[0u8]);
        h.update(&body);
        *h.finalize().as_bytes()
    }
}

impl WithinStepV1 {
    /// Nested enrich steps in order (skips nested `within` / filter).
    pub fn enrich_steps(&self) -> Vec<&EnrichStepV1> {
        self.steps
            .iter()
            .filter_map(|s| match s {
                FullPipelineStepV1::Enrich(e) => Some(e),
                FullPipelineStepV1::Within(_) | FullPipelineStepV1::Filter(_) => None,
            })
            .collect()
    }
}

enum RawPipelineStep {
    Enrich(String),
    Within(String),
    Where(String),
}

enum RawNestedStep {
    Enrich(String),
    Within(String),
    Where(String),
}

/// Compile full RQL-v1 source (Core + ordered enrich/within pipeline).
pub fn compile_rql_full(
    source: &str,
    bindings: &CollectionBindings,
) -> Result<CompiledRqlFull, Error> {
    if source.len() > crate::rql_app_core::MAX_RQL_SOURCE_BYTES {
        return Err(Error::QueryInvalid(format!(
            "RQL source exceeds {} bytes",
            crate::rql_app_core::MAX_RQL_SOURCE_BYTES
        )));
    }
    refuse_residual_constructs(source)?;

    let (without_project, project) = extract_brace_project(source)?;
    let (base_source, raw_steps) = split_full_clauses(&without_project)?;
    let base = compile_app_core(&base_source, bindings)?;
    let mut pipeline = Vec::with_capacity(raw_steps.len());
    for raw in raw_steps {
        match raw {
            RawPipelineStep::Enrich(body) => {
                pipeline.push(FullPipelineStepV1::Enrich(parse_enrich_step(
                    &body, bindings, None,
                )?));
            }
            RawPipelineStep::Within(body) => {
                pipeline.push(FullPipelineStepV1::Within(parse_within_step(
                    &body, bindings, None, 1,
                )?));
            }
            RawPipelineStep::Where(body) => {
                pipeline.push(FullPipelineStepV1::Filter(parse_filter_step(
                    &body, bindings, None,
                )?));
            }
        }
    }
    Ok(CompiledRqlFull {
        profile: RQL_FULL_PROFILE,
        base,
        pipeline,
        project,
        base_source,
    })
}

fn refuse_residual_constructs(source: &str) -> Result<(), Error> {
    let lower = source.to_ascii_lowercase();
    let padded = format!(" {} ", lower.replace('\t', " "));
    for (needle, label) in [
        (" at rank", "at rank"),
        (" sequential", "sequential access"),
        (" direct ", "direct access"),
        (" build ", "build access"),
    ] {
        if padded.contains(needle) {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_FULL_RESIDUAL}: `{label}` outside rql-full-v1 current slice"
            )));
        }
    }
    Ok(())
}

/// True when source uses constructs that require [`compile_rql_full`] /
/// [`execute_rql_full`] (not Application Core / op 118).
pub fn source_uses_rql_full_constructs(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    let padded = format!(" {} ", lower.replace('\t', " "));
    if padded.contains(" enrich ")
        || padded.contains("\nenrich ")
        || lower.split_whitespace().any(|t| t == "enrich" || t == "within")
        || padded.contains(" within ")
    {
        return true;
    }
    // Brace `project { … }` (flat Core `project a, b` is fine on Core wire).
    let bytes = lower.as_bytes();
    let mut i = 0usize;
    let mut depth = 0usize;
    while i < lower.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 && matches_kw(&lower, i, "project") {
            let after = i + "project".len();
            let rest = source.get(after..).unwrap_or("").trim_start();
            if rest.starts_with('{') {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Refuse full-language source on the Application Core / op **118** wire.
///
/// RQL-F2 honesty: enrich/within/brace-project are local façade only until a
/// dedicated wire package lands. Callers must use [`execute_rql_full`].
pub fn refuse_full_language_on_core_wire(source: &str) -> Result<(), Error> {
    if source_uses_rql_full_constructs(source) {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FEATURE_UNAVAILABLE}: enrich/within/brace-project require \
             execute_rql_full; not on Application Core op-118 wire"
        )));
    }
    Ok(())
}

/// Structured explain for full RQL (compile only; no row materialization).
pub fn explain_rql_full(
    source: &str,
    bindings: &CollectionBindings,
) -> Result<QueryExplanation, Error> {
    let compiled = compile_rql_full(source, bindings)?;
    Ok(QueryExplanation {
        plan_profile: RQL_FULL_PROFILE.into(),
        plan_hash: compiled.explain_hash(),
        tree: compiled.to_explain_tree(),
    })
}

/// Explain using collection names discovered on [`HeapClient`].
pub fn explain_rql_full_on_heap(
    client: &mut HeapClient,
    source: &str,
) -> Result<QueryExplanation, Error> {
    let infos = client.list_collections()?;
    let mut bindings = CollectionBindings::default();
    for info in &infos {
        bindings.bind(&info.name, info.collection_id);
    }
    explain_rql_full(source, &bindings)
}

fn pipeline_step_to_json(step: &FullPipelineStepV1) -> JsonValue {
    match step {
        FullPipelineStepV1::Enrich(e) => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("enrich".into()));
            m.insert("output".into(), JsonValue::String(e.output.clone()));
            m.insert("using".into(), JsonValue::String(e.using_name.clone()));
            m.insert(
                "using_id".into(),
                JsonValue::String(e.using_id.to_string()),
            );
            m.insert("left".into(), JsonValue::String(e.left.dotted()));
            m.insert("right".into(), JsonValue::String(e.right.dotted()));
            m.insert(
                "expect".into(),
                JsonValue::String(e.expect.as_str().into()),
            );
            match &e.candidate_where {
                Some(p) => m.insert("candidate_where".into(), p.to_canonical_json()),
                None => m.insert("candidate_where".into(), JsonValue::Null),
            };
            btree_to_json_obj(m)
        }
        FullPipelineStepV1::Within(w) => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("within".into()));
            m.insert("carrier".into(), JsonValue::String(w.carrier.dotted()));
            match &w.element_alias {
                Some(a) => m.insert("alias".into(), JsonValue::String(a.clone())),
                None => m.insert("alias".into(), JsonValue::Null),
            };
            m.insert(
                "steps".into(),
                JsonValue::Array(w.steps.iter().map(pipeline_step_to_json).collect()),
            );
            btree_to_json_obj(m)
        }
        FullPipelineStepV1::Filter(pred) => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("filter".into()));
            m.insert("predicate".into(), pred.to_canonical_json());
            btree_to_json_obj(m)
        }
    }
}

fn project_item_to_json(item: &ProjectItemV1) -> JsonValue {
    match item {
        ProjectItemV1::Leaf { output, source } => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("leaf".into()));
            m.insert("output".into(), JsonValue::String(output.clone()));
            m.insert("source".into(), JsonValue::String(source.dotted()));
            btree_to_json_obj(m)
        }
        ProjectItemV1::Nested { output, fields } => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("nested".into()));
            m.insert("output".into(), JsonValue::String(output.clone()));
            m.insert(
                "fields".into(),
                JsonValue::Array(fields.iter().map(project_item_to_json).collect()),
            );
            btree_to_json_obj(m)
        }
    }
}

fn btree_to_json_obj(m: BTreeMap<String, JsonValue>) -> JsonValue {
    let mut map = Map::new();
    for (k, v) in m {
        map.insert(k, v);
    }
    JsonValue::Object(map)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

/// Extract top-level nested `project { … }` (brace form). Flat Core `project a, b` stays.
fn extract_brace_project(
    source: &str,
) -> Result<(String, Option<Vec<ProjectItemV1>>), Error> {
    let lower = source.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut i = 0usize;
    let mut depth = 0usize;
    while i < lower.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 && matches_kw(&lower, i, "project") {
            let after = i + "project".len();
            let rest = source[after..].trim_start();
            if rest.starts_with('{') {
                let open_abs = source.len() - rest.len();
                let end = find_matching_brace_end(source, open_abs)?;
                let inner = source[open_abs + 1..end - 1].trim();
                let items = parse_project_items(inner, 1)?;
                let mut out = String::new();
                out.push_str(&source[..i]);
                out.push_str(&source[end..]);
                return Ok((out, Some(items)));
            }
        }
        i += 1;
    }
    Ok((source.to_string(), None))
}

fn find_matching_brace_end(source: &str, open_abs: usize) -> Result<usize, Error> {
    let bytes = source.as_bytes();
    if bytes.get(open_abs) != Some(&b'{') {
        return Err(Error::QueryInvalid("project: expected `{`".into()));
    }
    let mut depth = 0usize;
    let mut i = open_abs;
    while i < source.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(Error::QueryInvalid("project: unclosed `{`".into()))
}

fn parse_project_items(inner: &str, depth: usize) -> Result<Vec<ProjectItemV1>, Error> {
    if depth > MAX_PROJECT_DEPTH {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FULL_RESIDUAL}: project depth exceeds host bound {MAX_PROJECT_DEPTH}"
        )));
    }
    let mut p = Words::new(inner);
    let mut items = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    while !p.is_eof() {
        let first = p.next_ident()?;
        let item = if p.eat_char(b'{') {
            let nested_inner = p.take_brace_inner()?;
            let fields = parse_project_items(&nested_inner, depth + 1)?;
            ProjectItemV1::Nested {
                output: first,
                fields,
            }
        } else if p.eat_char(b':') {
            let source = Path::parse_dotted(&p.next_path()?)?;
            ProjectItemV1::Leaf {
                output: first,
                source,
            }
        } else {
            let mut segs = vec![first];
            loop {
                p.skip();
                if p.s.as_bytes().get(p.i) == Some(&b'.') {
                    p.i += 1;
                    segs.push(p.next_ident()?);
                } else {
                    break;
                }
            }
            let output = segs.last().cloned().expect("non-empty");
            let source = Path::from_segments(segs)?;
            ProjectItemV1::Leaf { output, source }
        };
        let out_name = match &item {
            ProjectItemV1::Leaf { output, .. } | ProjectItemV1::Nested { output, .. } => {
                output.clone()
            }
        };
        if !seen.insert(out_name.clone()) {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_PROJECTION_CONFLICT}: duplicate output `{out_name}`"
            )));
        }
        items.push(item);
        let _ = p.eat_char(b',');
    }
    Ok(items)
}

/// Split top-level `enrich` / `within` / post-attach `where`; return Core text + steps.
///
/// Pre-enrich/within `where` clauses stay in Core. `where` after the first
/// enrich/within becomes a pipeline [`FullPipelineStepV1::Filter`].
fn split_full_clauses(source: &str) -> Result<(String, Vec<RawPipelineStep>), Error> {
    let lower = source.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    // kind: 0=enrich 1=within 2=where
    let mut spans: Vec<(usize, usize, u8)> = Vec::new();
    let mut i = 0usize;
    let mut depth = 0usize;
    let mut seen_attach = false;
    while i < lower.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            if matches_kw(&lower, i, "enrich") {
                let after = i + "enrich".len();
                let end = find_enrich_end(&lower, after);
                spans.push((i, end, 0));
                seen_attach = true;
                i = end;
                continue;
            }
            if matches_kw(&lower, i, "within") {
                let after = i + "within".len();
                let end = find_within_end(&lower, after)?;
                spans.push((i, end, 1));
                seen_attach = true;
                i = end;
                continue;
            }
            if seen_attach && matches_kw(&lower, i, "where") {
                let after = i + "where".len();
                let end = find_root_where_end(&lower, after);
                spans.push((i, end, 2));
                i = end;
                continue;
            }
        }
        i += 1;
    }

    if spans.is_empty() {
        return Ok((source.to_string(), Vec::new()));
    }

    let mut core = String::new();
    let mut steps = Vec::new();
    let mut cursor = 0;
    for (start, end, kind) in spans {
        core.push_str(&source[cursor..start]);
        match kind {
            0 => {
                let body = source[start + "enrich".len()..end].trim();
                steps.push(RawPipelineStep::Enrich(body.to_string()));
            }
            1 => {
                let body = source[start + "within".len()..end].trim();
                steps.push(RawPipelineStep::Within(body.to_string()));
            }
            2 => {
                let body = source[start + "where".len()..end].trim();
                if body.is_empty() {
                    return Err(Error::QueryInvalid(
                        "pipeline where clause is empty".into(),
                    ));
                }
                steps.push(RawPipelineStep::Where(body.to_string()));
            }
            _ => unreachable!(),
        }
        cursor = end;
    }
    core.push_str(&source[cursor..]);
    let core = core.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok((core, steps))
}

fn find_root_where_end(lower: &str, after_where: usize) -> usize {
    let terminals = [
        " enrich ",
        " within ",
        " where ",
        " project ",
        " order ",
        " limit ",
        " page ",
        " coverage ",
        " consistency ",
        " budget ",
        " at rank",
        " after ",
        " access ",
    ];
    let padded = format!(" {} ", &lower[after_where..]);
    let mut best = None;
    for t in terminals {
        if let Some(rel) = padded.find(t) {
            let abs = after_where + rel;
            best = Some(best.map_or(abs, |b: usize| b.min(abs)));
        }
    }
    best.unwrap_or(lower.len())
}

fn matches_kw(lower: &str, start: usize, kw: &str) -> bool {
    let bytes = lower.as_bytes();
    if start + kw.len() > lower.len() {
        return false;
    }
    if &lower[start..start + kw.len()] != kw {
        return false;
    }
    let before_ok = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
    let after = start + kw.len();
    let after_ok = after >= lower.len() || !bytes[after].is_ascii_alphanumeric();
    before_ok && after_ok
}

fn find_enrich_end(lower: &str, after_enrich: usize) -> usize {
    // Grammar: enrich … [where …] expect <card> — stop after cardinality.
    let padded = format!(" {} ", &lower[after_enrich..]);
    if let Some(rel) = padded.find(" expect ") {
        let mut i = after_enrich + rel + " expect ".len();
        while i < lower.len() && lower.as_bytes()[i].is_ascii_whitespace() {
            i += 1;
        }
        while i < lower.len() {
            let b = lower.as_bytes()[i];
            if b.is_ascii_alphanumeric() || b == b'_' {
                i += 1;
            } else {
                break;
            }
        }
        return i;
    }

    let terminals = [
        " enrich ",
        " within ",
        " where ",
        " project ",
        " order ",
        " limit ",
        " page ",
        " coverage ",
        " consistency ",
        " budget ",
        " at rank",
        " after ",
        " access ",
    ];
    let mut best = None;
    for t in terminals {
        if let Some(rel) = padded.find(t) {
            let abs = after_enrich + rel;
            best = Some(best.map_or(abs, |b: usize| b.min(abs)));
        }
    }
    best.unwrap_or(lower.len())
}

fn find_within_end(lower: &str, after_within: usize) -> Result<usize, Error> {
    let Some(brace_rel) = lower[after_within..].find('{') else {
        return Err(Error::QueryInvalid(
            "within: expected `{` after path".into(),
        ));
    };
    let open = after_within + brace_rel;
    let bytes = lower.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < lower.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(Error::QueryInvalid("within: unclosed `{`".into()))
}

fn parse_within_step(
    body: &str,
    bindings: &CollectionBindings,
    outer_alias: Option<&str>,
    depth: usize,
) -> Result<WithinStepV1, Error> {
    if depth > MAX_WITHIN_DEPTH {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FULL_RESIDUAL}: within depth exceeds host bound {MAX_WITHIN_DEPTH}"
        )));
    }
    // body: <path> [as <alias>] { <nested steps> }
    let mut p = Words::new(body);
    let carrier = strip_alias_prefix(Path::parse_dotted(&p.next_path()?)?, outer_alias)?;
    let element_alias = if p.eat("as") {
        Some(p.next_ident()?)
    } else {
        None
    };
    if !p.eat_char(b'{') {
        return Err(Error::QueryInvalid(
            "within: expected `{` after path".into(),
        ));
    }
    let inner = p.take_brace_inner()?;
    if !p.is_eof() {
        return Err(Error::QueryInvalid(format!(
            "unexpected trailing within tokens near `{}`",
            p.rest()
        )));
    }

    let nested = split_nested_pipeline(&inner)?;
    let mut steps = Vec::with_capacity(nested.len());
    for step in nested {
        match step {
            RawNestedStep::Enrich(b) => {
                steps.push(FullPipelineStepV1::Enrich(parse_enrich_step(
                    &b,
                    bindings,
                    element_alias.as_deref(),
                )?));
            }
            RawNestedStep::Within(b) => {
                steps.push(FullPipelineStepV1::Within(parse_within_step(
                    &b,
                    bindings,
                    element_alias.as_deref(),
                    depth + 1,
                )?));
            }
            RawNestedStep::Where(b) => {
                steps.push(FullPipelineStepV1::Filter(parse_filter_step(
                    &b,
                    bindings,
                    element_alias.as_deref(),
                )?));
            }
        }
    }
    if steps.is_empty() {
        return Err(Error::QueryInvalid(
            "within block requires at least one enrich, within, or where".into(),
        ));
    }
    Ok(WithinStepV1 {
        carrier,
        element_alias,
        steps,
    })
}

/// Split nested `where` / `enrich` / `within` steps inside a `within` block.
fn split_nested_pipeline(source: &str) -> Result<Vec<RawNestedStep>, Error> {
    let lower = source.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut spans: Vec<(usize, usize, u8)> = Vec::new(); // 0=enrich 1=within 2=where
    let mut i = 0usize;
    let mut depth = 0usize;
    while i < lower.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            if matches_kw(&lower, i, "enrich") {
                let after = i + "enrich".len();
                let end = find_enrich_end(&lower, after);
                spans.push((i, end, 0));
                i = end;
                continue;
            }
            if matches_kw(&lower, i, "within") {
                let after = i + "within".len();
                let end = find_within_end(&lower, after)?;
                spans.push((i, end, 1));
                i = end;
                continue;
            }
            if matches_kw(&lower, i, "where") {
                let after = i + "where".len();
                let end = find_where_end(&lower, after);
                spans.push((i, end, 2));
                i = end;
                continue;
            }
        }
        i += 1;
    }

    if spans.is_empty() {
        if source.split_whitespace().next().is_some() {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_FULL_RESIDUAL}: within nested tokens not in this slice near `{source}`"
            )));
        }
        return Ok(Vec::new());
    }

    // Refuse leftover text between / around steps.
    let mut cursor = 0usize;
    let mut steps = Vec::new();
    for (start, end, kind) in spans {
        let gap = source[cursor..start].trim();
        if !gap.is_empty() {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_FULL_RESIDUAL}: within nested tokens not in this slice near `{gap}`"
            )));
        }
        match kind {
            0 => {
                let body = source[start + "enrich".len()..end].trim();
                steps.push(RawNestedStep::Enrich(body.to_string()));
            }
            1 => {
                let body = source[start + "within".len()..end].trim();
                steps.push(RawNestedStep::Within(body.to_string()));
            }
            2 => {
                let body = source[start + "where".len()..end].trim();
                if body.is_empty() {
                    return Err(Error::QueryInvalid(
                        "within where clause is empty".into(),
                    ));
                }
                steps.push(RawNestedStep::Where(body.to_string()));
            }
            _ => unreachable!(),
        }
        cursor = end;
    }
    let trailing = source[cursor..].trim();
    if !trailing.is_empty() {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FULL_RESIDUAL}: within nested tokens not in this slice near `{trailing}`"
        )));
    }
    Ok(steps)
}

fn find_where_end(lower: &str, after_where: usize) -> usize {
    let terminals = [" enrich ", " within ", " where "];
    let padded = format!(" {} ", &lower[after_where..]);
    let mut best = None;
    for t in terminals {
        if let Some(rel) = padded.find(t) {
            let abs = after_where + rel;
            best = Some(best.map_or(abs, |b: usize| b.min(abs)));
        }
    }
    best.unwrap_or(lower.len())
}

fn parse_filter_step(
    body: &str,
    bindings: &CollectionBindings,
    element_alias: Option<&str>,
) -> Result<Predicate, Error> {
    let using = bindings.by_name.keys().next().ok_or_else(|| {
        Error::QueryInvalid("within where requires at least one collection binding".into())
    })?;
    let fake = format!("from {using} where {body}");
    let compiled = compile_app_core(&fake, bindings)?;
    strip_alias_from_predicate(compiled.plan.where_pred, element_alias)
}

fn strip_alias_from_predicate(pred: Predicate, alias: Option<&str>) -> Result<Predicate, Error> {
    let Some(a) = alias else {
        return Ok(pred);
    };
    Ok(match pred {
        Predicate::True | Predicate::False => pred,
        Predicate::Cmp { cmp, left, right } => Predicate::Cmp {
            cmp,
            left: strip_alias_from_operand(left, a)?,
            right: strip_alias_from_operand(right, a)?,
        },
        Predicate::In {
            left,
            list,
            negated,
        } => Predicate::In {
            left: strip_alias_from_operand(left, a)?,
            list,
            negated,
        },
        Predicate::Present { path } => Predicate::Present {
            path: strip_alias_prefix(path, Some(a))?,
        },
        Predicate::Missing { path } => Predicate::Missing {
            path: strip_alias_prefix(path, Some(a))?,
        },
        Predicate::IsNull { path, negated } => Predicate::IsNull {
            path: strip_alias_prefix(path, Some(a))?,
            negated,
        },
        Predicate::StartsWith { path, prefix } => Predicate::StartsWith {
            path: strip_alias_prefix(path, Some(a))?,
            prefix,
        },
        Predicate::Contains { path, needle } => Predicate::Contains {
            path: strip_alias_prefix(path, Some(a))?,
            needle,
        },
        Predicate::And { args } => Predicate::And {
            args: args
                .into_iter()
                .map(|p| strip_alias_from_predicate(p, Some(a)))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Predicate::Or { args } => Predicate::Or {
            args: args
                .into_iter()
                .map(|p| strip_alias_from_predicate(p, Some(a)))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Predicate::Not { arg } => Predicate::Not {
            arg: Box::new(strip_alias_from_predicate(*arg, Some(a))?),
        },
    })
}

fn strip_alias_from_operand(op: crate::predicate::Operand, alias: &str) -> Result<crate::predicate::Operand, Error> {
    use crate::predicate::Operand;
    match op {
        Operand::Path { path } => Ok(Operand::Path {
            path: strip_alias_prefix(path, Some(alias))?,
        }),
        other => Ok(other),
    }
}

fn parse_enrich_step(
    body: &str,
    bindings: &CollectionBindings,
    element_alias: Option<&str>,
) -> Result<EnrichStepV1, Error> {
    // body: <output> using <source> [as <alias>] matching <left> = <right> [where …] expect <card>
    let mut p = Words::new(body);
    let output = p.next_ident()?;
    p.expect("using")?;
    let using_name = p.next_ident()?;
    let foreign_alias = if p.eat("as") {
        Some(p.next_ident()?)
    } else {
        None
    };
    p.expect("matching")?;
    let left = strip_alias_prefix(Path::parse_dotted(&p.next_path()?)?, element_alias)?;
    p.expect("=")?;
    let right = strip_alias_prefix(Path::parse_dotted(&p.next_path()?)?, foreign_alias.as_deref())?;
    let candidate_where = if p.eat("where") {
        let where_src = p.take_until_keyword("expect")?;
        if where_src.trim().is_empty() {
            return Err(Error::QueryInvalid(
                "enrich where clause is empty".into(),
            ));
        }
        let fake = format!("from {using_name} where {where_src}");
        let compiled = compile_app_core(&fake, bindings)?;
        Some(compiled.plan.where_pred)
    } else {
        None
    };
    p.expect("expect")?;
    let card_s = p.next_ident()?;
    let expect = match card_s.as_str() {
        "exactly_one" => EnrichCardinality::ExactlyOne,
        "optional" => EnrichCardinality::Optional,
        "many" => EnrichCardinality::Many,
        other => {
            return Err(Error::QueryInvalid(format!(
                "enrich expect must be exactly_one|optional|many, got `{other}`"
            )));
        }
    };
    if !p.is_eof() {
        return Err(Error::QueryInvalid(format!(
            "unexpected trailing enrich tokens near `{}`",
            p.rest()
        )));
    }
    let using_id = bindings
        .by_name
        .get(&using_name)
        .copied()
        .ok_or_else(|| {
            Error::QueryInvalid(format!(
                "unknown collection binding `{using_name}` for enrich using"
            ))
        })?;
    Ok(EnrichStepV1 {
        output,
        using_name,
        using_id,
        left,
        right,
        candidate_where,
        expect,
    })
}

fn strip_alias_prefix(path: Path, alias: Option<&str>) -> Result<Path, Error> {
    let Some(a) = alias else {
        return Ok(path);
    };
    if path.0.first().map(String::as_str) != Some(a) {
        return Ok(path);
    }
    if path.0.len() == 1 {
        return Err(Error::QueryInvalid(format!(
            "path `{a}` is only an alias; need a field under it"
        )));
    }
    Path::from_segments(path.0.into_iter().skip(1))
}

/// Tiny whitespace tokenizer for enrich / within clause bodies.
struct Words<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Words<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }
    fn skip(&mut self) {
        while self
            .s
            .as_bytes()
            .get(self.i)
            .copied()
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            self.i += 1;
        }
    }
    fn is_eof(&mut self) -> bool {
        self.skip();
        self.i >= self.s.len()
    }
    fn rest(&mut self) -> &'a str {
        self.skip();
        &self.s[self.i..]
    }
    fn eat_char(&mut self, c: u8) -> bool {
        self.skip();
        if self.s.as_bytes().get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn eat(&mut self, kw: &str) -> bool {
        self.skip();
        let r = self.rest();
        let lower = r.to_ascii_lowercase();
        if lower.starts_with(kw)
            && (r.len() == kw.len()
                || !r.as_bytes()[kw.len()].is_ascii_alphanumeric() && r.as_bytes()[kw.len()] != b'_')
        {
            self.i += kw.len();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, kw: &str) -> Result<(), Error> {
        if self.eat(kw) {
            Ok(())
        } else {
            Err(Error::QueryInvalid(format!(
                "expected `{kw}` near `{}`",
                self.rest().chars().take(24).collect::<String>()
            )))
        }
    }
    fn next_ident(&mut self) -> Result<String, Error> {
        self.skip();
        let r = self.rest();
        let mut n = 0;
        for c in r.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                n += c.len_utf8();
            } else {
                break;
            }
        }
        if n == 0 {
            return Err(Error::QueryInvalid(format!(
                "expected identifier near `{}`",
                r.chars().take(24).collect::<String>()
            )));
        }
        let id = r[..n].to_string();
        self.i += n;
        Ok(id)
    }
    fn next_path(&mut self) -> Result<String, Error> {
        let mut parts = vec![self.next_ident()?];
        loop {
            self.skip();
            if self.s.as_bytes().get(self.i) == Some(&b'.') {
                self.i += 1;
                parts.push(self.next_ident()?);
            } else {
                break;
            }
        }
        Ok(parts.join("."))
    }

    /// After consuming `{`, take inner text until the matching `}` (consumed).
    fn take_brace_inner(&mut self) -> Result<String, Error> {
        let start = self.i;
        let bytes = self.s.as_bytes();
        let mut depth = 1usize;
        while self.i < self.s.len() {
            match bytes[self.i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let inner = self.s[start..self.i].trim().to_string();
                        self.i += 1;
                        return Ok(inner);
                    }
                }
                _ => {}
            }
            self.i += 1;
        }
        Err(Error::QueryInvalid("within: unclosed `{`".into()))
    }

    /// Consume text until a keyword at a word boundary (keyword not consumed).
    fn take_until_keyword(&mut self, kw: &str) -> Result<String, Error> {
        self.skip();
        let start = self.i;
        let rest = &self.s[start..];
        let lower = rest.to_ascii_lowercase();
        let mut search = 0usize;
        let at = loop {
            let Some(rel) = lower[search..].find(kw) else {
                return Err(Error::QueryInvalid(format!(
                    "enrich where: expected `{kw}` after predicate"
                )));
            };
            let at = search + rel;
            let before_ok =
                at == 0 || lower.as_bytes()[at - 1].is_ascii_whitespace();
            let after = at + kw.len();
            let after_ok = after >= lower.len()
                || (!lower.as_bytes()[after].is_ascii_alphanumeric()
                    && lower.as_bytes()[after] != b'_');
            if before_ok && after_ok {
                break at;
            }
            search = at + 1;
        };
        let end = start + at;
        let taken = self.s[start..end].trim().to_string();
        self.i = end;
        Ok(taken)
    }
}

/// Keep rows where `pred` evaluates to true (SDA kernel — RQL-X4b).
pub fn filter_rows(
    rows: &[(String, JsonValue)],
    pred: &Predicate,
    params: &BTreeMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, Error> {
    let where_k = super::kernel::compile_where(pred, params)?;
    let mut kept = Vec::with_capacity(rows.len());
    for (k, v) in rows {
        if where_k.eval_doc(v)? {
            kept.push((k.clone(), v.clone()));
        }
    }
    Ok(kept)
}

/// Apply nested `project { … }` to materialised rows.
pub fn apply_project_rows(
    rows: &[(String, JsonValue)],
    fields: &[ProjectItemV1],
) -> Result<Vec<(String, JsonValue)>, Error> {
    let mut out = Vec::with_capacity(rows.len());
    for (k, v) in rows {
        let projected = project_value(v, fields)?;
        out.push((k.clone(), projected));
    }
    Ok(out)
}

fn project_value(doc: &JsonValue, fields: &[ProjectItemV1]) -> Result<JsonValue, Error> {
    let mut map = serde_json::Map::new();
    for item in fields {
        match item {
            ProjectItemV1::Leaf { output, source } => match resolve_path(doc, source) {
                Resolve::Present(v) => {
                    map.insert(output.clone(), v);
                }
                Resolve::Absent => {}
            },
            ProjectItemV1::Nested { output, fields } => {
                let carrier_path = Path::from_segments(vec![output.clone()])?;
                match resolve_path(doc, &carrier_path) {
                    Resolve::Absent => {}
                    Resolve::Present(JsonValue::Null) => {
                        map.insert(output.clone(), JsonValue::Null);
                    }
                    Resolve::Present(JsonValue::Object(obj)) => {
                        let nested = project_value(&JsonValue::Object(obj), fields)?;
                        map.insert(output.clone(), nested);
                    }
                    Resolve::Present(JsonValue::Array(arr)) => {
                        let mut mapped = Vec::with_capacity(arr.len());
                        for el in arr {
                            mapped.push(project_value(&el, fields)?);
                        }
                        map.insert(output.clone(), JsonValue::Array(mapped));
                    }
                    Resolve::Present(other) => {
                        return Err(Error::QueryInvalid(format!(
                            "{DIAG_RQL_PROJECT_TYPE}: `{output}` is {} (need product/optional/bag)",
                            json_type_name(&other)
                        )));
                    }
                }
            }
        }
    }
    Ok(JsonValue::Object(map))
}

/// Attach enrich fields onto already-materialised root JSON documents.
///
/// `foreign_docs` is the foreign candidate set (complete scan **or**
/// equality-index hits for observed left values). Callers must not label a
/// partial candidate set as a complete collection inventory.
///
/// `expect many` attaches a JSON array of matches, ordered by foreign key.
/// Optional [`EnrichStepV1::candidate_where`] filters foreign docs first.
pub fn attach_enrich_rows(
    roots: &[(String, JsonValue)],
    foreign_docs: &[(String, JsonValue)],
    step: &EnrichStepV1,
    params: &BTreeMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, Error> {
    let mut by_right: BTreeMap<String, Vec<(String, JsonValue)>> = BTreeMap::new();
    let candidate_k = match &step.candidate_where {
        Some(pred) => Some(super::kernel::compile_where(pred, params)?),
        None => None,
    };
    for (fk, doc) in foreign_docs {
        if let Some(ref where_k) = candidate_k {
            if !where_k.eval_doc(doc)? {
                continue;
            }
        }
        if let Resolve::Present(v) = resolve_path(doc, &step.right) {
            by_right
                .entry(canonical_match_key(&v))
                .or_default()
                .push((fk.clone(), doc.clone()));
        }
    }
    for entries in by_right.values_mut() {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    let mut out = Vec::with_capacity(roots.len());
    for (key, root) in roots {
        let left_key = match resolve_path(root, &step.left) {
            Resolve::Present(v) => canonical_match_key(&v),
            Resolve::Absent => {
                match step.expect {
                    EnrichCardinality::Optional => {
                        let mut row = root.clone();
                        if let JsonValue::Object(map) = &mut row {
                            map.insert(step.output.clone(), JsonValue::Null);
                        }
                        out.push((key.clone(), row));
                        continue;
                    }
                    EnrichCardinality::Many => {
                        let mut row = root.clone();
                        if let JsonValue::Object(map) = &mut row {
                            map.insert(step.output.clone(), JsonValue::Array(vec![]));
                        }
                        out.push((key.clone(), row));
                        continue;
                    }
                    EnrichCardinality::ExactlyOne => {
                        return Err(Error::QueryInvalid(format!(
                            "{DIAG_RQL_ENRICH_CARDINALITY}: missing left match path `{}` on key `{key}`",
                            step.left.0.join(".")
                        )));
                    }
                }
            }
        };
        let candidates = by_right
            .get(&left_key)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let attached = match (step.expect, candidates.len()) {
            (EnrichCardinality::ExactlyOne, 1) => candidates[0].1.clone(),
            (EnrichCardinality::ExactlyOne, n) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_ENRICH_CARDINALITY}: exactly_one expected 1 match, got {n} (key `{key}`)"
                )));
            }
            (EnrichCardinality::Optional, 0) => JsonValue::Null,
            (EnrichCardinality::Optional, 1) => candidates[0].1.clone(),
            (EnrichCardinality::Optional, n) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_ENRICH_CARDINALITY}: optional expected ≤1 match, got {n} (key `{key}`)"
                )));
            }
            (EnrichCardinality::Many, _) => JsonValue::Array(
                candidates.iter().map(|(_, d)| d.clone()).collect(),
            ),
        };
        let mut row = root.clone();
        match &mut row {
            JsonValue::Object(map) => {
                map.insert(step.output.clone(), attached);
            }
            _ => {
                return Err(Error::QueryInvalid(
                    "enrich attach requires object root documents".into(),
                ));
            }
        }
        out.push((key.clone(), row));
    }
    Ok(out)
}

/// Apply nested enrich / within steps to each element of a carrier array.
///
/// `foreign_by_using` maps collection name → complete foreign docs for that
/// using-collection. Absent / Null / non-array carriers fail with
/// [`DIAG_RQL_WITHIN_TYPE`] (never treated as empty bags).
pub fn attach_within_rows(
    roots: &[(String, JsonValue)],
    foreign_by_using: &BTreeMap<String, Vec<(String, JsonValue)>>,
    step: &WithinStepV1,
    params: &BTreeMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, Error> {
    let mut out = Vec::with_capacity(roots.len());
    for (key, root) in roots {
        let carrier = match resolve_path(root, &step.carrier) {
            Resolve::Present(JsonValue::Array(arr)) => arr,
            Resolve::Present(other) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_WITHIN_TYPE}: path `{}` is {} on key `{key}` (need sequence/bag)",
                    step.carrier.dotted(),
                    json_type_name(&other)
                )));
            }
            Resolve::Absent => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_WITHIN_TYPE}: path `{}` absent on key `{key}`",
                    step.carrier.dotted()
                )));
            }
        };
        let mut elements: Vec<(String, JsonValue)> = carrier
            .iter()
            .enumerate()
            .map(|(i, el)| (format!("{key}#{i}"), el.clone()))
            .collect();
        for nested in &step.steps {
            match nested {
                FullPipelineStepV1::Enrich(enrich) => {
                    let foreign = foreign_by_using.get(&enrich.using_name).ok_or_else(|| {
                        Error::QueryInvalid(format!(
                            "within attach missing foreign docs for `{}`",
                            enrich.using_name
                        ))
                    })?;
                    elements = attach_enrich_rows(&elements, foreign, enrich, params)?;
                }
                FullPipelineStepV1::Within(inner) => {
                    elements =
                        attach_within_rows(&elements, foreign_by_using, inner, params)?;
                }
                FullPipelineStepV1::Filter(pred) => {
                    elements = filter_rows(&elements, pred, params)?;
                }
            }
        }
        let new_arr: Vec<JsonValue> = elements.into_iter().map(|(_, v)| v).collect();
        let mut row = root.clone();
        set_at_path(&mut row, &step.carrier, JsonValue::Array(new_arr))?;
        out.push((key.clone(), row));
    }
    Ok(out)
}

fn json_type_name(v: &JsonValue) -> &'static str {
    match v {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}

fn set_at_path(doc: &mut JsonValue, path: &Path, value: JsonValue) -> Result<(), Error> {
    if path.0.is_empty() {
        return Err(Error::QueryInvalid("empty path for within replace".into()));
    }
    let mut cur = doc;
    for (i, seg) in path.0.iter().enumerate() {
        if i + 1 == path.0.len() {
            match cur {
                JsonValue::Object(map) => {
                    map.insert(seg.clone(), value);
                    return Ok(());
                }
                _ => {
                    return Err(Error::QueryInvalid(format!(
                        "within replace requires object at parent of `{}`",
                        path.dotted()
                    )));
                }
            }
        }
        match cur {
            JsonValue::Object(map) => {
                cur = map.get_mut(seg).ok_or_else(|| {
                    Error::QueryInvalid(format!(
                        "within replace missing segment `{seg}` on `{}`",
                        path.dotted()
                    ))
                })?;
            }
            _ => {
                return Err(Error::QueryInvalid(format!(
                    "within replace requires object along `{}`",
                    path.dotted()
                )));
            }
        }
    }
    Err(Error::QueryInvalid("within replace unreachable".into()))
}

/// Result of [`execute_rql_full`] (one base page + enrich/within attach).
#[derive(Debug, Clone, PartialEq)]
pub struct RqlFullPage {
    /// Profile label.
    pub profile: &'static str,
    /// Rows after pipeline (+ optional project).
    pub rows: Vec<(String, JsonValue)>,
    /// Underlying Application Core page (pre-enrich values).
    pub base: QueryPage,
    /// Compiled ordered pipeline applied after the base page.
    pub pipeline: Vec<FullPipelineStepV1>,
    /// Nested project applied after the pipeline, if any.
    pub project: Option<Vec<ProjectItemV1>>,
    /// Per root-enrich load mode (RQL-I1). Within nested enrich omitted here
    /// (still scan-loaded; see residual inventory).
    pub enrich_loads: Vec<EnrichLoadEvidence>,
}

impl RqlFullPage {
    /// Root enrich steps in pipeline order.
    pub fn enrich(&self) -> Vec<&EnrichStepV1> {
        self.pipeline
            .iter()
            .filter_map(|s| match s {
                FullPipelineStepV1::Enrich(e) => Some(e),
                FullPipelineStepV1::Within(_) | FullPipelineStepV1::Filter(_) => None,
            })
            .collect()
    }

    /// First root `within` step, if any.
    pub fn within(&self) -> Option<&WithinStepV1> {
        self.pipeline.iter().find_map(|s| match s {
            FullPipelineStepV1::Within(w) => Some(w),
            FullPipelineStepV1::Enrich(_) | FullPipelineStepV1::Filter(_) => None,
        })
    }
}

/// Façade: compile full RQL → ISA → decode → execute (RQL-X5b).
///
/// Discovers collection bindings from [`HeapClient::list_collections`].
/// Root `enrich` tries equality-index pushdown on the foreign match field
/// (RQL-I1); falls back to complete `list_keys`+`get` when no usable index.
/// Nested `within` enrich still scan-loads. Multipage: call again with
/// `options.after` from `base.next`.
pub fn execute_rql_full(
    client: &mut HeapClient,
    source: &str,
    parameters: &Parameters,
    options: QueryRunOptions,
) -> Result<RqlFullPage, Error> {
    execute_rql_full_with(
        client,
        source,
        parameters,
        RqlFullExecuteOptions {
            query: options,
            force_enrich_scan: false,
        },
    )
}

/// [`execute_rql_full`] with differential-oracle controls.
///
/// Authority is ISA: compile lowers to bytes, then [`execute_full_isa_with`]
/// decodes and runs. [`CompiledRqlFull`] is not an executable sidecar.
pub fn execute_rql_full_with(
    client: &mut HeapClient,
    source: &str,
    parameters: &Parameters,
    options: RqlFullExecuteOptions,
) -> Result<RqlFullPage, Error> {
    let infos = client.list_collections()?;
    let mut bindings = CollectionBindings::default();
    for info in &infos {
        bindings.bind(&info.name, info.collection_id);
    }
    let compiled = compile_rql_full(source, &bindings)?;
    let isa = encode_full_program(
        &compiled.base.plan,
        compiled.base.budget,
        &compiled.pipeline,
        &compiled.project,
    )?;
    execute_full_isa_with(client, &isa, parameters, options)
}

/// Full-language entry: decode ISA (must carry full section), then execute.
///
/// Base page uses Core [`execute_isa_bytes`] on a Core-only re-encode of the
/// decoded plan. Attach pipeline/project come only from the decoded full section.
pub fn execute_full_isa_with(
    client: &mut HeapClient,
    isa_bytes: &[u8],
    parameters: &Parameters,
    options: RqlFullExecuteOptions,
) -> Result<RqlFullPage, Error> {
    let prog = decode_isa(isa_bytes)?;
    if prog.profile != ISA_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "execute_full_isa: profile mismatch: got {:?}, want {ISA_PROFILE}",
            prog.profile
        )));
    }
    let full = prog.full.as_ref().ok_or_else(|| {
        Error::QueryInvalid(
            "execute_full_isa: Core-only ISA; use execute_isa_bytes".into(),
        )
    })?;

    let from_name = prog.core.from.source_name.clone();
    let mut base_col = client.open_collection(&from_name)?;
    let heap_id = base_col.heap_id();
    let collection_id = base_col.id();

    // Shared Core entry: ISA bytes → decode → page (not base_source / compile).
    let core_isa = encode_core_program(&prog.core, prog.budget)?;
    let page = execute_isa_bytes(
        &mut base_col,
        &core_isa,
        &parameters.values,
        &options.query,
        heap_id,
        collection_id,
    )?;

    let pipeline = full.pipeline.clone();
    let project = full.project.clone();

    let mut rows: Vec<(String, JsonValue)> = page
        .rows
        .iter()
        .map(|r| (r.key.clone(), r.value.clone()))
        .collect();

    let mut foreign_cache: BTreeMap<String, Vec<(String, JsonValue)>> = BTreeMap::new();
    let mut enrich_loads = Vec::new();
    for step in &pipeline {
        match step {
            FullPipelineStepV1::Enrich(e) => {
                let (foreign, mode) = load_foreign_docs_for_root_enrich(
                    client,
                    e,
                    &rows,
                    options.force_enrich_scan,
                )?;
                enrich_loads.push(EnrichLoadEvidence {
                    using: e.using_name.clone(),
                    output: e.output.clone(),
                    mode,
                });
                // Index hits are step-local; do not poison the within scan cache
                // with a partial collection view under the same using-name.
                if mode == EnrichAttachMode::Scan {
                    foreign_cache
                        .entry(e.using_name.clone())
                        .or_insert_with(|| foreign.clone());
                }
                rows = attach_enrich_rows(&rows, &foreign, e, &parameters.values)?;
            }
            FullPipelineStepV1::Within(w) => {
                collect_within_using_names(w, &mut foreign_cache, client)?;
                rows = attach_within_rows(&rows, &foreign_cache, w, &parameters.values)?;
            }
            FullPipelineStepV1::Filter(pred) => {
                rows = filter_rows(&rows, pred, &parameters.values)?;
            }
        }
    }

    if let Some(fields) = &project {
        rows = apply_project_rows(&rows, fields)?;
    }

    Ok(RqlFullPage {
        profile: RQL_FULL_PROFILE,
        rows,
        base: page,
        pipeline,
        project,
        enrich_loads,
    })
}

/// Load foreign docs for a **root** enrich: equality index when usable, else scan.
fn load_foreign_docs_for_root_enrich(
    client: &mut HeapClient,
    step: &EnrichStepV1,
    roots: &[(String, JsonValue)],
    force_scan: bool,
) -> Result<(Vec<(String, JsonValue)>, EnrichAttachMode), Error> {
    if force_scan {
        return Ok((
            load_collection_docs(client, &step.using_name)?,
            EnrichAttachMode::Scan,
        ));
    }
    let right_field = step.right.dotted();
    // Equality indexes today are single-field path labels (APB-7 T4).
    if right_field.is_empty() || right_field.contains('.') {
        return Ok((
            load_collection_docs(client, &step.using_name)?,
            EnrichAttachMode::Scan,
        ));
    }

    let left_values = collect_present_left_values(roots, &step.left);
    let mut col = client.open_collection(&step.using_name)?;

    if left_values.is_empty() {
        // Probe whether a usable equality index exists without loading all docs.
        match col.lookup_index_keys(&[(right_field.clone(), JsonValue::Null)])? {
            None => {
                drop(col);
                return Ok((
                    load_collection_docs(client, &step.using_name)?,
                    EnrichAttachMode::Scan,
                ));
            }
            Some(_) => return Ok((Vec::new(), EnrichAttachMode::EqualityIndex)),
        }
    }

    let mut by_key: BTreeMap<String, JsonValue> = BTreeMap::new();
    for val in left_values.values() {
        match col.lookup_index_keys(&[(right_field.clone(), val.clone())])? {
            None => {
                drop(col);
                return Ok((
                    load_collection_docs(client, &step.using_name)?,
                    EnrichAttachMode::Scan,
                ));
            }
            Some(keys) => {
                for k in keys {
                    if by_key.contains_key(&k) {
                        continue;
                    }
                    if let Some(doc) = col.get(&k)? {
                        by_key.insert(k, doc);
                    }
                }
            }
        }
    }
    Ok((
        by_key.into_iter().collect(),
        EnrichAttachMode::EqualityIndex,
    ))
}

fn collect_present_left_values(
    roots: &[(String, JsonValue)],
    left: &Path,
) -> BTreeMap<String, JsonValue> {
    let mut out = BTreeMap::new();
    for (_, root) in roots {
        if let Resolve::Present(v) = resolve_path(root, left) {
            out.insert(canonical_match_key(&v), v);
        }
    }
    out
}

fn ensure_foreign_docs(
    client: &mut HeapClient,
    name: &str,
    cache: &mut BTreeMap<String, Vec<(String, JsonValue)>>,
) -> Result<(), Error> {
    if !cache.contains_key(name) {
        let docs = load_collection_docs(client, name)?;
        cache.insert(name.to_string(), docs);
    }
    Ok(())
}

fn collect_within_using_names(
    step: &WithinStepV1,
    cache: &mut BTreeMap<String, Vec<(String, JsonValue)>>,
    client: &mut HeapClient,
) -> Result<(), Error> {
    for nested in &step.steps {
        match nested {
            FullPipelineStepV1::Enrich(e) => {
                ensure_foreign_docs(client, &e.using_name, cache)?;
            }
            FullPipelineStepV1::Within(w) => {
                collect_within_using_names(w, cache, client)?;
            }
            FullPipelineStepV1::Filter(_) => {}
        }
    }
    Ok(())
}

fn load_collection_docs(
    client: &mut HeapClient,
    name: &str,
) -> Result<Vec<(String, JsonValue)>, Error> {
    let mut col = client.open_collection(name)?;
    let mut foreign = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let batch = col.list_keys(Some(256), after.as_deref())?;
        if batch.is_empty() {
            break;
        }
        for k in &batch {
            if let Some(v) = col.get(k)? {
                foreign.push((k.clone(), v));
            }
        }
        after = batch.last().cloned();
        if batch.len() < 256 {
            break;
        }
    }
    Ok(foreign)
}

fn canonical_match_key(v: &JsonValue) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| format!("{v:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rql_app_core::DIAG_RQL_FEATURE_UNAVAILABLE;
    use residiuum_heap::CollectionId;
    use std::str::FromStr;

    fn bindings() -> CollectionBindings {
        let mut b = CollectionBindings::default();
        b.bind(
            "orders",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a1").unwrap(),
        );
        b.bind(
            "customers",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a2").unwrap(),
        );
        b.bind(
            "line_items",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a3").unwrap(),
        );
        b.bind(
            "products",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a4").unwrap(),
        );
        b
    }

    #[test]
    fn core_still_rejects_enrich() {
        let err = compile_app_core(
            "from orders enrich customer using customers matching customer_id = id expect exactly_one",
            &bindings(),
        )
        .unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_FEATURE_UNAVAILABLE));
    }

    #[test]
    fn compile_single_enrich() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id expect exactly_one
               page size 10"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.profile, RQL_FULL_PROFILE);
        assert_eq!(c.root_enrich().len(), 1);
        assert_eq!(c.root_enrich()[0].output, "customer");
        assert_eq!(c.root_enrich()[0].expect, EnrichCardinality::ExactlyOne);
        assert!(c.first_within().is_none());
        assert!(c.base_source.contains("from orders"));
        assert!(!c.base_source.contains("enrich"));
        assert_eq!(c.base.plan.page_size, 10);
    }

    #[test]
    fn compile_and_attach_within() {
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
               }"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.root_enrich().len(), 1);
        let w = c.first_within().unwrap();
        assert_eq!(w.carrier.dotted(), "items");
        assert_eq!(w.element_alias.as_deref(), Some("item"));
        assert_eq!(w.enrich_steps().len(), 1);
        assert_eq!(w.enrich_steps()[0].output, "product");
        assert_eq!(w.enrich_steps()[0].left.dotted(), "product_id");
        assert_eq!(w.enrich_steps()[0].right.dotted(), "id");

        let roots = vec![(
            "o1".into(),
            serde_json::json!({"order_id": "o1"}),
        )];
        let lines = vec![
            (
                "l1".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p1", "sku": "A"}),
            ),
            (
                "l2".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p2", "sku": "B"}),
            ),
        ];
        let products = vec![
            ("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"})),
            ("p2".into(), serde_json::json!({"id": "p2", "name": "Gadget"})),
        ];
        let after_many =
            attach_enrich_rows(&roots, &lines, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let bag = out[0].1["items"].as_array().unwrap();
        assert_eq!(bag.len(), 2);
        assert_eq!(bag[0]["product"]["name"], "Widget");
        assert_eq!(bag[1]["product"]["name"], "Gadget");
    }

    #[test]
    fn compile_and_attach_multi_root_enrich() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id expect exactly_one
               enrich items using line_items matching order_id = order_id expect many"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.root_enrich().len(), 2);
        assert_eq!(c.root_enrich()[0].output, "customer");
        assert_eq!(c.root_enrich()[1].output, "items");

        let roots = vec![(
            "o1".into(),
            serde_json::json!({"order_id": "o1", "customer_id": "c1"}),
        )];
        let customers = vec![("c1".into(), serde_json::json!({"id": "c1", "name": "Ada"}))];
        let lines = vec![
            ("l1".into(), serde_json::json!({"order_id": "o1", "sku": "A"})),
            ("l2".into(), serde_json::json!({"order_id": "o1", "sku": "B"})),
        ];
        let mid =
            attach_enrich_rows(&roots, &customers, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let out = attach_enrich_rows(&mid, &lines, c.root_enrich()[1], &BTreeMap::new()).unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");
        assert_eq!(out[0].1["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn compile_and_attach_multi_within_enrich() {
        let mut b = bindings();
        b.bind(
            "warehouses",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a5").unwrap(),
        );
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
                 enrich warehouse using warehouses as wh
                   matching item.warehouse_id = wh.id
                   expect optional
               }"#,
            &b,
        )
        .unwrap();
        let w = c.first_within().unwrap();
        assert_eq!(w.enrich_steps().len(), 2);
        assert_eq!(w.enrich_steps()[0].output, "product");
        assert_eq!(w.enrich_steps()[1].output, "warehouse");

        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let lines = vec![(
            "l1".into(),
            serde_json::json!({
                "order_id": "o1",
                "product_id": "p1",
                "warehouse_id": "w1"
            }),
        )];
        let products = vec![("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"}))];
        let warehouses = vec![("w1".into(), serde_json::json!({"id": "w1", "city": "Oslo"}))];
        let after_many =
            attach_enrich_rows(&roots, &lines, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        foreign.insert("warehouses".into(), warehouses);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let item = &out[0].1["items"].as_array().unwrap()[0];
        assert_eq!(item["product"]["name"], "Widget");
        assert_eq!(item["warehouse"]["city"], "Oslo");
    }

    #[test]
    fn within_type_error_on_absent() {
        let step = WithinStepV1 {
            carrier: Path::parse_dotted("items").unwrap(),
            element_alias: None,
            steps: vec![FullPipelineStepV1::Enrich(EnrichStepV1 {
                output: "product".into(),
                using_name: "products".into(),
                using_id: CollectionId::from_str("00000000-0000-4000-8000-0000000000a4")
                    .unwrap(),
                left: Path::parse_dotted("product_id").unwrap(),
                right: Path::parse_dotted("id").unwrap(),
                candidate_where: None,
                expect: EnrichCardinality::ExactlyOne,
            })],
        };
        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let foreign = BTreeMap::new();
        let err = attach_within_rows(&roots, &foreign, &step, &BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_WITHIN_TYPE));
    }

    #[test]
    fn compile_and_attach_nested_within() {
        let mut b = bindings();
        b.bind(
            "components",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a6").unwrap(),
        );
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich parts using components matching item.sku = parent_sku expect many
                 within item.parts as part {
                   enrich product using products as candidate
                     matching part.product_id = candidate.id
                     expect exactly_one
                 }
               }"#,
            &b,
        )
        .unwrap();
        let w = c.first_within().unwrap();
        assert_eq!(w.steps.len(), 2);
        assert!(matches!(&w.steps[0], FullPipelineStepV1::Enrich(_)));
        assert!(matches!(&w.steps[1], FullPipelineStepV1::Within(_)));
        let FullPipelineStepV1::Within(inner) = &w.steps[1] else {
            panic!("expected nested within");
        };
        assert_eq!(inner.carrier.dotted(), "parts");
        assert_eq!(inner.enrich_steps()[0].left.dotted(), "product_id");

        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let lines = vec![(
            "l1".into(),
            serde_json::json!({"order_id": "o1", "sku": "A"}),
        )];
        let components = vec![
            (
                "c1".into(),
                serde_json::json!({"parent_sku": "A", "product_id": "p1"}),
            ),
            (
                "c2".into(),
                serde_json::json!({"parent_sku": "A", "product_id": "p2"}),
            ),
        ];
        let products = vec![
            ("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"})),
            ("p2".into(), serde_json::json!({"id": "p2", "name": "Gadget"})),
        ];
        let after_many =
            attach_enrich_rows(&roots, &lines, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("components".into(), components);
        foreign.insert("products".into(), products);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let parts = out[0].1["items"].as_array().unwrap()[0]["parts"]
            .as_array()
            .unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["product"]["name"], "Widget");
        assert_eq!(parts[1]["product"]["name"], "Gadget");
    }

    #[test]
    fn compile_and_attach_enrich_after_within() {
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
               }
               enrich customer using customers matching customer_id = id expect exactly_one"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.pipeline.len(), 3);
        assert!(matches!(&c.pipeline[0], FullPipelineStepV1::Enrich(_)));
        assert!(matches!(&c.pipeline[1], FullPipelineStepV1::Within(_)));
        assert!(matches!(&c.pipeline[2], FullPipelineStepV1::Enrich(_)));
        assert_eq!(c.root_enrich().len(), 2);
        assert_eq!(c.root_enrich()[1].output, "customer");

        let roots = vec![(
            "o1".into(),
            serde_json::json!({"order_id": "o1", "customer_id": "c1"}),
        )];
        let lines = vec![(
            "l1".into(),
            serde_json::json!({"order_id": "o1", "product_id": "p1"}),
        )];
        let products = vec![("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"}))];
        let customers = vec![("c1".into(), serde_json::json!({"id": "c1", "name": "Ada"}))];
        let mid =
            attach_enrich_rows(&roots, &lines, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        let after_within =
            attach_within_rows(&mid, &foreign, c.first_within().unwrap(), &BTreeMap::new())
                .unwrap();
        let out =
            attach_enrich_rows(&after_within, &customers, c.root_enrich()[1], &BTreeMap::new())
                .unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");
        assert_eq!(
            out[0].1["items"].as_array().unwrap()[0]["product"]["name"],
            "Widget"
        );
    }

    #[test]
    fn refuse_within_depth_overflow() {
        let mut nested = String::from("enrich x using products matching a = b expect optional");
        for _ in 0..MAX_WITHIN_DEPTH {
            nested = format!("within nest {{ {nested} }}");
        }
        // depth 1..MAX are ok; one more exceeds
        nested = format!("within nest {{ {nested} }}");
        let src = format!("from orders {nested}");
        let err = compile_rql_full(&src, &bindings()).unwrap_err();
        assert!(
            err.to_string().contains(DIAG_RQL_FULL_RESIDUAL),
            "unexpected: {err}"
        );
        assert!(err.to_string().contains("depth exceeds"));
    }

    #[test]
    fn compile_and_attach_nested_where() {
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 where item.qty > 1
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
               }"#,
            &bindings(),
        )
        .unwrap();
        let w = c.first_within().unwrap();
        assert_eq!(w.steps.len(), 2);
        assert!(matches!(&w.steps[0], FullPipelineStepV1::Filter(_)));
        assert!(matches!(&w.steps[1], FullPipelineStepV1::Enrich(_)));

        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let lines = vec![
            (
                "l1".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p1", "qty": 1}),
            ),
            (
                "l2".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p2", "qty": 3}),
            ),
        ];
        let products = vec![
            ("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"})),
            ("p2".into(), serde_json::json!({"id": "p2", "name": "Gadget"})),
        ];
        let after_many =
            attach_enrich_rows(&roots, &lines, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let bag = out[0].1["items"].as_array().unwrap();
        assert_eq!(bag.len(), 1);
        assert_eq!(bag[0]["qty"], 3);
        assert_eq!(bag[0]["product"]["name"], "Gadget");
    }

    #[test]
    fn compile_and_attach_where_after_enrich_in_within() {
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
                 where item.product.name = "Widget"
               }"#,
            &bindings(),
        )
        .unwrap();
        let w = c.first_within().unwrap();
        assert!(matches!(&w.steps[0], FullPipelineStepV1::Enrich(_)));
        assert!(matches!(&w.steps[1], FullPipelineStepV1::Filter(_)));

        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let lines = vec![
            (
                "l1".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p1"}),
            ),
            (
                "l2".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p2"}),
            ),
        ];
        let products = vec![
            ("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"})),
            ("p2".into(), serde_json::json!({"id": "p2", "name": "Gadget"})),
        ];
        let after_many =
            attach_enrich_rows(&roots, &lines, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let bag = out[0].1["items"].as_array().unwrap();
        assert_eq!(bag.len(), 1);
        assert_eq!(bag[0]["product"]["name"], "Widget");
    }

    #[test]
    fn compile_and_attach_root_where_after_enrich() {
        let c = compile_rql_full(
            r#"from orders
               where status = "paid"
               enrich customer using customers matching customer_id = id expect exactly_one
               where customer.country = "TH"
               page size 10"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.pipeline.len(), 2);
        assert!(matches!(&c.pipeline[0], FullPipelineStepV1::Enrich(_)));
        assert!(matches!(&c.pipeline[1], FullPipelineStepV1::Filter(_)));
        assert!(c.base_source.contains("where status"));
        assert!(!c.base_source.contains("customer.country"));

        let roots = vec![
            (
                "o1".into(),
                serde_json::json!({"customer_id": "c1", "status": "paid"}),
            ),
            (
                "o2".into(),
                serde_json::json!({"customer_id": "c2", "status": "paid"}),
            ),
        ];
        let customers = vec![
            (
                "c1".into(),
                serde_json::json!({"id": "c1", "country": "TH", "name": "Ada"}),
            ),
            (
                "c2".into(),
                serde_json::json!({"id": "c2", "country": "US", "name": "Bob"}),
            ),
        ];
        let mid =
            attach_enrich_rows(&roots, &customers, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let FullPipelineStepV1::Filter(pred) = &c.pipeline[1] else {
            panic!("expected filter");
        };
        let out = filter_rows(&mid, pred, &BTreeMap::new()).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].1["customer"]["name"], "Ada");
    }

    #[test]
    fn compile_and_apply_nested_project() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id expect exactly_one
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
               }
               project {
                 order_id,
                 customer { name },
                 items { sku, product { name } }
               }"#,
            &bindings(),
        )
        .unwrap();
        let proj = c.project.as_ref().unwrap();
        assert_eq!(proj.len(), 3);
        assert!(c.base_source.contains("from orders"));
        assert!(!c.base_source.contains("project"));

        let roots = vec![(
            "o1".into(),
            serde_json::json!({"order_id": "o1", "customer_id": "c1"}),
        )];
        let customers = vec![("c1".into(), serde_json::json!({"id": "c1", "name": "Ada"}))];
        let lines = vec![(
            "l1".into(),
            serde_json::json!({"order_id": "o1", "product_id": "p1", "sku": "A"}),
        )];
        let products = vec![("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"}))];
        let mid =
            attach_enrich_rows(&roots, &customers, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        let mid2 =
            attach_enrich_rows(&mid, &lines, c.root_enrich()[1], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        let enriched =
            attach_within_rows(&mid2, &foreign, c.first_within().unwrap(), &BTreeMap::new())
                .unwrap();
        let out = apply_project_rows(&enriched, proj).unwrap();
        assert_eq!(out[0].1["order_id"], "o1");
        assert_eq!(out[0].1["customer"]["name"], "Ada");
        assert!(out[0].1["customer"].get("id").is_none());
        let item = &out[0].1["items"].as_array().unwrap()[0];
        assert_eq!(item["sku"], "A");
        assert_eq!(item["product"]["name"], "Widget");
        assert!(item.get("product_id").is_none());
    }

    #[test]
    fn project_conflict_and_type_error() {
        let err = compile_rql_full(
            r#"from orders project { id, id }"#,
            &bindings(),
        )
        .unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_PROJECTION_CONFLICT));

        let fields = vec![ProjectItemV1::Nested {
            output: "status".into(),
            fields: vec![ProjectItemV1::Leaf {
                output: "x".into(),
                source: Path::parse_dotted("x").unwrap(),
            }],
        }];
        let rows = vec![("o1".into(), serde_json::json!({"status": "paid"}))];
        let err = apply_project_rows(&rows, &fields).unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_PROJECT_TYPE));
    }

    #[test]
    fn compile_and_attach_many() {
        let c = compile_rql_full(
            "from orders enrich items using line_items matching order_id = order_id expect many",
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.root_enrich()[0].expect, EnrichCardinality::Many);

        let step = c.root_enrich()[0];
        let roots = vec![
            ("o1".into(), serde_json::json!({"order_id": "o1"})),
            ("o2".into(), serde_json::json!({"order_id": "o2"})),
        ];
        let foreign = vec![
            ("l2".into(), serde_json::json!({"order_id": "o1", "sku": "B"})),
            ("l1".into(), serde_json::json!({"order_id": "o1", "sku": "A"})),
            ("l3".into(), serde_json::json!({"order_id": "o2", "sku": "C"})),
        ];
        let out = attach_enrich_rows(&roots, &foreign, step, &BTreeMap::new()).unwrap();
        let bag = out[0].1["items"].as_array().unwrap();
        assert_eq!(bag.len(), 2);
        assert_eq!(bag[0]["sku"], "A");
        assert_eq!(bag[1]["sku"], "B");
        assert_eq!(out[1].1["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn attach_exactly_one_and_optional() {
        let step = EnrichStepV1 {
            output: "customer".into(),
            using_name: "customers".into(),
            using_id: CollectionId::from_str("00000000-0000-4000-8000-0000000000a2").unwrap(),
            left: Path::parse_dotted("customer_id").unwrap(),
            right: Path::parse_dotted("id").unwrap(),
            candidate_where: None,
            expect: EnrichCardinality::ExactlyOne,
        };
        let roots = vec![
            (
                "o1".into(),
                serde_json::json!({"customer_id": "c1", "n": 1}),
            ),
        ];
        let foreign = vec![
            ("c1".into(), serde_json::json!({"id": "c1", "name": "Ada"})),
            ("c2".into(), serde_json::json!({"id": "c2", "name": "Bob"})),
        ];
        let out = attach_enrich_rows(&roots, &foreign, &step, &BTreeMap::new()).unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");

        let mut opt = step.clone();
        opt.expect = EnrichCardinality::Optional;
        let roots2 = vec![("o2".into(), serde_json::json!({"customer_id": "missing"}))];
        let out2 = attach_enrich_rows(&roots2, &foreign, &opt, &BTreeMap::new()).unwrap();
        assert!(out2[0].1["customer"].is_null());
    }

    #[test]
    fn compile_and_attach_candidate_where() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id
               where active = true
               expect exactly_one"#,
            &bindings(),
        )
        .unwrap();
        assert!(c.root_enrich()[0].candidate_where.is_some());

        let roots = vec![("o1".into(), serde_json::json!({"customer_id": "c1"}))];
        let foreign = vec![
            (
                "c1a".into(),
                serde_json::json!({"id": "c1", "active": false, "name": "old"}),
            ),
            (
                "c1b".into(),
                serde_json::json!({"id": "c1", "active": true, "name": "Ada"}),
            ),
        ];
        let out =
            attach_enrich_rows(&roots, &foreign, c.root_enrich()[0], &BTreeMap::new()).unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");
    }
}
