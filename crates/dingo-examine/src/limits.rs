//! Host resource limits for examination streams (SDA_PROFILE §11 / OVERVIEW §11.4).

/// Bounds applied while projecting and paging examination units.
///
/// Exceeding a limit yields an **incomplete** page with uncertainty
/// `resource-limited`. It MUST NOT be reported as a complete empty success.
#[derive(Debug, Clone)]
pub struct ExamineLimits {
    /// Maximum number of units to return in one page. `None` = unbounded.
    pub max_units: Option<usize>,
    /// Maximum total source bytes to read across examination sources.
    pub max_bytes_read: Option<u64>,
    /// When false, verified payload bodies are not copied into units.
    pub materialize_payloads: bool,
}

impl Default for ExamineLimits {
    fn default() -> Self {
        Self {
            max_units: None,
            max_bytes_read: None,
            materialize_payloads: true,
        }
    }
}

impl ExamineLimits {
    /// Unbounded materializing examination.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cap units returned.
    pub fn max_units(mut self, n: usize) -> Self {
        self.max_units = Some(n);
        self
    }

    /// Cap bytes read from segment sources.
    pub fn max_bytes_read(mut self, n: u64) -> Self {
        self.max_bytes_read = Some(n);
        self
    }

    /// Do not copy payload bodies into units.
    pub fn without_payloads(mut self) -> Self {
        self.materialize_payloads = false;
        self
    }
}

/// One bounded page of examination units (SDA_PROFILE §11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExaminePage {
    /// Deterministic query / stream identifier (hex of content hash prefix).
    pub query_id: String,
    /// Zero-based page number within this examination stream.
    pub page_number: u64,
    /// Whether this page finishes the declared scope (no host truncation).
    pub complete: bool,
    /// Ordered examination units.
    pub units: Vec<crate::unit::ExaminationUnit>,
    /// Coverage summary for this page.
    pub coverage: PageCoverage,
    /// Opaque continuation when `complete` is false (Stage 5: simple index).
    pub continuation: Option<Vec<u8>>,
    /// Page-level uncertainty tags.
    pub uncertainty: Vec<String>,
}

/// Coverage product fields (single-node Stage 5 subset).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageCoverage {
    /// Catalog completeness (`complete`, `partial`, `absent`).
    pub catalogs: String,
    /// Index completeness.
    pub indexes: String,
    /// Tier names included.
    pub tiers: Vec<String>,
    /// Tier names excluded.
    pub excluded_tiers: Vec<String>,
}

impl PageCoverage {
    /// Catalog-free salvage: catalogs/indexes absent, hot tier only.
    pub fn salvage_default() -> Self {
        Self {
            catalogs: "absent".into(),
            indexes: "absent".into(),
            tiers: vec!["hot".into()],
            excluded_tiers: vec![],
        }
    }
}

impl ExaminePage {
    /// Convert to SDA `Prod` page shape (SDA_PROFILE §11).
    pub fn to_sda_value(&self) -> sda_core::Value {
        use sda_core::{ExactNum, Value};
        let num =
            |n: u64| Value::Num(ExactNum::parse_literal(&n.to_string()).expect("u64 literal"));
        Value::Prod(vec![
            ("query_id".into(), Value::Str(self.query_id.clone())),
            ("page_number".into(), num(self.page_number)),
            ("complete".into(), Value::Bool(self.complete)),
            (
                "units".into(),
                Value::Seq(
                    self.units
                        .iter()
                        .map(crate::unit::ExaminationUnit::to_sda_value)
                        .collect(),
                ),
            ),
            (
                "coverage".into(),
                Value::Prod(vec![
                    (
                        "catalogs".into(),
                        Value::Str(self.coverage.catalogs.clone()),
                    ),
                    ("indexes".into(), Value::Str(self.coverage.indexes.clone())),
                    ("requested_partitions".into(), Value::Set(vec![])),
                    ("completed_partitions".into(), Value::Set(vec![])),
                    ("unavailable_partitions".into(), Value::Set(vec![])),
                    ("partition_frontiers".into(), Value::Map(vec![])),
                    (
                        "tiers".into(),
                        Value::Set(
                            self.coverage
                                .tiers
                                .iter()
                                .map(|t| Value::Str(t.clone()))
                                .collect(),
                        ),
                    ),
                    (
                        "excluded_tiers".into(),
                        Value::Set(
                            self.coverage
                                .excluded_tiers
                                .iter()
                                .map(|t| Value::Str(t.clone()))
                                .collect(),
                        ),
                    ),
                ]),
            ),
            (
                "continuation".into(),
                match &self.continuation {
                    Some(b) => Value::Some_(Box::new(Value::Bytes(b.clone()))),
                    None => Value::None_,
                },
            ),
            (
                "uncertainty".into(),
                Value::Set(
                    self.uncertainty
                        .iter()
                        .map(|s| Value::Str(s.clone()))
                        .collect(),
                ),
            ),
        ])
    }
}
