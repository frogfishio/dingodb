//! Core VM phase helpers (RQL-VM2 / RQL-VM3 / RQL-VM3b).
//!
//! Product Core execute goes through Query VM opcodes → [`CoreFrame`] phases.
//! Each Core opcode owns a real phase body:
//! - **IndexEq** — host equality-index probe
//! - **Scan** — host key source / unfiltered doc load (no `where`)
//! - **Filter** — kernel `where` (+ key-stream get / early-stop)
//! - **Order** — sort-tuple order
//! - **Page** — limit / page-size / field-order resume
//! - **ProjectPaths** — path-project + page artefact
//!
//! **RQL-DEL1:** the demoted `execute_plan` / `run_core_page` orchestrator
//! wrappers (superseded by direct VM opcode dispatch) are deleted; only
//! [`CoreFrame`] phases remain as the shared implementation.
//! Decision 0 remains OPEN; RQL-C1 must not be accepted.

use crate::app_v1::{
    ConsistencyEvidence, HoleEvidence, QueryBudget, QueryId, QueryPage, QueryRow,
    QueryRunOptions,
};
use crate::cursor_v1::parameter_hash as cursor_parameter_hash;
use crate::error::Error;
use crate::plan_v1::OrderTerm;
use crate::predicate::{Path, Predicate};
use crate::rql_app_core::merge_budgets;
use super::vm_exec::{CoreOperands, VmPool};
use residiuum_heap::{CollectionId, HeapId};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::time::Instant;

use super::core_page::{equality_constraints, DocScan};
use super::kernel::CompiledKernelWhere;

fn json_byte_len(v: &JsonValue) -> u64 {
    serde_json::to_vec(v).map(|b| b.len() as u64).unwrap_or(0)
}

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
    where_k: &CompiledKernelWhere,
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
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Pending key source after Scan (RQL-VM3b). Filter consumes this.
enum PendingKeys {
    /// Field-order: docs already in `working`; Filter only applies where.
    Materialized,
    /// Key-stream index path: sorted candidate keys (after resume).
    Index(Vec<String>),
    /// Key-stream full scan: resume cursor for `list_keys`.
    Stream { after: Option<String> },
}

/// Mutable Core pipeline state driven by Query VM opcodes (RQL-VM2/VM3/VM3b).
///
/// Operands come from typed QVM immediates / pool identity — not a full plan body.
pub(crate) struct CoreFrame<'a> {
    plan_hash: [u8; 32],
    coverage: crate::app_v1::CoveragePolicy,
    consistency: crate::app_v1::ConsistencyMode,
    order: Vec<OrderTerm>,
    project: Option<Vec<Path>>,
    params: &'a BTreeMap<String, JsonValue>,
    options: &'a QueryRunOptions,
    heap_id: HeapId,
    collection_id: CollectionId,
    budget: Option<QueryBudget>,
    started: Instant,
    where_k: CompiledKernelWhere,
    page_size: usize,
    param_hash: String,
    last_sort_tuple_resume: Option<JsonValue>,
    remaining_limit: Option<u64>,
    key_only_order: bool,
    /// `None` until [`Self::index_eq`].
    index_keys: Option<Option<Vec<String>>>,
    /// Set by Scan; consumed by Filter (RQL-VM3b).
    pending_keys: Option<PendingKeys>,
    /// Full documents in the working bag (pre-project).
    working: Vec<(String, JsonValue)>,
    known_holes: Vec<HoleEvidence>,
    examined_docs: u64,
    examined_bytes: u64,
    used_index: bool,
    /// Key-stream / index early-stop: more candidates remain.
    index_more: bool,
    /// Field-order: truncated for page size.
    field_order_more: bool,
    saw_scan: bool,
    saw_filter: bool,
    saw_order: bool,
    saw_page: bool,
}

impl<'a> CoreFrame<'a> {
    /// Prepare frame after BindCollection from typed pool + Core opcode operands.
    ///
    /// `program_hash` is the canonical QVM hash of the **complete** program
    /// (including Full attach ops) — used for cursor identity, never an
    /// independent trusted wire field.
    pub fn begin(
        program_hash: [u8; 32],
        pool: &VmPool,
        core: &CoreOperands,
        params: &'a BTreeMap<String, JsonValue>,
        options: &'a QueryRunOptions,
        heap_id: HeapId,
        collection_id: CollectionId,
        source_budget: Option<QueryBudget>,
    ) -> Result<Self, Error> {
        let started = Instant::now();
        check_governance(options, started)?;
        let where_k = super::kernel::compile_where(&core.where_pred, params)?;
        let budget = merge_budgets(source_budget, options.budget);
        let page_size = super::ir_page::resolve_page_size(core.page_size, options.page_size);
        let param_hash = cursor_parameter_hash(params);
        let (last_sort_tuple_resume, remaining_limit) = if let Some(ref cont) = options.after {
            super::ir_page::decode_after(
                cont,
                heap_id,
                collection_id,
                &program_hash,
                &param_hash,
            )?
        } else {
            (None, core.limit)
        };
        let key_only_order = core
            .order
            .iter()
            .all(|t| t.tie_break || t.path.0 == ["$key"]);
        Ok(Self {
            plan_hash: program_hash,
            coverage: pool.coverage,
            consistency: pool.consistency,
            order: core.order.clone(),
            project: core.project.clone(),
            params,
            options,
            heap_id,
            collection_id,
            budget,
            started,
            where_k,
            page_size,
            param_hash,
            last_sort_tuple_resume,
            remaining_limit,
            key_only_order,
            index_keys: None,
            pending_keys: None,
            working: Vec::new(),
            known_holes: Vec::new(),
            examined_docs: 0,
            examined_bytes: 0,
            used_index: false,
            index_more: false,
            field_order_more: false,
            saw_scan: false,
            saw_filter: false,
            saw_order: false,
            saw_page: false,
        })
    }

    /// OpCode::IndexEq — real host equality-index probe (or force-scan skip).
    pub fn index_eq<S: DocScan>(
        &mut self,
        scan: &mut S,
        where_pred: &Predicate,
        force_scan: bool,
    ) -> Result<(), Error> {
        let keys = if force_scan {
            None
        } else {
            let eqs = equality_constraints(where_pred, self.params);
            if eqs.is_empty() {
                None
            } else {
                scan.try_equality_index_keys(&eqs)?
            }
        };
        self.index_keys = Some(keys);
        Ok(())
    }

    /// OpCode::Scan — establish key source or load unfiltered docs.
    ///
    /// Does **not** apply `where` (RQL-VM3b). Key-stream paths leave a
    /// [`PendingKeys`] cursor for Filter; field-order materializes docs now.
    pub fn scan<S: DocScan>(&mut self, scan: &mut S) -> Result<(), Error> {
        let index_keys = self.index_keys.as_ref().ok_or_else(|| {
            Error::QueryInvalid("core frame: Scan before IndexEq".into())
        })?;
        self.used_index = index_keys.is_some();
        let after_key = self
            .last_sort_tuple_resume
            .as_ref()
            .and_then(super::ir_order::key_from_sort_tuple);

        if let Some(mut candidates) = index_keys.clone() {
            candidates.sort();
            if self.key_only_order {
                if let Some(ak) = after_key.as_deref() {
                    candidates.retain(|k| k.as_str() > ak);
                }
                self.pending_keys = Some(PendingKeys::Index(candidates));
            } else {
                self.scan_index_materialize(scan, candidates)?;
                self.pending_keys = Some(PendingKeys::Materialized);
            }
        } else if self.key_only_order {
            self.pending_keys = Some(PendingKeys::Stream { after: after_key });
        } else {
            self.scan_full_unordered(scan)?;
            self.pending_keys = Some(PendingKeys::Materialized);
        }
        self.saw_scan = true;
        Ok(())
    }

    fn scan_index_materialize<S: DocScan>(
        &mut self,
        scan: &mut S,
        candidates: Vec<String>,
    ) -> Result<(), Error> {
        for key in candidates {
            check_governance(self.options, self.started)?;
            if let Some(doc) = scan.get_json(&key)? {
                self.examined_docs += 1;
                self.examined_bytes =
                    self.examined_bytes.saturating_add(json_byte_len(&doc));
                check_doc_budget(self.budget, self.examined_docs)?;
                check_bytes_budget(self.budget, self.examined_bytes)?;
                self.working.push((key, doc));
            } else {
                self.examined_docs += 1;
                check_doc_budget(self.budget, self.examined_docs)?;
                self.known_holes.push(HoleEvidence {
                    code: "index_key_absent".into(),
                    key: Some(key),
                });
            }
        }
        Ok(())
    }

    fn scan_full_unordered<S: DocScan>(&mut self, scan: &mut S) -> Result<(), Error> {
        let mut after: Option<String> = None;
        loop {
            check_governance(self.options, self.started)?;
            let batch = scan.list_keys(Some(256), after.as_deref())?;
            if batch.is_empty() {
                break;
            }
            for key in batch {
                check_governance(self.options, self.started)?;
                after = Some(key.clone());
                if let Some(doc) = scan.get_json(&key)? {
                    self.examined_docs += 1;
                    self.examined_bytes =
                        self.examined_bytes.saturating_add(json_byte_len(&doc));
                    check_doc_budget(self.budget, self.examined_docs)?;
                    check_bytes_budget(self.budget, self.examined_bytes)?;
                    self.working.push((key, doc));
                } else {
                    self.examined_docs += 1;
                    check_doc_budget(self.budget, self.examined_docs)?;
                    self.known_holes.push(HoleEvidence {
                        code: "key_listed_absent".into(),
                        key: Some(key),
                    });
                }
            }
        }
        Ok(())
    }

    /// OpCode::Filter — kernel where; key-stream also gets docs + early-stop.
    pub fn filter<S: DocScan>(&mut self, scan: &mut S) -> Result<(), Error> {
        if !self.saw_scan {
            return Err(Error::QueryInvalid("core frame: Filter before Scan".into()));
        }
        let pending = self.pending_keys.take().ok_or_else(|| {
            Error::QueryInvalid("core frame: Filter without Scan pending keys".into())
        })?;
        let need = super::ir_page::rows_needed(self.remaining_limit, self.page_size);
        match pending {
            PendingKeys::Materialized => {
                let mut kept = Vec::with_capacity(self.working.len());
                for (key, doc) in self.working.drain(..) {
                    check_governance(self.options, self.started)?;
                    if self.where_k.eval_doc(&doc)? {
                        kept.push((key, doc));
                    }
                }
                self.working = kept;
            }
            PendingKeys::Index(candidates) => {
                self.filter_index_stream(scan, candidates, need)?;
            }
            PendingKeys::Stream { after } => {
                self.filter_key_stream(scan, after, need)?;
            }
        }
        self.saw_filter = true;
        Ok(())
    }

    fn filter_index_stream<S: DocScan>(
        &mut self,
        scan: &mut S,
        candidates: Vec<String>,
        need: usize,
    ) -> Result<(), Error> {
        let mut iter = candidates.into_iter();
        while let Some(key) = iter.next() {
            check_governance(self.options, self.started)?;
            match scan.get_json(&key)? {
                None => {
                    self.examined_docs += 1;
                    check_doc_budget(self.budget, self.examined_docs)?;
                    self.known_holes.push(HoleEvidence {
                        code: "index_key_absent".into(),
                        key: Some(key),
                    });
                }
                Some(doc) => {
                    self.examined_docs += 1;
                    self.examined_bytes =
                        self.examined_bytes.saturating_add(json_byte_len(&doc));
                    check_doc_budget(self.budget, self.examined_docs)?;
                    check_bytes_budget(self.budget, self.examined_bytes)?;
                    if self.where_k.eval_doc(&doc)? {
                        self.working.push((key, doc));
                        if self.working.len() >= need {
                            self.index_more = iter.next().is_some();
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn filter_key_stream<S: DocScan>(
        &mut self,
        scan: &mut S,
        after_key: Option<String>,
        need: usize,
    ) -> Result<(), Error> {
        let mut after = after_key;
        'outer: loop {
            check_governance(self.options, self.started)?;
            let batch = scan.list_keys(Some(256), after.as_deref())?;
            if batch.is_empty() {
                break;
            }
            for key in batch {
                check_governance(self.options, self.started)?;
                after = Some(key.clone());
                match scan.get_json(&key)? {
                    None => {
                        self.known_holes.push(HoleEvidence {
                            code: "key_listed_absent".into(),
                            key: Some(key),
                        });
                        self.examined_docs += 1;
                        check_doc_budget(self.budget, self.examined_docs)?;
                    }
                    Some(doc) => {
                        self.examined_docs += 1;
                        self.examined_bytes =
                            self.examined_bytes.saturating_add(json_byte_len(&doc));
                        check_doc_budget(self.budget, self.examined_docs)?;
                        check_bytes_budget(self.budget, self.examined_bytes)?;
                        if self.where_k.eval_doc(&doc)? {
                            self.working.push((key, doc));
                            if self.working.len() >= need {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// OpCode::Order — sort-tuple order (no-op for key-only stream).
    pub fn order(&mut self) -> Result<(), Error> {
        if !self.saw_filter {
            return Err(Error::QueryInvalid("core frame: Order before Filter".into()));
        }
        if !self.key_only_order {
            self.working.sort_by(|(ka, va), (kb, vb)| {
                super::ir_order::compare_rows(ka, va, kb, vb, &self.order)
            });
        }
        self.saw_order = true;
        Ok(())
    }

    /// OpCode::Page — field-order resume + limit + page-size truncate.
    pub fn page(&mut self) -> Result<(), Error> {
        if !self.saw_order {
            return Err(Error::QueryInvalid("core frame: Page before Order".into()));
        }
        if !self.key_only_order {
            if let Some(ref lst) = self.last_sort_tuple_resume {
                super::ir_order::retain_after_sort_tuple(
                    &mut self.working,
                    &self.order,
                    lst,
                );
            }
            if let Some(n) = self.remaining_limit {
                self.working.truncate(n as usize);
            }
            if self.working.len() > self.page_size {
                self.field_order_more = true;
                self.working.truncate(self.page_size);
            }
        }
        self.saw_page = true;
        Ok(())
    }

    /// OpCode::ProjectPaths — path-project + coverage / cursor page artefact.
    pub fn project_paths<S: DocScan>(&mut self, scan: &mut S) -> Result<QueryPage, Error> {
        if !self.saw_page {
            return Err(Error::QueryInvalid(
                "core frame: ProjectPaths before Page".into(),
            ));
        }
        let _ = self.index_keys.take();

        let mut matched: Vec<(String, JsonValue)> = Vec::with_capacity(self.working.len());
        let mut result_bytes: u64 = 0;
        let mut last_full_for_cursor: Option<(String, JsonValue)> = None;

        for (key, doc) in self.working.drain(..) {
            check_governance(self.options, self.started)?;
            last_full_for_cursor = Some((key.clone(), doc.clone()));
            let value =
                super::ir_project::apply_project_paths(&doc, self.project.as_ref())?;
            let row_len = json_byte_len(&value);
            let next_result = result_bytes.saturating_add(row_len);
            check_result_budget(self.budget, next_result)?;
            result_bytes = next_result;
            matched.push((key, value));
        }

        let took = matched.len();
        let need = super::ir_page::rows_needed(self.remaining_limit, self.page_size);
        let exhausted = if self.used_index {
            if self.key_only_order {
                !self.index_more && took <= need
            } else {
                !self.field_order_more
            }
        } else if self.key_only_order {
            took < need
        } else {
            !self.field_order_more
        };

        let exhausted = if !self.used_index && self.key_only_order && took == need {
            if let Some((last_k, _)) = matched.last() {
                let more = scan.list_keys(Some(1), Some(last_k.as_str()))?;
                if more.is_empty() {
                    true
                } else {
                    !has_later_match(
                        scan,
                        last_k,
                        &self.where_k,
                        self.budget,
                        self.examined_docs,
                        self.examined_bytes,
                    )?
                }
            } else {
                true
            }
        } else if self.used_index && self.key_only_order {
            !self.index_more
        } else {
            exhausted
        };

        let rows: Vec<QueryRow> = matched
            .into_iter()
            .map(|(key, value)| QueryRow { key, value })
            .collect();

        let remaining_after = self
            .remaining_limit
            .map(|n| n.saturating_sub(rows.len() as u64));
        let next = if exhausted || rows.is_empty() {
            None
        } else {
            let (last_key, last_doc) = last_full_for_cursor.unwrap_or_else(|| {
                (
                    rows.last().map(|r| r.key.clone()).unwrap_or_default(),
                    JsonValue::Null,
                )
            });
            let last_tuple =
                super::ir_order::build_sort_tuple(&last_key, &last_doc, &self.order);
            Some(super::ir_page::mint_page_cursor(
                self.heap_id,
                self.collection_id,
                &self.plan_hash,
                &self.param_hash,
                &self.order,
                &last_tuple,
                remaining_after,
                self.page_size as u32,
                self.coverage,
                self.consistency,
            )?)
        };

        let coverage_mode =
            super::ir_page::resolve_coverage_mode(self.coverage, self.options.coverage);
        let coverage = super::ir_page::finish_coverage(
            coverage_mode,
            &self.known_holes,
            self.examined_docs,
        )?;

        let _ = result_bytes;
        let _ = self.examined_bytes;

        Ok(QueryPage {
            query_id: QueryId(residiuum_store::random_id().map_err(Error::from)?),
            plan_hash: self.plan_hash,
            heap_id: self.heap_id,
            collection_id: self.collection_id,
            rows,
            next: if exhausted { None } else { next },
            exhausted,
            coverage,
            consistency: ConsistencyEvidence {
                mode: self.options.consistency,
            },
            remaining_limit: remaining_after,
            known_holes: std::mem::take(&mut self.known_holes),
        })
    }
}

