//! SDK-native JSON filters (DX_SPEC §7.1–§7.2).
//!
//! Common field predicates without requiring callers to write SDA. Filters are
//! evaluated by scanning live collection entries (correct without secondary
//! indexes; DX_SPEC §8.1). Compilation of filters to SDA and raw examination
//! over recovery units live in the `dingo-examine` crate (Stage 5).

use crate::error::Error;
use serde_json::Value as JsonValue;

/// Sort direction for ordered queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Ascending (default).
    #[default]
    Asc,
    /// Descending.
    Desc,
}

/// Explicit resource budget for scans (DX_SPEC §8.1).
#[derive(Debug, Clone, Default)]
pub struct QueryBudget {
    /// Maximum live documents the engine may examine (index probe + scan).
    ///
    /// When unset, scans are unbounded (Stage 4 compatible default).
    pub max_docs_scanned: Option<usize>,
}

impl QueryBudget {
    /// Budget allowing at most `n` documents to be examined.
    pub fn max_docs(n: usize) -> Self {
        Self {
            max_docs_scanned: Some(n),
        }
    }
}

/// Options for [`crate::Collection::find_with`].
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Maximum number of matching rows to return. `None` means unbounded
    /// (caller is responsible for result size).
    pub limit: Option<usize>,
    /// Optional order-by field path (JSON path, dotted) and direction.
    ///
    /// When unset, results are in stable subject/key order (deterministic).
    pub order_by: Option<(String, SortOrder)>,
    /// Optional scan budget. When set and a scan would exceed it without a
    /// usable index, the query fails with [`ErrorCode::QueryBudgetRequired`].
    pub budget: Option<QueryBudget>,
    /// When true, force a full collection scan even if an index exists.
    pub force_scan: bool,
    /// When true, allow returning matches under incomplete cluster coverage
    /// (Stage 8e). Default `false`: incomplete coverage yields
    /// [`ErrorCode::CoverageIncomplete`] so partial results are never mistaken
    /// for a complete empty set (CLUSTER_SPEC §17.2).
    pub allow_partial_coverage: bool,
}

impl QueryOptions {
    /// Unbounded, key-ordered result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Cap the number of returned rows.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Order by a JSON field path.
    pub fn order_by(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.order_by = Some((field.into(), order));
        self
    }

    /// Attach a scan budget (DX_SPEC §8.1).
    pub fn budget(mut self, budget: QueryBudget) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Force scan path (skip secondary indexes).
    pub fn force_scan(mut self) -> Self {
        self.force_scan = true;
        self
    }

    /// Allow partial cluster coverage (return matches + incomplete coverage).
    pub fn allow_partial_coverage(mut self) -> Self {
        self.allow_partial_coverage = true;
        self
    }
}

/// Field-level predicate (DX_SPEC §7.1 portable vocabulary).
#[derive(Debug, Clone, PartialEq)]
pub enum Pred {
    /// Exact equality (JSON value equality).
    Eq(JsonValue),
    /// Inequality.
    Ne(JsonValue),
    /// Strict less-than (numbers and strings).
    Lt(JsonValue),
    /// Less-or-equal.
    Lte(JsonValue),
    /// Strict greater-than.
    Gt(JsonValue),
    /// Greater-or-equal.
    Gte(JsonValue),
    /// Membership in a list of values.
    In(Vec<JsonValue>),
    /// Field presence (`true`) or absence (`false`). `Null` counts as present.
    Exists(bool),
    /// String value starts with this prefix.
    Prefix(String),
    /// String contains substring, or array contains element.
    Contains(JsonValue),
}

/// A filter expression over a JSON document.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    /// Matches every document.
    Always,
    /// Field path + predicate. Paths use `.` separators (`"address.city"`).
    Field {
        /// Dotted JSON path.
        path: String,
        /// Predicate applied to the value at `path`.
        pred: Pred,
    },
    /// Conjunction.
    And(Vec<Filter>),
    /// Disjunction.
    Or(Vec<Filter>),
    /// Negation.
    Not(Box<Filter>),
}

impl Filter {
    /// Match everything.
    pub fn always() -> Self {
        Self::Always
    }

    /// Start a field predicate builder: `Filter::field("status").eq("active")`.
    pub fn field(path: impl Into<String>) -> FieldBuilder {
        FieldBuilder { path: path.into() }
    }

    /// Conjunction of filters (empty ⇒ always true).
    pub fn and(parts: impl IntoIterator<Item = Filter>) -> Self {
        let v: Vec<_> = parts.into_iter().collect();
        if v.is_empty() {
            Self::Always
        } else if v.len() == 1 {
            v.into_iter().next().unwrap()
        } else {
            Self::And(v)
        }
    }

    /// Disjunction of filters (empty ⇒ never matches).
    pub fn or(parts: impl IntoIterator<Item = Filter>) -> Self {
        let v: Vec<_> = parts.into_iter().collect();
        if v.is_empty() {
            // Empty OR matches nothing.
            Self::Not(Box::new(Self::Always))
        } else if v.len() == 1 {
            v.into_iter().next().unwrap()
        } else {
            Self::Or(v)
        }
    }

    /// Negation.
    pub fn not(inner: Filter) -> Self {
        Self::Not(Box::new(inner))
    }

    /// Parse a Mongo-style / DX_SPEC object filter from JSON.
    ///
    /// Supported shapes:
    /// - `{ "status": "active" }` → equality
    /// - `{ "age": { "$gte": 18 } }` → comparison / `$in` / `$ne` / `$exists` /
    ///   `$prefix` / `$contains`
    /// - `{ "$and": [ ... ] }`, `{ "$or": [ ... ] }`, `{ "$not": { ... } }`
    ///
    /// Multiple top-level keys are combined with AND.
    pub fn from_json(value: &JsonValue) -> Result<Self, Error> {
        parse_filter(value)
    }

    /// Whether `doc` satisfies this filter.
    pub fn matches(&self, doc: &JsonValue) -> bool {
        match self {
            Self::Always => true,
            Self::Field { path, pred } => match resolve_path(doc, path) {
                PathHit::Missing => match pred {
                    Pred::Exists(want) => !*want,
                    Pred::Ne(_) => true, // missing ≠ any concrete value
                    _ => false,
                },
                PathHit::Present(v) => pred_matches(v, pred),
            },
            Self::And(parts) => parts.iter().all(|p| p.matches(doc)),
            Self::Or(parts) => parts.iter().any(|p| p.matches(doc)),
            Self::Not(inner) => !inner.matches(doc),
        }
    }

    /// Encode this filter as a DX/Mongo-style JSON object (round-trips via [`Self::from_json`]).
    ///
    /// Used for remote `find` RPC: the server parses with [`Self::from_json`].
    pub fn to_json(&self) -> JsonValue {
        match self {
            Self::Always => JsonValue::Object(serde_json::Map::new()),
            Self::Field { path, pred } => {
                let mut map = serde_json::Map::new();
                map.insert(path.clone(), pred_to_json(pred));
                JsonValue::Object(map)
            }
            Self::And(parts) => {
                if parts.is_empty() {
                    return JsonValue::Object(serde_json::Map::new());
                }
                if parts.len() == 1 {
                    return parts[0].to_json();
                }
                // Prefer a flat multi-key object when every part is a distinct field predicate.
                let mut flat = serde_json::Map::new();
                let mut can_flat = true;
                for p in parts {
                    match p {
                        Self::Field { path, pred } if !flat.contains_key(path) => {
                            flat.insert(path.clone(), pred_to_json(pred));
                        }
                        _ => {
                            can_flat = false;
                            break;
                        }
                    }
                }
                if can_flat {
                    JsonValue::Object(flat)
                } else {
                    JsonValue::Object(serde_json::Map::from_iter([(
                        "$and".into(),
                        JsonValue::Array(parts.iter().map(|p| p.to_json()).collect()),
                    )]))
                }
            }
            Self::Or(parts) => {
                if parts.is_empty() {
                    // Empty OR matches nothing ≡ Not(Always).
                    return Self::not(Self::Always).to_json();
                }
                if parts.len() == 1 {
                    return parts[0].to_json();
                }
                JsonValue::Object(serde_json::Map::from_iter([(
                    "$or".into(),
                    JsonValue::Array(parts.iter().map(|p| p.to_json()).collect()),
                )]))
            }
            Self::Not(inner) => JsonValue::Object(serde_json::Map::from_iter([(
                "$not".into(),
                inner.to_json(),
            )])),
        }
    }
}

fn pred_to_json(pred: &Pred) -> JsonValue {
    match pred {
        // Bare value → equality (matches `from_json` bare-value rule).
        Pred::Eq(v) => v.clone(),
        Pred::Ne(v) => json_op("$ne", v.clone()),
        Pred::Lt(v) => json_op("$lt", v.clone()),
        Pred::Lte(v) => json_op("$lte", v.clone()),
        Pred::Gt(v) => json_op("$gt", v.clone()),
        Pred::Gte(v) => json_op("$gte", v.clone()),
        Pred::In(list) => json_op("$in", JsonValue::Array(list.clone())),
        Pred::Exists(b) => json_op("$exists", JsonValue::Bool(*b)),
        Pred::Prefix(s) => json_op("$prefix", JsonValue::String(s.clone())),
        Pred::Contains(v) => json_op("$contains", v.clone()),
    }
}

fn json_op(op: &str, rhs: JsonValue) -> JsonValue {
    JsonValue::Object(serde_json::Map::from_iter([(op.into(), rhs)]))
}

/// Builder for a single field predicate.
#[derive(Debug, Clone)]
pub struct FieldBuilder {
    path: String,
}

impl FieldBuilder {
    /// Equality.
    pub fn eq(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Eq(value.into()),
        }
    }

    /// Inequality.
    pub fn ne(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Ne(value.into()),
        }
    }

    /// Less-than.
    pub fn lt(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Lt(value.into()),
        }
    }

    /// Less-or-equal.
    pub fn lte(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Lte(value.into()),
        }
    }

    /// Greater-than.
    pub fn gt(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Gt(value.into()),
        }
    }

    /// Greater-or-equal.
    pub fn gte(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Gte(value.into()),
        }
    }

    /// Membership.
    pub fn is_in<I, V>(self, values: I) -> Filter
    where
        I: IntoIterator<Item = V>,
        V: Into<JsonValue>,
    {
        Filter::Field {
            path: self.path,
            pred: Pred::In(values.into_iter().map(Into::into).collect()),
        }
    }

    /// Field must exist (`true`) or must be absent (`false`).
    pub fn exists(self, present: bool) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Exists(present),
        }
    }

    /// String prefix match.
    pub fn prefix(self, prefix: impl Into<String>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Prefix(prefix.into()),
        }
    }

    /// String/array containment.
    pub fn contains(self, value: impl Into<JsonValue>) -> Filter {
        Filter::Field {
            path: self.path,
            pred: Pred::Contains(value.into()),
        }
    }
}

/// Fluent query builder (DX_SPEC §7.2).
///
/// ```ignore
/// let rows = users
///     .query()
///     .where_eq("status", "active")
///     .where_gte("age", 18)
///     .order_by("age", SortOrder::Desc)
///     .limit(100)
///     .collect()?;
/// ```
pub struct QueryBuilder<'c, 'a> {
    pub(crate) collection: &'c mut crate::collection::Collection<'a>,
    filters: Vec<Filter>,
    options: QueryOptions,
}

impl<'c, 'a> QueryBuilder<'c, 'a> {
    pub(crate) fn new(collection: &'c mut crate::collection::Collection<'a>) -> Self {
        Self {
            collection,
            filters: Vec::new(),
            options: QueryOptions::default(),
        }
    }

    /// Add an arbitrary filter (AND-combined with previous clauses).
    pub fn filter(mut self, f: Filter) -> Self {
        self.filters.push(f);
        self
    }

    /// `field == value`.
    pub fn where_eq(self, field: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.filter(Filter::field(field).eq(value))
    }

    /// `field != value`.
    pub fn where_ne(self, field: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.filter(Filter::field(field).ne(value))
    }

    /// `field < value`.
    pub fn where_lt(self, field: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.filter(Filter::field(field).lt(value))
    }

    /// `field <= value`.
    pub fn where_lte(self, field: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.filter(Filter::field(field).lte(value))
    }

    /// `field > value`.
    pub fn where_gt(self, field: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.filter(Filter::field(field).gt(value))
    }

    /// `field >= value`.
    pub fn where_gte(self, field: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.filter(Filter::field(field).gte(value))
    }

    /// `field` in `values`.
    pub fn where_in<I, V>(self, field: impl Into<String>, values: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<JsonValue>,
    {
        self.filter(Filter::field(field).is_in(values))
    }

    /// Field existence.
    pub fn where_exists(self, field: impl Into<String>, present: bool) -> Self {
        self.filter(Filter::field(field).exists(present))
    }

    /// String prefix.
    pub fn where_prefix(self, field: impl Into<String>, prefix: impl Into<String>) -> Self {
        self.filter(Filter::field(field).prefix(prefix))
    }

    /// Cap result size.
    pub fn limit(mut self, n: usize) -> Self {
        self.options.limit = Some(n);
        self
    }

    /// Order by field path.
    pub fn order_by(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.options.order_by = Some((field.into(), order));
        self
    }

    /// Execute and materialize matching rows.
    pub fn collect(self) -> Result<Vec<(String, JsonValue)>, Error> {
        let filter = Filter::and(self.filters);
        self.collection.find_with(&filter, self.options)
    }
}

// --- matching internals -------------------------------------------------------

enum PathHit<'a> {
    Missing,
    Present(&'a JsonValue),
}

/// Resolve a dotted JSON path to a present value (for index key extraction).
pub(crate) fn resolve_path_value<'a>(doc: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    match resolve_path(doc, path) {
        PathHit::Present(v) => Some(v),
        PathHit::Missing => None,
    }
}

fn resolve_path<'a>(doc: &'a JsonValue, path: &str) -> PathHit<'a> {
    if path.is_empty() {
        return PathHit::Present(doc);
    }
    let mut cur = doc;
    for part in path.split('.') {
        match cur {
            JsonValue::Object(map) => match map.get(part) {
                Some(next) => cur = next,
                None => return PathHit::Missing,
            },
            _ => return PathHit::Missing,
        }
    }
    PathHit::Present(cur)
}

fn pred_matches(value: &JsonValue, pred: &Pred) -> bool {
    match pred {
        Pred::Eq(rhs) => value == rhs,
        Pred::Ne(rhs) => value != rhs,
        Pred::Lt(rhs) => cmp_ord(value, rhs) == Some(std::cmp::Ordering::Less),
        Pred::Lte(rhs) => matches!(
            cmp_ord(value, rhs),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        ),
        Pred::Gt(rhs) => cmp_ord(value, rhs) == Some(std::cmp::Ordering::Greater),
        Pred::Gte(rhs) => matches!(
            cmp_ord(value, rhs),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        ),
        Pred::In(list) => list.iter().any(|v| v == value),
        Pred::Exists(want) => *want, // present ⇒ exists true; false would need Missing branch
        Pred::Prefix(p) => match value {
            JsonValue::String(s) => s.starts_with(p.as_str()),
            _ => false,
        },
        Pred::Contains(needle) => match (value, needle) {
            (JsonValue::String(hay), JsonValue::String(n)) => hay.contains(n.as_str()),
            (JsonValue::Array(arr), n) => arr.iter().any(|e| e == n),
            _ => false,
        },
    }
}

/// Compare numbers and strings only; mixed/other types yield `None` (predicate fails).
fn cmp_ord(a: &JsonValue, b: &JsonValue) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (JsonValue::Number(x), JsonValue::Number(y)) => {
            let xf = x.as_f64()?;
            let yf = y.as_f64()?;
            xf.partial_cmp(&yf)
        }
        (JsonValue::String(x), JsonValue::String(y)) => Some(x.cmp(y)),
        (JsonValue::Bool(x), JsonValue::Bool(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

fn parse_filter(value: &JsonValue) -> Result<Filter, Error> {
    match value {
        JsonValue::Object(map) if map.is_empty() => Ok(Filter::Always),
        JsonValue::Object(map) => {
            // Single logical combinator at top level.
            if map.len() == 1 {
                if let Some(arr) = map.get("$and") {
                    return parse_filter_list(arr, true);
                }
                if let Some(arr) = map.get("$or") {
                    return parse_filter_list(arr, false);
                }
                if let Some(inner) = map.get("$not") {
                    return Ok(Filter::not(parse_filter(inner)?));
                }
            }
            let mut parts = Vec::with_capacity(map.len());
            for (key, val) in map {
                if key.starts_with('$') {
                    // Combinators mixed with fields: only allow as sole key (handled above).
                    return Err(Error::QueryInvalid(format!(
                        "unexpected operator key {key:?} among field predicates"
                    )));
                }
                parts.push(parse_field_pred(key, val)?);
            }
            Ok(Filter::and(parts))
        }
        _ => Err(Error::QueryInvalid("filter must be a JSON object".into())),
    }
}

fn parse_filter_list(value: &JsonValue, is_and: bool) -> Result<Filter, Error> {
    let arr = value.as_array().ok_or_else(|| {
        Error::QueryInvalid(if is_and {
            "$and expects an array".into()
        } else {
            "$or expects an array".into()
        })
    })?;
    let mut parts = Vec::with_capacity(arr.len());
    for item in arr {
        parts.push(parse_filter(item)?);
    }
    Ok(if is_and {
        Filter::and(parts)
    } else {
        Filter::or(parts)
    })
}

fn parse_field_pred(path: &str, value: &JsonValue) -> Result<Filter, Error> {
    // Bare value → equality.
    if !value.is_object() {
        return Ok(Filter::field(path).eq(value.clone()));
    }
    let obj = value.as_object().unwrap();
    if obj.is_empty() {
        return Err(Error::QueryInvalid(format!(
            "empty operator object for field {path:?}"
        )));
    }
    // Multiple operators on one field → AND.
    let mut parts = Vec::with_capacity(obj.len());
    for (op, rhs) in obj {
        let pred = match op.as_str() {
            "$eq" => Pred::Eq(rhs.clone()),
            "$ne" => Pred::Ne(rhs.clone()),
            "$lt" => Pred::Lt(rhs.clone()),
            "$lte" => Pred::Lte(rhs.clone()),
            "$gt" => Pred::Gt(rhs.clone()),
            "$gte" => Pred::Gte(rhs.clone()),
            "$in" => {
                let arr = rhs
                    .as_array()
                    .ok_or_else(|| Error::QueryInvalid("$in expects an array".into()))?;
                Pred::In(arr.clone())
            }
            "$exists" => {
                let b = rhs
                    .as_bool()
                    .ok_or_else(|| Error::QueryInvalid("$exists expects a boolean".into()))?;
                Pred::Exists(b)
            }
            "$prefix" => {
                let s = rhs
                    .as_str()
                    .ok_or_else(|| Error::QueryInvalid("$prefix expects a string".into()))?;
                Pred::Prefix(s.to_string())
            }
            "$contains" => Pred::Contains(rhs.clone()),
            other => {
                return Err(Error::QueryInvalid(format!(
                    "unknown filter operator {other:?}"
                )));
            }
        };
        parts.push(Filter::Field {
            path: path.to_string(),
            pred,
        });
    }
    Ok(Filter::and(parts))
}

/// Compare two documents for order-by: missing fields sort first in Asc.
pub(crate) fn compare_field(
    a: &JsonValue,
    b: &JsonValue,
    path: &str,
    order: SortOrder,
) -> std::cmp::Ordering {
    let av = resolve_path(a, path);
    let bv = resolve_path(b, path);
    let base = match (av, bv) {
        (PathHit::Missing, PathHit::Missing) => std::cmp::Ordering::Equal,
        (PathHit::Missing, PathHit::Present(_)) => std::cmp::Ordering::Less,
        (PathHit::Present(_), PathHit::Missing) => std::cmp::Ordering::Greater,
        (PathHit::Present(x), PathHit::Present(y)) => cmp_ord(x, y).unwrap_or_else(|| {
            // Incomparable types: fall back to stringified JSON for stability.
            let xs = x.to_string();
            let ys = y.to_string();
            xs.cmp(&ys)
        }),
    };
    match order {
        SortOrder::Asc => base,
        SortOrder::Desc => base.reverse(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn eq_and_gte() {
        let f = Filter::and([
            Filter::field("status").eq("active"),
            Filter::field("age").gte(18),
        ]);
        assert!(f.matches(&json!({"status": "active", "age": 21})));
        assert!(!f.matches(&json!({"status": "active", "age": 17})));
        assert!(!f.matches(&json!({"status": "paused", "age": 30})));
    }

    #[test]
    fn from_json_object_filter() {
        let f = Filter::from_json(&json!({
            "status": "active",
            "age": { "$gte": 18 },
            "country": { "$in": ["TH", "SG"] }
        }))
        .unwrap();
        assert!(f.matches(&json!({"status":"active","age":20,"country":"TH"})));
        assert!(!f.matches(&json!({"status":"active","age":20,"country":"US"})));
    }

    #[test]
    fn missing_field_and_exists() {
        let doc = json!({"a": 1});
        assert!(!Filter::field("b").eq(1).matches(&doc));
        assert!(Filter::field("b").exists(false).matches(&doc));
        assert!(Filter::field("a").exists(true).matches(&doc));
        assert!(Filter::field("b").ne(1).matches(&doc));
    }

    #[test]
    fn prefix_and_contains() {
        let doc = json!({"name": "Alice", "tags": ["x", "y"]});
        assert!(Filter::field("name").prefix("Al").matches(&doc));
        assert!(Filter::field("name").contains("ice").matches(&doc));
        assert!(Filter::field("tags").contains("y").matches(&doc));
    }

    #[test]
    fn dotted_path() {
        let doc = json!({"address": {"city": "Bangkok"}});
        assert!(Filter::field("address.city").eq("Bangkok").matches(&doc));
        assert!(!Filter::field("address.city").eq("Singapore").matches(&doc));
    }

    #[test]
    fn to_json_roundtrip() {
        let f = Filter::and([
            Filter::field("status").eq("active"),
            Filter::field("age").gte(18),
        ]);
        let j = f.to_json();
        let back = Filter::from_json(&j).unwrap();
        let doc = json!({"status": "active", "age": 21});
        assert!(back.matches(&doc));
        assert!(!back.matches(&json!({"status": "active", "age": 10})));
    }
}
