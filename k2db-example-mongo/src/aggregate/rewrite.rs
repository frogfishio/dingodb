// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use mongodb::bson::{doc, Bson, Document};

use crate::config::OwnershipMode;
use crate::criteria::normalize_document_ids;
use crate::error::{K2DbError, ServiceError};
use crate::scope::Scope;

const RESERVED_FIRST_STAGE_OPS: &[&str] = &["$search", "$geoNear", "$vectorSearch"];

pub fn enforce_no_deleted_in_pipeline(pipeline: &[Document]) -> Vec<Document> {
    let mut cloned = pipeline.to_vec();
    let insert_idx = first_match_insertion_index(&cloned);
    cloned.insert(insert_idx, doc! { "$match": { "_deleted": { "$ne": true } } });
    cloned.into_iter().map(rewrite_no_deleted_stage).collect()
}

pub fn enforce_scope_in_pipeline(
    pipeline: &[Document],
    scope: Option<&Scope>,
    ownership_mode: OwnershipMode,
) -> Result<Vec<Document>, K2DbError> {
    match (ownership_mode, scope) {
        (OwnershipMode::Strict, None) => Err(K2DbError::new(
            ServiceError::BadRequest,
            "Scope is required in strict ownership mode",
            Some("sys_mdb_scope_required".to_owned()),
        )),
        (_, None) | (_, Some(Scope::All)) => Ok(pipeline.to_vec()),
        (_, Some(Scope::Owner(owner))) => {
            let mut cloned = pipeline.to_vec();
            let insert_idx = first_match_insertion_index(&cloned);
            cloned.insert(insert_idx, doc! { "$match": { "_owner": owner } });
            cloned
                .into_iter()
                .map(|stage| rewrite_scope_stage(stage, owner))
                .collect()
        }
    }
}

pub fn sanitize_pipeline_matches(pipeline: Vec<Document>) -> Vec<Document> {
    pipeline.into_iter().map(sanitize_stage).collect()
}

fn sanitize_stage(mut stage: Document) -> Document {
    if let Ok(inner) = stage.get_document("$match") {
        stage.insert("$match", Bson::Document(normalize_document_ids(inner.clone())));
        return stage;
    }

    if let Ok(lookup) = stage.get_document("$lookup") {
        let mut lookup = lookup.clone();
        if let Ok(pipeline) = lookup.get_array("pipeline") {
            let nested = pipeline
                .iter()
                .filter_map(|value| value.as_document().cloned())
                .collect::<Vec<_>>();
            lookup.insert(
                "pipeline",
                Bson::Array(
                    sanitize_pipeline_matches(nested)
                        .into_iter()
                        .map(Bson::Document)
                        .collect(),
                ),
            );
        }
        stage.insert("$lookup", lookup);
        return stage;
    }

    if let Ok(union_with) = stage.get_document("$unionWith") {
        let mut union_with = union_with.clone();
        if let Ok(pipeline) = union_with.get_array("pipeline") {
            let nested = pipeline
                .iter()
                .filter_map(|value| value.as_document().cloned())
                .collect::<Vec<_>>();
            union_with.insert(
                "pipeline",
                Bson::Array(
                    sanitize_pipeline_matches(nested)
                        .into_iter()
                        .map(Bson::Document)
                        .collect(),
                ),
            );
        }
        stage.insert("$unionWith", union_with);
        return stage;
    }

    if let Ok(facet) = stage.get_document("$facet") {
        let mut next = Document::new();
        for (key, value) in facet {
            if let Some(array) = value.as_array() {
                let nested = array
                    .iter()
                    .filter_map(|item| item.as_document().cloned())
                    .collect::<Vec<_>>();
                next.insert(
                    key,
                    Bson::Array(
                        sanitize_pipeline_matches(nested)
                            .into_iter()
                            .map(Bson::Document)
                            .collect(),
                    ),
                );
            } else {
                next.insert(key, value.clone());
            }
        }
        stage.insert("$facet", next);
    }

    stage
}

fn first_match_insertion_index(pipeline: &[Document]) -> usize {
    let mut index = 0;
    while let Some(stage) = pipeline.get(index) {
        let keys = stage.keys().cloned().collect::<Vec<_>>();
        if keys.len() == 1 && RESERVED_FIRST_STAGE_OPS.contains(&keys[0].as_str()) {
            index += 1;
            continue;
        }
        break;
    }
    index
}

fn rewrite_no_deleted_stage(mut stage: Document) -> Document {
    if let Ok(lookup) = stage.get_document("$lookup") {
        stage.insert("$lookup", rewrite_lookup_no_deleted(lookup.clone()));
        return stage;
    }

    if let Some(union_with_value) = stage.get("$unionWith") {
        stage.insert("$unionWith", rewrite_union_with_no_deleted(union_with_value.clone()));
        return stage;
    }

    if let Ok(graph_lookup) = stage.get_document("$graphLookup") {
        let mut graph_lookup = graph_lookup.clone();
        let mut restrict = graph_lookup
            .get_document("restrictSearchWithMatch")
            .cloned()
            .unwrap_or_default();
        restrict.insert("_deleted", Bson::Document(doc! { "$ne": true }));
        graph_lookup.insert("restrictSearchWithMatch", restrict);
        stage.insert("$graphLookup", graph_lookup);
        return stage;
    }

    if let Ok(facet) = stage.get_document("$facet") {
        let mut next = Document::new();
        for (key, value) in facet {
            if let Some(array) = value.as_array() {
                let nested = array
                    .iter()
                    .filter_map(|item| item.as_document().cloned())
                    .collect::<Vec<_>>();
                next.insert(
                    key,
                    Bson::Array(
                        enforce_no_deleted_in_pipeline(&nested)
                            .into_iter()
                            .map(Bson::Document)
                            .collect(),
                    ),
                );
            } else {
                next.insert(key, value.clone());
            }
        }
        stage.insert("$facet", next);
    }

    stage
}

fn rewrite_scope_stage(mut stage: Document, owner: &str) -> Result<Document, K2DbError> {
    if let Ok(lookup) = stage.get_document("$lookup") {
        stage.insert("$lookup", rewrite_lookup_scope(lookup.clone(), owner)?);
        return Ok(stage);
    }

    if let Some(union_with_value) = stage.get("$unionWith") {
        stage.insert("$unionWith", rewrite_union_with_scope(union_with_value.clone(), owner)?);
        return Ok(stage);
    }

    if let Ok(graph_lookup) = stage.get_document("$graphLookup") {
        let mut graph_lookup = graph_lookup.clone();
        let mut restrict = graph_lookup
            .get_document("restrictSearchWithMatch")
            .cloned()
            .unwrap_or_default();
        restrict.insert("_owner", owner);
        graph_lookup.insert("restrictSearchWithMatch", restrict);
        stage.insert("$graphLookup", graph_lookup);
        return Ok(stage);
    }

    if let Ok(facet) = stage.get_document("$facet") {
        let mut next = Document::new();
        for (key, value) in facet {
            if let Some(array) = value.as_array() {
                let nested = array
                    .iter()
                    .filter_map(|item| item.as_document().cloned())
                    .collect::<Vec<_>>();
                next.insert(
                    key,
                    Bson::Array(
                        enforce_scope_in_pipeline(&nested, Some(&Scope::owner(owner)), OwnershipMode::Lax)?
                        .into_iter()
                        .map(Bson::Document)
                        .collect(),
                    ),
                );
            } else {
                next.insert(key, value.clone());
            }
        }
        stage.insert("$facet", next);
    }

    Ok(stage)
}

fn rewrite_lookup_no_deleted(mut lookup: Document) -> Document {
    if let Ok(pipeline) = lookup.get_array("pipeline") {
        let nested = pipeline
            .iter()
            .filter_map(|item| item.as_document().cloned())
            .collect::<Vec<_>>();
        lookup.insert(
            "pipeline",
            Bson::Array(
                enforce_no_deleted_in_pipeline(&nested)
                    .into_iter()
                    .map(Bson::Document)
                    .collect(),
            ),
        );
        return lookup;
    }

    if let (Ok(local_field), Ok(foreign_field)) =
        (lookup.get_str("localField"), lookup.get_str("foreignField"))
    {
        let local_field = local_field.to_owned();
        let foreign_field = foreign_field.to_owned();
        lookup.insert("let", doc! { "k2lk": format!("${local_field}") });
        lookup.insert(
            "pipeline",
            Bson::Array(vec![Bson::Document(doc! {
                "$match": {
                    "$expr": {
                        "$and": [
                            {
                                "$cond": [
                                    { "$isArray": "$$k2lk" },
                                    { "$in": [format!("${foreign_field}"), "$$k2lk"] },
                                    { "$eq": [format!("${foreign_field}"), "$$k2lk"] }
                                ]
                            },
                            { "$ne": ["$_deleted", true] }
                        ]
                    }
                }
            })]),
        );
        lookup.remove("localField");
        lookup.remove("foreignField");
    }

    lookup
}

fn rewrite_lookup_scope(mut lookup: Document, owner: &str) -> Result<Document, K2DbError> {
    if let Ok(pipeline) = lookup.get_array("pipeline") {
        let nested = pipeline
            .iter()
            .filter_map(|item| item.as_document().cloned())
            .collect::<Vec<_>>();
        lookup.insert(
            "pipeline",
            Bson::Array(
                enforce_scope_in_pipeline(&nested, Some(&Scope::owner(owner)), OwnershipMode::Lax)?
                .into_iter()
                .map(Bson::Document)
                .collect(),
            ),
        );
        return Ok(lookup);
    }

    if let (Ok(local_field), Ok(foreign_field)) =
        (lookup.get_str("localField"), lookup.get_str("foreignField"))
    {
        let local_field = local_field.to_owned();
        let foreign_field = foreign_field.to_owned();
        lookup.insert(
            "let",
            doc! {
                "k2lk": format!("${local_field}"),
                "k2own": owner,
            },
        );
        lookup.insert(
            "pipeline",
            Bson::Array(vec![Bson::Document(doc! {
                "$match": {
                    "$expr": {
                        "$and": [
                            {
                                "$cond": [
                                    { "$isArray": "$$k2lk" },
                                    { "$in": [format!("${foreign_field}"), "$$k2lk"] },
                                    { "$eq": [format!("${foreign_field}"), "$$k2lk"] }
                                ]
                            },
                            { "$eq": ["$_owner", "$$k2own"] },
                            { "$ne": ["$_deleted", true] }
                        ]
                    }
                }
            })]),
        );
        lookup.remove("localField");
        lookup.remove("foreignField");
    }

    Ok(lookup)
}

fn rewrite_union_with_no_deleted(value: Bson) -> Bson {
    match value {
        Bson::String(coll) => Bson::Document(doc! {
            "coll": coll,
            "pipeline": [
                { "$match": { "_deleted": { "$ne": true } } }
            ]
        }),
        Bson::Document(mut docu) => {
            if let Ok(pipeline) = docu.get_array("pipeline") {
                let nested = pipeline
                    .iter()
                    .filter_map(|item| item.as_document().cloned())
                    .collect::<Vec<_>>();
                docu.insert(
                    "pipeline",
                    Bson::Array(
                        enforce_no_deleted_in_pipeline(&nested)
                            .into_iter()
                            .map(Bson::Document)
                            .collect(),
                    ),
                );
            }
            Bson::Document(docu)
        }
        other => other,
    }
}

fn rewrite_union_with_scope(value: Bson, owner: &str) -> Result<Bson, K2DbError> {
    Ok(match value {
        Bson::String(coll) => Bson::Document(doc! {
            "coll": coll,
            "pipeline": [
                { "$match": { "_owner": owner } },
                { "$match": { "_deleted": { "$ne": true } } }
            ]
        }),
        Bson::Document(mut docu) => {
            if let Ok(pipeline) = docu.get_array("pipeline") {
                let nested = pipeline
                    .iter()
                    .filter_map(|item| item.as_document().cloned())
                    .collect::<Vec<_>>();
                docu.insert(
                    "pipeline",
                    Bson::Array(
                        enforce_scope_in_pipeline(&nested, Some(&Scope::owner(owner)), OwnershipMode::Lax)?
                        .into_iter()
                        .map(Bson::Document)
                        .collect(),
                    ),
                );
            }
            Bson::Document(docu)
        }
        other => other,
    })
}
