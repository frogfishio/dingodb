//! Runnable PQH campaign CLI (plan §14 surface subset).
//!
//! ```text
//! residiuum-perf preflight --work <dir>
//! residiuum-perf run --work <dir> [--driver synthetic|real_store] [--max-cells N]
//! residiuum-perf analyze --campaign <dir>
//! residiuum-perf verify --campaign <dir>
//! residiuum-perf driver-smoke --work <dir> [--driver synthetic|real_store]
//! ```
//!
//! Synthetic/proxy results are always labelled non-product. Real-store driver
//! requires `--features store-driver` at build time. No optimisations applied.

use residiuum_perf::campaign::{
    build_campaign_reports, build_disclosure, campaign_plan_linux, campaign_plan_macos_apple_silicon,
    campaign_plan_synthetic, run_campaign, verify_bundle_hashes, write_evidence_bundle,
    CampaignConfig, PlatformClass,
};
use residiuum_perf::matrix::{build_matrix_cells, ScheduleSeed};
use residiuum_perf::runner::{
    environment_fingerprint, preflight_work_root, write_run_artifacts, BuildMode, PreflightConfig,
    RunBudgets,
};
use residiuum_perf::store_driver::{
    run_driver_cell, store_driver_compiled, DriverKind, DriverRunConfig,
};
use residiuum_perf::PROFILE_ID;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }
    let cmd = args.remove(0);
    let result = match cmd.as_str() {
        "preflight" => cmd_preflight(&args),
        "run" => cmd_run(&args),
        "analyze" => cmd_analyze(&args),
        "verify" => cmd_verify(&args),
        "driver-smoke" => cmd_driver_smoke(&args),
        other => Err(format!("unknown command {other}; see --help")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "residiuum-perf — Performance Qualification Harness CLI
profile: {PROFILE_ID}
store-driver compiled: {}

Commands:
  preflight --work <dir> [--qualification]
  run --work <dir> [--driver synthetic|real_store] [--max-cells N] [--seed N]
      [--platform synthetic|macos_as|linux]
  analyze --campaign <dir>
  verify --campaign <dir>
  driver-smoke --work <dir> [--driver synthetic|real_store]

Honesty:
  synthetic/proxy results are NON-PRODUCT.
  real_store requires build with --features store-driver.
  No optimisations are applied by this CLI.
",
        store_driver_compiled()
    );
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            return args.get(i + 1).cloned();
        }
        if let Some(rest) = args[i].strip_prefix(&format!("{name}=")) {
            return Some(rest.to_string());
        }
        i += 1;
    }
    None
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn cmd_preflight(args: &[String]) -> Result<(), String> {
    let work = PathBuf::from(flag_value(args, "--work").ok_or("--work required")?);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let qual = has_flag(args, "--qualification");
    let work_root_id = format!("pqh-work-{:x}", std::process::id());
    let cfg = PreflightConfig {
        work_root: work.clone(),
        work_root_id: work_root_id.clone(),
        run_id: "preflight".into(),
        repository_root: None,
        budgets: RunBudgets::default(),
        build_mode: if qual {
            BuildMode::Qualification
        } else {
            BuildMode::Diagnostic
        },
        enforce_release_for_qualification: qual,
    };
    let report = preflight_work_root(&cfg).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    );
    if report.outcome.is_ready() {
        let env = environment_fingerprint(Some(&work), cfg.build_mode).ok();
        let _ = write_run_artifacts(
            &work,
            &work_root_id,
            "preflight-scaffold",
            &report,
            env.as_ref(),
        );
    }
    Ok(())
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    let work = PathBuf::from(flag_value(args, "--work").ok_or("--work required")?);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let seed: u64 = flag_value(args, "--seed")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let max_cells: usize = flag_value(args, "--max-cells")
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    let driver = flag_value(args, "--driver").unwrap_or_else(|| "synthetic".into());
    let kind = DriverKind::parse(&driver).ok_or_else(|| format!("bad --driver {driver}"))?;
    if kind == DriverKind::RealStore && !store_driver_compiled() {
        return Err(
            "real_store driver not compiled; rebuild with --features store-driver".into(),
        );
    }

    let platform = flag_value(args, "--platform").unwrap_or_else(|| "synthetic".into());
    let mut plan = match platform.as_str() {
        "macos_as" | "macos" => campaign_plan_macos_apple_silicon(seed),
        "linux" => campaign_plan_linux(seed),
        _ => campaign_plan_synthetic(seed, max_cells),
    };
    plan.max_cells = max_cells;
    if kind == DriverKind::Synthetic {
        plan.platform = PlatformClass::SyntheticHarness;
        plan.notes
            .push("CLI run with synthetic driver — NON-PRODUCT".into());
    }

    let result = run_campaign(&CampaignConfig { plan: plan.clone() }).map_err(|e| e.to_string())?;
    let mut reports = build_campaign_reports(&result);
    if kind == DriverKind::Synthetic || matches!(plan.platform, PlatformClass::SyntheticHarness) {
        reports
            .notes
            .push("NON-PRODUCT: synthetic/proxy campaign; do not publish absolute MB/s".into());
        reports.multiproc_finding.overstates_product = false;
    }

    if kind == DriverKind::RealStore {
        let manifest = build_matrix_cells(ScheduleSeed { seed });
        if let Some(cell) = manifest.cells.first() {
            let d = run_driver_cell(&DriverRunConfig {
                cell: cell.clone(),
                seed,
                kind: DriverKind::RealStore,
                work_root: Some(work.clone()),
                durability_mutant: false,
                digest_mutant: false,
            })
            .map_err(|e| e.to_string())?;
            reports.notes.push(format!(
                "real_store smoke cell={} acked={} product_claim_eligible={}",
                d.cell.cell_id, d.cell.acknowledged, d.product_claim_eligible
            ));
            if let Some(p) = &d.plan {
                let plan_path = work.join("real_store_plan.json");
                std::fs::write(
                    &plan_path,
                    serde_json::to_string_pretty(p).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                reports
                    .notes
                    .push(format!("wrote store-native plan {}", plan_path.display()));
            }
        }
    }

    let disclosure = build_disclosure(&result, &reports);
    let campaign_dir = work.join("campaign").join(&plan.campaign_id);
    let bundle = write_evidence_bundle(&campaign_dir, &result, &reports, &disclosure)
        .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "campaign_dir": campaign_dir,
            "campaign_id": plan.campaign_id,
            "driver": kind.as_str(),
            "platform": plan.platform.as_str(),
            "allows_product_baseline": plan.platform.allows_product_baseline()
                && kind != DriverKind::Synthetic,
            "valid_runs": result.valid_runs,
            "content_hash": bundle.content_hash,
            "non_product": kind == DriverKind::Synthetic,
            "notes": reports.notes,
        })
    );
    Ok(())
}

fn cmd_analyze(args: &[String]) -> Result<(), String> {
    let campaign = PathBuf::from(flag_value(args, "--campaign").ok_or("--campaign required")?);
    let disclosure_md = campaign.join("DISCLOSURE.md");
    if disclosure_md.exists() {
        let s = std::fs::read_to_string(&disclosure_md).map_err(|e| e.to_string())?;
        println!("{s}");
    }
    let reports_path = campaign.join("reports.json");
    if reports_path.exists() {
        let raw = std::fs::read_to_string(&reports_path).map_err(|e| e.to_string())?;
        let reports: residiuum_perf::campaign::CampaignReports =
            serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        println!("\n## Ranked bottlenecks (from reports.json)\n");
        for b in &reports.ranked_bottlenecks {
            println!(
                "{}. {} ({}) runs={}",
                b.rank,
                b.verdict,
                b.confidence,
                b.run_ids.len()
            );
        }
        if reports.notes.iter().any(|n| n.contains("NON-PRODUCT")) {
            println!("\nNOTE: this campaign is labelled NON-PRODUCT.\n");
        }
        println!("No optimisations applied (PQH policy).\n");
    } else if !disclosure_md.exists() {
        return Err("campaign dir missing DISCLOSURE.md and reports.json".into());
    }
    Ok(())
}

fn cmd_verify(args: &[String]) -> Result<(), String> {
    let campaign = PathBuf::from(flag_value(args, "--campaign").ok_or("--campaign required")?);
    verify_bundle_hashes(&campaign).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({"ok": true, "campaign": campaign, "hashes": "verified"})
    );
    Ok(())
}

fn cmd_driver_smoke(args: &[String]) -> Result<(), String> {
    let work = PathBuf::from(flag_value(args, "--work").ok_or("--work required")?);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let driver = flag_value(args, "--driver").unwrap_or_else(|| "synthetic".into());
    let kind = DriverKind::parse(&driver).ok_or_else(|| format!("bad --driver {driver}"))?;
    if kind == DriverKind::RealStore && !store_driver_compiled() {
        return Err("rebuild with --features store-driver".into());
    }
    let manifest = build_matrix_cells(ScheduleSeed { seed: 1 });
    let cell = manifest.cells.first().cloned().ok_or("empty matrix")?;
    let report = run_driver_cell(&DriverRunConfig {
        cell,
        seed: 1,
        kind,
        work_root: Some(work),
        durability_mutant: false,
        digest_mutant: false,
    })
    .map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({
            "driver": report.driver_kind.as_str(),
            "surface": report.measurement_surface.as_str(),
            "product_claim_eligible": report.product_claim_eligible,
            "validity": report.cell.validity,
            "acknowledged": report.cell.acknowledged,
            "plan_source": report.plan_source,
            "planned_bytes": report.plan.as_ref().map(|p| p.planned_bytes),
            "notes": report.notes,
        })
    );
    Ok(())
}
