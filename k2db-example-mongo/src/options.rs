// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use mongodb::bson::Document;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProjectionMode {
    #[default]
    Default,
    All,
    Include(Vec<String>),
    Exclude(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FindOptions {
    pub projection: ProjectionMode,
    pub sort: Option<Document>,
    pub skip: u64,
    pub limit: u64,
    pub include_deleted: bool,
    pub deleted_only: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            projection: ProjectionMode::Default,
            sort: None,
            skip: 0,
            limit: 100,
            include_deleted: false,
            deleted_only: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnsureIndexesOptions {
    pub uuid_unique: bool,
    pub uuid_partial_unique: bool,
    pub owner_index: bool,
    pub deleted_index: bool,
}

impl Default for EnsureIndexesOptions {
    fn default() -> Self {
        Self {
            uuid_unique: false,
            uuid_partial_unique: true,
            owner_index: true,
            deleted_index: true,
        }
    }
}