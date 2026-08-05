//! Core path-project IR phase (RQL-IR1).
//!
//! Profile: **`residiuum-query-ir-project-v1`**
//! Normative: [QUERY_IR_PROJECT_V1.md](../../../../../doc/todo/rql/QUERY_IR_PROJECT_V1.md)
//!
//! Application Core `project` (path list) evaluates here — not an inline private
//! helper inside the page loop. Still a **Rust IR residual** (not an opcode
//! machine). Decision 0 remains OPEN; RQL-C1 must not be accepted.

use crate::error::Error;
use crate::predicate::{resolve_path, Path, Resolve};
use serde_json::Value as JsonValue;

/// IR profile id for Core path-project.
pub const PROJECT_IR_PROFILE: &str = "residiuum-query-ir-project-v1";

/// Compiled Core path-project (identity when paths empty / absent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProjectIr {
    /// Profile stamp.
    pub profile: &'static str,
    /// Paths to keep; `None` means identity (full document).
    paths: Option<Vec<Path>>,
}

impl CompiledProjectIr {
    /// Lower plan project list (or identity).
    pub fn lower(project: Option<&Vec<Path>>) -> Self {
        Self {
            profile: PROJECT_IR_PROFILE,
            paths: project.map(|p| p.to_vec()),
        }
    }

    /// Apply projection to one document.
    pub fn apply(&self, doc: &JsonValue) -> Result<JsonValue, Error> {
        apply_project_paths(doc, self.paths.as_ref())
    }

    /// Paths carried (None = identity).
    pub fn paths(&self) -> Option<&[Path]> {
        self.paths.as_deref()
    }
}

/// Apply Core path-project (identity when `paths` is None).
pub fn apply_project_paths(
    doc: &JsonValue,
    paths: Option<&Vec<Path>>,
) -> Result<JsonValue, Error> {
    let Some(paths) = paths else {
        return Ok(doc.clone());
    };
    let mut out = serde_json::Map::new();
    for p in paths {
        match resolve_path(doc, p) {
            Resolve::Present(v) => {
                // Flatten single-segment paths as object fields; multi-segment nest shallowly.
                if p.0.len() == 1 {
                    out.insert(p.0[0].clone(), v);
                } else {
                    out.insert(p.dotted(), v);
                }
            }
            Resolve::Absent => {}
        }
    }
    Ok(JsonValue::Object(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn project_ir_profile_constant() {
        assert_eq!(PROJECT_IR_PROFILE, "residiuum-query-ir-project-v1");
    }

    #[test]
    fn identity_when_no_project() {
        let ir = CompiledProjectIr::lower(None);
        assert_eq!(ir.profile, PROJECT_IR_PROFILE);
        let doc = json!({"a": 1, "b": 2});
        assert_eq!(ir.apply(&doc).unwrap(), doc);
    }

    #[test]
    fn single_and_multi_segment_paths() {
        let paths = vec![
            Path::parse_dotted("name").unwrap(),
            Path::parse_dotted("meta.sku").unwrap(),
        ];
        let ir = CompiledProjectIr::lower(Some(&paths));
        let doc = json!({"name": "Ada", "meta": {"sku": "A"}, "drop": true});
        let out = ir.apply(&doc).unwrap();
        assert_eq!(out["name"], "Ada");
        assert_eq!(out["meta.sku"], "A");
        assert!(out.get("drop").is_none());
        assert!(out.get("meta").is_none());
    }

    #[test]
    fn absent_paths_omitted() {
        let paths = vec![Path::parse_dotted("missing").unwrap()];
        let ir = CompiledProjectIr::lower(Some(&paths));
        let out = ir.apply(&json!({"a": 1})).unwrap();
        assert_eq!(out, json!({}));
    }
}
