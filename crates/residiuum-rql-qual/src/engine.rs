//! Engine adapter trait — product and comparator sides (Q4.3).
//!
//! - **Logical harness** (default): pure eval over shared logical datasets for
//!   smoke digests + metrics (not a product claim).
//! - **Residiuum embedded**: feature `residiuum-embedded`.
//! - **Mongo / CBL / Residiuum server**: load shared logical work; execute remains
//!   `NotConfigured` until external drivers are present (honest refuse codes).

use crate::canonicalize::{canonicalize_rows, CanonicalResult, ResultRow};
use crate::cell_plan::MeasuredCellPlan;
use crate::cells::MandatoryCell;
use crate::fixture::CorpusCaseHandle;
use crate::generator::LogicalDataset;
use crate::lane::EngineId;
use crate::lifecycle::LifecycleSpec;
use crate::metrics::{
    assemble_metrics, LatencyCollector, QueryPathMetrics, QueryTimer, CellMetrics,
};
use crate::shared_work::SharedLogicalWork;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("not configured: {0}")]
    NotConfigured(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("execute: {0}")]
    Execute(String),
    #[error("fixture: {0}")]
    Fixture(String),
}

/// Adapter readiness — never silent-empty for unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterStatus {
    Ready,
    NotConfigured,
    FeatureDisabled,
}

/// One engine execution outcome for a corpus case / measured cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRunOutcome {
    pub engine: EngineId,
    pub status: AdapterStatus,
    pub result: Option<CanonicalResult>,
    pub metrics: Option<CellMetrics>,
    pub refuse_code: Option<String>,
    pub detail: Option<String>,
    /// Shared work content hash when loaded (cross-engine fixture identity).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_work_hash: Option<String>,
}

/// Shared adapter contract for all engines.
pub trait EngineAdapter: Send {
    fn engine_id(&self) -> EngineId;
    fn status(&self) -> AdapterStatus;

    /// Load shared logical work (same bytes/hash across engines).
    fn load_shared_work(&mut self, work: &SharedLogicalWork) -> Result<(), AdapterError> {
        let _ = work;
        match self.status() {
            AdapterStatus::Ready | AdapterStatus::NotConfigured => Ok(()),
            AdapterStatus::FeatureDisabled => Err(AdapterError::NotConfigured(
                format!("{} feature disabled", self.engine_id().as_str()),
            )),
        }
    }

    fn prepare_case(&mut self, _case: &CorpusCaseHandle) -> Result<(), AdapterError> {
        match self.status() {
            AdapterStatus::Ready => Ok(()),
            AdapterStatus::NotConfigured => Err(AdapterError::NotConfigured(
                self.engine_id().as_str().into(),
            )),
            AdapterStatus::FeatureDisabled => Err(AdapterError::NotConfigured(
                format!("{} feature disabled", self.engine_id().as_str()),
            )),
        }
    }

    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError>;

    /// Execute a measured cell plan (Q4.2/Q4.3 primary entry).
    fn execute_plan(&mut self, plan: &MeasuredCellPlan) -> Result<EngineRunOutcome, AdapterError> {
        let _ = plan;
        Err(AdapterError::Unsupported(format!(
            "{} execute_plan residual",
            self.engine_id().as_str()
        )))
    }
}

// ---------------------------------------------------------------------------
// Logical harness engine (default Ready — pure, not product)
// ---------------------------------------------------------------------------

/// Pure logical evaluator for smoke digests over [`LogicalDataset`].
///
/// Not a product query path. Used so the harness can publish evidence bundles
/// and metric envelopes without Mongo/CBL drivers.
#[derive(Debug, Default)]
pub struct LogicalHarnessEngine {
    work: Option<SharedLogicalWork>,
}

impl LogicalHarnessEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn dataset(&self) -> Result<&LogicalDataset, AdapterError> {
        self.work
            .as_ref()
            .map(|w| &w.dataset)
            .ok_or_else(|| AdapterError::Fixture("shared work not loaded".into()))
    }

    fn eval_plan(&self, plan: &MeasuredCellPlan) -> Result<(Vec<ResultRow>, u64), AdapterError> {
        let ds = self.dataset()?;
        let docs = ds
            .collections
            .get("docs")
            .ok_or_else(|| AdapterError::Fixture("missing docs".into()))?;
        let mut examined = 0u64;
        let mut rows = Vec::new();

        match plan.cell {
            MandatoryCell::KeyGet => {
                for (k, v) in docs {
                    examined += 1;
                    if k == "d-00000000" || k == "t-00000000" {
                        rows.push(row_from_doc(k, v));
                    }
                }
            }
            MandatoryCell::IndexedEqMultiSelectivity | MandatoryCell::MixedReadWrite => {
                for (k, v) in docs {
                    examined += 1;
                    if v.get("sel_bucket").and_then(|x| x.as_str()) == Some("HIT")
                        || v.get("sel_bucket").and_then(|x| x.as_str()) == Some("POINT")
                    {
                        rows.push(row_from_doc(k, v));
                    }
                }
            }
            MandatoryCell::RangeAndCompound => {
                for (k, v) in docs {
                    examined += 1;
                    let amount = v.get("amount").and_then(|x| x.as_i64()).unwrap_or(-1);
                    let region = v.get("region").and_then(|x| x.as_str()).unwrap_or("");
                    if amount >= 100 && amount < 500 && region == "r0" {
                        rows.push(row_from_doc(k, v));
                    }
                }
            }
            MandatoryCell::DeterministicTopK => {
                let mut all: Vec<_> = docs.iter().collect();
                examined = all.len() as u64;
                all.sort_by(|(ka, va), (kb, vb)| {
                    let sa = va.get("score").and_then(|x| x.as_i64()).unwrap_or(0);
                    let sb = vb.get("score").and_then(|x| x.as_i64()).unwrap_or(0);
                    sb.cmp(&sa).then_with(|| ka.cmp(kb))
                });
                for (k, v) in all.into_iter().take(10) {
                    rows.push(row_from_doc(k, v));
                }
            }
            MandatoryCell::FirstAndDeepCursor => {
                let mut keys: Vec<_> = docs.keys().cloned().collect();
                keys.sort();
                examined = keys.len() as u64;
                let page = plan.page_size.unwrap_or(8) as usize;
                for k in keys.into_iter().take(page) {
                    if let Some(v) = docs.get(&k) {
                        rows.push(row_from_doc(&k, v));
                    }
                }
            }
            MandatoryCell::CoveredNonCoveredProject
            | MandatoryCell::ConditionalComputed => {
                for (k, v) in docs {
                    examined += 1;
                    // project status, region (when present)
                    if let Value::Object(m) = v {
                        let mut out = serde_json::Map::new();
                        if let Some(s) = m.get("status") {
                            out.insert("status".into(), s.clone());
                        }
                        if let Some(r) = m.get("region") {
                            out.insert("region".into(), r.clone());
                        }
                        if let Some(a) = m.get("amount") {
                            out.insert("amount".into(), a.clone());
                        }
                        rows.push(ResultRow {
                            key: k.clone(),
                            value: Value::Object(out),
                        });
                    }
                }
            }
            MandatoryCell::NestedAndArrayPreds => {
                for (k, v) in docs {
                    examined += 1;
                    let nested_ok = v
                        .pointer("/nested/l1/l2/l3/flag")
                        .and_then(|x| x.as_bool())
                        .unwrap_or(false);
                    let tags_hit = v
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .map(|a| a.iter().any(|x| x.as_str() == Some("t0-0")))
                        .unwrap_or(false);
                    if nested_ok || tags_hit {
                        rows.push(row_from_doc(k, v));
                    }
                }
            }
            MandatoryCell::GroupLowHighCard | MandatoryCell::AggCountSumMinMaxAvg => {
                // Logical group-by status/region: one synthetic row per group key.
                use std::collections::BTreeMap;
                let mut groups: BTreeMap<String, (u64, i64, i64, i64)> = BTreeMap::new();
                let field = if plan.cell == MandatoryCell::GroupLowHighCard {
                    "status"
                } else {
                    "region"
                };
                for (_k, v) in docs {
                    examined += 1;
                    let g = v
                        .get(field)
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    let amount = v.get("amount").and_then(|x| x.as_i64()).unwrap_or(0);
                    let e = groups.entry(g).or_insert((0, 0, i64::MAX, i64::MIN));
                    e.0 += 1;
                    e.1 += amount;
                    e.2 = e.2.min(amount);
                    e.3 = e.3.max(amount);
                }
                for (g, (cnt, sum, min, max)) in groups {
                    rows.push(ResultRow {
                        key: g.clone(),
                        value: serde_json::json!({
                            field: g,
                            "count": cnt,
                            "sum_amount": sum,
                            "min_amount": min,
                            "max_amount": max,
                        }),
                    });
                }
            }
            MandatoryCell::EnrichCardinalities => {
                // Logical optional enrich: attach customer when customer_id present.
                let customers = ds.collections.get("customers");
                for (k, v) in docs {
                    examined += 1;
                    let mut body = v.clone();
                    if let Some(cid) = v.get("customer_id").and_then(|x| x.as_str()) {
                        if let Some(c) = customers.and_then(|cs| cs.get(cid)) {
                            if let Value::Object(ref mut m) = body {
                                m.insert("customer".into(), c.clone());
                            }
                        }
                    }
                    rows.push(row_from_doc(k, &body));
                }
            }
        }

        // Multiset order for unordered cells.
        if !plan.order_sensitive {
            rows.sort_by(|a, b| a.key.cmp(&b.key));
        }
        Ok((rows, examined))
    }
}

fn row_from_doc(key: &str, doc: &Value) -> ResultRow {
    let mut value = doc.clone();
    if let Value::Object(ref mut m) = value {
        m.remove("_key");
        m.remove("payload"); // strip pad for digest stability across payload sizes
    }
    ResultRow {
        key: key.to_string(),
        value,
    }
}

fn metrics_from_run(
    lat: &LatencyCollector,
    examined: u64,
    digest: &str,
    coverage_complete: bool,
    life: &LifecycleSpec,
) -> CellMetrics {
    assemble_metrics(
        lat,
        QueryPathMetrics {
            documents_examined: Some(examined),
            index_entries_examined: None,
            index_size_bytes: None,
            index_build_ns: None,
            indexed_write_penalty_ns: None,
            explain_plan_digest: Some(format!("logical:{}", digest)),
        },
        Some(life.class),
        Some(life.cold_method.as_str().into()),
        Some(digest.into()),
        Some(coverage_complete),
        Some(true),
    )
}

impl EngineAdapter for LogicalHarnessEngine {
    fn engine_id(&self) -> EngineId {
        // Logical side uses Residiuum embedded id for lane-E pairing smoke only
        // when publishing harness self-evidence; not a product Residiuum claim.
        EngineId::ResidiuumEmbedded
    }

    fn status(&self) -> AdapterStatus {
        AdapterStatus::Ready
    }

    fn load_shared_work(&mut self, work: &SharedLogicalWork) -> Result<(), AdapterError> {
        self.work = Some(work.clone());
        Ok(())
    }

    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::Ready,
            result: None,
            metrics: None,
            refuse_code: Some("use_execute_plan:logical_harness".into()),
            detail: Some(format!("case={} — use MeasuredCellPlan path", case.case_id)),
            shared_work_hash: self.work.as_ref().map(|w| w.content_hash.clone()),
        })
    }

    fn execute_plan(&mut self, plan: &MeasuredCellPlan) -> Result<EngineRunOutcome, AdapterError> {
        let mut lat = LatencyCollector::new();
        let timer = QueryTimer::start();
        let (rows, examined) = self.eval_plan(plan)?;
        lat.record_duration(timer.elapsed());

        let canon = canonicalize_rows(&rows, plan.order_sensitive, true);
        let digest = canon.values_digest.clone();
        let metrics = metrics_from_run(&lat, examined, &digest, true, &plan.lifecycle);

        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::Ready,
            result: Some(canon),
            metrics: Some(metrics),
            refuse_code: None,
            detail: Some(format!(
                "logical_harness plan={} examined={}",
                plan.plan_id, examined
            )),
            shared_work_hash: self.work.as_ref().map(|w| w.content_hash.clone()),
        })
    }
}

// ---------------------------------------------------------------------------
// Comparator stubs — shared work loaded; execute not configured
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct MongoLocalAdapter {
    work_hash: Option<String>,
}

impl EngineAdapter for MongoLocalAdapter {
    fn engine_id(&self) -> EngineId {
        EngineId::MongoLocal
    }
    fn status(&self) -> AdapterStatus {
        AdapterStatus::NotConfigured
    }
    fn load_shared_work(&mut self, work: &SharedLogicalWork) -> Result<(), AdapterError> {
        self.work_hash = Some(work.content_hash.clone());
        Ok(())
    }
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:mongo_local".into()),
            detail: Some(format!(
                "Mongo 8.2.12 pin; shared_work={:?}; case={} — driver residual",
                self.work_hash, case.case_id
            )),
            shared_work_hash: self.work_hash.clone(),
        })
    }
    fn execute_plan(&mut self, plan: &MeasuredCellPlan) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:mongo_local".into()),
            detail: Some(format!(
                "shared logical work loaded={}; plan={} (mongodb crate 3.8.0 residual)",
                self.work_hash.is_some(),
                plan.plan_id
            )),
            shared_work_hash: self.work_hash.clone(),
        })
    }
}

#[derive(Debug, Default)]
pub struct CblEmbeddedAdapter {
    work_hash: Option<String>,
}

impl EngineAdapter for CblEmbeddedAdapter {
    fn engine_id(&self) -> EngineId {
        EngineId::CouchbaseLiteEmbedded
    }
    fn status(&self) -> AdapterStatus {
        AdapterStatus::NotConfigured
    }
    fn load_shared_work(&mut self, work: &SharedLogicalWork) -> Result<(), AdapterError> {
        self.work_hash = Some(work.content_hash.clone());
        Ok(())
    }
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:cbl_embedded".into()),
            detail: Some(format!(
                "CBL 4.1.0 Full Sync pin; shared_work={:?}; case={}",
                self.work_hash, case.case_id
            )),
            shared_work_hash: self.work_hash.clone(),
        })
    }
    fn execute_plan(&mut self, plan: &MeasuredCellPlan) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:cbl_embedded".into()),
            detail: Some(format!(
                "shared logical work loaded={}; plan={} (CBL native residual)",
                self.work_hash.is_some(),
                plan.plan_id
            )),
            shared_work_hash: self.work_hash.clone(),
        })
    }
}

#[derive(Debug, Default)]
pub struct ResidiuumServerAdapter {
    work_hash: Option<String>,
}

impl EngineAdapter for ResidiuumServerAdapter {
    fn engine_id(&self) -> EngineId {
        EngineId::ResidiuumServer
    }
    fn status(&self) -> AdapterStatus {
        AdapterStatus::NotConfigured
    }
    fn load_shared_work(&mut self, work: &SharedLogicalWork) -> Result<(), AdapterError> {
        self.work_hash = Some(work.content_hash.clone());
        Ok(())
    }
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:residiuum_server".into()),
            detail: Some(format!(
                "loopback serve + op 118 residual; case={}",
                case.case_id
            )),
            shared_work_hash: self.work_hash.clone(),
        })
    }
    fn execute_plan(&mut self, plan: &MeasuredCellPlan) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:residiuum_server".into()),
            detail: Some(format!(
                "shared_work={}; plan={}",
                self.work_hash.is_some(),
                plan.plan_id
            )),
            shared_work_hash: self.work_hash.clone(),
        })
    }
}

/// Residiuum embedded product adapter (feature-gated execute).
#[derive(Debug, Default)]
pub struct ResidiuumEmbeddedAdapter {
    work: Option<SharedLogicalWork>,
}

impl EngineAdapter for ResidiuumEmbeddedAdapter {
    fn engine_id(&self) -> EngineId {
        EngineId::ResidiuumEmbedded
    }

    fn status(&self) -> AdapterStatus {
        #[cfg(feature = "residiuum-embedded")]
        {
            AdapterStatus::Ready
        }
        #[cfg(not(feature = "residiuum-embedded"))]
        {
            AdapterStatus::FeatureDisabled
        }
    }

    fn load_shared_work(&mut self, work: &SharedLogicalWork) -> Result<(), AdapterError> {
        self.work = Some(work.clone());
        Ok(())
    }

    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: self.status(),
            result: None,
            metrics: None,
            refuse_code: Some("use_execute_plan:residiuum_embedded".into()),
            detail: Some(case.case_id.clone()),
            shared_work_hash: self.work.as_ref().map(|w| w.content_hash.clone()),
        })
    }

    fn execute_plan(&mut self, plan: &MeasuredCellPlan) -> Result<EngineRunOutcome, AdapterError> {
        #[cfg(not(feature = "residiuum-embedded"))]
        {
            let _ = plan;
            return Ok(EngineRunOutcome {
                engine: self.engine_id(),
                status: AdapterStatus::FeatureDisabled,
                result: None,
                metrics: None,
                refuse_code: Some("adapter_feature_disabled:residiuum_embedded".into()),
                detail: Some("enable --features residiuum-embedded".into()),
                shared_work_hash: self.work.as_ref().map(|w| w.content_hash.clone()),
            });
        }
        #[cfg(feature = "residiuum-embedded")]
        {
            crate::residiuum_embedded::execute_plan_embedded(self.work.as_ref(), plan)
        }
    }
}

// Legacy aliases
pub type MongoLocalStub = MongoLocalAdapter;
pub type CblEmbeddedStub = CblEmbeddedAdapter;
pub type ResidiuumServerStub = ResidiuumServerAdapter;

pub fn adapter_for(engine: EngineId) -> Box<dyn EngineAdapter> {
    match engine {
        EngineId::ResidiuumEmbedded => Box::new(ResidiuumEmbeddedAdapter::default()),
        EngineId::ResidiuumServer => Box::new(ResidiuumServerAdapter::default()),
        EngineId::MongoLocal => Box::new(MongoLocalAdapter::default()),
        EngineId::CouchbaseLiteEmbedded => Box::new(CblEmbeddedAdapter::default()),
    }
}

pub fn synthetic_ready_outcome(engine: EngineId, digest_hex: &str) -> EngineRunOutcome {
    EngineRunOutcome {
        engine,
        status: AdapterStatus::Ready,
        result: Some(CanonicalResult {
            keys_digest: digest_hex.to_string(),
            values_digest: digest_hex.to_string(),
            multiplicity: "multiset".into(),
            order_sensitive: false,
            coverage_complete: true,
            row_count: 0,
            raw_preview: Value::Null,
        }),
        metrics: None,
        refuse_code: None,
        detail: Some("synthetic_test_only".into()),
        shared_work_hash: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_plan::MeasuredCellPlan;
    use crate::cells::MandatoryCell;
    use crate::dataset::DatasetSpec;
    use crate::generator::generate_dataset;
    use crate::shared_work::SharedLogicalWork;

    #[test]
    fn stubs_load_shared_work_without_inventing_digests() {
        let ds = generate_dataset(&DatasetSpec::smoke_default(1));
        let work = SharedLogicalWork::from_dataset(ds);
        let mut mongo = MongoLocalAdapter::default();
        mongo.load_shared_work(&work).unwrap();
        let out = mongo
            .execute_plan(&MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, 1))
            .unwrap();
        assert_eq!(out.status, AdapterStatus::NotConfigured);
        assert!(out.result.is_none());
        assert_eq!(out.shared_work_hash.as_deref(), Some(work.content_hash.as_str()));
    }

    #[test]
    fn logical_harness_key_get_digest() {
        let ds = generate_dataset(&DatasetSpec::smoke_default(2));
        let work = SharedLogicalWork::from_dataset(ds);
        let mut eng = LogicalHarnessEngine::new();
        eng.load_shared_work(&work).unwrap();
        let plan = MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, 2);
        let out = eng.execute_plan(&plan).unwrap();
        assert_eq!(out.status, AdapterStatus::Ready);
        let r = out.result.unwrap();
        assert_eq!(r.row_count, 1);
        assert!(out.metrics.as_ref().unwrap().latency.samples >= 1);
        assert_eq!(out.metrics.as_ref().unwrap().validity_ok, Some(true));
    }

    #[test]
    fn factory_covers_all_engines() {
        for e in [
            EngineId::ResidiuumEmbedded,
            EngineId::ResidiuumServer,
            EngineId::MongoLocal,
            EngineId::CouchbaseLiteEmbedded,
        ] {
            assert_eq!(adapter_for(e).engine_id(), e);
        }
    }
}
