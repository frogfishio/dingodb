//! Application Core page semantics — owned by [`crate::query_bytecode_v1`].
//!
//! Ported from the former standalone `query_exec_v1` executor (Decision 0 /
//! RQL-X2b). Do not grow a second semantic entry outside this module tree.
//!
//! Normative: CORE plan §10 / §14 APP-6 (partial). Compiles via
//! [`crate::rql_app_core::compile_app_core`], scans with `list_keys` + `get`,
//! evaluates predicates via the ENR+SDA kernel ([`super::kernel`]).
//! Core path-project goes through [`super::ir_project`] (RQL-IR1).
//! Core order / sort-tuple goes through [`super::ir_order`] (RQL-IR2).
//! [`Predicate::eval`] is the test oracle only.
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
use crate::cursor_v1::{
    active_cursor_key_ring, mint, parameter_hash as cursor_parameter_hash, CursorLogical,
    VerifyContext, PROFILE as CURSOR_PROFILE,
};
use crate::error::Error;
use crate::plan_v1::{CollectionBindings, NullsOrder, OrderDir, OrderTerm, RqlPlanV1};
use crate::predicate::{CompareOp, Operand, Predicate};
use crate::rql_app_core::{compile_app_core, merge_budgets, CompiledAppCore};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
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

    let where_k = super::kernel::compile_where(&plan.where_pred, params)?;

    let budget = merge_budgets(source_budget, options.budget);
    let page_size = options
        .page_size
        .unwrap_or(plan.page_size)
        .clamp(1, 4_096) as usize;

    let param_hash = cursor_parameter_hash(params);
    let (last_sort_tuple_resume, remaining_limit) = if let Some(ref cont) = options.after {
        decode_after(
            cont,
            heap_id,
            collection_id,
            &plan.plan_hash(),
            &param_hash,
        )?
    } else {
        (None, plan.limit)
    };
    // Key-stream resume (when order is key-only): last element of sort tuple is the key.
    let after_key = last_sort_tuple_resume
        .as_ref()
        .and_then(super::ir_order::key_from_sort_tuple);

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
                        if where_k.eval_doc(&doc)? {
                            last_full_for_cursor = Some((key.clone(), doc.clone()));
                            let value = super::ir_project::apply_project_paths(
                                &doc,
                                plan.project.as_ref(),
                            )?;
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
                    if where_k.eval_doc(&doc)? {
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
            full.sort_by(|(ka, va), (kb, vb)| {
                super::ir_order::compare_rows(ka, va, kb, vb, &plan.order)
            });
            if let Some(ref lst) = last_sort_tuple_resume {
                super::ir_order::retain_after_sort_tuple(&mut full, &plan.order, lst);
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
                let value = super::ir_project::apply_project_paths(
                    &doc,
                    plan.project.as_ref(),
                )?;
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
                        if where_k.eval_doc(&doc)? {
                            last_full_for_cursor = Some((key.clone(), doc.clone()));
                            let value = super::ir_project::apply_project_paths(
                                &doc,
                                plan.project.as_ref(),
                            )?;
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
                    if where_k.eval_doc(&doc)? {
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
        full.sort_by(|(ka, va), (kb, vb)| {
            super::ir_order::compare_rows(ka, va, kb, vb, &plan.order)
        });
        // APP-6 T3: multipage field-order uses last_sort_tuple, not key position.
        if let Some(ref lst) = last_sort_tuple_resume {
            super::ir_order::retain_after_sort_tuple(&mut full, &plan.order, lst);
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
            let value = super::ir_project::apply_project_paths(
                &doc,
                plan.project.as_ref(),
            )?;
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
                    &where_k,
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
        let last_tuple = super::ir_order::build_sort_tuple(&last_key, &last_doc, &plan.order);
        Some(mint_page_cursor(
            heap_id,
            collection_id,
            &plan.plan_hash(),
            &param_hash,
            &plan.order,
            &last_tuple,
            remaining_after,
            page_size as u32,
            plan.coverage,
            plan.consistency,
        )?)
    };

    // APB-7 T9: coverage policy from the plan (RQL/builder); complete-by-default.
    // Run options may only *relax* to IncompleteAllowed when the plan already allows it.
    let coverage_mode = match (plan.coverage, options.coverage) {
        (CoveragePolicy::IncompleteAllowed, CoveragePolicy::IncompleteAllowed) => {
            CoveragePolicy::IncompleteAllowed
        }
        // Plan IncompleteAllowed + run default Complete still honors plan (RQL source).
        (CoveragePolicy::IncompleteAllowed, _) => CoveragePolicy::IncompleteAllowed,
        _ => CoveragePolicy::Complete,
    };
    let hole_count = known_holes.len() as u32;
    if hole_count > 0 && matches!(coverage_mode, CoveragePolicy::Complete) {
        let sample: Vec<&str> = known_holes
            .iter()
            .take(3)
            .map(|h| h.code.as_str())
            .collect();
        return Err(Error::CoverageIncomplete(format!(
            "complete coverage required but {hole_count} known hole(s); \
             sample codes={sample:?} (set coverage incomplete_allowed to allow)"
        )));
    }
    let coverage = if hole_count == 0 {
        CoverageEvidence::complete(coverage_mode, examined_docs)
    } else {
        CoverageEvidence::incomplete(coverage_mode, examined_docs, hole_count)
    };

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
        coverage,
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
    where_k: &super::kernel::CompiledKernelWhere,
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
                if where_k.eval_doc(&doc)? {
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

fn mint_page_cursor(
    heap_id: HeapId,
    collection_id: CollectionId,
    plan_hash: &[u8; 32],
    parameter_hash: &str,
    order: &[OrderTerm],
    last_sort_tuple: &JsonValue,
    remaining_limit: Option<u64>,
    page_size: u32,
    coverage: CoveragePolicy,
    consistency: crate::app_v1::ConsistencyMode,
) -> Result<Continuation, Error> {
    // APB-7 T10: product ring when installed; otherwise vector-lock default.
    let ring = active_cursor_key_ring();
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
        parameter_hash: parameter_hash.to_string(),
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
    parameter_hash: &str,
) -> Result<(Option<JsonValue>, Option<u64>), Error> {
    let ring = active_cursor_key_ring();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ctx = VerifyContext {
        heap_id: format_uuid(heap_id.as_bytes()),
        collection_id: format_uuid(collection_id.as_bytes()),
        plan_hash: Some(hex32(plan_hash)),
        // APB-7 T10: bind parameters into resume (fail-closed on mismatch).
        parameter_hash: Some(parameter_hash.to_string()),
    };
    let logical = crate::cursor_v1::verify(&cont.token, &ctx, &ring, now)?;
    let rem = if logical.remaining_limit == u64::MAX {
        None
    } else {
        Some(logical.remaining_limit)
    };
    Ok((Some(logical.last_sort_tuple), rem))
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
