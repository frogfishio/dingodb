//! Full-language enrich / within attach IR phase (RQL-IR4).
//!
//! Profile: **`residiuum-query-ir-attach-v1`**
//! Normative: [QUERY_IR_ATTACH_V1.md](../../../../../doc/todo/rql/QUERY_IR_ATTACH_V1.md)
//!
//! Ordered attach pipeline (enrich / within / filter) + brace project run here —
//! not as an inline loop inside [`super::full_attach::execute_full_isa_with`].
//! Still a **Rust IR residual** (not an opcode machine). Decision 0 remains OPEN;
//! RQL-C1 must not be accepted.

use crate::app_v1::{HeapClient, Parameters};
use crate::error::Error;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use super::full_attach::{
    apply_project_rows, attach_enrich_rows, attach_within_rows,
    collect_within_using_names, filter_rows, load_foreign_docs_for_root_enrich, EnrichAttachMode,
    EnrichLoadEvidence, FullPipelineStepV1, ProjectItemV1,
};

/// IR profile id for full-language attach.
pub const ATTACH_IR_PROFILE: &str = "residiuum-query-ir-attach-v1";

/// Compiled attach section (pipeline + optional brace project).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledAttachIr {
    /// Profile stamp.
    pub profile: &'static str,
    /// Ordered enrich / within / filter steps.
    pub pipeline: Vec<FullPipelineStepV1>,
    /// Optional brace project after the pipeline.
    pub project: Option<Vec<ProjectItemV1>>,
}

impl CompiledAttachIr {
    /// Lower decoded ISA full section.
    pub fn lower(
        pipeline: Vec<FullPipelineStepV1>,
        project: Option<Vec<ProjectItemV1>>,
    ) -> Self {
        Self {
            profile: ATTACH_IR_PROFILE,
            pipeline,
            project,
        }
    }

    /// Run attach against base-page rows.
    pub fn run(
        &self,
        client: &mut HeapClient,
        rows: Vec<(String, JsonValue)>,
        parameters: &Parameters,
        force_enrich_scan: bool,
    ) -> Result<(Vec<(String, JsonValue)>, Vec<EnrichLoadEvidence>), Error> {
        run_attach_pipeline(
            client,
            rows,
            &self.pipeline,
            &self.project,
            parameters,
            force_enrich_scan,
        )
    }
}

/// Run enrich / within / filter pipeline, then optional brace project.
pub fn run_attach_pipeline(
    client: &mut HeapClient,
    mut rows: Vec<(String, JsonValue)>,
    pipeline: &[FullPipelineStepV1],
    project: &Option<Vec<ProjectItemV1>>,
    parameters: &Parameters,
    force_enrich_scan: bool,
) -> Result<(Vec<(String, JsonValue)>, Vec<EnrichLoadEvidence>), Error> {
    let mut foreign_cache: BTreeMap<String, Vec<(String, JsonValue)>> = BTreeMap::new();
    let mut enrich_loads = Vec::new();
    for step in pipeline {
        match step {
            FullPipelineStepV1::Enrich(e) => {
                let (foreign, mode) = load_foreign_docs_for_root_enrich(
                    client,
                    e,
                    &rows,
                    force_enrich_scan,
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

    if let Some(fields) = project {
        rows = apply_project_rows(&rows, fields)?;
    }

    Ok((rows, enrich_loads))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_ir_profile_constant() {
        assert_eq!(ATTACH_IR_PROFILE, "residiuum-query-ir-attach-v1");
    }

    #[test]
    fn empty_pipeline_is_identity() {
        let ir = CompiledAttachIr::lower(Vec::new(), None);
        assert_eq!(ir.profile, ATTACH_IR_PROFILE);
        assert!(ir.pipeline.is_empty());
        assert!(ir.project.is_none());
    }
}
