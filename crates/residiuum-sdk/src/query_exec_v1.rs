//! Bounded Application Core page executor — APP-6 T2/T3 + APB-7 T2/T4.
//!
//! Normative: CORE plan §10 / §14 APP-6 (partial). Compiles via
//! [`crate::rql_app_core::compile_app_core`], scans with `list_keys` + `get`,
//! evaluates predicates with [`crate::predicate::Predicate::eval`].
//!
//! **T2 budgets:** `max_documents`, `max_bytes`, `max_result_bytes`.
//! **T3 multipage field-order:** cursor `last_sort_tuple` carries order-term
//! values (+ key); resume skips rows `<=` the prior page's last tuple (not
//! key-only). Key-stream order still uses key resume for streaming.
//! **T4 index pushdown:** when [`DocScan::try_equality_index_keys`] returns
//! candidates for field equalities, examine only those keys (still re-eval
//! full predicate). Fall back to full scan when no usable index.
//! **T8 deadline/cancel:** [`QueryRunOptions::deadline`] and
//! [`QueryRunOptions::cancel`] checked cooperatively between scan steps.
//!
//! **Not claimed:** product query qualification (APB-7 package accept), product
//! cursor secrets, remote op 118, snapshot reads.

use crate::app_v1::{
    ConsistencyEvidence, Continuation, CoverageEvidence, CoveragePolicy, HoleEvidence, Parameters,
    QueryBudget, QueryId, QueryPage, QueryRow, QueryRunOptions, QueryExplanation,
};
use crate::cursor_v1::{mint, CursorKeyRing, CursorLogical, VerifyContext, PROFILE as CURSOR_PROFILE};
use crate::error::Error;
use crate::plan_v1::{CollectionBindings, NullsOrder, OrderDir, OrderTerm, RqlPlanV1};
use crate::predicate::{
    resolve_path, CompareOp, Operand, Path, Predicate, Resolve,
};
use crate::rql_app_core::{compile_app_core, merge_budgets, CompiledAppCore};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::time::Instant;

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

    /// Optional equality-index acceleration (APB-7 T4).
    ///
    /// Default: no index → caller full-scans. Implementors may return candidate
    /// application keys for a shallow AND of field equalities.
    fn try_equality_index_keys(
        &mut self,
        _equalities: &[(String, JsonValue)],
    ) -> Result<Option<Vec<String>>, Error> {
        Ok(None)
    }
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
    let started = Instant::now();
    check_governance(options, started)?;

    let budget = merge_budgets(source_budget, options.budget);
    let page_size = options
        .page_size
        .unwrap_or(plan.page_size)
        .clamp(1, 4_096) as usize;

    let (last_sort_tuple_resume, remaining_limit) = if let Some(ref cont) = options.after {
        decode_after(cont, heap_id, collection_id, &plan.plan_hash())?
    } else {
        (None, plan.limit)
    };
    // Key-stream resume (when order is key-only): last element of sort tuple is the key.
    let after_key = last_sort_tuple_resume
        .as_ref()
        .and_then(key_from_sort_tuple);

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

    // APB-7 T4: equality index acceleration when available.
    let eqs = equality_constraints(&plan.where_pred, params);
    let index_keys = if eqs.is_empty() {
        None
    } else {
        scan.try_equality_index_keys(&eqs)?
    };
    let used_index = index_keys.is_some();

    // Matched rows keep projected values (key-only) or full docs until project (field-order).
    let mut matched: Vec<(String, JsonValue)> = Vec::new();
    // Last full document on the page (for field-order sort-tuple mint; pre-project).
    let mut last_full_for_cursor: Option<(String, JsonValue)> = None;
    let mut examined_docs: u64 = 0;
    let mut examined_bytes: u64 = 0;
    let mut result_bytes: u64 = 0;
    // Key-stream resume only for key-only order. Field-order always full-scans
    // then resumes with last_sort_tuple (key after would drop out-of-key-order rows).
    let mut after = if key_only_order {
        after_key.clone()
    } else {
        None
    };
    let mut known_holes = Vec::new();
    // When index path stops early, whether more candidate keys remain.
    let mut index_more = false;
    // Field-order: more rows after page truncate.
    let mut field_order_more = false;

    if let Some(mut candidates) = index_keys {
        // Index path: examine only candidate keys; re-eval full predicate.
        candidates.sort();
        if key_only_order {
            if let Some(ref ak) = after_key {
                candidates.retain(|k| k.as_str() > ak.as_str());
            }
            let mut iter = candidates.into_iter();
            while let Some(key) = iter.next() {
                check_governance(options, started)?;
                match scan.get_json(&key)? {
                    None => {
                        examined_docs += 1;
                        check_doc_budget(budget, examined_docs)?;
                        known_holes.push(HoleEvidence {
                            code: "index_key_absent".into(),
                            key: Some(key),
                        });
                    }
                    Some(doc) => {
                        examined_docs += 1;
                        examined_bytes = examined_bytes.saturating_add(json_byte_len(&doc));
                        check_doc_budget(budget, examined_docs)?;
                        check_bytes_budget(budget, examined_bytes)?;
                        if plan.where_pred.eval(&doc, params)? {
                            last_full_for_cursor = Some((key.clone(), doc.clone()));
                            let value = project_doc(&doc, plan.project.as_ref())?;
                            let row_len = json_byte_len(&value);
                            let next_result = result_bytes.saturating_add(row_len);
                            check_result_budget(budget, next_result)?;
                            result_bytes = next_result;
                            matched.push((key, value));
                            if matched.len() >= need {
                                index_more = iter.next().is_some();
                                break;
                            }
                        }
                    }
                }
            }
        } else {
            let mut full: Vec<(String, JsonValue)> = Vec::new();
            for key in candidates {
                check_governance(options, started)?;
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
                        code: "index_key_absent".into(),
                        key: Some(key),
                    });
                }
            }
            full.sort_by(|(ka, va), (kb, vb)| compare_rows(ka, va, kb, vb, &plan.order));
            if let Some(ref lst) = last_sort_tuple_resume {
                retain_after_sort_tuple(&mut full, &plan.order, lst);
            }
            // remaining_limit is rows still allowed after prior pages.
            if let Some(n) = total_limit {
                full.truncate(n as usize);
            }
            if full.len() > page_size {
                field_order_more = true;
                full.truncate(page_size);
            }
            for (key, doc) in full {
                check_governance(options, started)?;
                last_full_for_cursor = Some((key.clone(), doc.clone()));
                let value = project_doc(&doc, plan.project.as_ref())?;
                let row_len = json_byte_len(&value);
                let next_result = result_bytes.saturating_add(row_len);
                check_result_budget(budget, next_result)?;
                result_bytes = next_result;
                matched.push((key, value));
            }
        }
    } else if key_only_order {
        // Full key-stream until page full or scan ends.
        'outer: loop {
            check_governance(options, started)?;
            let batch = scan.list_keys(Some(256), after.as_deref())?;
            if batch.is_empty() {
                break;
            }
            for key in batch {
                check_governance(options, started)?;
                after = Some(key.clone());
                match scan.get_json(&key)? {
                    None => {
                        known_holes.push(HoleEvidence {
                            code: "key_listed_absent".into(),
                            key: Some(key),
                        });
                        examined_docs += 1;
                        check_doc_budget(budget, examined_docs)?;
                    }
                    Some(doc) => {
                        examined_docs += 1;
                        examined_bytes = examined_bytes.saturating_add(json_byte_len(&doc));
                        check_doc_budget(budget, examined_docs)?;
                        check_bytes_budget(budget, examined_bytes)?;
                        if plan.where_pred.eval(&doc, params)? {
                            last_full_for_cursor = Some((key.clone(), doc.clone()));
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
        // Full scan under budget, sort on full docs, resume by sort-tuple, page + project.
        let mut full: Vec<(String, JsonValue)> = Vec::new();
        loop {
            check_governance(options, started)?;
            let batch = scan.list_keys(Some(256), after.as_deref())?;
            if batch.is_empty() {
                break;
            }
            for key in batch {
                check_governance(options, started)?;
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
        // APP-6 T3: multipage field-order uses last_sort_tuple, not key position.
        if let Some(ref lst) = last_sort_tuple_resume {
            retain_after_sort_tuple(&mut full, &plan.order, lst);
        }
        // remaining_limit is rows still allowed after prior pages.
        if let Some(n) = total_limit {
            full.truncate(n as usize);
        }
        if full.len() > page_size {
            field_order_more = true;
            full.truncate(page_size);
        }
        for (key, doc) in full {
            check_governance(options, started)?;
            last_full_for_cursor = Some((key.clone(), doc.clone()));
            let value = project_doc(&doc, plan.project.as_ref())?;
            let row_len = json_byte_len(&value);
            let next_result = result_bytes.saturating_add(row_len);
            check_result_budget(budget, next_result)?;
            result_bytes = next_result;
            matched.push((key, value));
        }
    }

    let took = matched.len();
    let exhausted = if used_index {
        // Index candidates fully considered unless we stopped early for page size.
        if key_only_order {
            !index_more && took <= need
        } else {
            !field_order_more
        }
    } else if key_only_order {
        took < need
    } else {
        !field_order_more
    };

    // Full-scan key-only: if took == need, probe whether more matches exist.
    let exhausted = if !used_index && key_only_order && took == need {
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
    } else if used_index && key_only_order {
        !index_more
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
        let (last_key, last_doc) = last_full_for_cursor.unwrap_or_else(|| {
            (
                rows.last().map(|r| r.key.clone()).unwrap_or_default(),
                JsonValue::Null,
            )
        });
        let last_tuple = build_sort_tuple(&last_key, &last_doc, &plan.order);
        Some(mint_page_cursor(
            heap_id,
            collection_id,
            &plan.plan_hash(),
            &plan.order,
            &last_tuple,
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

/// Extract field equality constraints for index pushdown (shallow AND of `=` only).
///
/// Returns empty when the predicate is not a pure conjunction of field equalities
/// (or a single equality). Parameters are resolved from `params`.
pub(crate) fn equality_constraints(
    pred: &Predicate,
    params: &BTreeMap<String, JsonValue>,
) -> Vec<(String, JsonValue)> {
    match pred {
        Predicate::True => Vec::new(),
        Predicate::Cmp {
            cmp: CompareOp::Eq,
            left,
            right,
        } => match (left, right) {
            (Operand::Path { path }, other) | (other, Operand::Path { path }) => {
                if let Some(v) = operand_as_json(other, params) {
                    vec![(path.dotted(), v)]
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        },
        Predicate::And { args } => {
            let mut out = Vec::new();
            for a in args {
                let part = equality_constraints(a, params);
                if part.is_empty() {
                    // Non-equality conjunct → do not claim pure eq-AND for pushdown.
                    return Vec::new();
                }
                out.extend(part);
            }
            out
        }
        _ => Vec::new(),
    }
}

fn operand_as_json(op: &Operand, params: &BTreeMap<String, JsonValue>) -> Option<JsonValue> {
    match op {
        Operand::Literal { value } => Some(value.clone()),
        Operand::Param { name } => params.get(name).cloned(),
        Operand::Path { .. } => None,
    }
}

/// Cooperative deadline + cancellation (APB-7 T8).
fn check_governance(options: &QueryRunOptions, started: Instant) -> Result<(), Error> {
    if let Some(ref cancel) = options.cancel {
        cancel.check()?;
    }
    if let Some(deadline) = options.deadline {
        if started.elapsed() >= deadline {
            return Err(Error::DeadlineExceeded(
                "query deadline exceeded".into(),
            ));
        }
    }
    Ok(())
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
    order: &[OrderTerm],
    last_sort_tuple: &JsonValue,
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
        order_normalized: order_normalized_json(order),
        last_sort_tuple: last_sort_tuple.clone(),
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

/// Decode continuation → (`last_sort_tuple`, remaining limit).
fn decode_after(
    cont: &Continuation,
    heap_id: HeapId,
    collection_id: CollectionId,
    plan_hash: &[u8; 32],
) -> Result<(Option<JsonValue>, Option<u64>), Error> {
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
    let rem = if logical.remaining_limit == u64::MAX {
        None
    } else {
        Some(logical.remaining_limit)
    };
    Ok((Some(logical.last_sort_tuple), rem))
}

/// Key resume for key-stream order: last string element of the sort tuple.
fn key_from_sort_tuple(t: &JsonValue) -> Option<String> {
    let arr = t.as_array()?;
    // Prefer last element (key tie-break is last); fall back to first for legacy [key].
    arr.iter()
        .rev()
        .find_map(|v| v.as_str().map(|s| s.to_string()))
        .or_else(|| arr.first().and_then(|v| v.as_str().map(|s| s.to_string())))
}

fn order_normalized_json(order: &[OrderTerm]) -> JsonValue {
    JsonValue::Array(
        order
            .iter()
            .map(|t| {
                serde_json::json!({
                    "path": t.path.0,
                    "dir": match t.dir {
                        OrderDir::Asc => "asc",
                        OrderDir::Desc => "desc",
                    },
                    "nulls": match t.nulls {
                        NullsOrder::Last => "last",
                        NullsOrder::First => "first",
                    },
                    "tie_break": t.tie_break,
                })
            })
            .collect(),
    )
}

/// Sort-tuple for a full document (pre-projection), aligned with [`compare_rows`].
fn build_sort_tuple(key: &str, doc: &JsonValue, order: &[OrderTerm]) -> JsonValue {
    let mut parts = Vec::with_capacity(order.len());
    for term in order {
        if term.tie_break || term.path.0.as_slice() == ["$key"] {
            parts.push(JsonValue::String(key.to_string()));
        } else {
            match resolve_path(doc, &term.path) {
                Resolve::Present(v) => parts.push(v),
                // Distinct from JSON null so nulls placement matches compare_rows.
                Resolve::Absent => parts.push(serde_json::json!({"__rv":"absent"})),
            }
        }
    }
    JsonValue::Array(parts)
}

fn resolve_from_tuple_part(v: &JsonValue) -> Resolve {
    if v.get("__rv").and_then(|x| x.as_str()) == Some("absent") {
        Resolve::Absent
    } else {
        Resolve::Present(v.clone())
    }
}

fn cmp_sort_tuples(a: &JsonValue, b: &JsonValue, order: &[OrderTerm]) -> Ordering {
    let aa = a.as_array().map(|x| x.as_slice()).unwrap_or(&[]);
    let bb = b.as_array().map(|x| x.as_slice()).unwrap_or(&[]);
    for (i, term) in order.iter().enumerate() {
        let av = aa.get(i).unwrap_or(&JsonValue::Null);
        let bv = bb.get(i).unwrap_or(&JsonValue::Null);
        let c = if term.tie_break || term.path.0.as_slice() == ["$key"] {
            let as_ = av.as_str().unwrap_or("");
            let bs_ = bv.as_str().unwrap_or("");
            as_.cmp(bs_)
        } else {
            compare_resolve(&resolve_from_tuple_part(av), &resolve_from_tuple_part(bv), term.nulls)
        };
        if c != Ordering::Equal {
            return apply_dir(c, term.dir);
        }
    }
    Ordering::Equal
}

fn retain_after_sort_tuple(
    full: &mut Vec<(String, JsonValue)>,
    order: &[OrderTerm],
    last: &JsonValue,
) {
    full.retain(|(k, doc)| {
        let t = build_sort_tuple(k, doc, order);
        let c = cmp_sort_tuples(&t, last, order);
        c == Ordering::Greater
    });
}

#[cfg(test)]
mod sort_tuple_tests {
    use super::*;
    use crate::plan_v1::{NullsOrder, OrderDir, OrderTerm};
    use crate::predicate::Path;

    fn term_n() -> OrderTerm {
        OrderTerm {
            path: Path(vec!["n".into()]),
            dir: OrderDir::Asc,
            nulls: NullsOrder::Last,
            tie_break: false,
        }
    }
    fn term_key() -> OrderTerm {
        OrderTerm {
            path: Path(vec!["$key".into()]),
            dir: OrderDir::Asc,
            nulls: NullsOrder::Last,
            tie_break: true,
        }
    }

    #[test]
    fn after_c20_keeps_b30_and_d40() {
        let order = vec![term_n(), term_key()];
        let last = build_sort_tuple("c", &serde_json::json!({"n": 20}), &order);
        assert_eq!(last, serde_json::json!([20, "c"]));
        let mut full: Vec<(String, JsonValue)> = vec![
            ("a".to_string(), serde_json::json!({"n": 10})),
            ("b".to_string(), serde_json::json!({"n": 30})),
            ("c".to_string(), serde_json::json!({"n": 20})),
            ("d".to_string(), serde_json::json!({"n": 40})),
        ];
        full.sort_by(|(ka, va), (kb, vb)| compare_rows(ka, va, kb, vb, &order));
        retain_after_sort_tuple(&mut full, &order, &last);
        let keys: Vec<String> = full.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(keys, vec!["b".to_string(), "d".to_string()], "last={last:?}");
    }
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