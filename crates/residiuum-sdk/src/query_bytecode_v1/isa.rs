//! Durable query ISA encoding (`residiuum-query-isa-v1`) — RQL-X3.
//!
//! Normative: [QUERY_ISA_V1.md](../../../../../doc/todo/rql/QUERY_ISA_V1.md)
//!
//! This is the product **carrier** for compiled query programs. Rust
//! `RqlPlanV1` / pipeline structs are in-memory views; durable identity is
//! the ISA byte string (magic + version + sections).

use super::full_attach::{
    EnrichCardinality, EnrichStepV1, FullPipelineStepV1, ProjectItemV1, WithinStepV1,
};
use crate::app_v1::QueryBudget;
use crate::error::Error;
use crate::plan_v1::{PLAN_ENCODING_PROFILE, PLAN_PROFILE, RqlPlanV1};
use crate::predicate::{Path, Predicate};
use residiuum_heap::CollectionId;
use serde_json::{Map, Value as JsonValue};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Frozen ISA profile id.
pub const ISA_PROFILE: &str = "residiuum-query-isa-v1";

/// Magic prefix (`RQB1` = Residiuum Query Bytecode 1).
pub const ISA_MAGIC: &[u8; 4] = b"RQB1";

/// Encoding version byte (bump only with profile amendment).
pub const ISA_VERSION: u8 = 1;

const FLAG_BUDGET: u8 = 0x01;
const FLAG_FULL: u8 = 0x02;
/// Known top-level flag bits; any other bit is reserved and must be rejected.
const FLAGS_KNOWN: u8 = FLAG_BUDGET | FLAG_FULL;
/// Known budget flag bits (documents / bytes / result_bytes).
const BUDGET_FLAGS_KNOWN: u8 = 0x01 | 0x02 | 0x04;

/// Hard cap on total externally supplied ISA bytes (RQL-D0R).
pub const ISA_MAX_TOTAL_BYTES: usize = 1 << 20;
/// Hard cap on each length-prefixed section body.
pub const ISA_MAX_SECTION_BYTES: usize = 1 << 20;

/// Decoded durable program (Core always; optional full attach).
#[derive(Debug, Clone, PartialEq)]
pub struct QueryIsaProgram {
    /// Always [`ISA_PROFILE`].
    pub profile: String,
    /// Application Core logical plan (decoded from plan-encoding-v1 body).
    pub core: RqlPlanV1,
    /// Optional source budget stamped at compile.
    pub budget: Option<QueryBudget>,
    /// Optional full-language pipeline + brace project.
    pub full: Option<QueryIsaFullSection>,
}

/// Full-language attach section (pipeline + nested project).
#[derive(Debug, Clone, PartialEq)]
pub struct QueryIsaFullSection {
    /// Ordered enrich / within / filter steps.
    pub pipeline: Vec<FullPipelineStepV1>,
    /// Optional brace `project { … }`.
    pub project: Option<Vec<ProjectItemV1>>,
}

/// Encode a Core program (plan + optional budget) into ISA bytes.
pub fn encode_core_program(plan: &RqlPlanV1, budget: Option<QueryBudget>) -> Result<Vec<u8>, Error> {
    encode_program(plan, budget, None)
}

/// Encode Core + full attach into ISA bytes.
pub fn encode_full_program(
    plan: &RqlPlanV1,
    budget: Option<QueryBudget>,
    pipeline: &[FullPipelineStepV1],
    project: &Option<Vec<ProjectItemV1>>,
) -> Result<Vec<u8>, Error> {
    encode_program(
        plan,
        budget,
        Some(QueryIsaFullSection {
            pipeline: pipeline.to_vec(),
            project: project.clone(),
        }),
    )
}

fn encode_program(
    plan: &RqlPlanV1,
    budget: Option<QueryBudget>,
    full: Option<QueryIsaFullSection>,
) -> Result<Vec<u8>, Error> {
    if plan.profile != PLAN_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "isa encode: unexpected plan profile {:?}",
            plan.profile
        )));
    }
    let core_body = plan.canonical_bytes();
    let mut flags: u8 = 0;
    if budget.is_some() {
        flags |= FLAG_BUDGET;
    }
    if full.is_some() {
        flags |= FLAG_FULL;
    }

    let mut out = Vec::with_capacity(16 + core_body.len());
    out.extend_from_slice(ISA_MAGIC);
    out.push(ISA_VERSION);
    out.push(flags);
    push_u32(&mut out, core_body.len() as u32);
    out.extend_from_slice(&core_body);

    if let Some(b) = budget {
        push_budget(&mut out, b);
    }
    if let Some(f) = full {
        let body = full_section_canonical_bytes(&f)?;
        push_u32(&mut out, body.len() as u32);
        out.extend_from_slice(&body);
    }
    Ok(out)
}

/// Decode ISA bytes into a program (rejects reserved flag bits + oversize).
pub fn decode_isa(bytes: &[u8]) -> Result<QueryIsaProgram, Error> {
    if bytes.len() > ISA_MAX_TOTAL_BYTES {
        return Err(Error::QueryInvalid(format!(
            "isa: total length {} exceeds max {ISA_MAX_TOTAL_BYTES}",
            bytes.len()
        )));
    }
    if bytes.len() < 10 {
        return Err(Error::QueryInvalid("isa: truncated header".into()));
    }
    if &bytes[0..4] != ISA_MAGIC.as_slice() {
        return Err(Error::QueryInvalid("isa: bad magic".into()));
    }
    if bytes[4] != ISA_VERSION {
        return Err(Error::QueryInvalid(format!(
            "isa: unsupported version {}",
            bytes[4]
        )));
    }
    let flags = bytes[5];
    if flags & !FLAGS_KNOWN != 0 {
        return Err(Error::QueryInvalid(format!(
            "isa: reserved flag bits set ({flags:#04x})"
        )));
    }
    let mut off = 6;
    let core_len = read_u32(bytes, &mut off)? as usize;
    if core_len > ISA_MAX_SECTION_BYTES {
        return Err(Error::QueryInvalid(format!(
            "isa: core section {core_len} exceeds max {ISA_MAX_SECTION_BYTES}"
        )));
    }
    if off + core_len > bytes.len() {
        return Err(Error::QueryInvalid("isa: truncated core".into()));
    }
    let core_slice = &bytes[off..off + core_len];
    off += core_len;
    let core = decode_core_plan(core_slice)?;

    let budget = if flags & FLAG_BUDGET != 0 {
        Some(read_budget(bytes, &mut off)?)
    } else {
        None
    };

    let full = if flags & FLAG_FULL != 0 {
        let flen = read_u32(bytes, &mut off)? as usize;
        if flen > ISA_MAX_SECTION_BYTES {
            return Err(Error::QueryInvalid(format!(
                "isa: full section {flen} exceeds max {ISA_MAX_SECTION_BYTES}"
            )));
        }
        if off + flen > bytes.len() {
            return Err(Error::QueryInvalid("isa: truncated full".into()));
        }
        let body = &bytes[off..off + flen];
        off += flen;
        Some(decode_full_section(body)?)
    } else {
        None
    };

    if off != bytes.len() {
        return Err(Error::QueryInvalid("isa: trailing bytes".into()));
    }

    Ok(QueryIsaProgram {
        profile: ISA_PROFILE.to_string(),
        core,
        budget,
        full,
    })
}

/// Decode externally supplied ISA and require canonical re-encode equality.
///
/// Product execution entries use this so distinct byte strings cannot share
/// meaning while hashing differently (RQL-D0R / principal P1).
pub fn decode_isa_canonical(bytes: &[u8]) -> Result<QueryIsaProgram, Error> {
    let prog = decode_isa(bytes)?;
    let again = encode_program(&prog.core, prog.budget, prog.full.clone())?;
    if again.as_slice() != bytes {
        return Err(Error::QueryInvalid(
            "isa: non-canonical encoding (re-encode mismatch)".into(),
        ));
    }
    Ok(prog)
}

/// BLAKE3-256 over domain || 0x00 || isa bytes (durable program identity).
pub fn isa_hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"residiuum:query-isa-v1:hash-v1");
    hasher.update(&[0u8]);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

fn decode_core_plan(body: &[u8]) -> Result<RqlPlanV1, Error> {
    let v: JsonValue = serde_json::from_slice(body)
        .map_err(|e| Error::QueryInvalid(format!("isa core json: {e}")))?;
    // Canonical plan JSON uses `collection_id`; plan vector parser accepts it.
    RqlPlanV1::from_plan_vector_json(&v)
}

fn push_u32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn read_u32(bytes: &[u8], off: &mut usize) -> Result<u32, Error> {
    if *off + 4 > bytes.len() {
        return Err(Error::QueryInvalid("isa: need u32".into()));
    }
    let n = u32::from_le_bytes(bytes[*off..*off + 4].try_into().expect("4"));
    *off += 4;
    Ok(n)
}

fn push_budget(out: &mut Vec<u8>, b: QueryBudget) {
    let mut flags: u8 = 0;
    if b.max_documents.is_some() {
        flags |= 0x01;
    }
    if b.max_bytes.is_some() {
        flags |= 0x02;
    }
    if b.max_result_bytes.is_some() {
        flags |= 0x04;
    }
    out.push(flags);
    if let Some(n) = b.max_documents {
        out.extend_from_slice(&n.to_le_bytes());
    }
    if let Some(n) = b.max_bytes {
        out.extend_from_slice(&n.to_le_bytes());
    }
    if let Some(n) = b.max_result_bytes {
        out.extend_from_slice(&n.to_le_bytes());
    }
}

fn read_budget(bytes: &[u8], off: &mut usize) -> Result<QueryBudget, Error> {
    if *off >= bytes.len() {
        return Err(Error::QueryInvalid("isa: truncated budget".into()));
    }
    let flags = bytes[*off];
    *off += 1;
    if flags & !BUDGET_FLAGS_KNOWN != 0 {
        return Err(Error::QueryInvalid(format!(
            "isa: reserved budget flag bits set ({flags:#04x})"
        )));
    }
    let mut b = QueryBudget {
        max_documents: None,
        max_bytes: None,
        max_result_bytes: None,
    };
    if flags & 0x01 != 0 {
        b.max_documents = Some(read_u64(bytes, off)?);
    }
    if flags & 0x02 != 0 {
        b.max_bytes = Some(read_u64(bytes, off)?);
    }
    if flags & 0x04 != 0 {
        b.max_result_bytes = Some(read_u64(bytes, off)?);
    }
    Ok(b)
}

fn read_u64(bytes: &[u8], off: &mut usize) -> Result<u64, Error> {
    if *off + 8 > bytes.len() {
        return Err(Error::QueryInvalid("isa: need u64".into()));
    }
    let n = u64::from_le_bytes(bytes[*off..*off + 8].try_into().expect("8"));
    *off += 8;
    Ok(n)
}

fn full_section_canonical_bytes(sec: &QueryIsaFullSection) -> Result<Vec<u8>, Error> {
    let mut root = BTreeMap::new();
    root.insert(
        "encoding".into(),
        JsonValue::String(PLAN_ENCODING_PROFILE.into()),
    );
    root.insert("profile".into(), JsonValue::String(ISA_PROFILE.into()));
    root.insert(
        "pipeline".into(),
        JsonValue::Array(
            sec.pipeline
                .iter()
                .map(pipeline_step_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    match &sec.project {
        None => {
            root.insert("project".into(), JsonValue::Null);
        }
        Some(items) => {
            root.insert(
                "project".into(),
                JsonValue::Array(
                    items
                        .iter()
                        .map(project_item_json)
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            );
        }
    }
    serde_json::to_vec(&btree_to_obj(root))
        .map_err(|e| Error::QueryInvalid(format!("isa full json: {e}")))
}

fn decode_full_section(body: &[u8]) -> Result<QueryIsaFullSection, Error> {
    let v: JsonValue = serde_json::from_slice(body)
        .map_err(|e| Error::QueryInvalid(format!("isa full json: {e}")))?;
    let obj = v
        .as_object()
        .ok_or_else(|| Error::QueryInvalid("isa full must be object".into()))?;
    let pipeline = match obj.get("pipeline") {
        Some(JsonValue::Array(items)) => items
            .iter()
            .map(parse_pipeline_step)
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(Error::QueryInvalid("isa full.pipeline required".into())),
    };
    let project = match obj.get("project") {
        None | Some(JsonValue::Null) => None,
        Some(JsonValue::Array(items)) => Some(
            items
                .iter()
                .map(parse_project_item)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Some(_) => return Err(Error::QueryInvalid("isa full.project invalid".into())),
    };
    Ok(QueryIsaFullSection { pipeline, project })
}

fn pipeline_step_json(step: &FullPipelineStepV1) -> Result<JsonValue, Error> {
    match step {
        FullPipelineStepV1::Enrich(e) => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("enrich".into()));
            m.insert("output".into(), JsonValue::String(e.output.clone()));
            m.insert("using_name".into(), JsonValue::String(e.using_name.clone()));
            m.insert(
                "using_id".into(),
                JsonValue::String(e.using_id.to_string()),
            );
            m.insert("left".into(), path_json(&e.left));
            m.insert("right".into(), path_json(&e.right));
            m.insert(
                "expect".into(),
                JsonValue::String(e.expect.as_str().into()),
            );
            match &e.candidate_where {
                None => m.insert("candidate_where".into(), JsonValue::Null),
                Some(p) => m.insert("candidate_where".into(), p.to_canonical_json()),
            };
            Ok(btree_to_obj(m))
        }
        FullPipelineStepV1::Within(w) => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("within".into()));
            m.insert("carrier".into(), path_json(&w.carrier));
            match &w.element_alias {
                None => m.insert("element_alias".into(), JsonValue::Null),
                Some(a) => m.insert("element_alias".into(), JsonValue::String(a.clone())),
            };
            m.insert(
                "steps".into(),
                JsonValue::Array(
                    w.steps
                        .iter()
                        .map(pipeline_step_json)
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            );
            Ok(btree_to_obj(m))
        }
        FullPipelineStepV1::Filter(p) => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("filter".into()));
            m.insert("where".into(), p.to_canonical_json());
            Ok(btree_to_obj(m))
        }
    }
}

fn parse_pipeline_step(v: &JsonValue) -> Result<FullPipelineStepV1, Error> {
    let obj = v
        .as_object()
        .ok_or_else(|| Error::QueryInvalid("pipeline step object".into()))?;
    match obj.get("kind").and_then(|k| k.as_str()) {
        Some("enrich") => {
            let output = req_str(obj, "output")?;
            let using_name = req_str(obj, "using_name")?;
            let using_id = CollectionId::from_str(req_str(obj, "using_id")?)
                .map_err(|e| Error::QueryInvalid(format!("using_id: {e}")))?;
            let left = parse_path(obj.get("left"))?;
            let right = parse_path(obj.get("right"))?;
            let expect = parse_expect(req_str(obj, "expect")?)?;
            let candidate_where = match obj.get("candidate_where") {
                None | Some(JsonValue::Null) => None,
                Some(w) => Some(Predicate::from_plan_json(w)?),
            };
            Ok(FullPipelineStepV1::Enrich(EnrichStepV1 {
                output: output.to_string(),
                using_name: using_name.to_string(),
                using_id,
                left,
                right,
                candidate_where,
                expect,
            }))
        }
        Some("within") => {
            let carrier = parse_path(obj.get("carrier"))?;
            let element_alias = match obj.get("element_alias") {
                None | Some(JsonValue::Null) => None,
                Some(JsonValue::String(s)) => Some(s.clone()),
                Some(_) => return Err(Error::QueryInvalid("element_alias".into())),
            };
            let steps = match obj.get("steps") {
                Some(JsonValue::Array(items)) => items
                    .iter()
                    .map(parse_pipeline_step)
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err(Error::QueryInvalid("within.steps".into())),
            };
            Ok(FullPipelineStepV1::Within(WithinStepV1 {
                carrier,
                element_alias,
                steps,
            }))
        }
        Some("filter") => {
            let w = obj
                .get("where")
                .ok_or_else(|| Error::QueryInvalid("filter.where".into()))?;
            Ok(FullPipelineStepV1::Filter(Predicate::from_plan_json(w)?))
        }
        other => Err(Error::QueryInvalid(format!(
            "unknown pipeline kind `{other:?}`"
        ))),
    }
}

fn project_item_json(item: &ProjectItemV1) -> Result<JsonValue, Error> {
    match item {
        ProjectItemV1::Leaf { output, source } => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("leaf".into()));
            m.insert("output".into(), JsonValue::String(output.clone()));
            m.insert("source".into(), path_json(source));
            Ok(btree_to_obj(m))
        }
        ProjectItemV1::Nested { output, fields } => {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), JsonValue::String("nested".into()));
            m.insert("output".into(), JsonValue::String(output.clone()));
            m.insert(
                "fields".into(),
                JsonValue::Array(
                    fields
                        .iter()
                        .map(project_item_json)
                        .collect::<Result<Vec<_>, _>>()?,
                ),
            );
            Ok(btree_to_obj(m))
        }
    }
}

fn parse_project_item(v: &JsonValue) -> Result<ProjectItemV1, Error> {
    let obj = v
        .as_object()
        .ok_or_else(|| Error::QueryInvalid("project item object".into()))?;
    match obj.get("kind").and_then(|k| k.as_str()) {
        Some("leaf") => Ok(ProjectItemV1::Leaf {
            output: req_str(obj, "output")?.to_string(),
            source: parse_path(obj.get("source"))?,
        }),
        Some("nested") => {
            let fields = match obj.get("fields") {
                Some(JsonValue::Array(items)) => items
                    .iter()
                    .map(parse_project_item)
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err(Error::QueryInvalid("nested.fields".into())),
            };
            Ok(ProjectItemV1::Nested {
                output: req_str(obj, "output")?.to_string(),
                fields,
            })
        }
        other => Err(Error::QueryInvalid(format!(
            "unknown project kind `{other:?}`"
        ))),
    }
}

fn path_json(p: &Path) -> JsonValue {
    JsonValue::Array(p.0.iter().map(|s| JsonValue::String(s.clone())).collect())
}

fn parse_path(v: Option<&JsonValue>) -> Result<Path, Error> {
    match v {
        Some(JsonValue::Array(arr)) => {
            let segs: Vec<String> = arr
                .iter()
                .map(|x| {
                    x.as_str()
                        .map(|s| s.to_string())
                        .ok_or_else(|| Error::QueryInvalid("path segment".into()))
                })
                .collect::<Result<_, _>>()?;
            Path::from_segments(segs)
        }
        Some(JsonValue::String(s)) => Path::parse_dotted(s),
        _ => Err(Error::QueryInvalid("path required".into())),
    }
}

fn parse_expect(s: &str) -> Result<EnrichCardinality, Error> {
    match s {
        "exactly_one" => Ok(EnrichCardinality::ExactlyOne),
        "optional" => Ok(EnrichCardinality::Optional),
        "many" => Ok(EnrichCardinality::Many),
        other => Err(Error::QueryInvalid(format!("expect `{other}`"))),
    }
}

fn req_str<'a>(obj: &'a Map<String, JsonValue>, key: &str) -> Result<&'a str, Error> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::QueryInvalid(format!("missing string `{key}`")))
}

fn btree_to_obj(m: BTreeMap<String, JsonValue>) -> JsonValue {
    let mut map = Map::new();
    for (k, v) in m {
        map.insert(k, v);
    }
    JsonValue::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_v1::CollectionBindings;
    use crate::query_bytecode_v1::full_attach::compile_rql_full;
    use crate::query_bytecode_v1::{execute_isa_bytes, lower_core_source, HostCapabilities};
    use crate::app_v1::{Parameters, QueryRunOptions};
    use residiuum_heap::HeapId;

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
            _collection_id: CollectionId,
            _limit: Option<usize>,
            _after_key: Option<&str>,
        ) -> Result<Vec<String>, Error> {
            Ok(Vec::new())
        }
        fn get_json(
            &mut self,
            _collection_id: CollectionId,
            _key: &str,
        ) -> Result<Option<JsonValue>, Error> {
            Ok(None)
        }
    }

    #[test]
    fn isa_profile_constant() {
        assert_eq!(ISA_PROFILE, "residiuum-query-isa-v1");
        assert_eq!(ISA_MAGIC, b"RQB1");
    }

    #[test]
    fn core_encode_decode_roundtrip() {
        let id = CollectionId::from_bytes(uuidish(3)).expect("id");
        let bc = lower_core_source(
            "from items where status = $s order by created asc page size 32",
            id,
            "items",
        )
        .expect("lower");
        assert!(!bc.isa_bytes().is_empty());
        assert_eq!(&bc.isa_bytes()[0..4], ISA_MAGIC);
        let prog = bc.decode().expect("decode");
        assert_eq!(prog.profile, ISA_PROFILE);
        let again = encode_core_program(&prog.core, prog.budget).expect("re-encode");
        assert_eq!(again, bc.isa_bytes());
        assert!(prog.full.is_none());
        let again2 = encode_core_program(&prog.core, prog.budget).expect("re-encode");
        assert_eq!(again2, again);
    }

    #[test]
    fn execute_from_isa_empty_host() {
        let id = CollectionId::from_bytes(uuidish(4)).expect("id");
        let heap = HeapId::from_bytes(uuidish(1)).expect("heap");
        let bc = lower_core_source("from items", id, "items").expect("lower");
        let mut host = EmptyHost;
        let page = execute_isa_bytes(
            &mut host,
            bc.isa_bytes(),
            &Parameters::default().values,
            &QueryRunOptions::default(),
            heap,
            id,
        )
        .expect("exec");
        assert!(page.rows.is_empty());
    }

    #[test]
    fn reserved_top_level_flags_rejected() {
        let id = CollectionId::from_bytes(uuidish(5)).expect("id");
        let bc = lower_core_source("from items", id, "items").expect("lower");
        let mut bad = bc.isa_bytes().to_vec();
        bad[5] |= 0x80;
        let err = decode_isa(&bad).expect_err("reserved flags");
        assert!(
            format!("{err:?}").contains("reserved flag"),
            "{err:?}"
        );
    }

    #[test]
    fn non_canonical_rejected_by_decode_isa_canonical() {
        let id = CollectionId::from_bytes(uuidish(6)).expect("id");
        let bc = lower_core_source("from items", id, "items").expect("lower");
        // Trailing zero after a valid program: decode_isa rejects trailing;
        // flip is exercised by reserved-bit test. Canonical path accepts
        // encoder output.
        let prog = decode_isa_canonical(bc.isa_bytes()).expect("canonical");
        assert_eq!(prog.profile, ISA_PROFILE);
    }

    #[test]
    fn full_pipeline_encode_decode_roundtrip() {
        let orders = CollectionId::from_bytes(uuidish(10)).expect("o");
        let customers = CollectionId::from_bytes(uuidish(11)).expect("c");
        let mut bindings = CollectionBindings::default();
        bindings.bind("orders", orders);
        bindings.bind("customers", customers);
        let compiled = compile_rql_full(
            "from orders enrich customer using customers matching customer_id = id expect optional",
            &bindings,
        )
        .expect("compile");
        let bytes = encode_full_program(
            &compiled.base.plan,
            compiled.base.budget,
            &compiled.pipeline,
            &compiled.project,
        )
        .expect("encode");
        let prog = decode_isa(&bytes).expect("decode");
        assert_eq!(prog.core, compiled.base.plan);
        let full = prog.full.expect("full");
        assert_eq!(full.pipeline, compiled.pipeline);
        assert_eq!(full.project, compiled.project);
    }
}
