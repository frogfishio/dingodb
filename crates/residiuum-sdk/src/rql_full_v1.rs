//! Full RQL-v1 compile/execute (`rql-full-v1`) — Phase 3 labor.
//!
//! Normative: [RQL_SPEC.md](../../../doc/wip/query/RQL_SPEC.md) enrich clause.
//! Application Core (`rql-app-core-v1`) remains unchanged and still **rejects**
//! `enrich` / `within` / `at rank`.
//!
//! Current slice:
//! - compile single-level `enrich … matching … [where …] expect …`
//! - execute `exactly_one` / `optional` / `many` attach via foreign scan oracle
//! - candidate `where` filters foreign docs before cardinality
//! - [`execute_rql_full`] façade on [`HeapClient`] (base page + attach)
//! - refuse `within`, `at rank`, nested enrich
//!
//! Not package accept. Not a claim that full RQL-v1 is product-ready.

use crate::app_v1::{HeapClient, Parameters, QueryPage, QueryRunOptions};
use crate::error::Error;
use crate::plan_v1::CollectionBindings;
use crate::predicate::{resolve_path, Path, Predicate, Resolve};
use crate::rql_app_core::{compile_app_core, CompiledAppCore};
use residiuum_heap::CollectionId;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Full-language compile profile (Phase 3 kickoff).
pub const RQL_FULL_PROFILE: &str = "rql-full-v1";

/// Diagnostic when an enrich cardinality cannot be satisfied.
pub const DIAG_RQL_ENRICH_CARDINALITY: &str = "rql_enrich_cardinality";

/// Diagnostic when a full-language construct is still residual.
pub const DIAG_RQL_FULL_RESIDUAL: &str = "rql_full_residual";

/// Enrichment cardinality (RQL_SPEC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrichCardinality {
    /// Exactly one matching foreign document.
    ExactlyOne,
    /// Zero or one match.
    Optional,
    /// Zero or more matches (JSON array / bag).
    Many,
}

impl EnrichCardinality {
    /// Spec spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExactlyOne => "exactly_one",
            Self::Optional => "optional",
            Self::Many => "many",
        }
    }
}

/// One compiled enrich step (single-level).
#[derive(Debug, Clone, PartialEq)]
pub struct EnrichStepV1 {
    /// Field name attached onto the root row.
    pub output: String,
    /// Bound foreign collection name (diagnostics).
    pub using_name: String,
    /// Bound foreign collection id.
    pub using_id: CollectionId,
    /// Path on the **root** document (left side of matching).
    pub left: Path,
    /// Path on the **foreign** document (right side of matching).
    pub right: Path,
    /// Optional filter evaluated against each **foreign** candidate.
    pub candidate_where: Option<Predicate>,
    /// Cardinality expectation.
    pub expect: EnrichCardinality,
}

/// Compiled full-language query (Core base + enrich pipeline).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledRqlFull {
    /// Profile label.
    pub profile: &'static str,
    /// Application Core plan for the base `from`/`where`/`project`/`order`/…
    /// (enrich stripped before Core compile).
    pub base: CompiledAppCore,
    /// Ordered enrich steps (kickoff: 0 or 1).
    pub enrich: Vec<EnrichStepV1>,
    /// Source text with enrich clauses removed (Core surface).
    pub base_source: String,
}

/// Compile full RQL-v1 kickoff source (Core + optional single enrich).
pub fn compile_rql_full(
    source: &str,
    bindings: &CollectionBindings,
) -> Result<CompiledRqlFull, Error> {
    if source.len() > crate::rql_app_core::MAX_RQL_SOURCE_BYTES {
        return Err(Error::QueryInvalid(format!(
            "RQL source exceeds {} bytes",
            crate::rql_app_core::MAX_RQL_SOURCE_BYTES
        )));
    }
    refuse_residual_constructs(source)?;

    let (base_source, enrich_raw) = split_enrich_clauses(source)?;
    let base = compile_app_core(&base_source, bindings)?;
    let mut enrich = Vec::new();
    for raw in enrich_raw {
        enrich.push(parse_enrich_step(&raw, bindings)?);
    }
    if enrich.len() > 1 {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FULL_RESIDUAL}: multiple enrich steps (nested/chained attach) not in kickoff"
        )));
    }
    Ok(CompiledRqlFull {
        profile: RQL_FULL_PROFILE,
        base,
        enrich,
        base_source,
    })
}

fn refuse_residual_constructs(source: &str) -> Result<(), Error> {
    let lower = source.to_ascii_lowercase();
    let padded = format!(" {} ", lower.replace('\t', " "));
    for (needle, label) in [
        (" within ", "within"),
        (" at rank", "at rank"),
        (" sequential", "sequential access"),
        (" direct ", "direct access"),
        (" build ", "build access"),
    ] {
        if padded.contains(needle) {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_FULL_RESIDUAL}: `{label}` outside rql-full-v1 kickoff"
            )));
        }
    }
    Ok(())
}

/// Split `enrich … expect …` clauses out of the source, returning Core text +
/// raw enrich clause bodies (without the leading `enrich` keyword).
fn split_enrich_clauses(source: &str) -> Result<(String, Vec<String>), Error> {
    // Locate top-level `enrich` tokens (not inside strings — kickoff assumes no
    // string-embedded keyword collisions for labor corpus).
    let lower = source.to_ascii_lowercase();
    let mut enrich_spans: Vec<(usize, usize)> = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = lower[search_from..].find("enrich") {
        let start = search_from + rel;
        // Word boundary.
        let before_ok = start == 0
            || !lower.as_bytes()[start - 1].is_ascii_alphanumeric();
        let after = start + "enrich".len();
        let after_ok = after >= lower.len()
            || !lower.as_bytes()[after].is_ascii_alphanumeric();
        if !(before_ok && after_ok) {
            search_from = start + 1;
            continue;
        }
        // Enrich clause ends at next pipeline/terminal keyword or EOF.
        let end = find_enrich_end(&lower, after);
        enrich_spans.push((start, end));
        search_from = end;
    }

    if enrich_spans.is_empty() {
        return Ok((source.to_string(), Vec::new()));
    }

    let mut core = String::new();
    let mut raws = Vec::new();
    let mut cursor = 0;
    for (start, end) in &enrich_spans {
        core.push_str(&source[cursor..*start]);
        let body = source[*start + "enrich".len()..*end].trim();
        raws.push(body.to_string());
        cursor = *end;
    }
    core.push_str(&source[cursor..]);
    // Collapse leftover whitespace.
    let core = core.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok((core, raws))
}

fn find_enrich_end(lower: &str, after_enrich: usize) -> usize {
    let terminals = [
        " enrich ",
        " within ",
        " project ",
        " order ",
        " limit ",
        " page ",
        " coverage ",
        " consistency ",
        " budget ",
        " at rank",
        " after ",
        " access ",
    ];
    let padded = format!(" {} ", &lower[after_enrich..]);
    let mut best = None;
    for t in terminals {
        if let Some(rel) = padded.find(t) {
            // rel is in padded (leading space); map back.
            let abs = after_enrich + rel;
            best = Some(best.map_or(abs, |b: usize| b.min(abs)));
        }
    }
    best.unwrap_or(lower.len())
}

fn parse_enrich_step(body: &str, bindings: &CollectionBindings) -> Result<EnrichStepV1, Error> {
    // body: <output> using <source> [as <alias>] matching <left> = <right> [where …] expect <card>
    let mut p = Words::new(body);
    let output = p.next_ident()?;
    p.expect("using")?;
    let using_name = p.next_ident()?;
    if p.eat("as") {
        let _alias = p.next_ident()?;
    }
    p.expect("matching")?;
    let left = Path::parse_dotted(&p.next_path()?)?;
    p.expect("=")?;
    let right = Path::parse_dotted(&p.next_path()?)?;
    let candidate_where = if p.eat("where") {
        let where_src = p.take_until_keyword("expect")?;
        if where_src.trim().is_empty() {
            return Err(Error::QueryInvalid(
                "enrich where clause is empty".into(),
            ));
        }
        // Reuse Application Core predicate parser via a synthetic Core query.
        let fake = format!("from {using_name} where {where_src}");
        let compiled = compile_app_core(&fake, bindings)?;
        Some(compiled.plan.where_pred)
    } else {
        None
    };
    p.expect("expect")?;
    let card_s = p.next_ident()?;
    let expect = match card_s.as_str() {
        "exactly_one" => EnrichCardinality::ExactlyOne,
        "optional" => EnrichCardinality::Optional,
        "many" => EnrichCardinality::Many,
        other => {
            return Err(Error::QueryInvalid(format!(
                "enrich expect must be exactly_one|optional|many, got `{other}`"
            )));
        }
    };
    if !p.is_eof() {
        return Err(Error::QueryInvalid(format!(
            "unexpected trailing enrich tokens near `{}`",
            p.rest()
        )));
    }
    let using_id = bindings
        .by_name
        .get(&using_name)
        .copied()
        .ok_or_else(|| {
            Error::QueryInvalid(format!(
                "unknown collection binding `{using_name}` for enrich using"
            ))
        })?;
    Ok(EnrichStepV1 {
        output,
        using_name,
        using_id,
        left,
        right,
        candidate_where,
        expect,
    })
}

/// Tiny whitespace tokenizer for enrich clause bodies.
struct Words<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Words<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }
    fn skip(&mut self) {
        while self
            .s
            .as_bytes()
            .get(self.i)
            .copied()
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            self.i += 1;
        }
    }
    fn is_eof(&mut self) -> bool {
        self.skip();
        self.i >= self.s.len()
    }
    fn rest(&mut self) -> &'a str {
        self.skip();
        &self.s[self.i..]
    }
    fn eat(&mut self, kw: &str) -> bool {
        self.skip();
        let r = self.rest();
        let lower = r.to_ascii_lowercase();
        if lower.starts_with(kw)
            && (r.len() == kw.len()
                || !r.as_bytes()[kw.len()].is_ascii_alphanumeric() && r.as_bytes()[kw.len()] != b'_')
        {
            self.i += kw.len();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, kw: &str) -> Result<(), Error> {
        if self.eat(kw) {
            Ok(())
        } else {
            Err(Error::QueryInvalid(format!(
                "expected `{kw}` near `{}`",
                self.rest().chars().take(24).collect::<String>()
            )))
        }
    }
    fn next_ident(&mut self) -> Result<String, Error> {
        self.skip();
        let r = self.rest();
        let mut n = 0;
        for c in r.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                n += c.len_utf8();
            } else {
                break;
            }
        }
        if n == 0 {
            return Err(Error::QueryInvalid(format!(
                "expected identifier near `{}`",
                r.chars().take(24).collect::<String>()
            )));
        }
        let id = r[..n].to_string();
        self.i += n;
        Ok(id)
    }
    fn next_path(&mut self) -> Result<String, Error> {
        let mut parts = vec![self.next_ident()?];
        loop {
            self.skip();
            if self.s.as_bytes().get(self.i) == Some(&b'.') {
                self.i += 1;
                parts.push(self.next_ident()?);
            } else {
                break;
            }
        }
        Ok(parts.join("."))
    }

    /// Consume text until a keyword at a word boundary (keyword not consumed).
    fn take_until_keyword(&mut self, kw: &str) -> Result<String, Error> {
        self.skip();
        let start = self.i;
        let rest = &self.s[start..];
        let lower = rest.to_ascii_lowercase();
        let mut search = 0usize;
        let at = loop {
            let Some(rel) = lower[search..].find(kw) else {
                return Err(Error::QueryInvalid(format!(
                    "enrich where: expected `{kw}` after predicate"
                )));
            };
            let at = search + rel;
            let before_ok =
                at == 0 || lower.as_bytes()[at - 1].is_ascii_whitespace();
            let after = at + kw.len();
            let after_ok = after >= lower.len()
                || (!lower.as_bytes()[after].is_ascii_alphanumeric()
                    && lower.as_bytes()[after] != b'_');
            if before_ok && after_ok {
                break at;
            }
            search = at + 1;
        };
        let end = start + at;
        let taken = self.s[start..end].trim().to_string();
        self.i = end;
        Ok(taken)
    }
}

/// Attach enrich fields onto already-materialised root JSON documents.
///
/// `foreign_docs` is a complete list of `(key, json)` for the using-collection
/// (independent oracle / scan). Does **not** claim index pushdown.
///
/// `expect many` attaches a JSON array of matches, ordered by foreign key.
/// Optional [`EnrichStepV1::candidate_where`] filters foreign docs first.
pub fn attach_enrich_rows(
    roots: &[(String, JsonValue)],
    foreign_docs: &[(String, JsonValue)],
    step: &EnrichStepV1,
    params: &BTreeMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, Error> {
    // Index foreign by right-path JSON key; keep keys for stable many-order.
    let mut by_right: BTreeMap<String, Vec<(String, JsonValue)>> = BTreeMap::new();
    for (fk, doc) in foreign_docs {
        if let Some(pred) = &step.candidate_where {
            if !pred.eval(doc, params)? {
                continue;
            }
        }
        if let Resolve::Present(v) = resolve_path(doc, &step.right) {
            by_right
                .entry(canonical_match_key(&v))
                .or_default()
                .push((fk.clone(), doc.clone()));
        }
    }
    for entries in by_right.values_mut() {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    let mut out = Vec::with_capacity(roots.len());
    for (key, root) in roots {
        let left_key = match resolve_path(root, &step.left) {
            Resolve::Present(v) => canonical_match_key(&v),
            Resolve::Absent => {
                match step.expect {
                    EnrichCardinality::Optional => {
                        let mut row = root.clone();
                        if let JsonValue::Object(map) = &mut row {
                            map.insert(step.output.clone(), JsonValue::Null);
                        }
                        out.push((key.clone(), row));
                        continue;
                    }
                    EnrichCardinality::Many => {
                        let mut row = root.clone();
                        if let JsonValue::Object(map) = &mut row {
                            map.insert(step.output.clone(), JsonValue::Array(vec![]));
                        }
                        out.push((key.clone(), row));
                        continue;
                    }
                    EnrichCardinality::ExactlyOne => {
                        return Err(Error::QueryInvalid(format!(
                            "{DIAG_RQL_ENRICH_CARDINALITY}: missing left match path `{}` on key `{key}`",
                            step.left.0.join(".")
                        )));
                    }
                }
            }
        };
        let candidates = by_right
            .get(&left_key)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let attached = match (step.expect, candidates.len()) {
            (EnrichCardinality::ExactlyOne, 1) => candidates[0].1.clone(),
            (EnrichCardinality::ExactlyOne, n) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_ENRICH_CARDINALITY}: exactly_one expected 1 match, got {n} (key `{key}`)"
                )));
            }
            (EnrichCardinality::Optional, 0) => JsonValue::Null,
            (EnrichCardinality::Optional, 1) => candidates[0].1.clone(),
            (EnrichCardinality::Optional, n) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_ENRICH_CARDINALITY}: optional expected ≤1 match, got {n} (key `{key}`)"
                )));
            }
            (EnrichCardinality::Many, _) => JsonValue::Array(
                candidates.iter().map(|(_, d)| d.clone()).collect(),
            ),
        };
        let mut row = root.clone();
        match &mut row {
            JsonValue::Object(map) => {
                map.insert(step.output.clone(), attached);
            }
            _ => {
                return Err(Error::QueryInvalid(
                    "enrich attach requires object root documents".into(),
                ));
            }
        }
        out.push((key.clone(), row));
    }
    Ok(out)
}

/// Result of [`execute_rql_full`] (one base page + enrich attach).
#[derive(Debug, Clone, PartialEq)]
pub struct RqlFullPage {
    /// Profile label.
    pub profile: &'static str,
    /// Enriched rows `(key, json)` after attach (same page bounds as base).
    pub rows: Vec<(String, JsonValue)>,
    /// Underlying Application Core page (pre-enrich values).
    pub base: QueryPage,
    /// Compiled enrich steps applied (0 or 1 in current slice).
    pub enrich: Vec<EnrichStepV1>,
}

/// Façade: compile full RQL, run Core base page, attach enrich via foreign scan.
///
/// Discovers collection bindings from [`HeapClient::list_collections`]. Foreign
/// collection is loaded with `list_keys`+`get` (complete scan oracle — no index
/// claim). Multipage: call again with `options.after` from `base.next`.
pub fn execute_rql_full(
    client: &mut HeapClient,
    source: &str,
    parameters: &Parameters,
    options: QueryRunOptions,
) -> Result<RqlFullPage, Error> {
    let infos = client.list_collections()?;
    let mut bindings = CollectionBindings::default();
    for info in &infos {
        bindings.bind(&info.name, info.collection_id);
    }
    let compiled = compile_rql_full(source, &bindings)?;
    let from_name = compiled.base.plan.from.source_name.clone();
    let mut base_col = client.open_collection(&from_name)?;
    let page = base_col.rql(&compiled.base_source, parameters, options)?;

    if compiled.enrich.is_empty() {
        let rows = page
            .rows
            .iter()
            .map(|r| (r.key.clone(), r.value.clone()))
            .collect();
        return Ok(RqlFullPage {
            profile: RQL_FULL_PROFILE,
            rows,
            base: page,
            enrich: compiled.enrich,
        });
    }

    let step = &compiled.enrich[0];
    let mut foreign_col = client.open_collection(&step.using_name)?;
    let mut foreign = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let batch = foreign_col.list_keys(Some(256), after.as_deref())?;
        if batch.is_empty() {
            break;
        }
        for k in &batch {
            if let Some(v) = foreign_col.get(k)? {
                foreign.push((k.clone(), v));
            }
        }
        after = batch.last().cloned();
        if batch.len() < 256 {
            break;
        }
    }

    let roots: Vec<(String, JsonValue)> = page
        .rows
        .iter()
        .map(|r| (r.key.clone(), r.value.clone()))
        .collect();
    let rows = attach_enrich_rows(&roots, &foreign, step, &parameters.values)?;
    Ok(RqlFullPage {
        profile: RQL_FULL_PROFILE,
        rows,
        base: page,
        enrich: compiled.enrich,
    })
}

fn canonical_match_key(v: &JsonValue) -> String {
    // Stable enough for equality matching of scalars in kickoff tests.
    serde_json::to_string(v).unwrap_or_else(|_| format!("{v:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rql_app_core::DIAG_RQL_FEATURE_UNAVAILABLE;
    use residiuum_heap::CollectionId;
    use std::str::FromStr;

    fn bindings() -> CollectionBindings {
        let mut b = CollectionBindings::default();
        b.bind(
            "orders",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a1").unwrap(),
        );
        b.bind(
            "customers",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a2").unwrap(),
        );
        b
    }

    #[test]
    fn core_still_rejects_enrich() {
        let err = compile_app_core(
            "from orders enrich customer using customers matching customer_id = id expect exactly_one",
            &bindings(),
        )
        .unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_FEATURE_UNAVAILABLE));
    }

    #[test]
    fn compile_single_enrich() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id expect exactly_one
               page size 10"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.profile, RQL_FULL_PROFILE);
        assert_eq!(c.enrich.len(), 1);
        assert_eq!(c.enrich[0].output, "customer");
        assert_eq!(c.enrich[0].expect, EnrichCardinality::ExactlyOne);
        assert!(c.base_source.contains("from orders"));
        assert!(!c.base_source.contains("enrich"));
        assert_eq!(c.base.plan.page_size, 10);
    }

    #[test]
    fn refuse_within_still() {
        assert!(compile_rql_full(
            "from orders within items { enrich x using y matching a = b expect optional }",
            &bindings()
        )
        .is_err());
    }

    #[test]
    fn compile_and_attach_many() {
        let mut b = bindings();
        b.bind(
            "line_items",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a3").unwrap(),
        );
        let c = compile_rql_full(
            "from orders enrich items using line_items matching order_id = order_id expect many",
            &b,
        )
        .unwrap();
        assert_eq!(c.enrich[0].expect, EnrichCardinality::Many);

        let step = &c.enrich[0];
        let roots = vec![
            ("o1".into(), serde_json::json!({"order_id": "o1"})),
            ("o2".into(), serde_json::json!({"order_id": "o2"})),
        ];
        let foreign = vec![
            ("l2".into(), serde_json::json!({"order_id": "o1", "sku": "B"})),
            ("l1".into(), serde_json::json!({"order_id": "o1", "sku": "A"})),
            ("l3".into(), serde_json::json!({"order_id": "o2", "sku": "C"})),
        ];
        let out = attach_enrich_rows(&roots, &foreign, step, &BTreeMap::new()).unwrap();
        let bag = out[0].1["items"].as_array().unwrap();
        assert_eq!(bag.len(), 2);
        // Stable by foreign key: l1 then l2
        assert_eq!(bag[0]["sku"], "A");
        assert_eq!(bag[1]["sku"], "B");
        assert_eq!(out[1].1["items"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn attach_exactly_one_and_optional() {
        let step = EnrichStepV1 {
            output: "customer".into(),
            using_name: "customers".into(),
            using_id: CollectionId::from_str("00000000-0000-4000-8000-0000000000a2").unwrap(),
            left: Path::parse_dotted("customer_id").unwrap(),
            right: Path::parse_dotted("id").unwrap(),
            candidate_where: None,
            expect: EnrichCardinality::ExactlyOne,
        };
        let roots = vec![
            (
                "o1".into(),
                serde_json::json!({"customer_id": "c1", "n": 1}),
            ),
        ];
        let foreign = vec![
            ("c1".into(), serde_json::json!({"id": "c1", "name": "Ada"})),
            ("c2".into(), serde_json::json!({"id": "c2", "name": "Bob"})),
        ];
        let out = attach_enrich_rows(&roots, &foreign, &step, &BTreeMap::new()).unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");

        let mut opt = step.clone();
        opt.expect = EnrichCardinality::Optional;
        let roots2 = vec![("o2".into(), serde_json::json!({"customer_id": "missing"}))];
        let out2 = attach_enrich_rows(&roots2, &foreign, &opt, &BTreeMap::new()).unwrap();
        assert!(out2[0].1["customer"].is_null());
    }

    #[test]
    fn compile_and_attach_candidate_where() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id
               where active = true
               expect exactly_one"#,
            &bindings(),
        )
        .unwrap();
        assert!(c.enrich[0].candidate_where.is_some());

        let roots = vec![("o1".into(), serde_json::json!({"customer_id": "c1"}))];
        let foreign = vec![
            (
                "c1a".into(),
                serde_json::json!({"id": "c1", "active": false, "name": "old"}),
            ),
            (
                "c1b".into(),
                serde_json::json!({"id": "c1", "active": true, "name": "Ada"}),
            ),
        ];
        let out =
            attach_enrich_rows(&roots, &foreign, &c.enrich[0], &BTreeMap::new()).unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");
    }
}
