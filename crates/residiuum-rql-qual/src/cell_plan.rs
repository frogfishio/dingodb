//! Mandatory measured cell plans + concurrency matrix (programme §7.2).
//!
//! Q4.2: full plan registry (dataset + RQL intent + lifecycle + concurrency).
//! Execution against product engines remains Q4.3 / residual.

use crate::cells::{MandatoryCell, CONCURRENCY_LEVELS, OVERSUBSCRIBED_SLOT};
use crate::dataset::{
    CardinalityClass, DatasetSpec, DistributionKind, DocShape, MemoryRatio, PayloadClass,
    SelectivityClass,
};
use crate::lifecycle::{ColdMethod, LifecycleSpec};
use crate::metrics::LifecycleClass;
use serde::{Deserialize, Serialize};

/// Workload mix for mixed R/W cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadWriteMix {
    /// 90% read / 10% write.
    R90W10,
    /// 70% read / 30% write.
    R70W30,
}

impl ReadWriteMix {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::R90W10 => "rw_90_10",
            Self::R70W30 => "rw_70_30",
        }
    }

    pub fn read_pct(self) -> u8 {
        match self {
            Self::R90W10 => 90,
            Self::R70W30 => 70,
        }
    }
}

/// Index setup declaration for a cell (logical; adapters apply).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexPlan {
    pub name: String,
    pub fields: Vec<String>,
    pub required: bool,
}

/// Enrich cardinality expectation for §7.2 cell 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrichExpect {
    Optional,
    ExactlyOne,
    Many,
}

impl EnrichExpect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Optional => "optional",
            Self::ExactlyOne => "exactly_one",
            Self::Many => "many",
        }
    }
}

/// One runnable measured-cell plan (not yet a competitive result).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredCellPlan {
    pub cell: MandatoryCell,
    pub plan_id: String,
    pub dataset: DatasetSpec,
    pub lifecycle: LifecycleSpec,
    pub concurrency: u32,
    /// Host-declared oversubscribed slot (0 = not oversubscribed).
    pub oversubscribed: bool,
    /// Residiuum Application Core RQL source (intention; product path).
    pub rql_source: String,
    pub indexes: Vec<IndexPlan>,
    pub order_sensitive: bool,
    pub page_size: Option<u32>,
    /// When true, logical/product must multipage (first + deep) not first-only.
    pub require_deep_cursor: bool,
    pub rw_mix: Option<ReadWriteMix>,
    pub enrich_expect: Option<EnrichExpect>,
    pub server_lane_ineligible: bool,
    pub notes: String,
}

impl MeasuredCellPlan {
    /// Build the smoke-scale default plan for one mandatory cell.
    pub fn smoke_for(cell: MandatoryCell, seed: u64) -> Self {
        let mut dataset = DatasetSpec::smoke_default(seed.wrapping_add(cell.programme_index() as u64));
        let (rql, indexes, order_sensitive, page_size, rw_mix, shape, sel, card, notes) =
            cell_defaults(cell, &mut dataset);
        dataset.shape = shape;
        dataset.selectivity = sel;
        dataset.cardinality = card;

        let require_deep_cursor = matches!(cell, MandatoryCell::FirstAndDeepCursor);
        let enrich_expect = match cell {
            MandatoryCell::EnrichCardinalities => Some(EnrichExpect::Optional),
            _ => None,
        };
        Self {
            cell,
            plan_id: format!("{}_smoke_c1", cell.id()),
            dataset,
            lifecycle: LifecycleSpec::for_class(LifecycleClass::WarmSteady),
            concurrency: 1,
            oversubscribed: false,
            rql_source: rql,
            indexes,
            order_sensitive,
            page_size,
            require_deep_cursor,
            rw_mix,
            enrich_expect,
            server_lane_ineligible: cell.server_lane_ineligible_by_default(),
            notes,
        }
    }

    /// Expand concurrency matrix for a base plan (1,2,4,8 + optional oversub).
    pub fn with_concurrency(mut self, c: u32, oversubscribed: bool) -> Self {
        self.concurrency = c;
        self.oversubscribed = oversubscribed;
        self.plan_id = if oversubscribed {
            format!("{}_c{c}_oversub", self.cell.id())
        } else {
            format!("{}_c{c}", self.cell.id())
        };
        self
    }

    pub fn with_lifecycle(mut self, life: LifecycleSpec) -> Self {
        self.lifecycle = life;
        self.plan_id = format!("{}_{}", self.plan_id, self.lifecycle.class.as_str());
        self
    }
}

fn cell_defaults(
    cell: MandatoryCell,
    dataset: &mut DatasetSpec,
) -> (
    String,
    Vec<IndexPlan>,
    bool,
    Option<u32>,
    Option<ReadWriteMix>,
    DocShape,
    SelectivityClass,
    CardinalityClass,
    String,
) {
    match cell {
        MandatoryCell::KeyGet => (
            r#"from docs where _key = "d-00000000""#.into(),
            vec![],
            false,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::Point,
            CardinalityClass::High,
            "Key get by immutable _key.".into(),
        ),
        MandatoryCell::IndexedEqMultiSelectivity => {
            dataset.selectivity = SelectivityClass::S10;
            (
                r#"from docs where sel_bucket = "HIT""#.into(),
                vec![IndexPlan {
                    name: "by_sel_bucket".into(),
                    fields: vec!["sel_bucket".into()],
                    required: true,
                }],
                false,
                None,
                None,
                DocShape::Flat,
                SelectivityClass::S10,
                CardinalityClass::Medium,
                "Indexed equality; vary selectivity axis across matrix instances.".into(),
            )
        }
        MandatoryCell::RangeAndCompound => (
            r#"from docs where amount >= 100 and amount < 500 and region = "r0""#.into(),
            vec![IndexPlan {
                name: "by_region_amount".into(),
                fields: vec!["region".into(), "amount".into()],
                required: false,
            }],
            false,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::S10,
            CardinalityClass::Medium,
            "Range + compound equality/range.".into(),
        ),
        MandatoryCell::NestedAndArrayPreds => (
            r#"from docs where present(nested.l1.l2.l3.flag) or contains(tags, "t0-0")"#.into(),
            vec![],
            false,
            None,
            None,
            DocShape::ArrayHeavy,
            SelectivityClass::Broad,
            CardinalityClass::Medium,
            "Nested path and/or array contains (shape may be DeeplyNested or ArrayHeavy)."
                .into(),
        ),
        MandatoryCell::CoveredNonCoveredProject => (
            r#"from docs where status = "st-0000" project status, region"#.into(),
            vec![IndexPlan {
                name: "by_status".into(),
                fields: vec!["status".into()],
                required: false,
            }],
            false,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::S10,
            CardinalityClass::Low,
            "Projection covered vs non-covered (adapter records plan).".into(),
        ),
        MandatoryCell::DeterministicTopK => (
            r#"from docs order by score desc, _key asc limit 10"#.into(),
            vec![],
            true,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::Broad,
            CardinalityClass::High,
            "Deterministic top-k with key tie-break.".into(),
        ),
        MandatoryCell::FirstAndDeepCursor => (
            "from docs order by _key asc".into(),
            vec![],
            true,
            Some(8),
            None,
            DocShape::Flat,
            SelectivityClass::Broad,
            CardinalityClass::High,
            "First + deep multipage continuation (page_size=8); full concat = unpaged."
                .into(),
        ),
        MandatoryCell::EnrichCardinalities => (
            // Full enrich — lane S ineligible until wire (Q0.A4).
            "from docs enrich customer using customers matching customer_id = id expect optional"
                .into(),
            vec![],
            false,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::Broad,
            CardinalityClass::Medium,
            "Enrich optional default; variants exactly_one/many via enrich_variants()."
                .into(),
        ),
        MandatoryCell::GroupLowHighCard => {
            dataset.cardinality = CardinalityClass::Low;
            (
                "from docs group by status count".into(),
                vec![],
                false,
                None,
                None,
                DocShape::Flat,
                SelectivityClass::Broad,
                CardinalityClass::Low,
                "Grouping; flip cardinality axis Low/High in matrix.".into(),
            )
        }
        MandatoryCell::AggCountSumMinMaxAvg => (
            "from docs group by region count sum(amount) min(amount) max(amount) avg(amount)"
                .into(),
            vec![],
            false,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::Broad,
            CardinalityClass::Medium,
            "Aggregates count/sum/min/max/avg (logical always; product Core may refuse avg)."
                .into(),
        ),
        MandatoryCell::ConditionalComputed => (
            // Intention: high_band = amount when amount>=100 else null (logical).
            r#"from docs project amount, region, high_band"#.into(),
            vec![],
            false,
            None,
            None,
            DocShape::Flat,
            SelectivityClass::Broad,
            CardinalityClass::Medium,
            "Conditional high_band: amount if amount>=100 else null (logical computed).".into(),
        ),
        MandatoryCell::MixedReadWrite => (
            r#"from docs where sel_bucket = "HIT""#.into(),
            vec![],
            false,
            None,
            Some(ReadWriteMix::R90W10),
            DocShape::Flat,
            SelectivityClass::S10,
            CardinalityClass::Medium,
            "Mixed R/W 90/10 (and 70/30 via rw_mix_variants); logical performs writes."
                .into(),
        ),
    }
}

/// Smoke portfolio: one plan per mandatory cell @ concurrency 1, warm steady.
pub fn smoke_portfolio(seed: u64) -> Vec<MeasuredCellPlan> {
    MandatoryCell::ALL
        .iter()
        .map(|c| MeasuredCellPlan::smoke_for(*c, seed))
        .collect()
}

/// §7.2 expanded smoke: base 12 + enrich cardinality variants + R/W 70/30
/// + concurrency matrix on key-get (executed by runner).
pub fn section_7_2_expanded_portfolio(seed: u64) -> Vec<MeasuredCellPlan> {
    let mut out = smoke_portfolio(seed);
    out.extend(enrich_variants(seed));
    out.extend(rw_mix_variants(seed));
    let key = MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, seed);
    out.extend(concurrency_matrix(&key, 2));
    out
}

/// Enrich optional / exactly_one / many plan variants.
pub fn enrich_variants(seed: u64) -> Vec<MeasuredCellPlan> {
    [
        EnrichExpect::Optional,
        EnrichExpect::ExactlyOne,
        EnrichExpect::Many,
    ]
    .into_iter()
    .map(|ex| {
        let mut p = MeasuredCellPlan::smoke_for(MandatoryCell::EnrichCardinalities, seed);
        p.enrich_expect = Some(ex);
        p.rql_source = format!(
            "from docs enrich customer using customers matching customer_id = id expect {}",
            ex.as_str()
        );
        p.plan_id = format!("{}_{}", MandatoryCell::EnrichCardinalities.id(), ex.as_str());
        p.notes = format!("Enrich expect={}", ex.as_str());
        p
    })
    .collect()
}

/// Mixed R/W 90/10 and 70/30 variants.
pub fn rw_mix_variants(seed: u64) -> Vec<MeasuredCellPlan> {
    [ReadWriteMix::R90W10, ReadWriteMix::R70W30]
        .into_iter()
        .map(|mix| {
            let mut p = MeasuredCellPlan::smoke_for(MandatoryCell::MixedReadWrite, seed);
            p.rw_mix = Some(mix);
            p.plan_id = format!("{}_{}", MandatoryCell::MixedReadWrite.id(), mix.as_str());
            p.notes = format!("Mixed R/W {}", mix.as_str());
            p
        })
        .collect()
}

/// Concurrency matrix for one cell (levels 1,2,4,8 + one oversubscribed placeholder).
pub fn concurrency_matrix(base: &MeasuredCellPlan, oversub_factor: u32) -> Vec<MeasuredCellPlan> {
    let mut out = Vec::new();
    for &c in CONCURRENCY_LEVELS {
        out.push(base.clone().with_concurrency(c, false));
    }
    let over = CONCURRENCY_LEVELS.last().copied().unwrap_or(8).saturating_mul(oversub_factor.max(2));
    let mut p = base.clone().with_concurrency(over, true);
    p.notes = format!(
        "{}; oversubscribed slot ({OVERSUBSCRIBED_SLOT}) concurrency={over}",
        p.notes
    );
    out.push(p);
    out
}

/// Selectivity matrix for indexed-eq cell (point / 0.01% / 1% / 10% / broad).
pub fn selectivity_matrix(seed: u64) -> Vec<MeasuredCellPlan> {
    SelectivityClass::ALL
        .iter()
        .enumerate()
        .map(|(i, sel)| {
            let mut p = MeasuredCellPlan::smoke_for(MandatoryCell::IndexedEqMultiSelectivity, seed);
            p.dataset.selectivity = *sel;
            p.dataset.seed = seed.wrapping_add(100 + i as u64);
            p.plan_id = format!("{}_{}", MandatoryCell::IndexedEqMultiSelectivity.id(), sel.as_str());
            p.notes = format!("Indexed eq selectivity={}", sel.as_str());
            p
        })
        .collect()
}

/// Lifecycle matrix for key-get smoke (all §7.3 classes).
pub fn lifecycle_matrix(seed: u64) -> Vec<MeasuredCellPlan> {
    LifecycleSpec::all_programme_classes()
        .into_iter()
        .map(|life| {
            let mut p = MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, seed);
            // Larger-than-memory uses R400 scale identity.
            if life.class == LifecycleClass::LargerThanMemory {
                p.dataset.memory_ratio = MemoryRatio::R400;
                p.dataset = p.dataset.clone().with_scaled_docs(64);
            }
            p.with_lifecycle(life)
        })
        .collect()
}

/// Machine report for Q4.2 labor evidence.
pub fn q4_2_report_json() -> serde_json::Value {
    let smoke = smoke_portfolio(0x04_42);
    let expanded = section_7_2_expanded_portfolio(0x04_42);
    let conc = concurrency_matrix(&smoke[0], 2);
    let sel = selectivity_matrix(0x04_42);
    let life = lifecycle_matrix(0x04_42);
    serde_json::json!({
        "format": "residiuum-rql-q4-2-dataset-cells-report-v1",
        "harness_profile": crate::HARNESS_PROFILE,
        "summary": {
            "mandatory_cells": MandatoryCell::ALL.len(),
            "smoke_plans": smoke.len(),
            "section_7_2_expanded_plans": expanded.len(),
            "concurrency_matrix_len": conc.len(),
            "selectivity_matrix_len": sel.len(),
            "lifecycle_matrix_len": life.len(),
            "concurrency_levels": CONCURRENCY_LEVELS,
            "oversubscribed_slot": OVERSUBSCRIBED_SLOT,
            "cold_reopen_claims_device_cold": false,
            "default_cold_method_reopen": ColdMethod::StoreReopen.as_str(),
            "f2_real_cells": true,
        },
        "payload_classes": PayloadClass::PRIMARY.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
        "shapes": DocShape::ALL.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        "distributions": DistributionKind::ALL.iter().map(|d| d.as_str()).collect::<Vec<_>>(),
        "smoke_plan_ids": smoke.iter().map(|p| p.plan_id.clone()).collect::<Vec<_>>(),
        "non_claims": [
            "not_gate1",
            "not_competitive",
            "not_q4_package_accept",
            "execute_residual_q4_3"
        ],
        "authority": "doc/todo/rql/RQL_Q4_2_DATASET_CELLS.md",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::generate_dataset;
    use crate::lifecycle::validate_cold_claim;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn smoke_portfolio_twelve_plans() {
        let plans = smoke_portfolio(1);
        assert_eq!(plans.len(), 12);
        for p in &plans {
            assert!(!p.rql_source.is_empty());
            assert_eq!(p.concurrency, 1);
            validate_cold_claim(&p.lifecycle).unwrap();
        }
        let enrich = plans
            .iter()
            .find(|p| p.cell == MandatoryCell::EnrichCardinalities)
            .unwrap();
        assert!(enrich.server_lane_ineligible);
        let cursor = plans
            .iter()
            .find(|p| p.cell == MandatoryCell::FirstAndDeepCursor)
            .unwrap();
        assert!(cursor.require_deep_cursor);
        let agg = plans
            .iter()
            .find(|p| p.cell == MandatoryCell::AggCountSumMinMaxAvg)
            .unwrap();
        assert!(agg.rql_source.contains("avg"));
        let cond = plans
            .iter()
            .find(|p| p.cell == MandatoryCell::ConditionalComputed)
            .unwrap();
        assert!(cond.rql_source.contains("high_band"));
    }

    #[test]
    fn section_7_2_expanded_includes_variants() {
        let plans = section_7_2_expanded_portfolio(1);
        assert!(plans.len() >= 22);
        assert!(plans.iter().any(|p| p.plan_id.contains("exactly_one")));
        assert!(plans.iter().any(|p| p.plan_id.contains("many")));
        assert!(plans.iter().any(|p| p.plan_id.contains("rw_70_30")));
        assert!(plans.iter().any(|p| p.concurrency == 8));
    }

    #[test]
    fn concurrency_matrix_five_slots() {
        let base = MeasuredCellPlan::smoke_for(MandatoryCell::KeyGet, 9);
        let m = concurrency_matrix(&base, 2);
        assert_eq!(m.len(), 5); // 1,2,4,8 + oversub
        assert!(m.iter().any(|p| p.oversubscribed));
        assert_eq!(m[0].concurrency, 1);
        assert_eq!(m[3].concurrency, 8);
    }

    #[test]
    fn generator_matches_indexed_eq_plan() {
        let p = MeasuredCellPlan::smoke_for(MandatoryCell::IndexedEqMultiSelectivity, 3);
        let ds = generate_dataset(&p.dataset);
        assert_eq!(ds.collections["docs"].len() as u64, p.dataset.doc_count);
        assert!(!ds.content_hash.is_empty());
    }

    #[test]
    fn write_q4_2_report() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root.join("spec/rql/qualification/harness-v1/q4_2_dataset_cells_report.json");
        let report = q4_2_report_json();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        assert!(path.is_file());
        assert_eq!(report["summary"]["mandatory_cells"], 12);
        assert_eq!(report["summary"]["smoke_plans"], 12);
    }

    #[test]
    fn selectivity_matrix_five() {
        assert_eq!(selectivity_matrix(0).len(), 5);
    }

    #[test]
    fn lifecycle_matrix_seven() {
        assert_eq!(lifecycle_matrix(0).len(), 7);
    }
}