//! Core VM phase helpers (RQL-VM2).
//!
//! Product Core execute goes through Query VM opcodes → [`CoreFrame`] phases.
//! [`super::core_page::execute_plan`] is a demoted thin wrapper.
//! Decision 0 remains OPEN; RQL-C1 must not be accepted.

use crate::app_v1::{
    ConsistencyEvidence, HoleEvidence, QueryBudget, QueryId, QueryPage, QueryRow,
    QueryRunOptions,
};
use crate::cursor_v1::parameter_hash as cursor_parameter_hash;
use crate::error::Error;
use crate::plan_v1::RqlPlanV1;
use crate::rql_app_core::merge_budgets;
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::time::Instant;

use super::core_page::{equality_constraints, DocScan};

fn json_byte_len(v: &JsonValue) -> u64 {
    // Compact JSON encoding length — stable enough for budget accounting.
    serde_json::to_vec(v).map(|b| b.len() as u64).unwrap_or(0)
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


/// Mutable Core pipeline state driven by Query VM opcodes (RQL-VM2).
pub(crate) struct CoreFrame<'a> {
    plan: &'a RqlPlanV1,
    params: &'a BTreeMap<String, JsonValue>,
    options: &'a QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    budget: Option<QueryBudget>,
    /// `None` until [`Self::index_eq`].
    index_keys: Option<Option<Vec<String>>>,
    saw_scan: bool,
    saw_filter: bool,
    saw_order: bool,
    saw_page: bool,
}

impl<'a> CoreFrame<'a> {
    /// Prepare frame after BindCollection.
    pub fn begin(
        plan: &'a RqlPlanV1,
        params: &'a BTreeMap<String, JsonValue>,
        options: &'a QueryRunOptions,
        heap_id: HeapId,
        collection_id: CollectionId,
        source_budget: Option<QueryBudget>,
    ) -> Result<Self, Error> {
        Ok(Self {
            plan,
            params,
            options,
            heap_id,
            collection_id,
            budget: source_budget,
            index_keys: None,
            saw_scan: false,
            saw_filter: false,
            saw_order: false,
            saw_page: false,
        })
    }

    /// OpCode::IndexEq — real host equality-index probe.
    pub fn index_eq<S: DocScan>(&mut self, scan: &mut S) -> Result<(), Error> {
        let eqs = equality_constraints(&self.plan.where_pred, self.params);
        let keys = if eqs.is_empty() {
            None
        } else {
            scan.try_equality_index_keys(&eqs)?
        };
        self.index_keys = Some(keys);
        Ok(())
    }

    /// OpCode::Scan.
    pub fn scan(&mut self) -> Result<(), Error> {
        if self.index_keys.is_none() {
            return Err(Error::QueryInvalid("core frame: Scan before IndexEq".into()));
        }
        self.saw_scan = true;
        Ok(())
    }

    /// OpCode::Filter.
    pub fn filter(&mut self) -> Result<(), Error> {
        if !self.saw_scan {
            return Err(Error::QueryInvalid("core frame: Filter before Scan".into()));
        }
        self.saw_filter = true;
        Ok(())
    }

    /// OpCode::Order.
    pub fn order(&mut self) -> Result<(), Error> {
        if !self.saw_filter {
            return Err(Error::QueryInvalid("core frame: Order before Filter".into()));
        }
        self.saw_order = true;
        Ok(())
    }

    /// OpCode::Page.
    pub fn page(&mut self) -> Result<(), Error> {
        if !self.saw_order {
            return Err(Error::QueryInvalid("core frame: Page before Order".into()));
        }
        self.saw_page = true;
        Ok(())
    }

    /// OpCode::ProjectPaths — completes Core page via [`run_core_page`].
    ///
    /// Scan/Filter/Order/Page gate sequencing; IndexEq owns the host index probe.
    /// Materialize/page/project remain one function for APP-6 equivalence (honest residual).
    pub fn project_paths<S: DocScan>(&mut self, scan: &mut S) -> Result<QueryPage, Error> {
        if !self.saw_page {
            return Err(Error::QueryInvalid("core frame: ProjectPaths before Page".into()));
        }
        let pre = self.index_keys.take().ok_or_else(|| {
            Error::QueryInvalid("core frame: ProjectPaths without IndexEq".into())
        })?;
        run_core_page(
            scan,
            self.plan,
            self.params,
            self.options,
            self.heap_id,
            self.collection_id,
            self.budget,
            Some(pre),
        )
    }
}


/// Core page materialize (Scan→Project). Invoked from [`CoreFrame::project_paths`].
pub(crate) fn run_core_page<S: DocScan>(
    scan: &mut S,
    plan: &RqlPlanV1,
    params: &BTreeMap<String, JsonValue>,
    options: &QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    source_budget: Option<QueryBudget>,
    precomputed_index_keys: Option<Option<Vec<String>>>,
) -> Result<QueryPage, Error> {
    let started = Instant::now();
    check_governance(options, started)?;

    let where_k = super::kernel::compile_where(&plan.where_pred, params)?;

    let budget = merge_budgets(source_budget, options.budget);
    let page_size = super::ir_page::resolve_page_size(plan.page_size, options.page_size);

    let param_hash = cursor_parameter_hash(params);
    let (last_sort_tuple_resume, remaining_limit) = if let Some(ref cont) = options.after {
        super::ir_page::decode_after(
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
    let need = super::ir_page::rows_needed(total_limit, page_size);

    // Key-stream order when order is only key tie-break; else collect+sort.
    let key_only_order = plan
        .order
        .iter()
        .all(|t| t.tie_break || t.path.0 == ["$key"]);

    // APB-7 T4: equality index — prefer IndexEq phase probe when provided.
    let index_keys = if let Some(pre) = precomputed_index_keys {
        pre
    } else {
        let eqs = equality_constraints(&plan.where_pred, params);
        if eqs.is_empty() {
            None
        } else {
            scan.try_equality_index_keys(&eqs)?
        }
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
        Some(super::ir_page::mint_page_cursor(
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
    let coverage_mode =
        super::ir_page::resolve_coverage_mode(plan.coverage, options.coverage);
    let coverage =
        super::ir_page::finish_coverage(coverage_mode, &known_holes, examined_docs)?;

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
