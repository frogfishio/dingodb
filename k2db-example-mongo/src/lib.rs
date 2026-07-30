// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

pub mod aggregate;
pub mod client_pool;
pub mod config;
pub mod db;
pub mod document;
pub mod error;
pub mod observability;
pub mod options;
pub mod ops;
pub mod results;
pub mod schema;
pub mod secure_fields;
pub mod transaction;
pub mod criteria;
pub mod scope;

pub type Doc = mongodb::bson::Document;

pub use ratatouille;
pub use config::{AggregationMode, DatabaseConfig, EncryptionConfig, HostConfig, OwnershipMode};
pub use db::{K2Db, ScopedK2Db};
pub use error::{K2DbError, ServiceError};
pub use observability::{QueryHooks, RatatouilleLogger};
pub use options::{EnsureIndexesOptions, FindOptions, ProjectionMode};
pub use ops::{AggregateOp, CountOp, CreateOp, DeleteManyOp, FindOneOp, FindOp, GetOp, RestoreOp, UpdateManyOp, UpdateOneOp, VersionedUpdateOp};
pub use results::{CountResult, CreateResult, DeleteResult, DropResult, PurgeManyResult, PurgeResult, RestoreResult, UpdateResult, VersionInfo, VersionedUpdateResult};
pub use schema::SchemaMode;
pub use scope::Scope;
pub use transaction::{TransactionContext, TransactionFuture};