//! Full RQL-v1 compile/execute (`rql-full-v1`) — Phase 3 labor.
//!
//! Normative: [RQL_SPEC.md](../../../doc/wip/query/RQL_SPEC.md) enrich / within.
//! Application Core (`rql-app-core-v1`) remains unchanged and still **rejects**
//! `enrich` / `within` / `at rank`.
//!
//! Current slice:
//! - compile ordered root `enrich …` (chained / multi)
//! - compile one `within path [as alias] { enrich …; enrich … }` (multi nested enrich)
//! - execute `exactly_one` / `optional` / `many` attach via foreign scan oracle
//! - candidate `where` filters foreign docs before cardinality
//! - [`execute_rql_full`] façade on [`HeapClient`] (base page + attach pipeline)
//! - refuse `at rank`, nested `within`, multiple top-level `within`, enrich after within
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

/// Diagnostic when `within` path is absent, Null, or not a sequence/bag.
pub const DIAG_RQL_WITHIN_TYPE: &str = "rql_within_type";

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
    /// Field name attached onto the current row (root or within element).
    pub output: String,
    /// Bound foreign collection name (diagnostics).
    pub using_name: String,
    /// Bound foreign collection id.
    pub using_id: CollectionId,
    /// Path on the **current** document (left side of matching).
    pub left: Path,
    /// Path on the **foreign** document (right side of matching).
    pub right: Path,
    /// Optional filter evaluated against each **foreign** candidate.
    pub candidate_where: Option<Predicate>,
    /// Cardinality expectation.
    pub expect: EnrichCardinality,
}

/// One compiled `within` step (no nested `within`; multi nested enrich ok).
#[derive(Debug, Clone, PartialEq)]
pub struct WithinStepV1 {
    /// Carrier path on the root row (must resolve to a JSON array).
    pub carrier: Path,
    /// Optional element alias (`as item`); stripped from nested left paths.
    pub element_alias: Option<String>,
    /// Nested enrich steps applied in order to each carrier element.
    pub enrich: Vec<EnrichStepV1>,
}

/// Compiled full-language query (Core base + enrich / within pipeline).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledRqlFull {
    /// Profile label.
    pub profile: &'static str,
    /// Application Core plan for the base `from`/`where`/`project`/`order`/…
    /// (enrich/within stripped before Core compile).
    pub base: CompiledAppCore,
    /// Ordered root enrich steps (before optional within).
    pub enrich: Vec<EnrichStepV1>,
    /// Optional single `within` after root enrich.
    pub within: Option<WithinStepV1>,
    /// Source text with enrich/within clauses removed (Core surface).
    pub base_source: String,
}

enum RawPipelineStep {
    Enrich(String),
    Within(String),
}

/// Compile full RQL-v1 source (Core + optional enrich + optional within).
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

    let (base_source, raw_steps) = split_full_clauses(source)?;
    let base = compile_app_core(&base_source, bindings)?;
    let mut enrich = Vec::new();
    let mut within = None;
    for raw in raw_steps {
        match raw {
            RawPipelineStep::Enrich(body) => {
                if within.is_some() {
                    return Err(Error::QueryInvalid(format!(
                        "{DIAG_RQL_FULL_RESIDUAL}: enrich after within not in this slice"
                    )));
                }
                enrich.push(parse_enrich_step(&body, bindings, None)?);
            }
            RawPipelineStep::Within(body) => {
                if within.is_some() {
                    return Err(Error::QueryInvalid(format!(
                        "{DIAG_RQL_FULL_RESIDUAL}: multiple within steps not in this slice"
                    )));
                }
                within = Some(parse_within_step(&body, bindings)?);
            }
        }
    }
    Ok(CompiledRqlFull {
        profile: RQL_FULL_PROFILE,
        base,
        enrich,
        within,
        base_source,
    })
}

fn refuse_residual_constructs(source: &str) -> Result<(), Error> {
    let lower = source.to_ascii_lowercase();
    let padded = format!(" {} ", lower.replace('\t', " "));
    for (needle, label) in [
        (" at rank", "at rank"),
        (" sequential", "sequential access"),
        (" direct ", "direct access"),
        (" build ", "build access"),
    ] {
        if padded.contains(needle) {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_FULL_RESIDUAL}: `{label}` outside rql-full-v1 current slice"
            )));
        }
    }
    Ok(())
}

/// Split top-level `enrich` / `within` clauses; return Core text + ordered steps.
fn split_full_clauses(source: &str) -> Result<(String, Vec<RawPipelineStep>), Error> {
    let lower = source.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut spans: Vec<(usize, usize, bool)> = Vec::new();
    let mut i = 0usize;
    let mut depth = 0usize;
    while i < lower.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
                continue;
            }
            _ => {}
        }
        if depth == 0 {
            if matches_kw(&lower, i, "enrich") {
                let after = i + "enrich".len();
                let end = find_enrich_end(&lower, after);
                spans.push((i, end, false));
                i = end;
                continue;
            }
            if matches_kw(&lower, i, "within") {
                let after = i + "within".len();
                let end = find_within_end(&lower, after)?;
                spans.push((i, end, true));
                i = end;
                continue;
            }
        }
        i += 1;
    }

    if spans.is_empty() {
        return Ok((source.to_string(), Vec::new()));
    }

    let mut core = String::new();
    let mut steps = Vec::new();
    let mut cursor = 0;
    for (start, end, is_within) in spans {
        core.push_str(&source[cursor..start]);
        if is_within {
            let body = source[start + "within".len()..end].trim();
            steps.push(RawPipelineStep::Within(body.to_string()));
        } else {
            let body = source[start + "enrich".len()..end].trim();
            steps.push(RawPipelineStep::Enrich(body.to_string()));
        }
        cursor = end;
    }
    core.push_str(&source[cursor..]);
    let core = core.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok((core, steps))
}

fn matches_kw(lower: &str, start: usize, kw: &str) -> bool {
    let bytes = lower.as_bytes();
    if start + kw.len() > lower.len() {
        return false;
    }
    if &lower[start..start + kw.len()] != kw {
        return false;
    }
    let before_ok = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
    let after = start + kw.len();
    let after_ok = after >= lower.len() || !bytes[after].is_ascii_alphanumeric();
    before_ok && after_ok
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
            let abs = after_enrich + rel;
            best = Some(best.map_or(abs, |b: usize| b.min(abs)));
        }
    }
    best.unwrap_or(lower.len())
}

fn find_within_end(lower: &str, after_within: usize) -> Result<usize, Error> {
    let Some(brace_rel) = lower[after_within..].find('{') else {
        return Err(Error::QueryInvalid(
            "within: expected `{` after path".into(),
        ));
    };
    let open = after_within + brace_rel;
    let bytes = lower.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < lower.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(Error::QueryInvalid("within: unclosed `{`".into()))
}

fn parse_within_step(body: &str, bindings: &CollectionBindings) -> Result<WithinStepV1, Error> {
    // body: <path> [as <alias>] { <nested steps> }
    let mut p = Words::new(body);
    let carrier = Path::parse_dotted(&p.next_path()?)?;
    let element_alias = if p.eat("as") {
        Some(p.next_ident()?)
    } else {
        None
    };
    if !p.eat_char(b'{') {
        return Err(Error::QueryInvalid(
            "within: expected `{` after path".into(),
        ));
    }
    let inner = p.take_brace_inner()?;
    if !p.is_eof() {
        return Err(Error::QueryInvalid(format!(
            "unexpected trailing within tokens near `{}`",
            p.rest()
        )));
    }

    let inner_lower = format!(" {} ", inner.to_ascii_lowercase());
    if inner_lower.contains(" within ") {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FULL_RESIDUAL}: nested within not in this slice"
        )));
    }

    // Reuse top-level enrich splitter on a synthetic prefix-free body.
    let (_ignored, nested) = split_full_clauses(&inner)?;
    let mut enrich_bodies = Vec::new();
    for step in nested {
        match step {
            RawPipelineStep::Enrich(b) => enrich_bodies.push(b),
            RawPipelineStep::Within(_) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_FULL_RESIDUAL}: nested within not in this slice"
                )));
            }
        }
    }
    if enrich_bodies.is_empty() {
        return Err(Error::QueryInvalid(
            "within block requires at least one enrich".into(),
        ));
    }
    let mut enrich = Vec::with_capacity(enrich_bodies.len());
    for body in enrich_bodies {
        enrich.push(parse_enrich_step(
            &body,
            bindings,
            element_alias.as_deref(),
        )?);
    }
    Ok(WithinStepV1 {
        carrier,
        element_alias,
        enrich,
    })
}

fn parse_enrich_step(
    body: &str,
    bindings: &CollectionBindings,
    element_alias: Option<&str>,
) -> Result<EnrichStepV1, Error> {
    // body: <output> using <source> [as <alias>] matching <left> = <right> [where …] expect <card>
    let mut p = Words::new(body);
    let output = p.next_ident()?;
    p.expect("using")?;
    let using_name = p.next_ident()?;
    let foreign_alias = if p.eat("as") {
        Some(p.next_ident()?)
    } else {
        None
    };
    p.expect("matching")?;
    let left = strip_alias_prefix(Path::parse_dotted(&p.next_path()?)?, element_alias)?;
    p.expect("=")?;
    let right = strip_alias_prefix(Path::parse_dotted(&p.next_path()?)?, foreign_alias.as_deref())?;
    let candidate_where = if p.eat("where") {
        let where_src = p.take_until_keyword("expect")?;
        if where_src.trim().is_empty() {
            return Err(Error::QueryInvalid(
                "enrich where clause is empty".into(),
            ));
        }
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

fn strip_alias_prefix(path: Path, alias: Option<&str>) -> Result<Path, Error> {
    let Some(a) = alias else {
        return Ok(path);
    };
    if path.0.first().map(String::as_str) != Some(a) {
        return Ok(path);
    }
    if path.0.len() == 1 {
        return Err(Error::QueryInvalid(format!(
            "path `{a}` is only an alias; need a field under it"
        )));
    }
    Path::from_segments(path.0.into_iter().skip(1))
}

/// Tiny whitespace tokenizer for enrich / within clause bodies.
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
    fn eat_char(&mut self, c: u8) -> bool {
        self.skip();
        if self.s.as_bytes().get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
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

    /// After consuming `{`, take inner text until the matching `}` (consumed).
    fn take_brace_inner(&mut self) -> Result<String, Error> {
        let start = self.i;
        let bytes = self.s.as_bytes();
        let mut depth = 1usize;
        while self.i < self.s.len() {
            match bytes[self.i] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        let inner = self.s[start..self.i].trim().to_string();
                        self.i += 1;
                        return Ok(inner);
                    }
                }
                _ => {}
            }
            self.i += 1;
        }
        Err(Error::QueryInvalid("within: unclosed `{`".into()))
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

/// Apply nested enrich steps to each element of a carrier array on root rows.
///
/// `foreign_by_using` maps collection name → complete foreign docs for that
/// using-collection. Absent / Null / non-array carriers fail with
/// [`DIAG_RQL_WITHIN_TYPE`] (never treated as empty bags).
pub fn attach_within_rows(
    roots: &[(String, JsonValue)],
    foreign_by_using: &BTreeMap<String, Vec<(String, JsonValue)>>,
    step: &WithinStepV1,
    params: &BTreeMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, Error> {
    let mut out = Vec::with_capacity(roots.len());
    for (key, root) in roots {
        let carrier = match resolve_path(root, &step.carrier) {
            Resolve::Present(JsonValue::Array(arr)) => arr,
            Resolve::Present(other) => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_WITHIN_TYPE}: path `{}` is {} on key `{key}` (need sequence/bag)",
                    step.carrier.dotted(),
                    json_type_name(&other)
                )));
            }
            Resolve::Absent => {
                return Err(Error::QueryInvalid(format!(
                    "{DIAG_RQL_WITHIN_TYPE}: path `{}` absent on key `{key}`",
                    step.carrier.dotted()
                )));
            }
        };
        let mut elements: Vec<(String, JsonValue)> = carrier
            .iter()
            .enumerate()
            .map(|(i, el)| (format!("{key}#{i}"), el.clone()))
            .collect();
        for enrich in &step.enrich {
            let foreign = foreign_by_using.get(&enrich.using_name).ok_or_else(|| {
                Error::QueryInvalid(format!(
                    "within attach missing foreign docs for `{}`",
                    enrich.using_name
                ))
            })?;
            elements = attach_enrich_rows(&elements, foreign, enrich, params)?;
        }
        let new_arr: Vec<JsonValue> = elements.into_iter().map(|(_, v)| v).collect();
        let mut row = root.clone();
        set_at_path(&mut row, &step.carrier, JsonValue::Array(new_arr))?;
        out.push((key.clone(), row));
    }
    Ok(out)
}

fn json_type_name(v: &JsonValue) -> &'static str {
    match v {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}

fn set_at_path(doc: &mut JsonValue, path: &Path, value: JsonValue) -> Result<(), Error> {
    if path.0.is_empty() {
        return Err(Error::QueryInvalid("empty path for within replace".into()));
    }
    let mut cur = doc;
    for (i, seg) in path.0.iter().enumerate() {
        if i + 1 == path.0.len() {
            match cur {
                JsonValue::Object(map) => {
                    map.insert(seg.clone(), value);
                    return Ok(());
                }
                _ => {
                    return Err(Error::QueryInvalid(format!(
                        "within replace requires object at parent of `{}`",
                        path.dotted()
                    )));
                }
            }
        }
        match cur {
            JsonValue::Object(map) => {
                cur = map.get_mut(seg).ok_or_else(|| {
                    Error::QueryInvalid(format!(
                        "within replace missing segment `{seg}` on `{}`",
                        path.dotted()
                    ))
                })?;
            }
            _ => {
                return Err(Error::QueryInvalid(format!(
                    "within replace requires object along `{}`",
                    path.dotted()
                )));
            }
        }
    }
    Err(Error::QueryInvalid("within replace unreachable".into()))
}

/// Result of [`execute_rql_full`] (one base page + enrich/within attach).
#[derive(Debug, Clone, PartialEq)]
pub struct RqlFullPage {
    /// Profile label.
    pub profile: &'static str,
    /// Enriched rows `(key, json)` after attach (same page bounds as base).
    pub rows: Vec<(String, JsonValue)>,
    /// Underlying Application Core page (pre-enrich values).
    pub base: QueryPage,
    /// Compiled root enrich steps applied.
    pub enrich: Vec<EnrichStepV1>,
    /// Compiled within step applied (if any).
    pub within: Option<WithinStepV1>,
}

/// Façade: compile full RQL, run Core base page, attach enrich/within via scan.
///
/// Discovers collection bindings from [`HeapClient::list_collections`]. Foreign
/// collections are loaded with `list_keys`+`get` (complete scan oracle — no index
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

    let mut rows: Vec<(String, JsonValue)> = page
        .rows
        .iter()
        .map(|r| (r.key.clone(), r.value.clone()))
        .collect();

    for step in &compiled.enrich {
        let foreign = load_collection_docs(client, &step.using_name)?;
        rows = attach_enrich_rows(&rows, &foreign, step, &parameters.values)?;
    }
    if let Some(w) = &compiled.within {
        let mut foreign_by_using: BTreeMap<String, Vec<(String, JsonValue)>> = BTreeMap::new();
        for e in &w.enrich {
            if !foreign_by_using.contains_key(&e.using_name) {
                let docs = load_collection_docs(client, &e.using_name)?;
                foreign_by_using.insert(e.using_name.clone(), docs);
            }
        }
        rows = attach_within_rows(&rows, &foreign_by_using, w, &parameters.values)?;
    }

    Ok(RqlFullPage {
        profile: RQL_FULL_PROFILE,
        rows,
        base: page,
        enrich: compiled.enrich,
        within: compiled.within,
    })
}

fn load_collection_docs(
    client: &mut HeapClient,
    name: &str,
) -> Result<Vec<(String, JsonValue)>, Error> {
    let mut col = client.open_collection(name)?;
    let mut foreign = Vec::new();
    let mut after: Option<String> = None;
    loop {
        let batch = col.list_keys(Some(256), after.as_deref())?;
        if batch.is_empty() {
            break;
        }
        for k in &batch {
            if let Some(v) = col.get(k)? {
                foreign.push((k.clone(), v));
            }
        }
        after = batch.last().cloned();
        if batch.len() < 256 {
            break;
        }
    }
    Ok(foreign)
}

fn canonical_match_key(v: &JsonValue) -> String {
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
        b.bind(
            "line_items",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a3").unwrap(),
        );
        b.bind(
            "products",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a4").unwrap(),
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
        assert!(c.within.is_none());
        assert!(c.base_source.contains("from orders"));
        assert!(!c.base_source.contains("enrich"));
        assert_eq!(c.base.plan.page_size, 10);
    }

    #[test]
    fn compile_and_attach_within() {
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
               }"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.enrich.len(), 1);
        let w = c.within.as_ref().unwrap();
        assert_eq!(w.carrier.dotted(), "items");
        assert_eq!(w.element_alias.as_deref(), Some("item"));
        assert_eq!(w.enrich.len(), 1);
        assert_eq!(w.enrich[0].output, "product");
        assert_eq!(w.enrich[0].left.dotted(), "product_id");
        assert_eq!(w.enrich[0].right.dotted(), "id");

        let roots = vec![(
            "o1".into(),
            serde_json::json!({"order_id": "o1"}),
        )];
        let lines = vec![
            (
                "l1".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p1", "sku": "A"}),
            ),
            (
                "l2".into(),
                serde_json::json!({"order_id": "o1", "product_id": "p2", "sku": "B"}),
            ),
        ];
        let products = vec![
            ("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"})),
            ("p2".into(), serde_json::json!({"id": "p2", "name": "Gadget"})),
        ];
        let after_many =
            attach_enrich_rows(&roots, &lines, &c.enrich[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let bag = out[0].1["items"].as_array().unwrap();
        assert_eq!(bag.len(), 2);
        assert_eq!(bag[0]["product"]["name"], "Widget");
        assert_eq!(bag[1]["product"]["name"], "Gadget");
    }

    #[test]
    fn compile_and_attach_multi_root_enrich() {
        let c = compile_rql_full(
            r#"from orders
               enrich customer using customers matching customer_id = id expect exactly_one
               enrich items using line_items matching order_id = order_id expect many"#,
            &bindings(),
        )
        .unwrap();
        assert_eq!(c.enrich.len(), 2);
        assert_eq!(c.enrich[0].output, "customer");
        assert_eq!(c.enrich[1].output, "items");

        let roots = vec![(
            "o1".into(),
            serde_json::json!({"order_id": "o1", "customer_id": "c1"}),
        )];
        let customers = vec![("c1".into(), serde_json::json!({"id": "c1", "name": "Ada"}))];
        let lines = vec![
            ("l1".into(), serde_json::json!({"order_id": "o1", "sku": "A"})),
            ("l2".into(), serde_json::json!({"order_id": "o1", "sku": "B"})),
        ];
        let mid =
            attach_enrich_rows(&roots, &customers, &c.enrich[0], &BTreeMap::new()).unwrap();
        let out = attach_enrich_rows(&mid, &lines, &c.enrich[1], &BTreeMap::new()).unwrap();
        assert_eq!(out[0].1["customer"]["name"], "Ada");
        assert_eq!(out[0].1["items"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn compile_and_attach_multi_within_enrich() {
        let mut b = bindings();
        b.bind(
            "warehouses",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000a5").unwrap(),
        );
        let c = compile_rql_full(
            r#"from orders
               enrich items using line_items matching order_id = order_id expect many
               within items as item {
                 enrich product using products as candidate
                   matching item.product_id = candidate.id
                   expect exactly_one
                 enrich warehouse using warehouses as wh
                   matching item.warehouse_id = wh.id
                   expect optional
               }"#,
            &b,
        )
        .unwrap();
        let w = c.within.as_ref().unwrap();
        assert_eq!(w.enrich.len(), 2);
        assert_eq!(w.enrich[0].output, "product");
        assert_eq!(w.enrich[1].output, "warehouse");

        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let lines = vec![(
            "l1".into(),
            serde_json::json!({
                "order_id": "o1",
                "product_id": "p1",
                "warehouse_id": "w1"
            }),
        )];
        let products = vec![("p1".into(), serde_json::json!({"id": "p1", "name": "Widget"}))];
        let warehouses = vec![("w1".into(), serde_json::json!({"id": "w1", "city": "Oslo"}))];
        let after_many =
            attach_enrich_rows(&roots, &lines, &c.enrich[0], &BTreeMap::new()).unwrap();
        let mut foreign = BTreeMap::new();
        foreign.insert("products".into(), products);
        foreign.insert("warehouses".into(), warehouses);
        let out =
            attach_within_rows(&after_many, &foreign, w, &BTreeMap::new()).unwrap();
        let item = &out[0].1["items"].as_array().unwrap()[0];
        assert_eq!(item["product"]["name"], "Widget");
        assert_eq!(item["warehouse"]["city"], "Oslo");
    }

    #[test]
    fn within_type_error_on_absent() {
        let step = WithinStepV1 {
            carrier: Path::parse_dotted("items").unwrap(),
            element_alias: None,
            enrich: vec![EnrichStepV1 {
                output: "product".into(),
                using_name: "products".into(),
                using_id: CollectionId::from_str("00000000-0000-4000-8000-0000000000a4")
                    .unwrap(),
                left: Path::parse_dotted("product_id").unwrap(),
                right: Path::parse_dotted("id").unwrap(),
                candidate_where: None,
                expect: EnrichCardinality::ExactlyOne,
            }],
        };
        let roots = vec![("o1".into(), serde_json::json!({"order_id": "o1"}))];
        let foreign = BTreeMap::new();
        let err = attach_within_rows(&roots, &foreign, &step, &BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_WITHIN_TYPE));
    }

    #[test]
    fn refuse_nested_within() {
        let err = compile_rql_full(
            r#"from orders within items {
                 within nested { enrich x using products matching a = b expect optional }
               }"#,
            &bindings(),
        )
        .unwrap_err();
        assert!(err.to_string().contains(DIAG_RQL_FULL_RESIDUAL));
    }

    #[test]
    fn compile_and_attach_many() {
        let c = compile_rql_full(
            "from orders enrich items using line_items matching order_id = order_id expect many",
            &bindings(),
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
