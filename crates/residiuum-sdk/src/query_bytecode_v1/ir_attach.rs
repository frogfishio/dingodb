//! Full-language enrich / within attach IR phase (RQL-IR4).
//!
//! Profile: **`residiuum-query-ir-attach-v1`**
//! Normative: [QUERY_IR_ATTACH_V1.md](../../../../../doc/todo/rql/QUERY_IR_ATTACH_V1.md)
//!
//! **RQL-DEL1:** the [`run_attach_pipeline`] / [`CompiledAttachIr::run`]
//! Rust-loop orchestrator is **test-only** — product Full execute dispatches
//! the same enrich/within/filter steps as flat Query VM opcodes
//! ([`super::vm_exec::run_vm`] + [`super::vm_exec::emit_attach_pipeline`]).
//! The IR types ([`CompiledAttachIr`], [`ATTACH_IR_PROFILE`]) remain named
//! here for architecture-gate honesty; they are not a live product path.
//! Decision 0 remains OPEN; RQL-C1 must not be accepted.

#[cfg(test)]
use crate::app_v1::Parameters;
#[cfg(test)]
use crate::error::Error;
#[cfg(test)]
use residiuum_heap::CollectionId;
#[cfg(test)]
use serde_json::Value as JsonValue;
#[cfg(test)]
use std::collections::BTreeMap;

use super::full_attach::{FullPipelineStepV1, ProjectItemV1};
#[cfg(test)]
use super::full_attach::{
    apply_project_rows, attach_enrich_rows, attach_within_rows, collect_within_using_names,
    filter_rows, load_foreign_docs_for_root_enrich, EnrichAttachMode, EnrichLoadEvidence,
};
#[cfg(test)]
use super::HostCapabilities;

/// IR profile id for full-language attach.
pub const ATTACH_IR_PROFILE: &str = "residiuum-query-ir-attach-v1";

/// Compiled attach section (pipeline + optional brace project).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompiledAttachIr {
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

    /// Run attach against base-page rows via collection-qualified host (RQL-P1b).
    ///
    /// **Test-only** (RQL-DEL1): product Full execute runs attach as flat VM
    /// opcodes, not this Rust-loop orchestrator.
    #[cfg(test)]
    pub fn run<H: HostCapabilities>(
        &self,
        host: &mut H,
        rows: Vec<(String, JsonValue)>,
        parameters: &Parameters,
        force_enrich_scan: bool,
    ) -> Result<(Vec<(String, JsonValue)>, Vec<EnrichLoadEvidence>), Error> {
        run_attach_pipeline(
            host,
            rows,
            &self.pipeline,
            &self.project,
            parameters,
            force_enrich_scan,
        )
    }
}

/// Run enrich / within / filter pipeline, then optional brace project.
///
/// **Test-only** (RQL-DEL1): superseded by flat VM opcode dispatch in
/// [`super::vm_exec::run_vm`]; kept for IR unit tests / oracle comparison.
#[cfg(test)]
pub(crate) fn run_attach_pipeline<H: HostCapabilities>(
    host: &mut H,
    mut rows: Vec<(String, JsonValue)>,
    pipeline: &[FullPipelineStepV1],
    project: &Option<Vec<ProjectItemV1>>,
    parameters: &Parameters,
    force_enrich_scan: bool,
) -> Result<(Vec<(String, JsonValue)>, Vec<EnrichLoadEvidence>), Error> {
    let mut foreign_cache: BTreeMap<CollectionId, Vec<(String, JsonValue)>> = BTreeMap::new();
    let mut enrich_loads = Vec::new();
    for step in pipeline {
        match step {
            FullPipelineStepV1::Enrich(e) => {
                let (foreign, mode) = load_foreign_docs_for_root_enrich(
                    host,
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
                // with a partial collection view under the same using id.
                if mode == EnrichAttachMode::Scan {
                    foreign_cache
                        .entry(e.using_id)
                        .or_insert_with(|| foreign.clone());
                }
                rows = attach_enrich_rows(&rows, &foreign, e, &parameters.values)?;
            }
            FullPipelineStepV1::Within(w) => {
                collect_within_using_names(w, &mut foreign_cache, host)?;
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
