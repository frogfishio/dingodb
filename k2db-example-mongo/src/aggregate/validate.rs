// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use mongodb::bson::{Bson, Document};

use crate::config::AggregationMode;
use crate::error::{K2DbError, ServiceError};

pub fn validate_pipeline(
    mode: AggregationMode,
    pipeline: &[Document],
    limit: u64,
) -> Result<(), K2DbError> {
    if mode == AggregationMode::Loose {
        return Ok(());
    }

    let max_stages = 50;
    if pipeline.len() > max_stages {
        return Err(K2DbError::new(
            ServiceError::BadRequest,
            format!("Aggregation pipeline too long (max {max_stages} stages)"),
            Some("sys_mdb_ag_pipeline_too_long".to_owned()),
        ));
    }

    if limit == 0 {
        return Err(K2DbError::new(
            ServiceError::BadRequest,
            "Aggregation requires a positive limit in guarded/strict mode",
            Some("sys_mdb_ag_limit_required".to_owned()),
        ));
    }

    let max_limit = 1000;
    if limit > max_limit {
        return Err(K2DbError::new(
            ServiceError::BadRequest,
            format!("Aggregation limit too large (max {max_limit})"),
            Some("sys_mdb_ag_limit_too_large".to_owned()),
        ));
    }

    let ops = collect_stage_ops(pipeline);
    let deny_guarded = ["$out", "$merge", "$function", "$accumulator"];
    let allow_strict = ["$match", "$project", "$sort", "$skip", "$limit"];

    match mode {
        AggregationMode::Guarded => {
            if let Some(op) = ops.iter().find(|op| deny_guarded.contains(&op.as_str())) {
                return Err(K2DbError::new(
                    ServiceError::BadRequest,
                    format!("Aggregation stage {op} is not allowed in guarded mode"),
                    Some("sys_mdb_ag_stage_denied".to_owned()),
                ));
            }
        }
        AggregationMode::Strict => {
            if let Some(op) = ops.iter().find(|op| !allow_strict.contains(&op.as_str())) {
                return Err(K2DbError::new(
                    ServiceError::BadRequest,
                    format!("Aggregation stage {op} is not allowed in strict mode"),
                    Some("sys_mdb_ag_stage_denied".to_owned()),
                ));
            }
        }
        AggregationMode::Loose => {}
    }

    Ok(())
}

pub fn collect_stage_ops(pipeline: &[Document]) -> Vec<String> {
    pipeline
        .iter()
        .map(|stage| {
            let keys = stage.keys().cloned().collect::<Vec<_>>();
            if keys.len() == 1 && keys[0].starts_with('$') {
                keys[0].clone()
            } else {
                "__invalid__".to_owned()
            }
        })
        .collect()
}

pub fn assert_no_secure_field_refs_in_pipeline(
    pipeline: &[Document],
    secure_prefixes: &[String],
) -> Result<(), K2DbError> {
    if secure_prefixes.is_empty() {
        return Ok(());
    }

    if contains_secure_field_ref(
        &Bson::Array(pipeline.iter().cloned().map(Bson::Document).collect()),
        secure_prefixes,
    ) {
        return Err(K2DbError::new(
            ServiceError::BadRequest,
            "Aggregation pipeline references secure-prefixed field(s)",
            Some("sys_mdb_ag_secure_field_ref".to_owned()),
        ));
    }

    Ok(())
}

fn contains_secure_field_ref(value: &Bson, secure_prefixes: &[String]) -> bool {
    match value {
        Bson::Array(values) => values
            .iter()
            .any(|value| contains_secure_field_ref(value, secure_prefixes)),
        Bson::Document(document) => document.iter().any(|(key, value)| {
            string_has_secure_field_path(key, secure_prefixes)
                || contains_secure_field_ref(value, secure_prefixes)
        }),
        Bson::String(value) => string_has_secure_field_path(value, secure_prefixes),
        _ => false,
    }
}

fn string_has_secure_field_path(value: &str, secure_prefixes: &[String]) -> bool {
    let raw = value.trim();
    if raw.is_empty() {
        return false;
    }

    if let Some(rest) = raw.strip_prefix("$$") {
        if let Some((_, path)) = rest.split_once('.') {
            return path_has_secure_segment(path, secure_prefixes);
        }
        return false;
    }

    if let Some(path) = raw.strip_prefix('$') {
        return path_has_secure_segment(path, secure_prefixes);
    }

    path_has_secure_segment(raw, secure_prefixes)
}

fn path_has_secure_segment(path: &str, secure_prefixes: &[String]) -> bool {
    path.split('.')
        .filter(|segment| !segment.is_empty())
        .any(|segment| secure_prefixes.iter().any(|prefix| segment.starts_with(prefix)))
}
