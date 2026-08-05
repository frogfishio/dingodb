//! Portable dialect compile → Application Core / QVM (**RQL-DQ1**).
//!
//! sql / json / mongo lower to [`Filter`] (+ optional project), not SDA.
//! Product execute binds a collection and runs via Query VM.

use crate::error::Error;
use crate::filter::Filter;

/// Dialect compile result for QVM-bound frontends (not raw SDA).
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledPortable {
    /// Dialect id (`sql`, `json`, `mongo`).
    pub dialect: String,
    /// Portable where filter.
    pub filter: Filter,
    /// Optional SELECT column paths (dotted); `None` = identity / `*`.
    pub project: Option<Vec<String>>,
    /// Honesty notes (approximate mappings).
    pub notes: Vec<String>,
}

impl CompiledPortable {
    /// Construct a portable compile artefact.
    pub fn new(
        dialect: impl Into<String>,
        filter: Filter,
        project: Option<Vec<String>>,
        notes: Vec<String>,
    ) -> Self {
        Self {
            dialect: dialect.into(),
            filter,
            project,
            notes,
        }
    }
}

/// Product dialect compile result (**RQL-DQ1**).
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledDialect {
    /// Explicit raw SDA (`sda` dialect only on the dialect surface).
    Sda(super::CompiledSda),
    /// Portable filter (+ project) for QVM lower.
    Portable(CompiledPortable),
}

impl CompiledDialect {
    /// Dialect id string.
    pub fn dialect_id(&self) -> &str {
        match self {
            Self::Sda(s) => s.dialect.as_str(),
            Self::Portable(p) => p.dialect.as_str(),
        }
    }

    /// Borrow SDA artefact when present.
    pub fn as_sda(&self) -> Option<&super::CompiledSda> {
        match self {
            Self::Sda(s) => Some(s),
            Self::Portable(_) => None,
        }
    }

    /// Borrow portable artefact when present.
    pub fn as_portable(&self) -> Option<&CompiledPortable> {
        match self {
            Self::Sda(_) => None,
            Self::Portable(p) => Some(p),
        }
    }

    /// Honesty notes.
    pub fn notes(&self) -> &[String] {
        match self {
            Self::Sda(s) => &s.notes,
            Self::Portable(p) => &p.notes,
        }
    }
}

/// Refuse unknown / scaffold dialect ids with a stable message.
pub(super) fn unknown_dialect(id: &str) -> Error {
    Error::QueryInvalid(format!(
        "unknown query dialect {id:?}; known: sda, rql, json, mongo, sql, graphql \
         (rql refused on this surface — use CollectionClient::rql)"
    ))
}
