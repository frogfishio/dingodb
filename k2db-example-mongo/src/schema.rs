// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use jsonschema::JSONSchema;
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{K2DbError, ServiceError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaMode {
    Strict,
    Strip,
    Passthrough,
}

impl Default for SchemaMode {
    fn default() -> Self {
        Self::Strip
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredSchema {
    raw: Value,
    mode: SchemaMode,
    full: Arc<JSONSchema>,
    partial: Arc<JSONSchema>,
}

impl RegisteredSchema {
    pub fn compile(schema: Value, mode: SchemaMode) -> Result<Self, K2DbError> {
        let full_schema = adjust_root_schema(schema.clone(), mode, false);
        let partial_schema = adjust_root_schema(schema.clone(), mode, true);

        let full = JSONSchema::compile(&full_schema).map_err(|error| {
            K2DbError::new(
                ServiceError::ConfigurationError,
                format!("Invalid schema: {error}"),
                Some("sys_mdb_schema_invalid".to_owned()),
            )
        })?;
        let partial = JSONSchema::compile(&partial_schema).map_err(|error| {
            K2DbError::new(
                ServiceError::ConfigurationError,
                format!("Invalid schema: {error}"),
                Some("sys_mdb_schema_invalid".to_owned()),
            )
        })?;

        Ok(Self {
            raw: schema,
            mode,
            full: Arc::new(full),
            partial: Arc::new(partial),
        })
    }

    pub fn apply(&self, document: &Document, partial: bool) -> Result<Document, K2DbError> {
        let mut value = serde_json::to_value(document).map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::ValidationError,
                Some("sys_mdb_schema_validation".to_owned()),
                "Schema validation failed",
            )
        })?;

        if self.mode == SchemaMode::Strip {
            strip_unknown_root_fields(&mut value, &self.raw);
        }

        let validator = if partial { &self.partial } else { &self.full };
        if let Err(errors) = validator.validate(&value) {
            let message = errors.map(|error| error.to_string()).collect::<Vec<_>>().join("; ");
            return Err(K2DbError::new(
                ServiceError::ValidationError,
                message,
                Some("sys_mdb_schema_validation".to_owned()),
            ));
        }

        serde_json::from_value(value).map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::ValidationError,
                Some("sys_mdb_schema_validation".to_owned()),
                "Schema validation failed",
            )
        })
    }
}

fn adjust_root_schema(mut schema: Value, mode: SchemaMode, partial: bool) -> Value {
    let Some(root) = schema.as_object_mut() else {
        return schema;
    };
    let is_object_schema = match root.get("type") {
        Some(Value::String(kind)) => kind == "object",
        Some(Value::Array(kinds)) => kinds.iter().any(|kind| kind.as_str() == Some("object")),
        _ => root.contains_key("properties"),
    };
    if !is_object_schema {
        return schema;
    }

    if partial {
        root.remove("required");
    }

    if mode == SchemaMode::Strict {
        root.insert("additionalProperties".to_owned(), Value::Bool(false));
    }

    schema
}

fn strip_unknown_root_fields(value: &mut Value, schema: &Value) {
    let Some(document) = value.as_object_mut() else {
        return;
    };
    let Some(schema_object) = schema.as_object() else {
        return;
    };
    let Some(properties) = schema_object.get("properties").and_then(Value::as_object) else {
        return;
    };

    document.retain(|key, _| properties.contains_key(key));
}

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;
    use serde_json::json;

    use super::{RegisteredSchema, SchemaMode};

    #[test]
    fn strip_mode_removes_unknown_root_fields() {
        let schema = RegisteredSchema::compile(
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "required": ["name"]
            }),
            SchemaMode::Strip,
        )
        .unwrap();

        let applied = schema.apply(&doc! { "name": "ok", "extra": true }, false).unwrap();
        assert_eq!(applied, doc! { "name": "ok" });
    }

    #[test]
    fn partial_mode_drops_root_required() {
        let schema = RegisteredSchema::compile(
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "count": { "type": "integer" }
                },
                "required": ["name", "count"]
            }),
            SchemaMode::Strict,
        )
        .unwrap();

        schema.apply(&doc! { "name": "ok" }, true).unwrap();
        let error = schema.apply(&doc! { "name": "ok" }, false).unwrap_err();
        assert_eq!(error.key.as_deref(), Some("sys_mdb_schema_validation"));
    }

    #[test]
    fn non_object_schema_keeps_partial_behavior_unchanged() {
        let schema = RegisteredSchema::compile(json!({ "type": "string" }), SchemaMode::Strip).unwrap();

        let patch_error = schema.apply(&doc! { "name": "ok" }, true).unwrap_err();
        let replace_error = schema.apply(&doc! { "name": "ok" }, false).unwrap_err();

        assert_eq!(patch_error.key.as_deref(), Some("sys_mdb_schema_validation"));
        assert_eq!(replace_error.key.as_deref(), Some("sys_mdb_schema_validation"));
    }
}