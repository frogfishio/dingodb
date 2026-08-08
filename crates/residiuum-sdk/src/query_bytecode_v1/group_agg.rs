//! Group-by + aggregate IR phase (RQL-Q2 `pkg_group_aggregate`).
//!
//! Profile stamp: **`residiuum-query-ir-group-agg-v1`**
//! Normative: [RQL_SPEC.md](../../../../../doc/wip/query/RQL_SPEC.md) §9a
//!
//! Lowers through Core `ProjectPaths` immediate (`ProjectImm` with group payload).
//! Phase body is a Rust IR residual (Decision 0 still OPEN) — same honesty class
//! as order/page/project. **Not** RQL-C1 / pure micro-VM.

use crate::error::Error;
use crate::plan_v1::{AggFn, AggregateSpec, GroupAggSpec};
use crate::predicate::{resolve_path, Path, Resolve};
use serde_json::{Map, Number, Value as JsonValue};
use std::collections::BTreeMap;

/// IR profile id for group/aggregate.
pub const GROUP_AGG_IR_PROFILE: &str = "residiuum-query-ir-group-agg-v1";

/// Apply group-by + aggregates to the working bag.
///
/// - Empty [`GroupAggSpec`] is a no-op (returns `working` unchanged).
/// - Empty `group_by` with aggregates ⇒ one global group.
/// - `count()` counts input rows in the group (including null/absent fields).
/// - `sum` / `min` / `max` / `avg` ignore null, absent, and non-numeric present
///   sources (heterogeneous document bags — same skip class as null).
/// - Output row keys are deterministic: `g:` + canonical group-key encoding.
pub(crate) fn apply_group_agg(
    working: Vec<(String, JsonValue)>,
    spec: &GroupAggSpec,
) -> Result<Vec<(String, JsonValue)>, Error> {
    if !spec.is_active() {
        return Ok(working);
    }

    // Bucket: canonical key string → (first-seen key values, rows)
    let mut buckets: BTreeMap<String, (Vec<JsonValue>, Vec<JsonValue>)> = BTreeMap::new();

    for (_doc_key, doc) in working {
        let mut key_vals = Vec::with_capacity(spec.group_by.len());
        for path in &spec.group_by {
            key_vals.push(match resolve_path(&doc, path) {
                Resolve::Present(v) => v,
                Resolve::Absent => JsonValue::Null,
            });
        }
        let canon = canonical_group_key(&key_vals);
        let entry = buckets
            .entry(canon)
            .or_insert_with(|| (key_vals, Vec::new()));
        entry.1.push(doc);
    }

    let mut out = Vec::with_capacity(buckets.len());
    for (canon, (key_vals, rows)) in buckets {
        let mut obj = Map::new();
        for (i, path) in spec.group_by.iter().enumerate() {
            let field = output_field_name(path);
            obj.insert(field, key_vals[i].clone());
        }
        for agg in &spec.aggregates {
            let v = evaluate_agg(agg, &rows)?;
            obj.insert(agg.output.clone(), v);
        }
        let key = format!("g:{canon}");
        out.push((key, JsonValue::Object(obj)));
    }
    Ok(out)
}

fn output_field_name(path: &Path) -> String {
    path.0.last().cloned().unwrap_or_else(|| path.dotted())
}

fn canonical_group_key(vals: &[JsonValue]) -> String {
    // Stable, collision-resistant enough for in-memory buckets (BLAKE3 of JSON).
    let body = serde_json::to_vec(vals).unwrap_or_else(|_| b"[]".to_vec());
    let hash = blake3::hash(&body);
    bytes_to_hex(hash.as_bytes())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

fn evaluate_agg(agg: &AggregateSpec, rows: &[JsonValue]) -> Result<JsonValue, Error> {
    match agg.fun {
        AggFn::Count => Ok(JsonValue::Number(Number::from(rows.len() as u64))),
        AggFn::Sum => {
            let path = agg
                .source
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("sum() requires a field path".into()))?;
            let mut sum = 0.0f64;
            let mut any = false;
            for row in rows {
                if let Some(n) = numeric_present(row, path)? {
                    sum += n;
                    any = true;
                }
            }
            if !any {
                return Ok(JsonValue::Null);
            }
            json_number(sum)
        }
        AggFn::Min => {
            let path = agg
                .source
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("min() requires a field path".into()))?;
            extremum(rows, path, true)
        }
        AggFn::Max => {
            let path = agg
                .source
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("max() requires a field path".into()))?;
            extremum(rows, path, false)
        }
        AggFn::Avg => {
            let path = agg
                .source
                .as_ref()
                .ok_or_else(|| Error::QueryInvalid("avg() requires a field path".into()))?;
            let mut sum = 0.0f64;
            let mut n = 0u64;
            for row in rows {
                if let Some(v) = numeric_present(row, path)? {
                    sum += v;
                    n += 1;
                }
            }
            if n == 0 {
                return Ok(JsonValue::Null);
            }
            json_number(sum / (n as f64))
        }
    }
}

fn extremum(rows: &[JsonValue], path: &Path, want_min: bool) -> Result<JsonValue, Error> {
    let mut best: Option<f64> = None;
    for row in rows {
        if let Some(v) = numeric_present(row, path)? {
            best = Some(match best {
                None => v,
                Some(b) if want_min => b.min(v),
                Some(b) => b.max(v),
            });
        }
    }
    match best {
        None => Ok(JsonValue::Null),
        Some(v) => json_number(v),
    }
}

fn numeric_present(doc: &JsonValue, path: &Path) -> Result<Option<f64>, Error> {
    match resolve_path(doc, path) {
        Resolve::Absent => Ok(None),
        Resolve::Present(JsonValue::Null) => Ok(None),
        Resolve::Present(JsonValue::Number(n)) => {
            match n.as_f64() {
                Some(f) if f.is_finite() => Ok(Some(f)),
                // Non-finite JSON numbers (if any) skip like null.
                _ => Ok(None),
            }
        }
        // Heterogeneous bags: non-numeric present values skip (do not fail the query).
        Resolve::Present(JsonValue::String(s)) => {
            // Accept plain decimal strings when generators encode amounts as text.
            // Reject NaN/Inf tokens and non-numeric labels.
            let t = s.trim();
            if t.eq_ignore_ascii_case("nan")
                || t.eq_ignore_ascii_case("inf")
                || t.eq_ignore_ascii_case("+inf")
                || t.eq_ignore_ascii_case("-inf")
                || t.eq_ignore_ascii_case("infinity")
                || t.eq_ignore_ascii_case("+infinity")
                || t.eq_ignore_ascii_case("-infinity")
            {
                return Ok(None);
            }
            match t.parse::<f64>() {
                Ok(f) if f.is_finite() => Ok(Some(f)),
                _ => Ok(None),
            }
        }
        Resolve::Present(_) => Ok(None),
    }
}

fn json_number(v: f64) -> Result<JsonValue, Error> {
    if !v.is_finite() {
        return Err(Error::QueryInvalid(
            "aggregate produced non-finite number".into(),
        ));
    }
    // Prefer integer encoding when the value is integral and in i64 range.
    if v.fract() == 0.0 && v >= i64::MIN as f64 && v <= i64::MAX as f64 {
        return Ok(JsonValue::Number(Number::from(v as i64)));
    }
    Number::from_f64(v)
        .map(JsonValue::Number)
        .ok_or_else(|| Error::QueryInvalid("aggregate number encode failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan_v1::AggregateSpec;

    fn path(s: &str) -> Path {
        Path::parse_dotted(s).unwrap()
    }

    #[test]
    fn count_by_region() {
        let working = vec![
            ("a".into(), serde_json::json!({"region":"us","amount":1})),
            ("b".into(), serde_json::json!({"region":"eu","amount":2})),
            ("c".into(), serde_json::json!({"region":"us","amount":3})),
        ];
        let spec = GroupAggSpec {
            group_by: vec![path("region")],
            aggregates: vec![AggregateSpec {
                fun: AggFn::Count,
                source: None,
                output: "order_count".into(),
            }],
        };
        let out = apply_group_agg(working, &spec).unwrap();
        assert_eq!(out.len(), 2);
        let mut by_region = BTreeMap::new();
        for (_, v) in out {
            let r = v
                .get("region")
                .and_then(|x| x.as_str())
                .unwrap()
                .to_string();
            let c = v.get("order_count").and_then(|x| x.as_u64()).unwrap();
            by_region.insert(r, c);
        }
        assert_eq!(by_region.get("us"), Some(&2));
        assert_eq!(by_region.get("eu"), Some(&1));
    }

    #[test]
    fn global_min_max() {
        let working = vec![
            ("a".into(), serde_json::json!({"amount": 10})),
            ("b".into(), serde_json::json!({"amount": 3})),
            ("c".into(), serde_json::json!({"amount": 7})),
        ];
        let spec = GroupAggSpec {
            group_by: vec![],
            aggregates: vec![
                AggregateSpec {
                    fun: AggFn::Min,
                    source: Some(path("amount")),
                    output: "min_amount".into(),
                },
                AggregateSpec {
                    fun: AggFn::Max,
                    source: Some(path("amount")),
                    output: "max_amount".into(),
                },
            ],
        };
        let out = apply_group_agg(working, &spec).unwrap();
        assert_eq!(out.len(), 1);
        let v = &out[0].1;
        assert_eq!(v.get("min_amount").and_then(|x| x.as_i64()), Some(3));
        assert_eq!(v.get("max_amount").and_then(|x| x.as_i64()), Some(10));
    }
}
