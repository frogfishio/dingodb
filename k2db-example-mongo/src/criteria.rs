// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use mongodb::bson::{Bson, Document};

use crate::config::OwnershipMode;
use crate::document::normalize_id;
use crate::error::{K2DbError, ServiceError};
use crate::scope::Scope;

pub fn normalize_criteria_ids(value: Bson) -> Bson {
    match value {
        Bson::Document(document) => Bson::Document(normalize_document_ids(document)),
        Bson::Array(values) => Bson::Array(values.into_iter().map(normalize_criteria_ids).collect()),
        other => other,
    }
}

pub fn normalize_document_ids(document: Document) -> Document {
    document
        .into_iter()
        .map(|(key, value)| {
            let normalized = match key.as_str() {
                "_uuid" => normalize_uuid_field(value),
                "_owner" => normalize_owner_field(value),
                _ => normalize_criteria_ids(value),
            };
            (key, normalized)
        })
        .collect()
}

pub fn normalize_uuid_field(value: Bson) -> Bson {
    match value {
        Bson::String(value) => Bson::String(value.to_ascii_uppercase()),
        Bson::Array(values) => Bson::Array(values.into_iter().map(normalize_uuid_field).collect()),
        Bson::Document(document) => Bson::Document(normalize_operator_values(document, normalize_uuid_field)),
        other => other,
    }
}

pub fn normalize_owner_field(value: Bson) -> Bson {
    match value {
        Bson::String(value) => Bson::String(value.trim().to_owned()),
        Bson::Array(values) => Bson::Array(values.into_iter().map(normalize_owner_field).collect()),
        Bson::Document(document) => Bson::Document(normalize_operator_values(document, normalize_owner_field)),
        other => other,
    }
}

fn normalize_operator_values(document: Document, transform: fn(Bson) -> Bson) -> Document {
    document
        .into_iter()
        .map(|(key, value)| {
            let normalized = match key.as_str() {
                "$in" | "$nin" | "$eq" | "$ne" | "$all" => transform(value),
                _ => normalize_criteria_ids(value),
            };
            (key, normalized)
        })
        .collect()
}

pub fn apply_default_non_deleted(mut criteria: Document) -> Document {
    if !criteria.contains_key("_deleted") {
        criteria.insert(
            "_deleted",
            Bson::Document(Document::from_iter([(String::from("$ne"), Bson::Boolean(true))])),
        );
    }
    criteria
}

pub fn apply_scope(
    mut criteria: Document,
    scope: Option<&Scope>,
    ownership_mode: OwnershipMode,
) -> Result<Document, K2DbError> {
    match (ownership_mode, scope) {
        (OwnershipMode::Strict, None) => Err(K2DbError::new(
            ServiceError::BadRequest,
            "Scope is required in strict ownership mode",
            Some("sys_mdb_scope_required".to_owned()),
        )),
        (_, None) | (_, Some(Scope::All)) => Ok(criteria),
        (_, Some(Scope::Owner(owner))) => {
            let normalized_owner = normalize_id(owner);
            if let Some(existing) = criteria.get_str("_owner").ok() {
                if existing.trim() != normalized_owner {
                    return Err(K2DbError::new(
                        ServiceError::BadRequest,
                        "Conflicting _owner in criteria and provided scope",
                        Some("sys_mdb_scope_conflict".to_owned()),
                    ));
                }
            }
            criteria.insert("_owner", normalized_owner);
            Ok(criteria)
        }
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::{doc, Bson};

    use crate::config::OwnershipMode;
    use crate::scope::Scope;

    use super::{apply_default_non_deleted, apply_scope, normalize_document_ids};

    #[test]
    fn normalize_document_ids_updates_uuid_and_owner() {
        let input = doc! {
            "_uuid": "abc-def",
            "nested": { "_uuid": "ghi-jkl" },
            "_owner": "  owner1  "
        };
        let actual = normalize_document_ids(input);
        assert_eq!(actual.get_str("_uuid").unwrap(), "ABC-DEF");
        assert_eq!(actual.get_document("nested").unwrap().get_str("_uuid").unwrap(), "GHI-JKL");
        assert_eq!(actual.get_str("_owner").unwrap(), "owner1");
    }

    #[test]
    fn apply_default_non_deleted_adds_default_filter() {
        let actual = apply_default_non_deleted(doc! {});
        assert_eq!(actual.get("_deleted"), Some(&Bson::Document(doc! { "$ne": true })));
    }

    #[test]
    fn strict_scope_requires_scope() {
        let error = apply_scope(doc! {}, None, OwnershipMode::Strict).unwrap_err();
        assert_eq!(error.key.as_deref(), Some("sys_mdb_scope_required"));
    }

    #[test]
    fn owner_scope_is_injected() {
        let actual = apply_scope(doc! { "name": "x" }, Some(&Scope::owner("owner1")), OwnershipMode::Lax).unwrap();
        assert_eq!(actual.get_str("_owner").unwrap(), "owner1");
    }
}