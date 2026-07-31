//! Multi-process multi-repetition campaign execution over matrix cells.

use super::plan::CampaignPlan;
use super::CampaignError;
use crate::matrix::{
    build_matrix_cells, run_cell, CellRunReport, MatrixManifest, RunCellConfig, ScheduleSeed,
};
use serde::{Deserialize, Serialize};

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
    pub report: CellRunReport,
}

#[derive(Debug, Clone)]
pub struct CampaignConfig {
    pub plan: CampaignPlan,
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
}

/// Execute campaign: for each selected cell, run `processes × reps_per_process`
/// so total reps ≥ plan.repetitions across ≥ plan.processes process slots.
pub fn run_campaign(cfg: &CampaignConfig) -> Result<CampaignResult, CampaignError> {
    cfg.plan
        .validate()
        .map_err(|e| CampaignError::Msg(e))?;

    let manifest: MatrixManifest = build_matrix_cells(ScheduleSeed {
        seed: cfg.plan.seed,
    });

    let cells: Vec<_> = manifest
        .cells
        .iter()
        .take(cfg.plan.max_cells)
        .cloned()
        .collect();

    // Distribute repetitions across processes (ceil division).
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

    for cell in &cells {
        for slot in &process_slots {
            for rep in 0..reps_per_proc {
                let seed = slot
                    .process_seed
                    .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    .wrapping_add(rep as u64)
                    .wrapping_add(cell.order_rank as u64);
                let report = run_cell(&RunCellConfig {
                    cell: cell.clone(),
                    seed,
                    durability_mutant: false,
                    digest_mutant: false,
                })?;
                let run_id = format!(
                    "{}-p{}-r{}-{}",
                    cfg.plan.campaign_id, slot.process_id, rep, cell.cell_id
                );
                if report.validity == "valid" {
                    valid_runs += 1;
                } else {
                    invalid_runs += 1;
                }
                repetitions.push(CellRepetition {
                    run_id,
                    cell_id: cell.cell_id.clone(),
                    process_id: slot.process_id,
                    rep,
                    report,
                });
            }
        }
    }

    // Optional multiproc 4K/8K finding cells (concurrency>1 at fixed sizes).
    if cfg.plan.include_multiproc_finding {
        let finding_cells: Vec<_> = manifest
            .cells
            .iter()
            .filter(|c| {
                (c.payload_size == 4096 || c.payload_size == 8192)
                    && c.concurrency >= 2
            })
            .take(4)
            .cloned()
            .collect();
        for cell in finding_cells {
            // Avoid double-counting if already in the first max_cells slice.
            if cells.iter().any(|c| c.cell_id == cell.cell_id) {
                continue;
            }
            for slot in &process_slots {
                for rep in 0..reps_per_proc {
                    let seed = slot
                        .process_seed
                        .wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
                        .wrapping_add(rep as u64);
                    let report = run_cell(&RunCellConfig {
                        cell: cell.clone(),
                        seed,
                        durability_mutant: false,
                        digest_mutant: false,
                    })?;
                    let run_id = format!(
                        "{}-mp-p{}-r{}-{}",
                        cfg.plan.campaign_id, slot.process_id, rep, cell.cell_id
                    );
                    if report.validity == "valid" {
                        valid_runs += 1;
                    } else {
                        invalid_runs += 1;
                    }
                    repetitions.push(CellRepetition {
                        run_id,
                        cell_id: cell.cell_id.clone(),
                        process_id: slot.process_id,
                        rep,
                        report,
                    });
                }
            }
        }
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
    })
}

/// Process-slot seed discriminator (ASCII-ish "P9PR").
const PROCESS_SEED_TAG: u64 = 0x5039_5052;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::plan::campaign_plan_synthetic;

    #[test]
    fn campaign_meets_rep_and_process_mins() {
        let plan = campaign_plan_synthetic(42, 3);
        let result = run_campaign(&CampaignConfig { plan: plan.clone() }).unwrap();
        assert_eq!(result.process_slots.len() as u32, plan.processes);
        assert!(result.valid_runs > 0);
        assert_eq!(result.invalid_runs, 0);

        // Each of first cells should have process_id covering 0..processes-1
        let mut by_cell: std::collections::HashMap<String, std::collections::HashSet<u32>> =
            std::collections::HashMap::new();
        for r in &result.repetitions {
            by_cell
                .entry(r.cell_id.clone())
                .or_default()
                .insert(r.process_id);
        }
        for (_cell, procs) in by_cell {
            assert!(procs.len() as u32 >= plan.processes.min(2));
        }

        // Per cell, total reps across processes >= MIN when only first cells
        let first_cell = &result.repetitions[0].cell_id;
        let n = result
            .repetitions
            .iter()
            .filter(|r| r.cell_id == *first_cell)
            .count();
        assert!(n as u32 >= plan.repetitions);
    }
}