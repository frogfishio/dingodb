//! Pluggable query **dialects** that compile into pure SDA.
//!
//! SDA (+ ENR1) is the mathematical language ([`SDA_SPEC`](../../../../../SDA_SPEC.md)).
//! Dialects are comfortable, imperfect frontends — never a redefinition of the
//! algebra and **not** a hybrid of co-equal languages. Foreign surfaces cannot
//! losslessly express every algebraic distinction (especially Null vs absence);
//! when that precision is required, callers use pure SDA.
//! See [doc/SDA/DIALECTS.md](../../../../../doc/SDA/DIALECTS.md).
//!
//! Builtin ids: `sda`, `rql` (official human dialect → ENR1+SDA), `json`,
//! `mongo` (alias of `json`), `sql`, `graphql` (scaffold / refuse).
//! Hosts may register more via [`DialectRegistry`].

mod rql;
mod sql;

use crate::error::Error;
use crate::filter::Filter;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Profile / docs tag for the dialect compilation surface.
pub const DIALECT_PROFILE: &str = "dingo-query-dialects-v0.1";

/// Shape of the compiled pure SDA artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdaShape {
    /// Boolean expression over a single document bound as `input`.
    ///
    /// Suitable for [`crate::Collection::filter_sda`] and for evaluating one
    /// row at a time.
    DocumentPredicate,
    /// Full SDA program. Binding `input` is host-defined (often a sequence of
    /// documents for projection dialects).
    Program,
}

/// Result of compiling dialect source into pure SDA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSda {
    /// Dialect id that produced this compilation (`json`, `sql`, …).
    pub dialect: String,
    /// Pure SDA source text.
    pub sda: String,
    /// Whether `sda` is a document predicate or a full program.
    pub shape: SdaShape,
    /// Non-fatal mapping notes (mimicry caveats, ignored clauses, …).
    pub notes: Vec<String>,
}

impl CompiledSda {
    /// Construct a document-predicate compilation.
    pub fn predicate(
        dialect: impl Into<String>,
        sda: impl Into<String>,
        notes: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            dialect: dialect.into(),
            sda: sda.into(),
            shape: SdaShape::DocumentPredicate,
            notes: notes.into_iter().collect(),
        }
    }

    /// Construct a full-program compilation.
    pub fn program(
        dialect: impl Into<String>,
        sda: impl Into<String>,
        notes: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            dialect: dialect.into(),
            sda: sda.into(),
            shape: SdaShape::Program,
            notes: notes.into_iter().collect(),
        }
    }
}

/// A frontend that compiles foreign notation into pure SDA.
///
/// Implementations MUST refuse unmappable constructs rather than silently
/// weaken SDA semantics. Approximate mappings SHOULD attach [`CompiledSda::notes`].
pub trait QueryDialect: Send + Sync {
    /// Stable dialect id (e.g. `"sql"`, `"json"`).
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// One-line description of coverage and limits.
    fn description(&self) -> &str;

    /// Compile `source` into pure SDA.
    fn compile(&self, source: &str) -> Result<CompiledSda, Error>;
}

/// Static metadata for discovery (CLI, docs, explain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialectInfo {
    /// Stable id.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Coverage summary.
    pub description: &'static str,
    /// Whether compilation is implemented for a useful subset.
    pub implemented: bool,
}

/// Builtin dialect identifiers recognized by [`compile_dialect`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinDialect {
    /// Pure SDA / ENR1 source (parse-checked pass-through).
    Sda,
    /// Official Residiuum Query Language → pure ENR1 + SDA ([`DQL_SPEC`](../../../../../DQL_SPEC.md)).
    Rql,
    /// DX/Mongo-style JSON filter object → document predicate.
    Json,
    /// Alias of [`Self::Json`] for Mongo-familiar callers.
    Mongo,
    /// Tiny SQL `SELECT` / `WHERE` mimicry (partial).
    Sql,
    /// Reserved; compilation fails closed until designed.
    Graphql,
}

impl BuiltinDialect {
    /// Parse a dialect id (case-insensitive). Unknown → `None`.
    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "sda" => Some(Self::Sda),
            "rql"  => Some(Self::Rql),
            "json" | "json-filter" | "filter" => Some(Self::Json),
            "mongo" | "mongodb" => Some(Self::Mongo),
            "sql" => Some(Self::Sql),
            "graphql" | "gql" => Some(Self::Graphql),
            _ => None,
        }
    }

    /// Canonical id string.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Sda => "sda",
            Self::Rql => "rql",
            Self::Json => "json",
            Self::Mongo => "mongo",
            Self::Sql => "sql",
            Self::Graphql => "graphql",
        }
    }

    /// Human name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sda => "Pure SDA",
            Self::Rql => "Residiuum Query Language",
            Self::Json => "JSON filter",
            Self::Mongo => "Mongo-style filter",
            Self::Sql => "SQL mimicry",
            Self::Graphql => "GraphQL (scaffold)",
        }
    }

    /// Coverage blurb.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Sda => "Mathematical SDA/ENR1 source; parse-checked identity",
            Self::Rql => "Official human dialect; lowers to ENR1 Match/enrich/cardinality",
            Self::Json => "DX portable filter object; complete for §7.1 vocabulary",
            Self::Mongo => "Alias of json (Mongo-style $ops object filter)",
            Self::Sql => "Partial SELECT/WHERE → SDA; not full SQL",
            Self::Graphql => "Id reserved; not implemented",
        }
    }

    /// Whether a useful subset is implemented.
    pub const fn implemented(self) -> bool {
        !matches!(self, Self::Graphql)
    }

    /// Compile `source` with this builtin dialect.
    pub fn compile(self, source: &str) -> Result<CompiledSda, Error> {
        match self {
            Self::Sda => compile_sda(source),
            Self::Rql => rql::compile_rql(source),
            Self::Json | Self::Mongo => compile_json_filter(self.id(), source),
            Self::Sql => sql::compile_sql(source),
            Self::Graphql => Err(Error::QueryInvalid(
                "dialect 'graphql' is reserved but not implemented; \
                 use pure SDA, rql, json/mongo filter, or sql mimicry (see doc/SDA/DIALECTS.md)"
                    .into(),
            )),
        }
    }
}

impl QueryDialect for BuiltinDialect {
    fn id(&self) -> &str {
        BuiltinDialect::id(*self)
    }

    fn name(&self) -> &str {
        BuiltinDialect::name(*self)
    }

    fn description(&self) -> &str {
        BuiltinDialect::description(*self)
    }

    fn compile(&self, source: &str) -> Result<CompiledSda, Error> {
        BuiltinDialect::compile(*self, source)
    }
}

/// All builtin dialect metadata (including scaffold-only ids).
pub fn list_builtin_dialects() -> &'static [DialectInfo] {
    const LIST: &[DialectInfo] = &[
        DialectInfo {
            id: "sda",
            name: "Pure SDA",
            description: "Mathematical SDA/ENR1 source; parse-checked identity",
            implemented: true,
        },
        DialectInfo {
            id: "rql",
            name: "Residiuum Query Language",
            description: "Official human dialect; lowers to ENR1 Match/enrich/cardinality",
            implemented: true,
        },
        DialectInfo {
            id: "json",
            name: "JSON filter",
            description: "DX portable filter object; complete for §7.1 vocabulary",
            implemented: true,
        },
        DialectInfo {
            id: "mongo",
            name: "Mongo-style filter",
            description: "Alias of json (Mongo-style $ops object filter)",
            implemented: true,
        },
        DialectInfo {
            id: "sql",
            name: "SQL mimicry",
            description: "Partial SELECT/WHERE → SDA; not full SQL",
            implemented: true,
        },
        DialectInfo {
            id: "graphql",
            name: "GraphQL (scaffold)",
            description: "Id reserved; not implemented",
            implemented: false,
        },
    ];
    LIST
}

/// Compile `source` with a builtin dialect id (`json`, `sql`, `sda`, …).
///
/// Custom dialects require a [`DialectRegistry`].
pub fn compile_dialect(dialect_id: &str, source: &str) -> Result<CompiledSda, Error> {
    match BuiltinDialect::from_id(dialect_id) {
        Some(d) => d.compile(source),
        None => Err(Error::QueryInvalid(format!(
            "unknown query dialect {dialect_id:?}; known: sda, rql, json, mongo, sql, graphql \
             (see doc/SDA/DIALECTS.md)"
        ))),
    }
}

/// Registry of builtin + caller-registered dialects.
#[derive(Clone, Default)]
pub struct DialectRegistry {
    custom: HashMap<String, Arc<dyn QueryDialect>>,
}

impl DialectRegistry {
    /// Empty registry (still resolves builtins via [`Self::compile`]).
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom dialect. Id is stored lowercased; must not collide
    /// with a builtin id unless intentionally shadowing (shadowing is refused).
    pub fn register(&mut self, dialect: Arc<dyn QueryDialect>) -> Result<(), Error> {
        let id = dialect.id().trim().to_ascii_lowercase();
        if id.is_empty() {
            return Err(Error::QueryInvalid("dialect id must be non-empty".into()));
        }
        if BuiltinDialect::from_id(&id).is_some() {
            return Err(Error::QueryInvalid(format!(
                "cannot register dialect {id:?}: id is reserved for a builtin"
            )));
        }
        if self.custom.contains_key(&id) {
            return Err(Error::QueryInvalid(format!(
                "dialect {id:?} is already registered"
            )));
        }
        self.custom.insert(id, dialect);
        Ok(())
    }

    /// Compile with builtin or custom dialect.
    pub fn compile(&self, dialect_id: &str, source: &str) -> Result<CompiledSda, Error> {
        let key = dialect_id.trim().to_ascii_lowercase();
        if let Some(d) = BuiltinDialect::from_id(&key) {
            return d.compile(source);
        }
        if let Some(d) = self.custom.get(&key) {
            return d.compile(source);
        }
        let mut known: Vec<&str> = list_builtin_dialects().iter().map(|d| d.id).collect();
        known.extend(self.custom.keys().map(|s| s.as_str()));
        known.sort_unstable();
        Err(Error::QueryInvalid(format!(
            "unknown query dialect {dialect_id:?}; known: {}",
            known.join(", ")
        )))
    }

    /// List builtin metadata plus registered custom ids.
    pub fn list(&self) -> Vec<DialectInfoOwned> {
        let mut out: Vec<DialectInfoOwned> = list_builtin_dialects()
            .iter()
            .map(DialectInfoOwned::from_static)
            .collect();
        for (id, d) in &self.custom {
            out.push(DialectInfoOwned {
                id: id.clone(),
                name: d.name().to_string(),
                description: d.description().to_string(),
                implemented: true,
            });
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }
}

/// Owned dialect metadata (for registries that include custom ids).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialectInfoOwned {
    /// Stable id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Coverage summary.
    pub description: String,
    /// Whether compilation is implemented.
    pub implemented: bool,
}

impl DialectInfoOwned {
    fn from_static(info: &DialectInfo) -> Self {
        Self {
            id: info.id.to_string(),
            name: info.name.to_string(),
            description: info.description.to_string(),
            implemented: info.implemented,
        }
    }
}

fn compile_sda(source: &str) -> Result<CompiledSda, Error> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(Error::QueryInvalid(
            "dialect 'sda': empty program".into(),
        ));
    }
    sda_core::Program::parse(trimmed).map_err(|e| {
        Error::QueryInvalid(format!("dialect 'sda' parse failed: {e}"))
    })?;
    Ok(CompiledSda::program(
        "sda",
        trimmed,
        std::iter::empty::<String>(),
    ))
}

fn compile_json_filter(dialect_id: &str, source: &str) -> Result<CompiledSda, Error> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(Error::QueryInvalid(format!(
            "dialect '{dialect_id}': empty filter"
        )));
    }
    let value: JsonValue = serde_json::from_str(trimmed).map_err(|e| {
        Error::QueryInvalid(format!(
            "dialect '{dialect_id}': filter must be a JSON object: {e}"
        ))
    })?;
    let filter = Filter::from_json(&value)?;
    let sda = filter.to_sda();
    // Sanity: compiled predicate must parse as SDA.
    sda_core::Program::parse(&sda).map_err(|e| {
        Error::QueryInvalid(format!(
            "dialect '{dialect_id}': internal SDA compile failed: {e}; sda={sda}"
        ))
    })?;
    Ok(CompiledSda::predicate(
        dialect_id,
        sda,
        std::iter::empty::<String>(),
    ))
}

/// Compile a JSON value (not a string) with the `json` dialect.
pub fn compile_json_value(filter: &JsonValue) -> Result<CompiledSda, Error> {
    let f = Filter::from_json(filter)?;
    let sda = f.to_sda();
    sda_core::Program::parse(&sda).map_err(|e| {
        Error::QueryInvalid(format!(
            "dialect 'json': internal SDA compile failed: {e}; sda={sda}"
        ))
    })?;
    Ok(CompiledSda::predicate(
        "json",
        sda,
        std::iter::empty::<String>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_includes_scaffolds() {
        let ids: Vec<_> = list_builtin_dialects().iter().map(|d| d.id).collect();
        assert!(ids.contains(&"sda"));
        assert!(ids.contains(&"rql"));
        assert!(ids.contains(&"json"));
        assert!(ids.contains(&"mongo"));
        assert!(ids.contains(&"sql"));
        assert!(ids.contains(&"graphql"));
        assert!(list_builtin_dialects()
            .iter()
            .find(|d| d.id == "rql")
            .unwrap()
            .implemented);
        assert!(!list_builtin_dialects()
            .iter()
            .find(|d| d.id == "graphql")
            .unwrap()
            .implemented);
    }

    #[test]
    fn dql_dialect_compiles_to_enr1() {
        let c = compile_dialect(
            "rql",
            r#"
            from orders
            enrich customer using customers
              matching customer_id = id
              expect exactly_one
        "#,
        )
        .unwrap();
        assert_eq!(c.dialect, "rql");
        assert_eq!(c.shape, SdaShape::Program);
        assert!(c.sda.contains("Match("));
        assert!(c.sda.contains("one!("));
    }

    #[test]
    fn json_dialect_compiles_and_matches() {
        let src = r#"{"status":"active","age":{"$gte":18}}"#;
        let compiled = compile_dialect("json", src).unwrap();
        assert_eq!(compiled.dialect, "json");
        assert_eq!(compiled.shape, SdaShape::DocumentPredicate);
        assert!(compiled.sda.contains("getPath"));
        let prog = sda_core::Program::parse(&compiled.sda).unwrap();
        let hit = prog
            .run_json("input", json!({"status": "active", "age": 21}))
            .unwrap();
        assert_eq!(hit, json!(true));
        let miss = prog
            .run_json("input", json!({"status": "active", "age": 10}))
            .unwrap();
        assert_eq!(miss, json!(false));
    }

    #[test]
    fn mongo_alias_matches_json() {
        let src = r#"{"x":1}"#;
        let a = compile_dialect("mongo", src).unwrap();
        let b = compile_dialect("json", src).unwrap();
        assert_eq!(a.sda, b.sda);
        assert_eq!(a.dialect, "mongo");
    }

    #[test]
    fn sda_pass_through() {
        let src = r#"getPath(input, Seq["a"]) = Some(1)"#;
        let c = compile_dialect("sda", src).unwrap();
        assert_eq!(c.shape, SdaShape::Program);
        assert_eq!(c.sda, src);
    }

    #[test]
    fn graphql_refuses() {
        let err = compile_dialect("graphql", "{ user { id } }").unwrap_err();
        assert!(err.to_string().contains("not implemented"));
    }

    #[test]
    fn unknown_dialect() {
        let err = compile_dialect("cypher", "MATCH (n)").unwrap_err();
        assert!(err.to_string().contains("unknown query dialect"));
    }

    #[test]
    fn sql_select_star_where() {
        let c = compile_dialect(
            "sql",
            "SELECT * WHERE status = 'active' AND age >= 18",
        )
        .unwrap();
        assert_eq!(c.shape, SdaShape::DocumentPredicate);
        let prog = sda_core::Program::parse(&c.sda).unwrap();
        assert_eq!(
            prog.run_json("input", json!({"status": "active", "age": 20}))
                .unwrap(),
            json!(true)
        );
        assert_eq!(
            prog.run_json("input", json!({"status": "active", "age": 10}))
                .unwrap(),
            json!(false)
        );
    }

    #[test]
    fn sql_projection_program() {
        let c = compile_dialect(
            "sql",
            "SELECT name, city WHERE status = 'active'",
        )
        .unwrap();
        assert_eq!(c.shape, SdaShape::Program);
        assert!(!c.notes.is_empty());
        let prog = sda_core::Program::parse(&c.sda).unwrap();
        let out = prog
            .run_json(
                "input",
                json!([
                    {"name": "Ada", "city": "LA", "status": "active"},
                    {"name": "Bob", "city": "NY", "status": "idle"},
                ]),
            )
            .unwrap();
        // Seq of projected maps; field values are getPath Option carriers
        // (SQL mimicry is imperfect — bare SQL null is not SDA absence).
        let arr = out.as_array().expect("seq of projections");
        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0]["name"],
            json!({"$type": "some", "$value": "Ada"})
        );
        assert_eq!(
            arr[0]["city"],
            json!({"$type": "some", "$value": "LA"})
        );
    }

    #[test]
    fn registry_custom_dialect() {
        struct Echo;
        impl QueryDialect for Echo {
            fn id(&self) -> &str {
                "echo"
            }
            fn name(&self) -> &str {
                "Echo"
            }
            fn description(&self) -> &str {
                "test"
            }
            fn compile(&self, source: &str) -> Result<CompiledSda, Error> {
                Ok(CompiledSda::predicate(
                    "echo",
                    "true",
                    [format!("echoed:{source}")],
                ))
            }
        }
        let mut reg = DialectRegistry::new();
        reg.register(Arc::new(Echo)).unwrap();
        let c = reg.compile("echo", "hi").unwrap();
        assert_eq!(c.sda, "true");
        assert!(c.notes.iter().any(|n| n.contains("hi")));
        assert!(reg.register(Arc::new(Echo)).is_err());
        assert!(reg.register(Arc::new(BuiltinDialect::Json)).is_err());
    }

    #[test]
    fn compile_json_value_api() {
        let c = compile_json_value(&json!({"a": {"$exists": true}})).unwrap();
        assert_eq!(c.dialect, "json");
        assert!(c.sda.contains("getPath"));
    }
}
