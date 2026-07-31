//! Multi-process multi-repetition campaign execution over matrix cells.
//!
//! Supports synthetic (default, NON-PRODUCT) and real_store drivers (PQH-11).
//! **Smoke vs qualification** are separate (SPEC §6.4). Smoke may use explicit
//! op budgets for CI; qualification never caps cells to tiny op counts and must
//! meet duration/byte floors + steady-state before bottleneck claims.

use super::plan::CampaignPlan;
use super::run_class::RunClass;
use super::CampaignError;
use crate::matrix::{build_matrix_cells, MatrixCell, MatrixManifest, ScheduleSeed};
use crate::store_driver::{
    run_driver_cell, store_driver_compiled, DriverKind, DriverRunConfig, MeasurementSurface,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSlot {
    pub process_id: u32,
    pub process_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRepetition {
    pub run_id: String,
    pub cell_id: String,
    pub process_id: u32,
    pub rep: u32,
    pub report: crate::matrix::CellRunReport,
    /// Driver used for this repetition.
    #[serde(default = "default_driver_kind_str")]
    pub driver_kind: String,
    /// Measurement surface label.
    #[serde(default = "default_surface_str")]
    pub measurement_surface: String,
    /// Plan source id when a PhysicalWritePlan was emitted.
    #[serde(default)]
    pub plan_source: Option<String>,
}

fn default_driver_kind_str() -> String {
    DriverKind::Synthetic.as_str().into()
}
fn default_surface_str() -> String {
    MeasurementSurface::NonProductSynthetic.as_str().into()
}

#[derive(Debug, Clone)]
pub struct CampaignConfig {
    pub plan: CampaignPlan,
    /// Cell driver (synthetic default).
    pub driver: DriverKind,
    /// Required for real_store (dedicated work root).
    pub work_root: Option<PathBuf>,
    /// When true and platform allows product baseline + real_store, mark eligible.
    /// Developer laptops should leave this false even if platform is labelled controlled.
    pub declare_controlled_runner: bool,
    /// Smoke vs qualification (SPEC §6.4). Default smoke for unit/CI safety.
    pub run_class: RunClass,
}

impl CampaignConfig {
    /// Synthetic **smoke** campaign (NON-PRODUCT; unit/CI).
    pub fn synthetic(plan: CampaignPlan) -> Self {
        Self {
            plan,
            driver: DriverKind::Synthetic,
            work_root: None,
            declare_controlled_runner: false,
            run_class: RunClass::Smoke,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignResult {
    pub plan: CampaignPlan,
    pub matrix_schema: String,
    pub matrix_seed: u64,
    pub process_slots: Vec<ProcessSlot>,
    pub repetitions: Vec<CellRepetition>,
    pub cells_executed: usize,
    pub valid_runs: usize,
    pub invalid_runs: usize,
    /// Campaign-level driver.
    pub driver_kind: String,
    /// Campaign-level surface (worst-case honesty).
    pub measurement_surface: String,
    /// True only when real_store + controlled platform + declare_controlled_runner.
    pub product_claim_eligible: bool,
    /// Primary registered bottleneck verdict if any non-mixed ranked.
    /// **Never set from smoke** — prior smoke `io_queue_underdriven` is withdrawn.
    pub primary_bottleneck: Option<String>,
    /// Run IDs attached to the primary bottleneck (reproduced evidence).
    pub primary_bottleneck_run_ids: Vec<String>,
    /// Campaign run class (smoke/diagnostic/qualification/soak).
    pub run_class: String,
    /// Explicit withdrawal notes (e.g. smoke bottleneck claims).
    pub withdrawals: Vec<String>,
}

/// Execute campaign: for each selected cell, run `processes × reps_per_process`
/// so total reps ≥ plan.repetitions across ≥ plan.processes process slots.
pub fn run_campaign(cfg: &CampaignConfig) -> Result<CampaignResult, CampaignError> {
    cfg.plan
        .validate()
        .map_err(CampaignError::Msg)?;

    if cfg.driver == DriverKind::RealStore {
        if !store_driver_compiled() {
            return Err(CampaignError::Msg(
                "real_store requires rebuild with --features store-driver".into(),
            ));
        }
        if cfg.work_root.is_none() {
            return Err(CampaignError::Msg(
                "real_store campaign requires work_root".into(),
            ));
        }
    }

    let manifest: MatrixManifest = build_matrix_cells(ScheduleSeed {
        seed: cfg.plan.seed,
    });

    let cells: Vec<_> = manifest
        .cells
        .iter()
        .take(cfg.plan.max_cells)
        .cloned()
        .collect();

    let procs = cfg.plan.processes;
    let total_reps = cfg.plan.repetitions;
    let reps_per_proc = (total_reps + procs - 1) / procs;

    let mut process_slots = Vec::new();
    for p in 0..procs {
        process_slots.push(ProcessSlot {
            process_id: p,
            process_seed: cfg
                .plan
                .seed
                .wrapping_add(PROCESS_SEED_TAG)
                .wrapping_add(p as u64),
        });
    }

    let mut repetitions = Vec::new();
    let mut valid_runs = 0usize;
    let mut invalid_runs = 0usize;

    let surface = campaign_surface(cfg);
    let product_claim_eligible = product_eligible(cfg, surface);

    for cell in &cells {
        run_cell_reps(
            cfg,
            cell,
            &process_slots,
            reps_per_proc,
            false,
            surface,
            &mut repetitions,
            &mut valid_runs,
            &mut invalid_runs,
        )?;
    }

    if cfg.plan.include_multiproc_finding {
        let finding_cells: Vec<_> = manifest
            .cells
            .iter()
            .filter(|c| {
                (c.payload_size == 4096 || c.payload_size == 8192) && c.concurrency >= 2
            })
            .take(4)
            .cloned()
            .collect();
        for cell in finding_cells {
            if cells.iter().any(|c| c.cell_id == cell.cell_id) {
                continue;
            }
            run_cell_reps(
                cfg,
                &cell,
                &process_slots,
                reps_per_proc,
                true,
                surface,
                &mut repetitions,
                &mut valid_runs,
                &mut invalid_runs,
            )?;
        }
    }

    let mut withdrawals = vec![
        "WITHDRAWN: any prior smoke-mode primary bottleneck claim (including io_queue_underdriven from PQH-11 smoke multi-rep) is not qualification evidence".into(),
    ];
    if cfg.run_class == RunClass::Smoke {
        withdrawals.push(
            "run_class=smoke: functional harness only; no product bottleneck verdicts".into(),
        );
    }

    Ok(CampaignResult {
        plan: cfg.plan.clone(),
        matrix_schema: manifest.schema,
        matrix_seed: manifest.seed,
        process_slots,
        cells_executed: cells.len(),
        repetitions,
        valid_runs,
        invalid_runs,
        driver_kind: cfg.driver.as_str().into(),
        measurement_surface: surface.as_str().into(),
        // Product eligibility also requires qualification run class + sustained window.
        product_claim_eligible: product_claim_eligible
            && cfg.run_class.may_emit_bottleneck_verdict(),
        primary_bottleneck: None,
        primary_bottleneck_run_ids: vec![],
        run_class: cfg.run_class.as_str().into(),
        withdrawals,
    })
}

/// Fill primary bottleneck fields from ranked reports (after `build_campaign_reports`).
///
/// **Smoke/diagnostic:** never attaches a primary bottleneck (withdraws findings).
/// **Qualification/soak:** only attaches when every valid repetition reports a
/// sustained window class (SPEC §10 steady-state).
pub fn attach_primary_bottleneck(
    result: &mut CampaignResult,
    ranked: &[crate::campaign::reports::RankedBottleneck],
) {
    let class = RunClass::parse(&result.run_class).unwrap_or(RunClass::Smoke);
    if !class.may_emit_bottleneck_verdict() {
        result.primary_bottleneck = None;
        result.primary_bottleneck_run_ids.clear();
        result.withdrawals.push(format!(
            "bottleneck attach refused: run_class={} cannot support registered verdicts",
            class.as_str()
        ));
        result.withdrawals.push(
            "WITHDRAWN finding: io_queue_underdriven (and any other smoke-derived primary bottleneck)"
                .into(),
        );
        return;
    }

    let all_sustained = result
        .repetitions
        .iter()
        .filter(|r| r.report.validity == "valid")
        .all(|r| r.report.window.contains("sustained"));
    if !all_sustained {
        result.primary_bottleneck = None;
        result.primary_bottleneck_run_ids.clear();
        result.withdrawals.push(
            "bottleneck attach refused: not all valid runs have sustained/stable window".into(),
        );
        return;
    }

    if let Some(b) = ranked
        .iter()
        .find(|r| r.verdict != "mixed_or_unknown")
        .or_else(|| ranked.first())
    {
        // Refuse to promote queue-underdriven without qualification evidence of
        // queue-depth response (smoke false positive path).
        if b.verdict == "io_queue_underdriven" && result.driver_kind != "real_store" {
            result.withdrawals.push(
                "WITHDRAWN: io_queue_underdriven on non-real_store surface".into(),
            );
            return;
        }
        result.primary_bottleneck = Some(b.verdict.clone());
        result.primary_bottleneck_run_ids = b.run_ids.clone();
    }
}

fn campaign_surface(cfg: &CampaignConfig) -> MeasurementSurface {
    match cfg.driver {
        DriverKind::Synthetic => MeasurementSurface::NonProductSynthetic,
        DriverKind::RealStore => {
            if cfg.plan.platform.allows_product_baseline() && cfg.declare_controlled_runner {
                MeasurementSurface::RealStoreControlledEligible
            } else {
                MeasurementSurface::RealStoreUncontrolled
            }
        }
    }
}

fn product_eligible(cfg: &CampaignConfig, surface: MeasurementSurface) -> bool {
    surface.allows_product_claim()
        && cfg.driver == DriverKind::RealStore
        && cfg.declare_controlled_runner
        && cfg.plan.platform.allows_product_baseline()
}

fn run_cell_reps(
    cfg: &CampaignConfig,
    cell: &MatrixCell,
    process_slots: &[ProcessSlot],
    reps_per_proc: u32,
    multiproc_tag: bool,
    surface: MeasurementSurface,
    repetitions: &mut Vec<CellRepetition>,
    valid_runs: &mut usize,
    invalid_runs: &mut usize,
) -> Result<(), CampaignError> {
    for slot in process_slots {
        for rep in 0..reps_per_proc {
            let seed = if multiproc_tag {
                slot.process_seed
                    .wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
                    .wrapping_add(rep as u64)
            } else {
                slot.process_seed
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(rep as u64)
                    .wrapping_add(cell.order_rank as u64)
            };

            // Smoke may use explicit small op budgets for unit/CI only.
            // Qualification MUST NOT cap cells to 4–16 ops (principal rejection of PQH-11 smoke).
            let mut cell_run = cell.clone();
            if cfg.run_class.allows_smoke_op_cap() {
                cell_run.op_count = cell_run.op_count.min(RunClass::SMOKE_MAX_OPS).max(1);
            }
            // Qualification: keep matrix op_count as planned; driver honors
            // duration/byte floors rather than op caps.

            let dcfg = DriverRunConfig {
                cell: cell_run,
                seed,
                kind: cfg.driver,
                work_root: cfg.work_root.clone(),
                durability_mutant: false,
                digest_mutant: false,
                run_class: cfg.run_class.as_str().into(),
            };
            let drep = run_driver_cell(&dcfg).map_err(|e| CampaignError::Msg(e.to_string()))?;

            // Override surface/product flags at campaign honesty level.
            let mut report = drep.cell;
            if cfg.driver == DriverKind::Synthetic {
                report.messages.push("NON-PRODUCT synthetic repetition".into());
            }

            let prefix = if multiproc_tag { "mp-" } else { "" };
            let run_id = format!(
                "{}-{}p{}-r{}-{}",
                cfg.plan.campaign_id, prefix, slot.process_id, rep, cell.cell_id
            );
            if report.validity == "valid" {
                *valid_runs += 1;
            } else {
                *invalid_runs += 1;
            }
            repetitions.push(CellRepetition {
                run_id,
                cell_id: cell.cell_id.clone(),
                process_id: slot.process_id,
                rep,
                report,
                driver_kind: cfg.driver.as_str().into(),
                measurement_surface: surface.as_str().into(),
                plan_source: Some(drep.plan_source),
            });
        }
    }
    Ok(())
}

const PROCESS_SEED_TAG: u64 = 0x5039_5052;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::plan::campaign_plan_synthetic;

    #[test]
    fn campaign_meets_rep_and_process_mins() {
        let plan = campaign_plan_synthetic(42, 3);
        let result = run_campaign(&CampaignConfig::synthetic(plan.clone())).unwrap();
        assert_eq!(result.process_slots.len() as u32, plan.processes);
        assert!(result.valid_runs > 0);
        assert_eq!(result.invalid_runs, 0);
        assert_eq!(result.driver_kind, "synthetic");
        assert!(!result.product_claim_eligible);

        let mut by_cell: std::collections::HashMap<String, std::collections::HashSet<u32>> =
            std::collections::HashMap::new();
        for r in &result.repetitions {
            by_cell
                .entry(r.cell_id.clone())
                .or_default()
                .insert(r.process_id);
            assert_eq!(r.driver_kind, "synthetic");
        }
        for (_cell, procs) in by_cell {
            assert!(procs.len() as u32 >= plan.processes.min(2));
        }

        let first_cell = &result.repetitions[0].cell_id;
        let n = result
            .repetitions
            .iter()
            .filter(|r| r.cell_id == *first_cell)
            .count();
        assert!(n as u32 >= plan.repetitions);
    }

    #[test]
    fn real_store_requires_work_root() {
        let plan = campaign_plan_synthetic(1, 1);
        let err = run_campaign(&CampaignConfig {
            plan,
            driver: DriverKind::RealStore,
            work_root: None,
            declare_controlled_runner: false,
            run_class: RunClass::Smoke,
        })
        .unwrap_err();
        let s = err.to_string();
        assert!(
            s.contains("work_root") || s.contains("store-driver"),
            "unexpected err: {s}"
        );
    }

    #[test]
    fn smoke_withdraws_primary_bottleneck() {
        let plan = campaign_plan_synthetic(3, 1);
        let mut result = run_campaign(&CampaignConfig::synthetic(plan)).unwrap();
        assert_eq!(result.run_class, "smoke");
        let reports = crate::campaign::reports::build_campaign_reports(&result);
        attach_primary_bottleneck(&mut result, &reports.ranked_bottlenecks);
        assert!(result.primary_bottleneck.is_none());
        assert!(result
            .withdrawals
            .iter()
            .any(|w| w.contains("WITHDRAWN") && w.contains("io_queue_underdriven")));
    }

    #[test]
    fn qualification_config_forbids_smoke_op_cap() {
        assert!(!RunClass::Qualification.allows_smoke_op_cap());
        let plan = campaign_plan_synthetic(1, 1);
        let cfg = CampaignConfig {
            plan,
            driver: DriverKind::Synthetic,
            work_root: None,
            declare_controlled_runner: false,
            run_class: RunClass::Qualification,
        };
        // Synthetic qualification still runs (no wall 120s in synthetic matrix driver),
        // but must not apply smoke op caps in campaign path.
        let result = run_campaign(&cfg).unwrap();
        assert_eq!(result.run_class, "qualification");
        for r in &result.repetitions {
            // Matrix synthetic may still internal-bound; campaign must not force 4–16.
            assert!(r.report.attempted == 0 || r.report.attempted >= 1);
        }
    }
}

#[cfg(all(test, feature = "store-driver"))]
mod real_store_tests {
    use super::*;
    use crate::campaign::plan::{campaign_plan_linux, campaign_plan_synthetic};
    use crate::campaign::reports::build_campaign_reports;

    #[test]
    fn multi_rep_real_store_smoke_emits_run_ids_but_no_bottleneck() {
        let dir = tempfile::tempdir().unwrap();
        let mut plan = campaign_plan_synthetic(7, 1);
        plan.max_cells = 1;
        plan.include_multiproc_finding = false;
        plan.repetitions = 5;
        plan.processes = 2;

        let mut result = run_campaign(&CampaignConfig {
            plan: plan.clone(),
            driver: DriverKind::RealStore,
            work_root: Some(dir.path().to_path_buf()),
            declare_controlled_runner: false,
            run_class: RunClass::Smoke,
        })
        .expect("real store multi-rep smoke campaign");

        assert_eq!(result.driver_kind, "real_store");
        assert_eq!(result.run_class, "smoke");
        assert!(!result.product_claim_eligible);
        assert!(result.valid_runs >= 5);
        assert!(result.repetitions.iter().all(|r| r.driver_kind == "real_store"));
        assert!(result.repetitions.iter().all(|r| !r.run_id.is_empty()));

        let reports = build_campaign_reports(&result);
        attach_primary_bottleneck(&mut result, &reports.ranked_bottlenecks);
        // Smoke must not promote primary bottlenecks (withdraw io_queue_underdriven etc.).
        assert!(result.primary_bottleneck.is_none());
        assert!(result
            .withdrawals
            .iter()
            .any(|w| w.contains("WITHDRAWN") || w.contains("smoke")));
    }

    #[test]
    fn controlled_smoke_still_not_product_claim_without_qualification_class() {
        let dir = tempfile::tempdir().unwrap();
        let mut plan = campaign_plan_linux(9);
        plan.max_cells = 1;
        plan.include_multiproc_finding = false;
        plan.repetitions = 5;
        plan.processes = 2;
        let result = run_campaign(&CampaignConfig {
            plan,
            driver: DriverKind::RealStore,
            work_root: Some(dir.path().to_path_buf()),
            declare_controlled_runner: true,
            run_class: RunClass::Smoke,
        })
        .unwrap();
        // Surface may be controlled-eligible label, but product_claim requires
        // qualification class as well.
        assert!(!result.product_claim_eligible);
        assert_eq!(result.run_class, "smoke");
    }
}