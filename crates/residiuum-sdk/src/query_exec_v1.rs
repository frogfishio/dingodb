//! Bounded Application Core page executor — APP-6 T2 + APB-7 T2 hardening.
//!
//! Normative: CORE plan §10 / §14 APP-6 (partial). Compiles via
//! [`crate::rql_app_core::compile_app_core`], scans with `list_keys` + `get`,
//! evaluates predicates with [`crate::predicate::Predicate::eval`].
//!
//! **T2 budgets:** `max_documents`, `max_bytes` (examined JSON), and
//! `max_result_bytes` (projected page payload) all fail closed with
//! [`Error::ResourceLimit`]. Field order sorts on **full documents**, then
//! projects (so order paths need not appear in `project`).
//!
//! **Not claimed:** product query qualification (APB-7 package accept), product
//! cursor secrets (vector-lock ring only), remote op 118, index pushdown,
//! multi-page field-order continuation tuples, snapshot reads.

use crate::app_v1::{
    ConsistencyEvidence, Continuation, CoverageEvidence, CoveragePolicy, HoleEvidence, Parameters,
    QueryBudget, QueryId, QueryPage, QueryRow, QueryRunOptions, QueryExplanation,
};
use crate::cursor_v1::{mint, CursorKeyRing, CursorLogical, VerifyContext, PROFILE as CURSOR_PROFILE};
use crate::error::Error;
use crate::plan_v1::{CollectionBindings, NullsOrder, OrderDir, OrderTerm, RqlPlanV1};
use crate::predicate::{resolve_path, Path, Predicate, Resolve};
use crate::rql_app_core::{compile_app_core, merge_budgets, CompiledAppCore};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Profile label for this executor cut (not full product query).
pub const EXEC_PROFILE: &str = "residiuum-app-core-exec-v1";

/// Document accessor used by the scan executor (embedded or remote collection plane).
pub trait DocScan {
    /// List keys in deterministic order.
    fn list_keys(
        &mut self,
        limit: Option<usize>,
        after_key: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// JSON get (None when absent).
    fn get_json(&mut self, key: &str) -> Result<Option<JsonValue>, Error>;
}

/// Execute Application Core RQL source against a document scan.
pub fn execute_rql<S: DocScan>(
    scan: &mut S,
    source: &str,
    parameters: &Parameters,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    collection_name: &str,
) -> Result<QueryPage, Error> {
    let mut bindings = CollectionBindings {
        by_name: BTreeMap::new(),
    };
    bindings.bind(collection_name, collection_id);
    let compiled = compile_app_core(source, &bindings)?;
    if options.explain || compiled.explain {
        return Err(Error::QueryInvalid(
            "use explain_rql for explain; rql executes rows".into(),
        ));
    }
    // Prefer live collection id over name-bound plan source if they differ.
    let mut plan = compiled.plan;
    if plan.from.collection_id != collection_id {
        plan.from.collection_id = collection_id;
    }
    execute_plan(
        scan,
        &plan,
        &parameters.values,
        options,
        heap_id,
        collection_id,
        compiled.budget,
    )
}

/// Explain Application Core source (plan tree + hash; no row materialization).
pub fn explain_rql_source(
    source: &str,
    collection_id: CollectionId,
    collection_name: &str,
) -> Result<QueryExplanation, Error> {
    let mut bindings = CollectionBindings {
        by_name: BTreeMap::new(),
    };
    bindings.bind(collection_name, collection_id);
    let compiled = compile_app_core(source, &bindings)?;
    Ok(QueryExplanation {
        plan_profile: compiled.plan.profile.clone(),
        plan_hash: compiled.plan.plan_hash(),
        tree: compiled.plan.to_canonical_json(),
    })
}

/// Execute a validated plan (one page).
pub fn execute_plan<S: DocScan>(
    scan: &mut S,
    plan: &RqlPlanV1,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    source_budget: Option<QueryBudget>,
) -> Result<QueryPage, Error> {
    let budget = merge_budgets(source_budget, options.budget);
    let page_size = options
        .page_size
        .unwrap_or(plan.page_size)
        .clamp(1, 4_096) as usize;

    let (after_key, remaining_limit) = if let Some(ref cont) = options.after {
        decode_after(cont, heap_id, collection_id, &plan.plan_hash())?
    } else {
        (None, plan.limit)
    };

    let total_limit = remaining_limit;
    let need = match total_limit {
        Some(n) => (n as usize).min(page_size),
        None => page_size,
    };

    // Key-stream order when order is only key tie-break; else collect+sort.
    let key_only_order = plan
        .order
        .iter()
        .all(|t| t.tie_break || t.path.0 == ["$key"]);

    // Matched rows keep full documents until final project (field-order sort).
    let mut matched: Vec<(String, JsonValue)> = Vec::new();
    let mut examined_docs: u64 = 0;
    let mut examined_bytes: u64 = 0;
    let mut result_bytes: u64 = 0;
    let mut after = after_key.clone();
    let mut known_holes = Vec::new();

    if key_only_order {
        // Stream until page full or scan ends; project on the way for result budget.
        'outer: loop {
            let batch = scan.list_keys(Some(256), after.as_deref())?;
            if batch.is_empty() {
                break;
            }
            for key in batch {
                after = Some(key.clone());
                match scan.get_json(&key)? {
                    None => {
                        known_holes.push(HoleEvidence {
                            code: "key_listed_absent".into(),
                            key: Some(key),
                        });
                        // Listing still counts as one examination for documents.
                        examined_docs += 1;
                        check_doc_budget(budget, examined_docs)?;
                    }
                    Some(doc) => {
                        examined_docs += 1;
                        examined_bytes = examined_bytes.saturating_add(json_byte_len(&doc));
                        check_doc_budget(budget, examined_docs)?;
                        check_bytes_budget(budget, examined_bytes)?;
                        if plan.where_pred.eval(&doc, params)? {
                            let value = project_doc(&doc, plan.project.as_ref())?;
                            let row_len = json_byte_len(&value);
                            let next_result = result_bytes.saturating_add(row_len);
                            check_result_budget(budget, next_result)?;
                            result_bytes = next_result;
                            matched.push((key, value));
                            if matched.len() >= need {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Full scan under budget, sort on full docs, then page + project.
        let mut full: Vec<(String, JsonValue)> = Vec::new();
        loop {
            let batch = scan.list_keys(Some(256), after.as_deref())?;
            if batch.is_empty() {
                break;
            }
            for key in batch {
                after = Some(key.clone());
                if let Some(doc) = scan.get_json(&key)? {
                    examined_docs += 1;
                    examined_bytes = examined_bytes.saturating_add(json_byte_len(&doc));
                    check_doc_budget(budget, examined_docs)?;
                    check_bytes_budget(budget, examined_bytes)?;
                    if plan.where_pred.eval(&doc, params)? {
                        full.push((key, doc));
                    }
                } else {
                    examined_docs += 1;
                    check_doc_budget(budget, examined_docs)?;
                    known_holes.push(HoleEvidence {
                        code: "key_listed_absent".into(),
                        key: Some(key),
                    });
                }
            }
        }
        full.sort_by(|(ka, va), (kb, vb)| compare_rows(ka, va, kb, vb, &plan.order));
        if let Some(n) = total_limit {
            full.truncate(n as usize);
        }
        // Residual multi-page field-order: continuation is key-based only.
        if let Some(ref ak) = after_key {
            if let Some(pos) = full.iter().position(|(k, _)| k == ak) {
                full = full.split_off(pos + 1);
            }
        }
        if full.len() > page_size {
            full.truncate(page_size);
        }
        for (key, doc) in full {
            let value = project_doc(&doc, plan.project.as_ref())?;
            let row_len = json_byte_len(&value);
            let next_result = result_bytes.saturating_add(row_len);
            check_result_budget(budget, next_result)?;
            result_bytes = next_result;
            matched.push((key, value));
        }
    }

    let took = matched.len();
    let exhausted = if key_only_order {
        took < need
    } else {
        // If we truncated to page_size after sort, may have more.
        took < page_size
    };

    // More precise exhausted for key-only: if took < need, scan ended.
    // If took == need, probe one more key after last match.
    let exhausted = if key_only_order && took == need {
        if let Some((last_k, _)) = matched.last() {
            let more = scan.list_keys(Some(1), Some(last_k.as_str()))?;
            if more.is_empty() {
                true
            } else {
                !has_later_match(
                    scan,
                    last_k,
                    &plan.where_pred,
                    params,
                    budget,
                    examined_docs,
                    examined_bytes,
                )?
            }
        } else {
            true
        }
    } else {
        exhausted
    };

    let rows: Vec<QueryRow> = matched
        .into_iter()
        .map(|(key, value)| QueryRow { key, value })
        .collect();

    let remaining_after = total_limit.map(|n| n.saturating_sub(rows.len() as u64));
    let next = if exhausted || rows.is_empty() {
        None
    } else {
        let last_key = rows.last().map(|r| r.key.clone()).unwrap_or_default();
        Some(mint_page_cursor(
            heap_id,
            collection_id,
            &plan.plan_hash(),
            &last_key,
            remaining_after,
            page_size as u32,
            plan.coverage,
            plan.consistency,
        )?)
    };

    let coverage_complete = known_holes.is_empty()
        && matches!(options.coverage, CoveragePolicy::Complete | CoveragePolicy::IncompleteAllowed);

    let _ = result_bytes; // accounted during fill; residual: surface on QueryPage later
    let _ = examined_bytes;

    Ok(QueryPage {
        query_id: QueryId(residiuum_store::random_id().map_err(Error::from)?),
        plan_hash: plan.plan_hash(),
        heap_id,
        collection_id,
        rows,
        next: if exhausted { None } else { next },
        exhausted,
        coverage: CoverageEvidence {
            complete: coverage_complete,
            mode: options.coverage,
        },
        consistency: ConsistencyEvidence {
            mode: options.consistency,
        },
        remaining_limit: remaining_after,
        known_holes,
    })
}

fn json_byte_len(v: &JsonValue) -> u64 {
    // Compact JSON encoding length — stable enough for budget accounting.
    serde_json::to_vec(v).map(|b| b.len() as u64).unwrap_or(0)
}

fn check_doc_budget(budget: Option<QueryBudget>, examined: u64) -> Result<(), Error> {
    if let Some(max) = budget.and_then(|b| b.max_documents) {
        if examined > max {
            return Err(Error::ResourceLimit(format!(
                "query budget max_documents={max} exceeded"
            )));
        }
    }
    Ok(())
}

fn check_bytes_budget(budget: Option<QueryBudget>, examined_bytes: u64) -> Result<(), Error> {
    if let Some(max) = budget.and_then(|b| b.max_bytes) {
        if examined_bytes > max {
            return Err(Error::ResourceLimit(format!(
                "query budget max_bytes={max} exceeded (examined_bytes={examined_bytes})"
            )));
        }
    }
    Ok(())
}

fn check_result_budget(budget: Option<QueryBudget>, result_bytes: u64) -> Result<(), Error> {
    if let Some(max) = budget.and_then(|b| b.max_result_bytes) {
        if result_bytes > max {
            return Err(Error::ResourceLimit(format!(
                "query budget max_result_bytes={max} exceeded (result_bytes={result_bytes})"
            )));
        }
    }
    Ok(())
}

fn has_later_match<S: DocScan>(
    scan: &mut S,
    after_key: &str,
    pred: &Predicate,
    params: &BTreeMap<String, JsonValue>,
    budget: Option<QueryBudget>,
    mut examined_docs: u64,
    mut examined_bytes: u64,
) -> Result<bool, Error> {
    let mut after = Some(after_key.to_string());
    let mut probes = 0usize;
    while probes < 64 {
        let batch = scan.list_keys(Some(32), after.as_deref())?;
        if batch.is_empty() {
            return Ok(false);
        }
        for key in batch {
            after = Some(key.clone());
            probes += 1;
            if let Some(doc) = scan.get_json(&key)? {
                examined_docs += 1;
                examined_bytes = examined_bytes.saturating_add(json_byte_len(&doc));
                if budget.and_then(|b| b.max_documents).is_some_and(|m| examined_docs > m) {
                    return Ok(false);
                }
                if budget.and_then(|b| b.max_bytes).is_some_and(|m| examined_bytes > m) {
                    return Ok(false);
                }
                if pred.eval(&doc, params)? {
                    return Ok(true);
                }
            } else {
                examined_docs += 1;
                if budget.and_then(|b| b.max_documents).is_some_and(|m| examined_docs > m) {
                    return Ok(false);
                }
            }
            if probes >= 64 {
                // Residual: may still have matches further — conservative not exhausted.
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn project_doc(doc: &JsonValue, project: Option<&Vec<Path>>) -> Result<JsonValue, Error> {
    let Some(paths) = project else {
        return Ok(doc.clone());
    };
    let mut out = serde_json::Map::new();
    for p in paths {
        match resolve_path(doc, p) {
            Resolve::Present(v) => {
                // Flatten single-segment paths as object fields; multi-segment nest shallowly.
                if p.0.len() == 1 {
                    out.insert(p.0[0].clone(), v);
                } else {
                    out.insert(p.dotted(), v);
                }
            }
            Resolve::Absent => {}
        }
    }
    Ok(JsonValue::Object(out))
}

fn compare_rows(
    ka: &str,
    va: &JsonValue,
    kb: &str,
    vb: &JsonValue,
    order: &[OrderTerm],
) -> Ordering {
    for term in order {
        if term.tie_break || term.path.0 == ["$key"] {
            let c = ka.cmp(kb);
            return apply_dir(c, term.dir);
        }
        let ra = resolve_path(va, &term.path);
        let rb = resolve_path(vb, &term.path);
        let c = compare_resolve(&ra, &rb, term.nulls);
        if c != Ordering::Equal {
            return apply_dir(c, term.dir);
        }
    }
    ka.cmp(kb)
}

fn apply_dir(c: Ordering, dir: OrderDir) -> Ordering {
    match dir {
        OrderDir::Asc => c,
        OrderDir::Desc => c.reverse(),
    }
}

fn compare_resolve(a: &Resolve, b: &Resolve, nulls: NullsOrder) -> Ordering {
    match (a, b) {
        (Resolve::Absent, Resolve::Absent) => Ordering::Equal,
        (Resolve::Absent, Resolve::Present(_)) => match nulls {
            NullsOrder::Last => Ordering::Greater,
            NullsOrder::First => Ordering::Less,
        },
        (Resolve::Present(_), Resolve::Absent) => match nulls {
            NullsOrder::Last => Ordering::Less,
            NullsOrder::First => Ordering::Greater,
        },
        (Resolve::Present(x), Resolve::Present(y)) => json_ord(x, y),
    }
}

fn json_ord(a: &JsonValue, b: &JsonValue) -> Ordering {
    match (a, b) {
        (JsonValue::Null, JsonValue::Null) => Ordering::Equal,
        (JsonValue::Null, _) => Ordering::Less,
        (_, JsonValue::Null) => Ordering::Greater,
        (JsonValue::Bool(x), JsonValue::Bool(y)) => x.cmp(y),
        (JsonValue::Number(x), JsonValue::Number(y)) => {
            match (x.as_f64(), y.as_f64()) {
                (Some(xf), Some(yf)) => xf.partial_cmp(&yf).unwrap_or(Ordering::Equal),
                _ => x.to_string().cmp(&y.to_string()),
            }
        }
        (JsonValue::String(x), JsonValue::String(y)) => x.cmp(y),
        _ => a.to_string().cmp(&b.to_string()),
    }
}

fn mint_page_cursor(
    heap_id: HeapId,
    collection_id: CollectionId,
    plan_hash: &[u8; 32],
    last_key: &str,
    remaining_limit: Option<u64>,
    page_size: u32,
    coverage: CoveragePolicy,
    consistency: crate::app_v1::ConsistencyMode,
) -> Result<Continuation, Error> {
    let ring = CursorKeyRing::vector_lock();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| Error::Internal(format!("clock: {e}")))?;
    let logical = CursorLogical {
        cursor_profile: CURSOR_PROFILE.into(),
        key_id: ring.current.key_id.clone(),
        heap_id: format_uuid(heap_id.as_bytes()),
        collection_id: format_uuid(collection_id.as_bytes()),
        authority_epoch: 1,
        plan_hash: hex32(plan_hash),
        parameter_hash: "00".repeat(32),
        order_normalized: serde_json::json!([{"path":["$key"],"dir":"asc","tie_break":true}]),
        last_sort_tuple: serde_json::json!([last_key]),
        source_frontier: serde_json::json!({"generation": 0}),
        remaining_limit: remaining_limit.unwrap_or(u64::MAX),
        page_size,
        coverage_mode: match coverage {
            CoveragePolicy::Complete => "complete".into(),
            CoveragePolicy::IncompleteAllowed => "incomplete_allowed".into(),
        },
        consistency_mode: match consistency {
            crate::app_v1::ConsistencyMode::Available => "available".into(),
            crate::app_v1::ConsistencyMode::Current => "current".into(),
        },
        issued_at: now,
        expires_at: now.saturating_add(crate::cursor_v1::TTL_SECONDS),
    };
    mint(&logical, &ring)
}

fn decode_after(
    cont: &Continuation,
    heap_id: HeapId,
    collection_id: CollectionId,
    plan_hash: &[u8; 32],
) -> Result<(Option<String>, Option<u64>), Error> {
    let ring = CursorKeyRing::vector_lock();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ctx = VerifyContext {
        heap_id: format_uuid(heap_id.as_bytes()),
        collection_id: format_uuid(collection_id.as_bytes()),
        plan_hash: Some(hex32(plan_hash)),
        parameter_hash: None,
    };
    let logical = crate::cursor_v1::verify(&cont.token, &ctx, &ring, now)?;
    let after = logical
        .last_sort_tuple
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let rem = if logical.remaining_limit == u64::MAX {
        None
    } else {
        Some(logical.remaining_limit)
    };
    Ok((after, rem))
}

fn format_uuid(bytes: &[u8; 16]) -> String {
    // UUID text form from 16 bytes (hyphenated).
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

fn hex32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// Silence unused import if CompiledAppCore only used in docs
#[allow(dead_code)]
fn _compiled_type(_: &CompiledAppCore) {}