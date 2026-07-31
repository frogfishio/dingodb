//! Benchmark Disclosure summary for a campaign (no overstated product claims).

use super::reports::CampaignReports;
use super::run::CampaignResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureSummary {
    pub schema: String,
    pub campaign_id: String,
    pub profile: String,
    pub platform: String,
    pub allows_product_baseline: bool,
    pub durability_modes_seen: Vec<String>,
    pub layers_seen: Vec<String>,
    pub valid_runs: usize,
    pub invalid_runs: usize,
    pub repetitions_requested: u32,
    pub processes_requested: u32,
    pub residual_store_driver: bool,
    pub warnings: Vec<String>,
    pub required_fields_checklist: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub field: String,
    pub status: String,
    pub notes: String,
}

pub fn build_disclosure(result: &CampaignResult, reports: &CampaignReports) -> DisclosureSummary {
    let mut durs = std::collections::BTreeSet::new();
    let mut layers = std::collections::BTreeSet::new();
    for r in &result.repetitions {
        durs.insert(r.report.durability.clone());
        layers.insert(r.report.layer.clone());
    }

    let allows = result.plan.platform.allows_product_baseline();
    let mut warnings = reports.notes.clone();
    warnings.push(
        "Real residiuum-store attachment remains residual; matrix driver is harness simulation"
            .into(),
    );
    if !allows {
        warnings.push(
            "Platform is not a controlled product baseline host — do not publish absolute throughput"
                .into(),
        );
    }

    let checklist = vec![
        item("profile", "ok", result.plan.profile.clone()),
        item(
            "platform",
            if allows { "ok" } else { "harness_only" },
            result.plan.platform.as_str(),
        ),
        item(
            "durability_modes",
            "ok",
            durs.iter().cloned().collect::<Vec<_>>().join(","),
        ),
        item(
            "layers",
            "ok",
            layers.iter().cloned().collect::<Vec<_>>().join(","),
        ),
        item(
            "repetitions",
            if result.plan.repetitions >= 5 {
                "ok"
            } else {
                "insufficient"
            },
            result.plan.repetitions.to_string(),
        ),
        item(
            "processes",
            if result.plan.processes >= 2 {
                "ok"
            } else {
                "insufficient"
            },
            result.plan.processes.to_string(),
        ),
        item(
            "correctness_interlock",
            "ok",
            "ack ledger + independent digest on harness cells",
        ),
        item(
            "store_driver",
            "residual",
            "pending real store attachment; proxies only",
        ),
        item(
            "absolute_throughput_claim",
            if allows {
                "controlled_runner_required"
            } else {
                "forbidden_on_synthetic"
            },
            "PQH-9 does not auto-publish MB/s product claims",
        ),
        item(
            "optimization",
            "none",
            "follow-up cards are stubs with run IDs only",
        ),
    ];

    DisclosureSummary {
        schema: "residiuum-performance-disclosure-v1".into(),
        campaign_id: result.plan.campaign_id.clone(),
        profile: result.plan.profile.clone(),
        platform: result.plan.platform.as_str().into(),
        allows_product_baseline: allows,
        durability_modes_seen: durs.into_iter().collect(),
        layers_seen: layers.into_iter().collect(),
        valid_runs: result.valid_runs,
        invalid_runs: result.invalid_runs,
        repetitions_requested: result.plan.repetitions,
        processes_requested: result.plan.processes,
        residual_store_driver: true,
        warnings,
        required_fields_checklist: checklist,
    }
}

fn item(field: &str, status: &str, notes: impl AsRef<str>) -> ChecklistItem {
    ChecklistItem {
        field: field.into(),
        status: status.into(),
        notes: notes.as_ref().into(),
    }
}

pub fn render_disclosure_markdown(
    disclosure: &DisclosureSummary,
    reports: &CampaignReports,
) -> String {
    let mut md = String::new();
    md.push_str("# Benchmark Disclosure summary (PQH-9)\n\n");
    md.push_str(&format!("Campaign: `{}`\n\n", disclosure.campaign_id));
    md.push_str(&format!("Profile: `{}`\n\n", disclosure.profile));
    md.push_str(&format!(
        "Platform: `{}` (product baseline allowed: {})\n\n",
        disclosure.platform, disclosure.allows_product_baseline
    ));
    md.push_str("## Checklist\n\n");
    md.push_str("| Field | Status | Notes |\n|---|---|---|\n");
    for c in &disclosure.required_fields_checklist {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            c.field, c.status, c.notes
        ));
    }
    md.push_str("\n## Runs\n\n");
    md.push_str(&format!(
        "- valid: {}\n- invalid: {}\n- repetitions requested: {}\n- processes requested: {}\n\n",
        disclosure.valid_runs,
        disclosure.invalid_runs,
        disclosure.repetitions_requested,
        disclosure.processes_requested
    ));
    md.push_str("## Multiproc finding (4 KiB / 8 KiB)\n\n");
    md.push_str(&format!("{}\n\n", reports.multiproc_finding.statement));
    md.push_str("## Ranked bottlenecks\n\n");
    if reports.ranked_bottlenecks.is_empty() {
        md.push_str("(none isolated — see mixed_or_unknown / residual store driver)\n\n");
    } else {
        for b in &reports.ranked_bottlenecks {
            md.push_str(&format!(
                "{}. **{}** ({}) runs={}\n",
                b.rank,
                b.verdict,
                b.confidence,
                b.run_ids.len()
            ));
            if !b.falsification_ids.is_empty() {
                md.push_str(&format!(
                    "   - falsify: {}\n",
                    b.falsification_ids.join(", ")
                ));
            }
        }
        md.push('\n');
    }
    md.push_str("## Follow-up optimization cards (stubs only)\n\n");
    if reports.follow_up_cards.is_empty() {
        md.push_str("None.\n\n");
    } else {
        for c in &reports.follow_up_cards {
            md.push_str(&format!(
                "- `{}` {} — parameter `{}` — runs: {}\n",
                c.card_id,
                c.title,
                c.suggested_parameter,
                c.related_run_ids.join(", ")
            ));
        }
        md.push('\n');
    }
    md.push_str("## Warnings\n\n");
    for w in &disclosure.warnings {
        md.push_str(&format!("- {w}\n"));
    }
    md.push('\n');
    md.push_str(
        "This disclosure does **not** authorize product marketing numbers without a controlled-runner accept of PQH-9.\n",
    );
    md
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::plan::campaign_plan_synthetic;
    use crate::campaign::reports::build_campaign_reports;
    use crate::campaign::run::{run_campaign, CampaignConfig};

    #[test]
    fn disclosure_forbids_synthetic_product_claim() {
        let plan = campaign_plan_synthetic(3, 2);
        let result = run_campaign(&CampaignConfig { plan }).unwrap();
        let reports = build_campaign_reports(&result);
        let d = build_disclosure(&result, &reports);
        assert!(!d.allows_product_baseline);
        assert!(d.residual_store_driver);
        let md = render_disclosure_markdown(&d, &reports);
        assert!(md.contains("Benchmark Disclosure"));
        assert!(md.contains("does **not** authorize") || md.contains("does not authorize") || md.contains("**not**"));
    }
}