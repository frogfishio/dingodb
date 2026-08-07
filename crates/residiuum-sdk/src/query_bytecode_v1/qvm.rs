//! Durable Query VM bytecode (**RQL-QVM1**).
//!
//! Profile: **`residiuum-query-vm-v1`** · magic **`QVM1`** (sole durable form).
//! Normative: [QUERY_VM_V1.md](../../../../../doc/todo/rql/QUERY_VM_V1.md)
//!
//! This is the **public executable** authority: opcode stream + typed immediates
//! + policy pool (coverage / consistency). Cursor identity is the domain hash
//! of the **complete canonical QVM bytes** ([`qvm_hash`]) — not an embedded
//! trusted field. Legacy `RQB1` is not accepted (Q0.A10).
//! Decision 0 remains OPEN; **RQL-C1 must not be accepted.**

use crate::app_v1::{ConsistencyMode, CoveragePolicy, QueryBudget};
use crate::error::Error;
use crate::plan_v1::{NullsOrder, OrderDir, OrderTerm};
use crate::predicate::{Path, Predicate};
use residiuum_heap::CollectionId;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use super::full_attach::FullPipelineStepV1;
use super::full_imm_json::{
    parse_pipeline_step, parse_project_item, pipeline_step_json, project_item_json,
};
use super::vm::{OpCode, VM_PROFILE, VM_VERSION};
use super::vm_exec::{verify_vm_program, VmImm, VmInstr, VmPool, VmProgram};

/// Durable QVM magic (`QVM1`).
pub const QVM_MAGIC: &[u8; 4] = b"QVM1";

/// Hard cap on total QVM bytes.
pub const QVM_MAX_TOTAL_BYTES: usize = 1 << 20;
/// Hard cap on each length-prefixed blob.
pub const QVM_MAX_BLOB_BYTES: usize = 1 << 20;
/// Hard cap on opcode count (allocation bound).
pub const QVM_MAX_OPS: usize = 4_096;

/// Domain-separated hash over durable QVM bytes (public program identity).
pub fn qvm_hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"residiuum:query-vm-v1:hash-v1");
    hasher.update(&[0u8]);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

const FLAG_BUDGET: u8 = 0x01;
const FLAGS_KNOWN: u8 = FLAG_BUDGET;

const IMM_NONE: u8 = 0;
const IMM_COLLECTION: u8 = 1;
const IMM_INDEX_EQ: u8 = 2;
const IMM_WHERE: u8 = 3;
const IMM_ORDER: u8 = 4;
const IMM_PAGE: u8 = 5;
const IMM_PROJECT: u8 = 6;
const IMM_ENRICH: u8 = 7;
const IMM_WITHIN: u8 = 8;
const IMM_FILTER_ATTACH: u8 = 9;
const IMM_PROJECT_BRACE: u8 = 10;

/// Encode a lowered [`VmProgram`] into durable QVM bytes (crate-internal).
///
/// Public callers use [`crate::query_bytecode_v1::QueryBytecodeV1`] or
/// [`validate_qvm`] / [`qvm_hash`] byte APIs — [`VmProgram`] is not public.
pub(crate) fn encode_qvm(prog: &VmProgram) -> Result<Vec<u8>, Error> {
    if prog.profile != VM_PROFILE {
        return Err(Error::QueryInvalid(format!(
            "qvm encode: profile mismatch {:?}",
            prog.profile
        )));
    }
    verify_vm_program(prog)?;
    if prog.ops.len() > QVM_MAX_OPS {
        return Err(Error::QueryInvalid(format!(
            "qvm: op_count {} exceeds max {QVM_MAX_OPS}",
            prog.ops.len()
        )));
    }
    let mut flags: u8 = 0;
    if prog.budget.is_some() {
        flags |= FLAG_BUDGET;
    }
    let mut out = Vec::with_capacity(64 + prog.ops.len() * 16);
    out.extend_from_slice(QVM_MAGIC);
    out.push(VM_VERSION);
    out.push(flags);
    // Policy pool only — no trusted plan_hash field on the wire.
    out.push(coverage_to_u8(prog.pool.coverage));
    out.push(consistency_to_u8(prog.pool.consistency));
    if let Some(b) = prog.budget {
        push_budget(&mut out, b);
    }
    push_u32(&mut out, prog.ops.len() as u32);
    for instr in &prog.ops {
        out.push(instr.op.as_u8());
        encode_imm(&mut out, &instr.imm)?;
    }
    if out.len() > QVM_MAX_TOTAL_BYTES {
        return Err(Error::QueryInvalid(format!(
            "qvm: encoded length {} exceeds max {QVM_MAX_TOTAL_BYTES}",
            out.len()
        )));
    }
    Ok(out)
}

/// Decode durable QVM bytes into a [`VmProgram`] (crate-internal).
///
/// Enforces **canonical** form: `encode_qvm(decode(bytes)) == bytes`.
/// Sets [`VmProgram::program_hash`] to [`qvm_hash`] of the input bytes
/// (complete program identity for cursors).
pub(crate) fn decode_qvm(bytes: &[u8]) -> Result<VmProgram, Error> {
    let prog = decode_qvm_structural(bytes)?;
    // Canonical re-encode: reject non-canonical JSON/imm padding.
    let again = encode_qvm(&prog)?;
    if again.as_slice() != bytes {
        return Err(Error::QueryInvalid(
            "qvm: non-canonical encoding (encode(decode(bytes)) != bytes)".into(),
        ));
    }
    Ok(VmProgram {
        program_hash: qvm_hash(bytes),
        ..prog
    })
}

/// Structural decode + verify (no canonical re-encode). Used by encode path
/// internal tests; product entry uses [`decode_qvm`].
fn decode_qvm_structural(bytes: &[u8]) -> Result<VmProgram, Error> {
    if bytes.len() > QVM_MAX_TOTAL_BYTES {
        return Err(Error::QueryInvalid(format!(
            "qvm: total length {} exceeds max {QVM_MAX_TOTAL_BYTES}",
            bytes.len()
        )));
    }
    // Header: magic(4) + ver(1) + flags(1) + cov(1) + cons(1) = 8
    if bytes.len() < 8 {
        return Err(Error::QueryInvalid("qvm: truncated header".into()));
    }
    if &bytes[0..4] != QVM_MAGIC.as_slice() {
        return Err(Error::QueryInvalid("qvm: bad magic (want QVM1)".into()));
    }
    if bytes[4] != VM_VERSION {
        return Err(Error::QueryInvalid(format!(
            "qvm: unsupported version {}",
            bytes[4]
        )));
    }
    let flags = bytes[5];
    if flags & !FLAGS_KNOWN != 0 {
        return Err(Error::QueryInvalid(format!(
            "qvm: reserved flag bits set ({flags:#04x})"
        )));
    }
    let mut off = 6;
    let coverage = coverage_from_u8(bytes[off])?;
    off += 1;
    let consistency = consistency_from_u8(bytes[off])?;
    off += 1;
    let budget = if flags & FLAG_BUDGET != 0 {
        Some(read_budget(bytes, &mut off)?)
    } else {
        None
    };
    let op_count = read_u32(bytes, &mut off)? as usize;
    if op_count > QVM_MAX_OPS {
        return Err(Error::QueryInvalid(format!(
            "qvm: op_count {op_count} exceeds max {QVM_MAX_OPS}"
        )));
    }
    // Bound allocation against remaining bytes (each op ≥ 2 bytes: opcode + tag).
    let remaining = bytes.len().saturating_sub(off);
    if op_count > remaining {
        return Err(Error::QueryInvalid(format!(
            "qvm: op_count {op_count} exceeds remaining {remaining} bytes"
        )));
    }
    if op_count.saturating_mul(2) > remaining {
        return Err(Error::QueryInvalid(format!(
            "qvm: op_count {op_count} cannot fit in remaining {remaining} bytes"
        )));
    }
    let mut ops = Vec::with_capacity(op_count);
    for _ in 0..op_count {
        if off >= bytes.len() {
            return Err(Error::QueryInvalid("qvm: truncated ops".into()));
        }
        let op = OpCode::from_u8(bytes[off]).ok_or_else(|| {
            Error::QueryInvalid(format!("qvm: unknown opcode {:#04x}", bytes[off]))
        })?;
        off += 1;
        let imm = decode_imm(bytes, &mut off)?;
        ops.push(VmInstr { op, imm });
    }
    if off != bytes.len() {
        return Err(Error::QueryInvalid(format!(
            "qvm: trailing {} bytes",
            bytes.len() - off
        )));
    }
    let prog = VmProgram {
        profile: VM_PROFILE,
        ops,
        pool: VmPool {
            coverage,
            consistency,
        },
        budget,
        program_hash: [0u8; 32],
    };
    verify_vm_program(&prog)?;
    Ok(prog)
}

/// Validate durable QVM bytes (decode + canonical + verify). Public byte API.
pub fn validate_qvm(bytes: &[u8]) -> Result<(), Error> {
    let _ = decode_qvm(bytes)?;
    Ok(())
}

/// Lower → encode → decode so product execute consumes QVM authority (RQL-QVM1).
pub(crate) fn materialize_qvm(prog: &VmProgram) -> Result<VmProgram, Error> {
    let bytes = encode_qvm(prog)?;
    decode_qvm(&bytes)
}

fn coverage_to_u8(c: CoveragePolicy) -> u8 {
    match c {
        CoveragePolicy::Complete => 0,
        CoveragePolicy::IncompleteAllowed => 1,
    }
}

fn coverage_from_u8(b: u8) -> Result<CoveragePolicy, Error> {
    match b {
        0 => Ok(CoveragePolicy::Complete),
        1 => Ok(CoveragePolicy::IncompleteAllowed),
        other => Err(Error::QueryInvalid(format!(
            "qvm: unknown coverage {other}"
        ))),
    }
}

fn consistency_to_u8(c: ConsistencyMode) -> u8 {
    match c {
        ConsistencyMode::Available => 0,
        ConsistencyMode::Current => 1,
    }
}

fn consistency_from_u8(b: u8) -> Result<ConsistencyMode, Error> {
    match b {
        0 => Ok(ConsistencyMode::Available),
        1 => Ok(ConsistencyMode::Current),
        other => Err(Error::QueryInvalid(format!(
            "qvm: unknown consistency {other}"
        ))),
    }
}

fn encode_imm(out: &mut Vec<u8>, imm: &VmImm) -> Result<(), Error> {
    match imm {
        VmImm::None => out.push(IMM_NONE),
        VmImm::Collection(id) => {
            out.push(IMM_COLLECTION);
            out.extend_from_slice(id.as_bytes());
        }
        VmImm::IndexEq { force_scan } => {
            out.push(IMM_INDEX_EQ);
            out.push(if *force_scan { 1 } else { 0 });
        }
        VmImm::Where(p) => {
            out.push(IMM_WHERE);
            push_predicate(out, p)?;
        }
        VmImm::Order(terms) => {
            out.push(IMM_ORDER);
            push_order(out, terms)?;
        }
        VmImm::Page { page_size, limit } => {
            out.push(IMM_PAGE);
            out.extend_from_slice(&page_size.to_le_bytes());
            match limit {
                None => out.push(0),
                Some(n) => {
                    out.push(1);
                    out.extend_from_slice(&n.to_le_bytes());
                }
            }
        }
        VmImm::Project(paths) => {
            out.push(IMM_PROJECT);
            push_project(out, paths)?;
        }
        VmImm::Enrich(e) => {
            out.push(IMM_ENRICH);
            let body = serde_json::to_vec(&pipeline_step_json(&FullPipelineStepV1::Enrich(
                e.clone(),
            ))?)
            .map_err(|e| Error::QueryInvalid(format!("qvm enrich json: {e}")))?;
            push_blob(out, &body)?;
        }
        VmImm::Within(w) => {
            out.push(IMM_WITHIN);
            let body = serde_json::to_vec(&pipeline_step_json(&FullPipelineStepV1::Within(
                w.clone(),
            ))?)
            .map_err(|e| Error::QueryInvalid(format!("qvm within json: {e}")))?;
            push_blob(out, &body)?;
        }
        VmImm::FilterAttach(p) => {
            out.push(IMM_FILTER_ATTACH);
            push_predicate(out, p)?;
        }
        VmImm::ProjectBrace(fields) => {
            out.push(IMM_PROJECT_BRACE);
            let arr: Result<Vec<_>, _> = fields.iter().map(project_item_json).collect();
            let body = serde_json::to_vec(&JsonValue::Array(arr?))
                .map_err(|e| Error::QueryInvalid(format!("qvm project json: {e}")))?;
            push_blob(out, &body)?;
        }
    }
    Ok(())
}

fn decode_imm(bytes: &[u8], off: &mut usize) -> Result<VmImm, Error> {
    if *off >= bytes.len() {
        return Err(Error::QueryInvalid("qvm: truncated imm tag".into()));
    }
    let tag = bytes[*off];
    *off += 1;
    match tag {
        IMM_NONE => Ok(VmImm::None),
        IMM_COLLECTION => {
            if *off + 16 > bytes.len() {
                return Err(Error::QueryInvalid("qvm: truncated collection id".into()));
            }
            let mut idb = [0u8; 16];
            idb.copy_from_slice(&bytes[*off..*off + 16]);
            *off += 16;
            let id = CollectionId::from_bytes(idb).map_err(|e| {
                Error::QueryInvalid(format!("qvm: collection id: {e}"))
            })?;
            Ok(VmImm::Collection(id))
        }
        IMM_INDEX_EQ => {
            if *off >= bytes.len() {
                return Err(Error::QueryInvalid("qvm: truncated IndexEq flags".into()));
            }
            let force_scan = bytes[*off] != 0;
            *off += 1;
            Ok(VmImm::IndexEq { force_scan })
        }
        IMM_WHERE => Ok(VmImm::Where(read_predicate(bytes, off)?)),
        IMM_ORDER => Ok(VmImm::Order(read_order(bytes, off)?)),
        IMM_PAGE => {
            if *off + 5 > bytes.len() {
                return Err(Error::QueryInvalid("qvm: truncated Page imm".into()));
            }
            let page_size = u32::from_le_bytes(bytes[*off..*off + 4].try_into().expect("4"));
            *off += 4;
            let has_limit = bytes[*off];
            *off += 1;
            let limit = if has_limit == 0 {
                None
            } else if has_limit == 1 {
                if *off + 8 > bytes.len() {
                    return Err(Error::QueryInvalid("qvm: truncated Page limit".into()));
                }
                let n = u64::from_le_bytes(bytes[*off..*off + 8].try_into().expect("8"));
                *off += 8;
                Some(n)
            } else {
                return Err(Error::QueryInvalid(format!(
                    "qvm: bad Page limit tag {has_limit}"
                )));
            };
            Ok(VmImm::Page { page_size, limit })
        }
        IMM_PROJECT => Ok(VmImm::Project(read_project(bytes, off)?)),
        IMM_ENRICH => {
            let body = read_blob(bytes, off)?;
            let v: JsonValue = serde_json::from_slice(&body)
                .map_err(|e| Error::QueryInvalid(format!("qvm enrich json: {e}")))?;
            match parse_pipeline_step(&v)? {
                FullPipelineStepV1::Enrich(e) => Ok(VmImm::Enrich(e)),
                _ => Err(Error::QueryInvalid("qvm: enrich imm kind mismatch".into())),
            }
        }
        IMM_WITHIN => {
            let body = read_blob(bytes, off)?;
            let v: JsonValue = serde_json::from_slice(&body)
                .map_err(|e| Error::QueryInvalid(format!("qvm within json: {e}")))?;
            match parse_pipeline_step(&v)? {
                FullPipelineStepV1::Within(w) => Ok(VmImm::Within(w)),
                _ => Err(Error::QueryInvalid("qvm: within imm kind mismatch".into())),
            }
        }
        IMM_FILTER_ATTACH => Ok(VmImm::FilterAttach(read_predicate(bytes, off)?)),
        IMM_PROJECT_BRACE => {
            let body = read_blob(bytes, off)?;
            let v: JsonValue = serde_json::from_slice(&body)
                .map_err(|e| Error::QueryInvalid(format!("qvm project json: {e}")))?;
            let arr = v
                .as_array()
                .ok_or_else(|| Error::QueryInvalid("qvm project must be array".into()))?;
            let fields = arr
                .iter()
                .map(parse_project_item)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(VmImm::ProjectBrace(fields))
        }
        other => Err(Error::QueryInvalid(format!(
            "qvm: unknown imm tag {other:#04x}"
        ))),
    }
}

fn push_predicate(out: &mut Vec<u8>, p: &Predicate) -> Result<(), Error> {
    let body = serde_json::to_vec(&p.to_canonical_json())
        .map_err(|e| Error::QueryInvalid(format!("qvm predicate json: {e}")))?;
    push_blob(out, &body)
}

fn read_predicate(bytes: &[u8], off: &mut usize) -> Result<Predicate, Error> {
    let body = read_blob(bytes, off)?;
    let v: JsonValue = serde_json::from_slice(&body)
        .map_err(|e| Error::QueryInvalid(format!("qvm predicate json: {e}")))?;
    Predicate::from_plan_json(&v)
}

fn push_order(out: &mut Vec<u8>, terms: &[OrderTerm]) -> Result<(), Error> {
    let arr: Vec<JsonValue> = terms
        .iter()
        .map(|t| {
            let mut m = BTreeMap::new();
            m.insert(
                "path".into(),
                JsonValue::Array(
                    t.path
                        .0
                        .iter()
                        .map(|s| JsonValue::String(s.clone()))
                        .collect(),
                ),
            );
            m.insert(
                "dir".into(),
                JsonValue::String(
                    match t.dir {
                        OrderDir::Asc => "asc",
                        OrderDir::Desc => "desc",
                    }
                    .into(),
                ),
            );
            m.insert(
                "nulls".into(),
                JsonValue::String(
                    match t.nulls {
                        NullsOrder::Last => "last",
                        NullsOrder::First => "first",
                    }
                    .into(),
                ),
            );
            m.insert("tie_break".into(), JsonValue::Bool(t.tie_break));
            JsonValue::Object(m.into_iter().collect())
        })
        .collect();
    let body = serde_json::to_vec(&JsonValue::Array(arr))
        .map_err(|e| Error::QueryInvalid(format!("qvm order json: {e}")))?;
    push_blob(out, &body)
}

fn read_order(bytes: &[u8], off: &mut usize) -> Result<Vec<OrderTerm>, Error> {
    let body = read_blob(bytes, off)?;
    let v: JsonValue = serde_json::from_slice(&body)
        .map_err(|e| Error::QueryInvalid(format!("qvm order json: {e}")))?;
    let arr = v
        .as_array()
        .ok_or_else(|| Error::QueryInvalid("qvm order must be array".into()))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let o = item
            .as_object()
            .ok_or_else(|| Error::QueryInvalid("qvm order term must be object".into()))?;
        let path_v = o
            .get("path")
            .ok_or_else(|| Error::QueryInvalid("qvm order.path required".into()))?;
        let segs = path_v
            .as_array()
            .ok_or_else(|| Error::QueryInvalid("qvm order.path must be array".into()))?
            .iter()
            .map(|s| {
                s.as_str()
                    .map(|x| x.to_string())
                    .ok_or_else(|| Error::QueryInvalid("qvm order.path segment".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let dir = match o.get("dir").and_then(|d| d.as_str()).unwrap_or("asc") {
            "asc" => OrderDir::Asc,
            "desc" => OrderDir::Desc,
            other => {
                return Err(Error::QueryInvalid(format!(
                    "qvm unknown order dir `{other}`"
                )))
            }
        };
        let nulls = match o.get("nulls").and_then(|n| n.as_str()).unwrap_or("last") {
            "last" => NullsOrder::Last,
            "first" => NullsOrder::First,
            other => {
                return Err(Error::QueryInvalid(format!(
                    "qvm unknown nulls order `{other}`"
                )))
            }
        };
        let tie_break = o.get("tie_break").and_then(|t| t.as_bool()).unwrap_or(false);
        out.push(OrderTerm {
            path: Path::from_segments(segs)?,
            dir,
            nulls,
            tie_break,
        });
    }
    Ok(out)
}

fn push_project(out: &mut Vec<u8>, paths: &Option<Vec<Path>>) -> Result<(), Error> {
    let body = match paths {
        None => b"null".to_vec(),
        Some(ps) => {
            let arr: Vec<JsonValue> = ps
                .iter()
                .map(|p| {
                    JsonValue::Array(p.0.iter().map(|s| JsonValue::String(s.clone())).collect())
                })
                .collect();
            serde_json::to_vec(&JsonValue::Array(arr))
                .map_err(|e| Error::QueryInvalid(format!("qvm project paths: {e}")))?
        }
    };
    push_blob(out, &body)
}

fn read_project(bytes: &[u8], off: &mut usize) -> Result<Option<Vec<Path>>, Error> {
    let body = read_blob(bytes, off)?;
    let v: JsonValue = serde_json::from_slice(&body)
        .map_err(|e| Error::QueryInvalid(format!("qvm project json: {e}")))?;
    if v.is_null() {
        return Ok(None);
    }
    let arr = v
        .as_array()
        .ok_or_else(|| Error::QueryInvalid("qvm project must be array or null".into()))?;
    let mut paths = Vec::with_capacity(arr.len());
    for item in arr {
        let segs = item
            .as_array()
            .ok_or_else(|| Error::QueryInvalid("qvm project path must be array".into()))?
            .iter()
            .map(|s| {
                s.as_str()
                    .map(|x| x.to_string())
                    .ok_or_else(|| Error::QueryInvalid("qvm project segment".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        paths.push(Path::from_segments(segs)?);
    }
    Ok(Some(paths))
}

fn push_blob(out: &mut Vec<u8>, body: &[u8]) -> Result<(), Error> {
    if body.len() > QVM_MAX_BLOB_BYTES {
        return Err(Error::QueryInvalid(format!(
            "qvm: blob {} exceeds max {QVM_MAX_BLOB_BYTES}",
            body.len()
        )));
    }
    push_u32(out, body.len() as u32);
    out.extend_from_slice(body);
    Ok(())
}

fn read_blob(bytes: &[u8], off: &mut usize) -> Result<Vec<u8>, Error> {
    let len = read_u32(bytes, off)? as usize;
    if len > QVM_MAX_BLOB_BYTES {
        return Err(Error::QueryInvalid(format!(
            "qvm: blob {len} exceeds max {QVM_MAX_BLOB_BYTES}"
        )));
    }
    if *off + len > bytes.len() {
        return Err(Error::QueryInvalid("qvm: truncated blob".into()));
    }
    let body = bytes[*off..*off + len].to_vec();
    *off += len;
    Ok(body)
}

fn push_u32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}

fn read_u32(bytes: &[u8], off: &mut usize) -> Result<u32, Error> {
    if *off + 4 > bytes.len() {
        return Err(Error::QueryInvalid("qvm: need u32".into()));
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
        return Err(Error::QueryInvalid("qvm: truncated budget flags".into()));
    }
    let flags = bytes[*off];
    *off += 1;
    if flags & !0x07 != 0 {
        return Err(Error::QueryInvalid(format!(
            "qvm: reserved budget flags ({flags:#04x})"
        )));
    }
    let mut b = QueryBudget {
        max_documents: None,
        max_bytes: None,
        max_result_bytes: None,
    };
    if flags & 0x01 != 0 {
        if *off + 8 > bytes.len() {
            return Err(Error::QueryInvalid("qvm: truncated max_documents".into()));
        }
        b.max_documents = Some(u64::from_le_bytes(
            bytes[*off..*off + 8].try_into().expect("8"),
        ));
        *off += 8;
    }
    if flags & 0x02 != 0 {
        if *off + 8 > bytes.len() {
            return Err(Error::QueryInvalid("qvm: truncated max_bytes".into()));
        }
        b.max_bytes = Some(u64::from_le_bytes(
            bytes[*off..*off + 8].try_into().expect("8"),
        ));
        *off += 8;
    }
    if flags & 0x04 != 0 {
        if *off + 8 > bytes.len() {
            return Err(Error::QueryInvalid("qvm: truncated max_result_bytes".into()));
        }
        b.max_result_bytes = Some(u64::from_le_bytes(
            bytes[*off..*off + 8].try_into().expect("8"),
        ));
        *off += 8;
    }
    Ok(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_v1::{CollectionBindings, PlanBuilder};
    use crate::query_bytecode_v1::vm_exec::lower_core;
    use residiuum_heap::CollectionId;

    fn uuidish(seed: u8) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = seed;
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        b
    }

    #[test]
    fn qvm_roundtrip_core() {
        let id = CollectionId::from_bytes(uuidish(9)).expect("id");
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        let core = PlanBuilder::from_source("items")
            .compile(&bindings)
            .expect("plan");
        let prog = lower_core(core, None);
        let bytes = encode_qvm(&prog).expect("encode");
        assert_eq!(&bytes[0..4], QVM_MAGIC);
        let again = decode_qvm(&bytes).expect("decode");
        assert_eq!(again.ops.len(), prog.ops.len());
        assert_ne!(again.program_hash, [0u8; 32]);
        assert_eq!(again.program_hash, qvm_hash(&bytes));
        assert_eq!(again.ops[0].op, OpCode::BindCollection);
        assert!(matches!(again.ops[1].imm, VmImm::IndexEq { .. }));
        // Canonical: re-encode equals original.
        assert_eq!(encode_qvm(&again).unwrap(), bytes);
    }

    #[test]
    fn qvm_rejects_huge_op_count() {
        let id = CollectionId::from_bytes(uuidish(10)).expect("id");
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        let core = PlanBuilder::from_source("items")
            .compile(&bindings)
            .expect("plan");
        let prog = lower_core(core, None);
        let mut bytes = encode_qvm(&prog).expect("encode");
        // Corrupt op_count at fixed offset: magic4+ver1+flags1+cov1+cons1 = 8
        let op_count_off = 8;
        bytes[op_count_off..op_count_off + 4]
            .copy_from_slice(&(u32::MAX).to_le_bytes());
        let err = decode_qvm(&bytes).unwrap_err();
        assert!(
            err.to_string().contains("op_count") || err.to_string().contains("qvm:"),
            "got {err}"
        );
    }

    #[test]
    fn qvm_mutation_of_opcode_fails_or_changes() {
        let id = CollectionId::from_bytes(uuidish(11)).expect("id");
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        let core = PlanBuilder::from_source("items")
            .compile(&bindings)
            .expect("plan");
        let prog = lower_core(core, None);
        let mut bytes = encode_qvm(&prog).expect("encode");
        // Corrupt Halt opcode (penultimate byte region: opcode then IMM_NONE).
        let corrupt_at = bytes.len() - 2;
        bytes[corrupt_at] = 0xEE;
        let err = decode_qvm(&bytes).unwrap_err();
        assert!(
            err.to_string().contains("unknown opcode") || err.to_string().contains("qvm:"),
            "got {err}"
        );
    }

    #[test]
    fn qvm_rejects_missing_terminal_halt_via_verify() {
        let id = CollectionId::from_bytes(uuidish(12)).expect("id");
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        let core = PlanBuilder::from_source("items")
            .compile(&bindings)
            .expect("plan");
        let mut prog = lower_core(core, None);
        prog.ops.pop(); // drop Halt
        let err = encode_qvm(&prog).unwrap_err();
        assert!(err.to_string().contains("Halt") || err.to_string().contains("verify"));
    }

    #[test]
    fn qvm_program_hash_covers_full_attach() {
        use crate::query_bytecode_v1::full_attach::{EnrichCardinality, EnrichStepV1, FullPipelineStepV1};
        use crate::query_bytecode_v1::vm_exec::lower_full;
        use crate::predicate::Path;
        let id = CollectionId::from_bytes(uuidish(20)).expect("id");
        let fid = CollectionId::from_bytes(uuidish(21)).expect("id");
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        let core = PlanBuilder::from_source("items").compile(&bindings).expect("plan");
        let core_only = encode_qvm(&lower_core(core.clone(), None)).expect("enc core");
        let pipeline = vec![FullPipelineStepV1::Enrich(EnrichStepV1 {
            output: "f".into(),
            using_name: "foreign".into(),
            using_id: fid,
            left: Path::parse_dotted("a").unwrap(),
            right: Path::parse_dotted("b").unwrap(),
            candidate_where: None,
            expect: EnrichCardinality::Optional,
        })];
        let full = encode_qvm(&lower_full(core, None, pipeline, None)).expect("enc full");
        assert_ne!(qvm_hash(&core_only), qvm_hash(&full));
        let d = decode_qvm(&full).expect("decode");
        assert_eq!(d.program_hash, qvm_hash(&full));
    }

    #[test]
    fn index_eq_force_scan_has_no_where_imm() {
        let id = CollectionId::from_bytes(uuidish(22)).expect("id");
        let mut bindings = CollectionBindings::default();
        bindings.bind("items", id);
        let core = PlanBuilder::from_source("items")
            .where_(crate::predicate::Predicate::True)
            .compile(&bindings)
            .expect("plan");
        let prog = lower_core(core, None);
        match &prog.ops[1].imm {
            VmImm::IndexEq { force_scan } => assert!(!force_scan),
            other => panic!("expected IndexEq, got {other:?}"),
        }
        assert!(matches!(prog.ops[3].imm, VmImm::Where(_)));
    }
}
