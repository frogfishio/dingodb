//! RQL Application Core compiler (`rql-app-core-v1`) — APP-5 first cut.
//!
//! Normative:
//! - CORE plan §9 / §14 APP-5
//! - [RQL_SPEC.md](../../../doc/wip/query/RQL_SPEC.md) (subset; no alternative syntax)
//!
//! Compiles Application Core source text into the same [`RqlPlanV1`] as
//! [`PlanBuilder`]. Full RQL v1 is **not** claimed; unsupported constructs
//! fail with [`Error::QueryInvalid`] and diagnostic `rql_feature_unavailable`
//! when appropriate.
//!
//! Residual (later APP-5 cuts): budget-clause source surface, after-clause
//! (APP-6), ranked access, full conformance corpus fixtures, parser fuzz.

use crate::app_v1::{ConsistencyMode, CoveragePolicy};
use crate::error::Error;
use crate::plan_v1::{
    CollectionBindings, NullsOrder, OrderDir, PlanBuilder, RqlPlanV1, DEFAULT_PAGE_SIZE,
    MAX_PAGE_SIZE,
};
use crate::predicate::{param, CompareOp, Operand, Path, Predicate};
use serde_json::{Number, Value as JsonValue};

/// Conformance profile id (same as [`crate::app_v1::RQL_APP_CORE_PROFILE`]).
pub const APP_CORE_PROFILE: &str = "rql-app-core-v1";

/// Hard ceiling on UTF-8 RQL source (CORE plan §9.2).
pub const MAX_RQL_SOURCE_BYTES: usize = 1_048_576;

/// Diagnostic tag for unsupported Application Core features (error map).
pub const DIAG_RQL_FEATURE_UNAVAILABLE: &str = "rql_feature_unavailable";

/// Result of compiling Application Core source.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledAppCore {
    /// Validated plan.
    pub plan: RqlPlanV1,
    /// True when source began with `explain`.
    pub explain: bool,
    /// Profile string for advertising.
    pub profile: &'static str,
}

/// Compile Application Core RQL into [`RqlPlanV1`].
///
/// `bindings` must resolve the `from` source name to an immutable collection id.
pub fn compile_app_core(source: &str, bindings: &CollectionBindings) -> Result<CompiledAppCore, Error> {
    if source.len() > MAX_RQL_SOURCE_BYTES {
        return Err(Error::QueryInvalid(format!(
            "RQL source exceeds {MAX_RQL_SOURCE_BYTES} bytes"
        )));
    }
    reject_excluded_features(source)?;

    let mut p = Parser::new(source);
    p.skip_ws();
    let explain = if p.eat_keyword("explain") {
        p.skip_ws();
        true
    } else {
        false
    };

    p.expect_keyword("from")?;
    p.skip_ws();
    let source_name = p.parse_ident_or_string()?;
    p.skip_ws();
    if p.eat_keyword("as") {
        p.skip_ws();
        let _alias = p.parse_ident_or_string()?;
        p.skip_ws();
        // Alias is readability-only for CollectionClient::rql; plan binds the
        // handle's collection via `source_name` resolution below.
    }

    let mut builder = PlanBuilder::from_source(source_name);
    // CORE §9 EBNF allows repeated `where` clauses — they AND together.
    let mut where_parts: Vec<Predicate> = Vec::new();

    loop {
        p.skip_ws();
        if p.is_eof() {
            break;
        }
        if p.eat_keyword("where") {
            p.skip_ws();
            where_parts.push(p.parse_or()?);
            continue;
        }
        if p.eat_keyword("project") {
            p.skip_ws();
            let fields = p.parse_project_list()?;
            builder = builder.project(fields)?;
            continue;
        }
        if p.eat_keyword("order") {
            p.skip_ws();
            p.expect_keyword("by")?;
            p.skip_ws();
            loop {
                let path = p.parse_path_dotted()?;
                p.skip_ws();
                let dir = if p.eat_keyword("desc") {
                    OrderDir::Desc
                } else {
                    let _ = p.eat_keyword("asc");
                    OrderDir::Asc
                };
                p.skip_ws();
                let nulls = if p.eat_keyword("nulls") {
                    p.skip_ws();
                    if p.eat_keyword("first") {
                        NullsOrder::First
                    } else if p.eat_keyword("last") {
                        NullsOrder::Last
                    } else {
                        return Err(Error::QueryInvalid(
                            "order by … nulls requires first|last".into(),
                        ));
                    }
                } else {
                    NullsOrder::Last
                };
                p.skip_ws();
                builder = builder.order_by_nulls(&path, dir, nulls)?;
                p.skip_ws();
                if p.eat_char(',') {
                    p.skip_ws();
                    continue;
                }
                break;
            }
            continue;
        }
        if p.eat_keyword("limit") {
            p.skip_ws();
            let n = p.parse_u64()?;
            builder = builder.limit(n);
            continue;
        }
        if p.eat_keyword("page") {
            p.skip_ws();
            p.expect_keyword("size")?;
            p.skip_ws();
            let n = p.parse_u64()?;
            if n == 0 || n > u64::from(MAX_PAGE_SIZE) {
                return Err(Error::QueryInvalid(format!(
                    "page size must be 1..={MAX_PAGE_SIZE}"
                )));
            }
            builder = builder.page_size(n as u32)?;
            continue;
        }
        if p.eat_keyword("coverage") {
            p.skip_ws();
            if p.eat_keyword("complete") {
                builder = builder.coverage(CoveragePolicy::Complete);
            } else if p.eat_keyword("incomplete") || p.eat_keyword("allow_incomplete") {
                // "incomplete allowed" style — accept incomplete / incomplete_allowed tokens
                let _ = p.eat_keyword("allowed");
                builder = builder.coverage(CoveragePolicy::IncompleteAllowed);
            } else {
                return Err(Error::QueryInvalid(
                    "coverage expects complete|incomplete".into(),
                ));
            }
            continue;
        }
        if p.eat_keyword("consistency") {
            p.skip_ws();
            if p.eat_keyword("available") {
                builder = builder.consistency(ConsistencyMode::Available);
            } else if p.eat_keyword("current") {
                builder = builder.consistency(ConsistencyMode::Current);
            } else {
                return Err(Error::QueryInvalid(
                    "consistency expects available|current".into(),
                ));
            }
            continue;
        }
        if p.eat_keyword("budget") {
            return feature_unavailable("budget clause (surface residual; set via run options)");
        }
        if p.eat_keyword("after") {
            return feature_unavailable("after / continuation clause (APP-6)");
        }
        return Err(Error::QueryInvalid(format!(
            "unexpected token near `{}`",
            p.snippet()
        )));
    }

    // Default page size when omitted (CORE §9.2) — PlanBuilder fills DEFAULT_PAGE_SIZE.
    let _ = DEFAULT_PAGE_SIZE;

    if !where_parts.is_empty() {
        let combined = if where_parts.len() == 1 {
            where_parts.pop().expect("len==1")
        } else {
            Predicate::And { args: where_parts }
        };
        builder = builder.where_(combined);
    }

    let plan = builder.compile(bindings)?;
    Ok(CompiledAppCore {
        plan,
        explain,
        profile: APP_CORE_PROFILE,
    })
}

fn feature_unavailable(what: &str) -> Result<CompiledAppCore, Error> {
    Err(Error::QueryInvalid(format!(
        "{DIAG_RQL_FEATURE_UNAVAILABLE}: {what}"
    )))
}

fn reject_excluded_features(source: &str) -> Result<(), Error> {
    let lower = source.to_ascii_lowercase();
    // Word-ish checks to avoid rejecting identifiers that merely contain letters.
    let banned = [
        (" enrich ", "enrich"),
        ("\nenrich ", "enrich"),
        (" within ", "within"),
        (" at rank", "at rank"),
        (" sequential", "sequential access policy"),
        (" direct ", "direct access policy"),
        (" build ", "build access policy"),
    ];
    let padded = format!(" {} ", lower.replace('\t', " "));
    for (needle, label) in banned {
        if padded.contains(needle) || lower.starts_with(needle.trim()) {
            return Err(Error::QueryInvalid(format!(
                "{DIAG_RQL_FEATURE_UNAVAILABLE}: `{label}` is outside rql-app-core-v1"
            )));
        }
    }
    // bare "enrich" as first token after explain
    if lower.split_whitespace().any(|t| t == "enrich" || t == "within") {
        return Err(Error::QueryInvalid(format!(
            "{DIAG_RQL_FEATURE_UNAVAILABLE}: enrich/within outside rql-app-core-v1"
        )));
    }
    Ok(())
}

// --- lexer/parser ------------------------------------------------------------

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    fn is_eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }

    fn snippet(&self) -> String {
        let r = self.rest();
        r.chars().take(24).collect()
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let mut it = self.rest().char_indices();
        let (_, c) = it.next()?;
        let next = it.next().map(|(o, _)| o).unwrap_or(self.rest().len());
        self.i += next;
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.bump();
            } else if c == '-' && self.rest().starts_with("--") {
                // line comment
                while let Some(c) = self.bump() {
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn eat_char(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_keyword(&mut self, kw: &str) -> bool {
        let r = self.rest();
        if r.len() >= kw.len()
            && r[..kw.len()].eq_ignore_ascii_case(kw)
            && r[kw.len()..]
                .chars()
                .next()
                .map(|c| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(true)
        {
            self.i += kw.len();
            true
        } else {
            false
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), Error> {
        if self.eat_keyword(kw) {
            Ok(())
        } else {
            Err(Error::QueryInvalid(format!(
                "expected `{kw}` near `{}`",
                self.snippet()
            )))
        }
    }

    fn parse_ident_or_string(&mut self) -> Result<String, Error> {
        self.skip_ws();
        if self.peek() == Some('"') {
            return self.parse_string();
        }
        let start = self.i;
        let mut first = true;
        while let Some(c) = self.peek() {
            if first {
                if !(c.is_ascii_alphabetic() || c == '_' || c == '$') {
                    break;
                }
                first = false;
                self.bump();
            } else if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                self.bump();
            } else {
                break;
            }
        }
        if self.i == start {
            return Err(Error::QueryInvalid(format!(
                "expected identifier near `{}`",
                self.snippet()
            )));
        }
        Ok(self.s[start..self.i].to_string())
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        if !self.eat_char('"') {
            return Err(Error::QueryInvalid("expected string".into()));
        }
        let mut out = String::new();
        while let Some(c) = self.bump() {
            match c {
                '"' => return Ok(out),
                '\\' => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other),
                    None => return Err(Error::QueryInvalid("unterminated string escape".into())),
                },
                other => out.push(other),
            }
        }
        Err(Error::QueryInvalid("unterminated string".into()))
    }

    fn parse_path_dotted(&mut self) -> Result<String, Error> {
        // paths like status, customer.country, $key
        let mut parts = vec![self.parse_ident_or_string()?];
        while self.eat_char('.') {
            parts.push(self.parse_ident_or_string()?);
        }
        Ok(parts.join("."))
    }

    fn parse_u64(&mut self) -> Result<u64, Error> {
        self.skip_ws();
        let start = self.i;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        if self.i == start {
            return Err(Error::QueryInvalid(format!(
                "expected unsigned integer near `{}`",
                self.snippet()
            )));
        }
        self.s[start..self.i]
            .parse()
            .map_err(|_| Error::QueryInvalid("integer overflow".into()))
    }

    fn parse_project_list(&mut self) -> Result<Vec<String>, Error> {
        // project a, b, c   or project [a, b]
        let mut fields = Vec::new();
        let bracket = self.eat_char('[');
        self.skip_ws();
        loop {
            fields.push(self.parse_path_dotted()?);
            self.skip_ws();
            if self.eat_char(',') {
                self.skip_ws();
                continue;
            }
            break;
        }
        if bracket {
            self.skip_ws();
            if !self.eat_char(']') {
                return Err(Error::QueryInvalid("expected `]` after project list".into()));
            }
        }
        Ok(fields)
    }

    // predicate: or / and / not / cmp
    fn parse_or(&mut self) -> Result<Predicate, Error> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if self.eat_keyword("or") {
                self.skip_ws();
                let right = self.parse_and()?;
                left = Predicate::Or {
                    args: vec![left, right],
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Predicate, Error> {
        let mut left = self.parse_not()?;
        loop {
            self.skip_ws();
            if self.eat_keyword("and") {
                self.skip_ws();
                let right = self.parse_not()?;
                left = Predicate::And {
                    args: vec![left, right],
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Predicate, Error> {
        self.skip_ws();
        if self.eat_keyword("not") {
            self.skip_ws();
            return Ok(Predicate::Not {
                arg: Box::new(self.parse_not()?),
            });
        }
        self.parse_primary_pred()
    }

    fn parse_primary_pred(&mut self) -> Result<Predicate, Error> {
        self.skip_ws();
        if self.eat_char('(') {
            let inner = self.parse_or()?;
            self.skip_ws();
            if !self.eat_char(')') {
                return Err(Error::QueryInvalid("expected `)`".into()));
            }
            return Ok(inner);
        }
        if self.eat_keyword("true") {
            return Ok(Predicate::True);
        }
        if self.eat_keyword("false") {
            return Ok(Predicate::False);
        }
        if self.eat_keyword("present") {
            self.skip_ws();
            self.expect_char('(')?;
            self.skip_ws();
            let path = Path::parse_dotted(&self.parse_path_dotted()?)?;
            self.skip_ws();
            self.expect_char(')')?;
            return Ok(Predicate::Present { path });
        }
        if self.eat_keyword("missing") {
            self.skip_ws();
            self.expect_char('(')?;
            self.skip_ws();
            let path = Path::parse_dotted(&self.parse_path_dotted()?)?;
            self.skip_ws();
            self.expect_char(')')?;
            return Ok(Predicate::Missing { path });
        }
        if self.eat_keyword("starts_with") {
            self.skip_ws();
            self.expect_char('(')?;
            self.skip_ws();
            let path = Path::parse_dotted(&self.parse_path_dotted()?)?;
            self.skip_ws();
            self.expect_char(',')?;
            self.skip_ws();
            let prefix = self.parse_string()?;
            self.skip_ws();
            self.expect_char(')')?;
            return Ok(Predicate::StartsWith { path, prefix });
        }
        if self.eat_keyword("contains") {
            self.skip_ws();
            self.expect_char('(')?;
            self.skip_ws();
            let path = Path::parse_dotted(&self.parse_path_dotted()?)?;
            self.skip_ws();
            self.expect_char(',')?;
            self.skip_ws();
            let needle = self.parse_literal_value()?;
            self.skip_ws();
            self.expect_char(')')?;
            return Ok(Predicate::Contains { path, needle });
        }

        // path op operand | path [not] in [...] | path is [not] null
        let path_s = self.parse_path_dotted()?;
        let path = Path::parse_dotted(&path_s)?;
        self.skip_ws();

        if self.eat_keyword("is") {
            self.skip_ws();
            let neg = self.eat_keyword("not");
            self.skip_ws();
            self.expect_keyword("null")?;
            return Ok(Predicate::IsNull {
                path,
                negated: neg,
            });
        }

        // `not in` / `in`
        let not_in = self.eat_keyword("not");
        self.skip_ws();
        if self.eat_keyword("in") {
            self.skip_ws();
            let list = self.parse_literal_list()?;
            return Ok(Predicate::In {
                left: Operand::path(path),
                list,
                negated: not_in,
            });
        }
        if not_in {
            return Err(Error::QueryInvalid(format!(
                "expected `in` after `not` near path `{path_s}`"
            )));
        }

        let cmp = if self.rest().starts_with("!=") {
            self.i += 2;
            CompareOp::Ne
        } else if self.rest().starts_with("<>") {
            self.i += 2;
            CompareOp::Ne
        } else if self.eat_char('=') {
            CompareOp::Eq
        } else if self.rest().starts_with("<=") {
            self.i += 2;
            CompareOp::Lte
        } else if self.rest().starts_with(">=") {
            self.i += 2;
            CompareOp::Gte
        } else if self.eat_char('<') {
            CompareOp::Lt
        } else if self.eat_char('>') {
            CompareOp::Gt
        } else {
            return Err(Error::QueryInvalid(format!(
                "expected comparison after path `{path_s}`"
            )));
        };
        self.skip_ws();
        let right = self.parse_operand()?;
        Ok(Predicate::Cmp {
            cmp,
            left: Operand::path(path),
            right,
        })
    }

    fn parse_literal_list(&mut self) -> Result<Vec<JsonValue>, Error> {
        self.skip_ws();
        self.expect_char('[')?;
        let mut list = Vec::new();
        self.skip_ws();
        if self.eat_char(']') {
            return Ok(list);
        }
        loop {
            list.push(self.parse_literal_value()?);
            self.skip_ws();
            if self.eat_char(',') {
                self.skip_ws();
                continue;
            }
            break;
        }
        self.skip_ws();
        self.expect_char(']')?;
        Ok(list)
    }

    /// Parse a JSON-ish literal (string, number, bool, null) — not a path or param.
    fn parse_literal_value(&mut self) -> Result<JsonValue, Error> {
        self.skip_ws();
        if self.peek() == Some('"') {
            return Ok(JsonValue::String(self.parse_string()?));
        }
        if self.eat_keyword("true") {
            return Ok(JsonValue::Bool(true));
        }
        if self.eat_keyword("false") {
            return Ok(JsonValue::Bool(false));
        }
        if self.eat_keyword("null") {
            return Ok(JsonValue::Null);
        }
        let start = self.i;
        if self.eat_char('-') {
            // keep
        }
        let dig_start = self.i;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        if self.i == dig_start {
            self.i = start;
            return Err(Error::QueryInvalid(format!(
                "expected literal near `{}`",
                self.snippet()
            )));
        }
        // optional fractional
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
            }
        }
        let num = &self.s[start..self.i];
        if let Ok(i) = num.parse::<i64>() {
            return Ok(JsonValue::Number(i.into()));
        }
        if let Ok(f) = num.parse::<f64>() {
            if let Some(n) = Number::from_f64(f) {
                return Ok(JsonValue::Number(n));
            }
        }
        Err(Error::QueryInvalid(format!("bad number `{num}`")))
    }

    fn expect_char(&mut self, ch: char) -> Result<(), Error> {
        if self.eat_char(ch) {
            Ok(())
        } else {
            Err(Error::QueryInvalid(format!(
                "expected `{ch}` near `{}`",
                self.snippet()
            )))
        }
    }

    fn parse_operand(&mut self) -> Result<Operand, Error> {
        self.skip_ws();
        if self.peek() == Some('$') {
            self.bump();
            let name = self.parse_ident_or_string()?;
            return Ok(param(name));
        }
        if self.peek() == Some('"') {
            return Ok(Operand::literal(JsonValue::String(self.parse_string()?)));
        }
        if self.eat_keyword("true") {
            return Ok(Operand::literal(JsonValue::Bool(true)));
        }
        if self.eat_keyword("false") {
            return Ok(Operand::literal(JsonValue::Bool(false)));
        }
        if self.eat_keyword("null") {
            return Ok(Operand::literal(JsonValue::Null));
        }
        // number
        let start = self.i;
        if self.eat_char('-') {
            // keep
        }
        let dig_start = self.i;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }
        if self.i == dig_start && self.s.as_bytes().get(start) == Some(&b'-') {
            // only minus
            self.i = start;
        } else if self.i > dig_start {
            let num = &self.s[start..self.i];
            if let Ok(i) = num.parse::<i64>() {
                return Ok(Operand::literal(JsonValue::Number(i.into())));
            }
            if let Ok(f) = num.parse::<f64>() {
                if let Some(n) = Number::from_f64(f) {
                    return Ok(Operand::literal(JsonValue::Number(n)));
                }
            }
            return Err(Error::QueryInvalid(format!("bad number `{num}`")));
        }
        // bare path as right-hand? treat as path operand
        let path_s = self.parse_path_dotted()?;
        Ok(Operand::path(Path::parse_dotted(&path_s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_v1::{ConsistencyMode, CoveragePolicy};
    use crate::predicate::{field, param as pred_param, CompareOp, Operand, Path, Predicate};
    use residiuum_heap::CollectionId;
    use std::str::FromStr;

    fn bindings() -> CollectionBindings {
        let mut b = CollectionBindings::default();
        b.bind(
            "orders",
            CollectionId::from_str("00000000-0000-4000-8000-0000000000aa").unwrap(),
        );
        b
    }

    #[test]
    fn compile_simple_where_eq() {
        let c = compile_app_core(
            r#"from orders where status = "paid" limit 10 page size 5"#,
            &bindings(),
        )
        .unwrap();
        assert!(!c.explain);
        assert_eq!(c.profile, APP_CORE_PROFILE);
        assert_eq!(c.plan.limit, Some(10));
        assert_eq!(c.plan.page_size, 5);
        assert_eq!(c.plan.from.source_name, "orders");
    }

    #[test]
    fn reject_enrich() {
        let err = compile_app_core("from orders enrich other", &bindings()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains(DIAG_RQL_FEATURE_UNAVAILABLE), "{msg}");
    }

    #[test]
    fn explain_flag() {
        let c = compile_app_core("explain from orders", &bindings()).unwrap();
        assert!(c.explain);
    }

    #[test]
    fn builder_and_rql_same_hash() {
        use crate::plan_v1::PlanBuilder;
        let b = bindings();
        let from_builder = PlanBuilder::from_source("orders")
            .where_(field("status").unwrap().eq(pred_param("st")))
            .limit(100)
            .page_size(10)
            .unwrap()
            .compile(&b)
            .unwrap();
        let from_rql = compile_app_core(
            r#"from orders where status = $st limit 100 page size 10"#,
            &b,
        )
        .unwrap()
        .plan;
        assert_eq!(from_builder.plan_hash_hex(), from_rql.plan_hash_hex());
    }

    #[test]
    fn multi_clause_hash_matches_builder() {
        use crate::plan_v1::{NullsOrder, OrderDir, PlanBuilder};
        let b = bindings();
        let from_builder = PlanBuilder::from_source("orders")
            .where_(field("status").unwrap().eq(pred_param("st")))
            .project(["id", "status"])
            .unwrap()
            .order_by_nulls("created_at", OrderDir::Desc, NullsOrder::First)
            .unwrap()
            .limit(1000)
            .page_size(100)
            .unwrap()
            .coverage(CoveragePolicy::IncompleteAllowed)
            .consistency(ConsistencyMode::Current)
            .compile(&b)
            .unwrap();
        let src = r#"
            from orders
            where status = $st
            project id, status
            order by created_at desc nulls first
            limit 1000
            page size 100
            coverage incomplete
            consistency current
        "#;
        let from_rql = compile_app_core(src, &b).unwrap().plan;
        assert_eq!(from_builder.plan_hash_hex(), from_rql.plan_hash_hex());
        assert_eq!(from_rql.order[0].nulls, NullsOrder::First);
        assert!(matches!(
            from_rql.coverage,
            CoveragePolicy::IncompleteAllowed
        ));
        assert!(matches!(from_rql.consistency, ConsistencyMode::Current));
    }

    #[test]
    fn repeated_where_and_and_or_not() {
        let b = bindings();
        let c = compile_app_core(
            r#"from orders where status = "paid" where not missing(customer.id) or present(tags)"#,
            &b,
        )
        .unwrap();
        // Two where clauses → And of (eq, Or(Not(Missing), Present))
        match &c.plan.where_pred {
            Predicate::And { args } => {
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[0], Predicate::Cmp { cmp: CompareOp::Eq, .. }));
                assert!(matches!(&args[1], Predicate::Or { .. }));
            }
            other => panic!("expected And of two where clauses, got {other:?}"),
        }
    }

    #[test]
    fn present_missing_is_null_in_starts_contains() {
        let b = bindings();
        let c = compile_app_core(
            r#"from orders where present(status) and missing(gone) and tags is not null and region in ["us", "eu"] and starts_with(name, "Acme") and contains(notes, "rush")"#,
            &b,
        )
        .unwrap();
        // Flatten not required — just ensure it compiles and is And tree
        assert!(!matches!(c.plan.where_pred, Predicate::True));
        let c2 = compile_app_core(
            r#"from orders where status is null and code not in [1, 2, 3]"#,
            &b,
        )
        .unwrap();
        match &c2.plan.where_pred {
            Predicate::And { args } => {
                assert!(matches!(
                    &args[0],
                    Predicate::IsNull { negated: false, .. }
                ));
                assert!(matches!(
                    &args[1],
                    Predicate::In { negated: true, .. }
                ));
            }
            other => panic!("expected And, got {other:?}"),
        }
        // starts_with / contains forms
        let c3 = compile_app_core(
            r#"from orders where starts_with(sku, "AB") and contains(desc, "x")"#,
            &b,
        )
        .unwrap();
        match &c3.plan.where_pred {
            Predicate::And { args } => {
                assert!(matches!(&args[0], Predicate::StartsWith { prefix, .. } if prefix == "AB"));
                assert!(matches!(&args[1], Predicate::Contains { .. }));
            }
            other => panic!("expected And, got {other:?}"),
        }
        let _ = Path::parse_dotted("status").unwrap();
        let _ = Operand::path(Path::parse_dotted("x").unwrap());
    }

    #[test]
    fn comment_and_as_alias() {
        let b = bindings();
        let c = compile_app_core(
            r#"
            -- line comment
            from orders as o
            where status = "open"
            "#,
            &b,
        )
        .unwrap();
        assert_eq!(c.plan.from.source_name, "orders");
    }
}