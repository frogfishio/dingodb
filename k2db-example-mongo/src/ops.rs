// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use mongodb::bson::Document;

use crate::options::FindOptions;
use crate::scope::Scope;

/// Typed input for `create_with`.
///
/// ```no_run
/// use k2db::CreateOp;
/// use mongodb::bson::doc;
///
/// let op = CreateOp {
///     collection: "widgets".to_owned(),
///     owner: "alice".to_owned(),
///     data: doc! { "name": "widget" },
/// };
/// # let _ = op;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOp {
    pub collection: String,
    pub owner: String,
    pub data: Document,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetOp {
    pub collection: String,
    pub id: String,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FindOneOp {
    pub collection: String,
    pub criteria: Document,
    pub fields: Option<Vec<String>>,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountOp {
    pub collection: String,
    pub criteria: Document,
    pub scope: Option<Scope>,
}

/// Typed input for `find_with`.
///
/// ```no_run
/// use k2db::{FindOp, FindOptions, Scope};
/// use mongodb::bson::doc;
///
/// let op = FindOp {
///     collection: "widgets".to_owned(),
///     criteria: doc! { "status": "active" },
///     options: FindOptions::default(),
///     scope: Some(Scope::owner("alice")),
/// };
/// # let _ = op;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FindOp {
    pub collection: String,
    pub criteria: Document,
    pub options: FindOptions,
    pub scope: Option<Scope>,
}

/// Typed input for `aggregate_with`.
///
/// ```no_run
/// use k2db::{AggregateOp, Scope};
/// use mongodb::bson::doc;
///
/// let op = AggregateOp {
///     collection: "widgets".to_owned(),
///     pipeline: vec![doc! { "$match": { "status": "active" } }],
///     skip: 0,
///     limit: 25,
///     scope: Some(Scope::owner("alice")),
/// };
/// # let _ = op;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateOp {
    pub collection: String,
    pub pipeline: Vec<Document>,
    pub skip: u64,
    pub limit: u64,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateOneOp {
    pub collection: String,
    pub id: String,
    pub data: Document,
    pub replace: bool,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateManyOp {
    pub collection: String,
    pub criteria: Document,
    pub values: Document,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteManyOp {
    pub collection: String,
    pub criteria: Document,
    pub scope: Option<Scope>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestoreOp {
    pub collection: String,
    pub criteria: Document,
    pub scope: Option<Scope>,
}

/// Typed input for `update_versioned_with`.
///
/// ```no_run
/// use k2db::{Scope, VersionedUpdateOp};
/// use mongodb::bson::doc;
///
/// let op = VersionedUpdateOp {
///     collection: "widgets".to_owned(),
///     id: "0194f3f3-8a34-79d0-bd03-59a59c4f3f11".to_owned(),
///     data: doc! { "name": "updated widget" },
///     replace: false,
///     max_versions: Some(10),
///     scope: Some(Scope::owner("alice")),
/// };
/// # let _ = op;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct VersionedUpdateOp {
    pub collection: String,
    pub id: String,
    pub data: Document,
    pub replace: bool,
    pub max_versions: Option<u64>,
    pub scope: Option<Scope>,
}