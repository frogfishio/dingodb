//! Engine adapter trait — product and comparator sides.
//!
//! Mongo / CBL return [`AdapterStatus::NotConfigured`] until Q4.3 wiring.
//! Residiuum embedded is optional via feature `residiuum-embedded`.

use crate::canonicalize::CanonicalResult;
use crate::fixture::CorpusCaseHandle;
use crate::lane::EngineId;
use crate::metrics::CellMetrics;
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
    /// When status is Ready and execution succeeded.
    pub result: Option<CanonicalResult>,
    pub metrics: Option<CellMetrics>,
    /// Stable refuse code when the engine refused (not a silent empty success).
    pub refuse_code: Option<String>,
    pub detail: Option<String>,
}

/// Shared adapter contract for all engines.
pub trait EngineAdapter: Send {
    fn engine_id(&self) -> EngineId;
    fn status(&self) -> AdapterStatus;

    /// Load / seed fixtures for this case (no-op when not configured).
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

    /// Execute the case intention (RQL / pipeline / SQL++ as appropriate).
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError>;
}

/// Stub Mongo adapter — local c/s lane comparator (Q4.3 wires driver).
#[derive(Debug, Default)]
pub struct MongoLocalStub;

impl EngineAdapter for MongoLocalStub {
    fn engine_id(&self) -> EngineId {
        EngineId::MongoLocal
    }
    fn status(&self) -> AdapterStatus {
        AdapterStatus::NotConfigured
    }
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:mongo_local".into()),
            detail: Some(format!(
                "Q4.1 stub; case={} (Mongo 8.2.12 pin — wire in Q4.3)",
                case.case_id
            )),
        })
    }
}

/// Stub Couchbase Lite adapter — embedded lane comparator.
#[derive(Debug, Default)]
pub struct CblEmbeddedStub;

impl EngineAdapter for CblEmbeddedStub {
    fn engine_id(&self) -> EngineId {
        EngineId::CouchbaseLiteEmbedded
    }
    fn status(&self) -> AdapterStatus {
        AdapterStatus::NotConfigured
    }
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:cbl_embedded".into()),
            detail: Some(format!(
                "Q4.1 stub; case={} (CBL 4.1.0 pin — wire in Q4.3)",
                case.case_id
            )),
        })
    }
}

/// Residiuum server (lane S) — not wired in Q4.1 (op 118 path residual).
#[derive(Debug, Default)]
pub struct ResidiuumServerStub;

impl EngineAdapter for ResidiuumServerStub {
    fn engine_id(&self) -> EngineId {
        EngineId::ResidiuumServer
    }
    fn status(&self) -> AdapterStatus {
        AdapterStatus::NotConfigured
    }
    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        Ok(EngineRunOutcome {
            engine: self.engine_id(),
            status: AdapterStatus::NotConfigured,
            result: None,
            metrics: None,
            refuse_code: Some("adapter_not_configured:residiuum_server".into()),
            detail: Some(format!(
                "Q4.1 stub; case={} (loopback serve + op 118 — Q4.3)",
                case.case_id
            )),
        })
    }
}

/// Residiuum embedded adapter.
///
/// Without feature `residiuum-embedded`: reports FeatureDisabled.
/// With feature: still a **scaffold** in Q4.1 — prepare/execute return honest
/// residual until Q4.2/Q4.3 cell runners land (no competitive digest claim).
#[derive(Debug, Default)]
pub struct ResidiuumEmbeddedAdapter {
    /// Optional last prepare note for diagnostics.
    pub last_case: Option<String>,
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

    fn prepare_case(&mut self, case: &CorpusCaseHandle) -> Result<(), AdapterError> {
        self.last_case = Some(case.case_id.clone());
        match self.status() {
            AdapterStatus::Ready => Ok(()),
            AdapterStatus::FeatureDisabled => Err(AdapterError::NotConfigured(
                "enable feature residiuum-embedded".into(),
            )),
            AdapterStatus::NotConfigured => Err(AdapterError::NotConfigured(
                "residiuum_embedded".into(),
            )),
        }
    }

    fn execute_case(&mut self, case: &CorpusCaseHandle) -> Result<EngineRunOutcome, AdapterError> {
        match self.status() {
            AdapterStatus::FeatureDisabled | AdapterStatus::NotConfigured => Ok(EngineRunOutcome {
                engine: self.engine_id(),
                status: self.status(),
                result: None,
                metrics: None,
                refuse_code: Some("adapter_feature_disabled:residiuum_embedded".into()),
                detail: Some(format!(
                    "rebuild with --features residiuum-embedded; case={}",
                    case.case_id
                )),
            }),
            AdapterStatus::Ready => {
                // Q4.1: scaffold only — do not claim product digests yet.
                // Q4.2+ will materialise fixtures and call CollectionClient::rql.
                let _src = case.rql_source.as_deref().unwrap_or("");
                Ok(EngineRunOutcome {
                    engine: self.engine_id(),
                    status: AdapterStatus::Ready,
                    result: None,
                    metrics: None,
                    refuse_code: Some("scaffold_pending_q4_2_execute:residiuum_embedded".into()),
                    detail: Some(format!(
                        "adapter ready; execute residual until Q4.2 cell runner (case={})",
                        case.case_id
                    )),
                })
            }
        }
    }
}

/// Factory for adapters by engine id (stubs by default).
pub fn adapter_for(engine: EngineId) -> Box<dyn EngineAdapter> {
    match engine {
        EngineId::ResidiuumEmbedded => Box::new(ResidiuumEmbeddedAdapter::default()),
        EngineId::ResidiuumServer => Box::new(ResidiuumServerStub),
        EngineId::MongoLocal => Box::new(MongoLocalStub),
        EngineId::CouchbaseLiteEmbedded => Box::new(CblEmbeddedStub),
    }
}

/// Escape hatch for tests that need a synthetic ready result (never product).
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::CorpusCaseHandle;

    fn dummy_case() -> CorpusCaseHandle {
        CorpusCaseHandle {
            case_id: "t-1".into(),
            tier: "A".into(),
            domain: "test".into(),
            plain_english_intent: None,
            generator_id: None,
            seed: None,
            rql_source: Some("from orders".into()),
            server_lane_ineligible: false,
            lane_hint: None,
        }
    }

    #[test]
    fn stubs_never_invent_digests() {
        let mut mongo = MongoLocalStub;
        let out = mongo.execute_case(&dummy_case()).unwrap();
        assert_eq!(out.status, AdapterStatus::NotConfigured);
        assert!(out.result.is_none());
        assert!(out.refuse_code.as_ref().unwrap().contains("not_configured"));

        let mut cbl = CblEmbeddedStub;
        let out = cbl.execute_case(&dummy_case()).unwrap();
        assert!(out.result.is_none());
    }

    #[test]
    fn factory_covers_all_engines() {
        for e in [
            EngineId::ResidiuumEmbedded,
            EngineId::ResidiuumServer,
            EngineId::MongoLocal,
            EngineId::CouchbaseLiteEmbedded,
        ] {
            let a = adapter_for(e);
            assert_eq!(a.engine_id(), e);
        }
    }
}
