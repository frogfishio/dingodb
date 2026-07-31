//! Canonical logical plan (`rql-plan-v1`) + encoding/hash (APP-4).
//!
//! Normative companions:
//! - [doc/wip/query/RQL_SPEC.md](../../../doc/wip/query/RQL_SPEC.md) §15–§16
//! - [CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md](../../../doc/todo/application-baseline/CORE_APPLICATION_API_IMPLEMENTATION_PLAN.md) §9 / §14 APP-4
//! - `spec/app/v1/plan_vectors_v1.json`
//!
//! This cut freezes **canonical JSON bytes** and a domain-separated BLAKE3 plan
//! hash so cursors and explain can bind plan identity. Full RQL source parsing
//! is APP-5; builder + fixture compilation land here.

use crate::app_v1::{ConsistencyMode, CoveragePolicy};
use crate::error::Error;
use crate::predicate::{field, param, Path, Predicate};
use residiuum_heap::CollectionId;
use serde_json::{json, Map, Value as JsonValue};
use std::collections::BTreeMap;
use std::str::FromStr;

/// Logical plan profile (same as [`crate::RQL_PLAN_PROFILE`]).
pub const PLAN_PROFILE: &str = "rql-plan-v1";

/// Companion encoding profile for persistent / exchanged plan bytes.
pub const PLAN_ENCODING_PROFILE: &str = "rql-plan-encoding-v1";

/// Domain separation tag for plan hashes (BLAKE3 over domain || 0x00 || body).
pub const PLAN_HASH_DOMAIN: &str = "residiuum:rql-plan-v1:canonical-v1";

/// Implicit key tie-break path used in Application Core order.
pub const KEY_TIE_BREAK_PATH: &str = "$key";

/// Default page size (CORE plan §9.2).
pub const DEFAULT_PAGE_SIZE: u32 = 64;

/// Maximum page size.
pub const MAX_PAGE_SIZE: u32 = 4_096;

/// Maximum order terms before the key tie-break.
pub const MAX_ORDER_TERMS: usize = 16;

/// Maximum projected items.
pub const MAX_PROJECT_ITEMS: usize = 1_024;

/// Sort direction on a plan order term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDir {
    /// Ascending.
    Asc,
    /// Descending.
    Desc,
}

impl OrderDir {
    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }

    fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            other => Err(Error::QueryInvalid(format!("unknown order dir `{other}`"))),
        }
    }
}

/// Null/missing placement for ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullsOrder {
    /// Nulls/missing last (default Application Core).
    Last,
    /// Nulls/missing first.
    First,
}

impl NullsOrder {
    fn as_str(self) -> &'static str {
        match self {
            Self::Last => "last",
            Self::First => "first",
        }
    }

    fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "last" => Ok(Self::Last),
            "first" => Ok(Self::First),
            other => Err(Error::QueryInvalid(format!("unknown nulls order `{other}`"))),
        }
    }
}

/// One order term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderTerm {
    /// Path segments (`$key` for the immutable document key).
    pub path: Path,
    /// Direction.
    pub dir: OrderDir,
    /// Null/missing placement.
    pub nulls: NullsOrder,
    /// True when this is the implicit key tie-break.
    pub tie_break: bool,
}

/// Bound collection source after name → immutable id binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanSource {
    /// Display / RQL source name (for diagnostics).
    pub source_name: String,
    /// Immutable collection identity (Heap-confined).
    pub collection_id: CollectionId,
}

/// Validated Application Core logical plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RqlPlanV1 {
    /// Always [`PLAN_PROFILE`].
    pub profile: String,
    /// Predicate profile label.
    pub predicate_profile: String,
    /// Bound source.
    pub from: PlanSource,
    /// Normalized where predicate (`True` when absent).
    pub where_pred: Predicate,
    /// Projection paths; `None` means full document.
    pub project: Option<Vec<Path>>,
    /// Order terms including key tie-break as last term.
    pub order: Vec<OrderTerm>,
    /// Optional total limit across pages.
    pub limit: Option<u64>,
    /// Page size (1..=4096).
    pub page_size: u32,
    /// Coverage policy.
    pub coverage: CoveragePolicy,
    /// Consistency mode.
    pub consistency: ConsistencyMode,
}

impl RqlPlanV1 {
    /// Domain-separated BLAKE3-256 plan hash over [`Self::canonical_bytes`].
    pub fn plan_hash(&self) -> [u8; 32] {
        plan_hash_bytes(&self.canonical_bytes())
    }

    /// Hex encoding of [`Self::plan_hash`].
    pub fn plan_hash_hex(&self) -> String {
        bytes_to_hex(&self.plan_hash())
    }

    /// Canonical JSON UTF-8 bytes (stable key order).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let v = self.to_canonical_json();
        // serde_json Map iteration is insertion-ordered; we insert keys in sorted
        // order via BTreeMap intermediate.
        serde_json::to_vec(&v).expect("canonical plan json")
    }

    /// Canonical JSON value (for fixtures / explain).
    pub fn to_canonical_json(&self) -> JsonValue {
        let mut root = BTreeMap::new();
        root.insert("profile".into(), JsonValue::String(self.profile.clone()));
        root.insert(
            "predicate_profile".into(),
            JsonValue::String(self.predicate_profile.clone()),
        );

        let mut from = BTreeMap::new();
        from.insert(
            "source_name".into(),
            JsonValue::String(self.from.source_name.clone()),
        );
        from.insert(
            "collection_id".into(),
            JsonValue::String(format_collection_id(&self.from.collection_id)),
        );
        root.insert("from".into(), btree_to_obj(from));

        root.insert("where".into(), self.where_pred.to_canonical_json());

        match &self.project {
            None => {
                root.insert("project".into(), JsonValue::Null);
            }
            Some(paths) => {
                root.insert(
                    "project".into(),
                    JsonValue::Array(
                        paths
                            .iter()
                            .map(|p| {
                                JsonValue::Array(
                                    p.0.iter()
                                        .map(|s| JsonValue::String(s.clone()))
                                        .collect(),
                                )
                            })
                            .collect(),
                    ),
                );
            }
        }

        let order: Vec<JsonValue> = self
            .order
            .iter()
            .map(|t| {
                let mut m = BTreeMap::new();
                m.insert(
                    "path".into(),
                    JsonValue::Array(
                        t.path
                            .0
                            .iter()
                            .map(|s| JsonValue::String(s.clone()))
                            .collect(),
                    ),
                );
                m.insert("dir".into(), JsonValue::String(t.dir.as_str().into()));
                m.insert("nulls".into(), JsonValue::String(t.nulls.as_str().into()));
                m.insert("tie_break".into(), JsonValue::Bool(t.tie_break));
                btree_to_obj(m)
            })
            .collect();
        root.insert("order".into(), JsonValue::Array(order));

        root.insert(
            "limit".into(),
            match self.limit {
                Some(n) => json!(n),
                None => JsonValue::Null,
            },
        );
        root.insert("page_size".into(), json!(self.page_size));
        root.insert(
            "coverage".into(),
            JsonValue::String(match self.coverage {
                CoveragePolicy::Complete => "complete".into(),
                CoveragePolicy::IncompleteAllowed => "incomplete_allowed".into(),
            }),
        );
        root.insert(
            "consistency".into(),
            JsonValue::String(match self.consistency {
                ConsistencyMode::Available => "available".into(),
                ConsistencyMode::Current => "current".into(),
            }),
        );

        btree_to_obj(root)
    }

    /// Parse a plan vector `logical_plan` object (APP-0 fixtures).
    pub fn from_plan_vector_json(v: &JsonValue) -> Result<Self, Error> {
        let obj = v
            .as_object()
            .ok_or_else(|| Error::QueryInvalid("logical_plan must be object".into()))?;
        let profile = obj
            .get("profile")
            .and_then(|p| p.as_str())
            .unwrap_or(PLAN_PROFILE)
            .to_string();
        if profile != PLAN_PROFILE {
            return Err(Error::QueryInvalid(format!(
                "unexpected plan profile `{profile}`"
            )));
        }

        let from_v = obj
            .get("from")
            .ok_or_else(|| Error::QueryInvalid("logical_plan missing from".into()))?;
        let from_obj = from_v
            .as_object()
            .ok_or_else(|| Error::QueryInvalid("from must be object".into()))?;
        let source_name = from_obj
            .get("source_name")
            .and_then(|s| s.as_str())
            .ok_or_else(|| Error::QueryInvalid("from.source_name required".into()))?
            .to_string();
        let id_str = from_obj
            .get("collection_id_bound")
            .or_else(|| from_obj.get("collection_id"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| Error::QueryInvalid("from.collection_id_bound required".into()))?;
        let collection_id = parse_collection_id(id_str)?;

        let where_pred = match obj.get("where") {
            None | Some(JsonValue::Null) => Predicate::True,
            Some(w) => Predicate::from_plan_json(w)?,
        };

        let project = match obj.get("project") {
            None | Some(JsonValue::Null) => None,
            Some(JsonValue::Array(items)) => {
                if items.len() > MAX_PROJECT_ITEMS {
                    return Err(Error::QueryInvalid("project exceeds ceiling".into()));
                }
                let mut paths = Vec::with_capacity(items.len());
                for it in items {
                    if let Some(s) = it.as_str() {
                        paths.push(Path::parse_dotted(s)?);
                    } else if let Some(arr) = it.as_array() {
                        let segs: Vec<String> = arr
                            .iter()
                            .map(|x| {
                                x.as_str()
                                    .map(|s| s.to_string())
                                    .ok_or_else(|| Error::QueryInvalid("path segment".into()))
                            })
                            .collect::<Result<_, _>>()?;
                        paths.push(Path::from_segments(segs)?);
                    } else {
                        return Err(Error::QueryInvalid("project item invalid".into()));
                    }
                }
                Some(paths)
            }
            Some(_) => return Err(Error::QueryInvalid("project must be array or null".into())),
        };

        let mut order = Vec::new();
        if let Some(JsonValue::Array(terms)) = obj.get("order") {
            if terms.len() > MAX_ORDER_TERMS + 1 {
                return Err(Error::QueryInvalid("too many order terms".into()));
            }
            for t in terms {
                let o = t
                    .as_object()
                    .ok_or_else(|| Error::QueryInvalid("order term object".into()))?;
                let path = match o.get("path") {
                    Some(JsonValue::Array(arr)) => {
                        let segs: Vec<String> = arr
                            .iter()
                            .map(|x| {
                                x.as_str()
                                    .map(|s| s.to_string())
                                    .ok_or_else(|| Error::QueryInvalid("order path".into()))
                            })
                            .collect::<Result<_, _>>()?;
                        Path::from_segments(segs)?
                    }
                    Some(JsonValue::String(s)) => Path::parse_dotted(s)?,
                    _ => return Err(Error::QueryInvalid("order path required".into())),
                };
                let dir = OrderDir::parse(o.get("dir").and_then(|d| d.as_str()).unwrap_or("asc"))?;
                let nulls =
                    NullsOrder::parse(o.get("nulls").and_then(|n| n.as_str()).unwrap_or("last"))?;
                let tie_break = o.get("tie_break").and_then(|b| b.as_bool()).unwrap_or(false);
                order.push(OrderTerm {
                    path,
                    dir,
                    nulls,
                    tie_break,
                });
            }
        }
        ensure_key_tie_break(&mut order)?;

        let limit = match obj.get("limit") {
            None | Some(JsonValue::Null) => None,
            Some(JsonValue::Number(n)) => Some(
                n.as_u64()
                    .ok_or_else(|| Error::QueryInvalid("limit must be u64".into()))?,
            ),
            Some(_) => return Err(Error::QueryInvalid("limit invalid".into())),
        };

        let page_size = match obj.get("page_size") {
            None => DEFAULT_PAGE_SIZE,
            Some(JsonValue::Number(n)) => {
                let p = n
                    .as_u64()
                    .ok_or_else(|| Error::QueryInvalid("page_size invalid".into()))?;
                if p == 0 || p > u64::from(MAX_PAGE_SIZE) {
                    return Err(Error::QueryInvalid(format!(
                        "page_size must be 1..={MAX_PAGE_SIZE}"
                    )));
                }
                p as u32
            }
            Some(_) => return Err(Error::QueryInvalid("page_size invalid".into())),
        };

        let coverage = match obj.get("coverage").and_then(|c| c.as_str()).unwrap_or("complete") {
            "complete" => CoveragePolicy::Complete,
            "incomplete_allowed" | "allow_incomplete" => CoveragePolicy::IncompleteAllowed,
            other => {
                return Err(Error::QueryInvalid(format!("unknown coverage `{other}`")));
            }
        };
        let consistency = match obj
            .get("consistency")
            .and_then(|c| c.as_str())
            .unwrap_or("available")
        {
            "available" => ConsistencyMode::Available,
            "current" => ConsistencyMode::Current,
            other => {
                return Err(Error::QueryInvalid(format!(
                    "unknown consistency `{other}`"
                )));
            }
        };

        let plan = Self {
            profile,
            predicate_profile: crate::PREDICATE_PROFILE.to_string(),
            from: PlanSource {
                source_name,
                collection_id,
            },
            where_pred,
            project,
            order,
            limit,
            page_size,
            coverage,
            consistency,
        };
        plan.where_pred.check_node_ceiling()?;
        Ok(plan)
    }
}

/// Name → collection id binding table for plan compilation.
#[derive(Debug, Clone, Default)]
pub struct CollectionBindings {
    /// Map of source name → immutable collection id.
    pub by_name: BTreeMap<String, CollectionId>,
}

impl CollectionBindings {
    /// Insert a binding.
    pub fn bind(&mut self, name: impl Into<String>, id: CollectionId) {
        self.by_name.insert(name.into(), id);
    }

    /// Resolve a source name.
    pub fn resolve(&self, name: &str) -> Result<CollectionId, Error> {
        self.by_name
            .get(name)
            .copied()
            .ok_or_else(|| Error::QueryInvalid(format!("unknown collection source `{name}`")))
    }
}

/// Builder that compiles to [`RqlPlanV1`] (CORE plan §9.1 example surface).
#[derive(Debug, Clone)]
pub struct PlanBuilder {
    source_name: String,
    where_pred: Predicate,
    project: Option<Vec<Path>>,
    order: Vec<OrderTerm>,
    limit: Option<u64>,
    page_size: Option<u32>,
    coverage: CoveragePolicy,
    consistency: ConsistencyMode,
}

impl PlanBuilder {
    /// Start a plan for source name `from`.
    pub fn from_source(name: impl Into<String>) -> Self {
        Self {
            source_name: name.into(),
            where_pred: Predicate::True,
            project: None,
            order: Vec::new(),
            limit: None,
            page_size: None,
            coverage: CoveragePolicy::Complete,
            consistency: ConsistencyMode::Available,
        }
    }

    /// Attach a where predicate.
    pub fn where_(mut self, pred: Predicate) -> Self {
        self.where_pred = pred;
        self
    }

    /// Projection field paths (dotted).
    pub fn project<I, S>(mut self, fields: I) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut paths = Vec::new();
        for f in fields {
            paths.push(Path::parse_dotted(f.as_ref())?);
        }
        if paths.len() > MAX_PROJECT_ITEMS {
            return Err(Error::QueryInvalid("project exceeds ceiling".into()));
        }
        self.project = Some(paths);
        Ok(self)
    }

    /// Add an order term (key tie-break inserted at compile).
    pub fn order_by(mut self, path: &str, dir: OrderDir) -> Result<Self, Error> {
        if self.order.len() >= MAX_ORDER_TERMS {
            return Err(Error::QueryInvalid("too many order terms".into()));
        }
        self.order.push(OrderTerm {
            path: Path::parse_dotted(path)?,
            dir,
            nulls: NullsOrder::Last,
            tie_break: false,
        });
        Ok(self)
    }

    /// Total limit across pages.
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Page size.
    pub fn page_size(mut self, n: u32) -> Result<Self, Error> {
        if n == 0 || n > MAX_PAGE_SIZE {
            return Err(Error::QueryInvalid(format!(
                "page_size must be 1..={MAX_PAGE_SIZE}"
            )));
        }
        self.page_size = Some(n);
        Ok(self)
    }

    /// Coverage policy.
    pub fn coverage(mut self, c: CoveragePolicy) -> Self {
        self.coverage = c;
        self
    }

    /// Consistency mode.
    pub fn consistency(mut self, c: ConsistencyMode) -> Self {
        self.consistency = c;
        self
    }

    /// Bind source name and produce a validated plan.
    pub fn compile(self, bindings: &CollectionBindings) -> Result<RqlPlanV1, Error> {
        let collection_id = bindings.resolve(&self.source_name)?;
        let mut order = self.order;
        ensure_key_tie_break(&mut order)?;
        self.where_pred.check_node_ceiling()?;
        Ok(RqlPlanV1 {
            profile: PLAN_PROFILE.into(),
            predicate_profile: crate::PREDICATE_PROFILE.into(),
            from: PlanSource {
                source_name: self.source_name,
                collection_id,
            },
            where_pred: self.where_pred,
            project: self.project,
            order,
            limit: self.limit,
            page_size: self.page_size.unwrap_or(DEFAULT_PAGE_SIZE),
            coverage: self.coverage,
            consistency: self.consistency,
        })
    }
}

/// Convenience: `field("status").eq(param("status"))` for plan builders.
pub fn where_field_eq_param(path: &str, param_name: &str) -> Result<Predicate, Error> {
    Ok(field(path)?.eq(param(param_name)))
}

fn ensure_key_tie_break(order: &mut Vec<OrderTerm>) -> Result<(), Error> {
    let has_tb = order.iter().any(|t| t.tie_break);
    if !has_tb {
        order.push(OrderTerm {
            path: Path::from_segments([KEY_TIE_BREAK_PATH])?,
            dir: OrderDir::Asc,
            nulls: NullsOrder::Last,
            tie_break: true,
        });
    }
    // Key tie-break must be last.
    if let Some(pos) = order.iter().position(|t| t.tie_break) {
        if pos != order.len() - 1 {
            let tb = order.remove(pos);
            order.push(tb);
        }
    }
    Ok(())
}

fn plan_hash_bytes(canonical_body: &[u8]) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(PLAN_HASH_DOMAIN.as_bytes());
    h.update(&[0u8]);
    h.update(canonical_body);
    *h.finalize().as_bytes()
}

fn btree_to_obj(m: BTreeMap<String, JsonValue>) -> JsonValue {
    // BTreeMap iterates in key order → stable object key order in serde_json Map.
    let mut map = Map::new();
    for (k, v) in m {
        map.insert(k, v);
    }
    JsonValue::Object(map)
}

fn format_collection_id(id: &CollectionId) -> String {
    id.to_string()
}

fn parse_collection_id(s: &str) -> Result<CollectionId, Error> {
    CollectionId::from_str(s).map_err(|e| Error::QueryInvalid(format!("collection id: {e}")))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{field, param};

    fn orders_id() -> CollectionId {
        parse_collection_id("00000000-0000-4000-8000-0000000000a1").unwrap()
    }

    #[test]
    fn builder_matches_vector_shape_hash_stable() {
        let mut bindings = CollectionBindings::default();
        bindings.bind("orders", orders_id());
        let plan = PlanBuilder::from_source("orders")
            .where_(field("status").unwrap().eq(param("status")))
            .project(["id", "status"])
            .unwrap()
            .order_by("created_at", OrderDir::Desc)
            .unwrap()
            .limit(1000)
            .page_size(100)
            .unwrap()
            .compile(&bindings)
            .unwrap();
        let h1 = plan.plan_hash();
        let h2 = plan.plan_hash();
        assert_eq!(h1, h2);
        assert_ne!(h1, [0u8; 32]);
        // Re-parse canonical json round-trip via vector parser is not identity
        // for where form, but hash of same builder plan is stable.
        assert_eq!(plan.page_size, 100);
        assert_eq!(plan.order.len(), 2);
        assert!(plan.order.last().unwrap().tie_break);
    }

    #[test]
    fn defaults_only_plan() {
        let mut bindings = CollectionBindings::default();
        bindings.bind("orders", orders_id());
        let plan = PlanBuilder::from_source("orders")
            .compile(&bindings)
            .unwrap();
        assert_eq!(plan.page_size, DEFAULT_PAGE_SIZE);
        assert!(matches!(plan.where_pred, Predicate::True));
        assert_eq!(plan.order.len(), 1);
        assert!(plan.order[0].tie_break);
    }

    #[test]
    fn unknown_source_fails() {
        let bindings = CollectionBindings::default();
        let err = PlanBuilder::from_source("orders")
            .compile(&bindings)
            .unwrap_err();
        assert!(matches!(err, Error::QueryInvalid(_)));
    }
}
