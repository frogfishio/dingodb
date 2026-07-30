// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::error::Error as _;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use std::sync::RwLock as StdRwLock;

use futures_util::TryStreamExt;
use mongodb::{bson::{doc, Bson, Document}, Collection, Client, Database};
use mongodb::options::IndexOptions;
use mongodb::IndexModel;
use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::aggregate::{
    assert_no_secure_field_refs_in_pipeline, enforce_no_deleted_in_pipeline,
    enforce_scope_in_pipeline, sanitize_pipeline_matches, validate_pipeline,
};
use crate::client_pool::{self, ClientCacheKey};
use crate::config::DatabaseConfig;
use crate::criteria::{apply_default_non_deleted, apply_scope, normalize_document_ids};
use crate::document::{generate_k2_id, normalize_id, require_owner, secure_projection_allowed, strip_reserved_fields, strip_secure_fields_deep, without_mongo_id};
use crate::error::{K2DbError, ServiceError};
use crate::options::{EnsureIndexesOptions, FindOptions, ProjectionMode};
use crate::ops::{AggregateOp, CountOp, CreateOp, DeleteManyOp, FindOneOp, FindOp, GetOp, RestoreOp, UpdateManyOp, UpdateOneOp, VersionedUpdateOp};
use crate::observability::RatatouilleLogger;
use crate::results::{CountResult, CreateResult, DeleteResult, DropResult, PurgeManyResult, PurgeResult, RestoreResult, UpdateResult, VersionInfo, VersionedUpdateResult};
use crate::schema::{RegisteredSchema, SchemaMode};
use crate::secure_fields::{decrypt_secure_fields_deep, encrypt_secure_fields_deep, has_secure_encryption};
use crate::scope::Scope;
use crate::transaction::{TransactionContext, TransactionFuture};

#[path = "db/admin.rs"]
mod admin;
#[path = "db/crud.rs"]
mod crud;
#[path = "db/scoped.rs"]
mod scoped;
#[path = "db/versioning.rs"]
mod versioning;

#[derive(Debug)]
struct State {
    client: Option<Arc<Client>>,
    database: Option<Database>,
    client_key: Option<ClientCacheKey>,
}

impl State {
    fn empty() -> Self {
        Self {
            client: None,
            database: None,
            client_key: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct InnerDb {
    pub(crate) config: DatabaseConfig,
    state: RwLock<State>,
    schemas: StdRwLock<HashMap<String, RegisteredSchema>>,
    logger: StdRwLock<Option<RatatouilleLogger>>,
}

#[derive(Debug, Clone)]
pub struct K2Db {
    inner: Arc<InnerDb>,
}

#[derive(Debug, Clone)]
pub struct ScopedK2Db {
    db: K2Db,
    scope: Scope,
}

impl K2Db {
    pub fn new(config: DatabaseConfig) -> Result<Self, K2DbError> {
        config.validate()?;
        Ok(Self {
            inner: Arc::new(InnerDb {
                config,
                state: RwLock::new(State::empty()),
                schemas: StdRwLock::new(HashMap::new()),
                logger: StdRwLock::new(None),
            }),
        })
    }

    /// Attaches the canonical Ratatouille logger used for `k2db:debug` and
    /// `k2db:error` emission.
    ///
    /// ```
    /// use k2db::{
    ///     AggregationMode, DatabaseConfig, HostConfig, K2Db, OwnershipMode,
    ///     QueryHooks, RatatouilleLogger, ratatouille,
    /// };
    ///
    /// let config = DatabaseConfig {
    ///     name: "example".to_owned(),
    ///     hosts: vec![HostConfig {
    ///         host: "localhost".to_owned(),
    ///         port: Some(27017),
    ///     }],
    ///     user: None,
    ///     password: None,
    ///     auth_source: None,
    ///     replica_set: None,
    ///     slow_query_ms: Some(25),
    ///     ownership_mode: OwnershipMode::default(),
    ///     aggregation_mode: AggregationMode::default(),
    ///     secure_field_prefixes: Vec::new(),
    ///     secure_field_encryption: None,
    ///     hooks: QueryHooks::default()
    ///         .with_before_query(|op, details| {
    ///             let _ = (op, details);
    ///         })
    ///         .with_after_query(|op, details, duration_ms| {
    ///             let _ = (op, details, duration_ms);
    ///         }),
    /// };
    ///
    /// let db = K2Db::new(config).unwrap();
    /// db.set_logger(RatatouilleLogger::stdout(ratatouille::LoggerConfig {
    ///     filter: Some("*".to_owned()),
    ///     ..Default::default()
    /// }))
    /// .unwrap();
    /// ```
    pub fn set_logger(&self, logger: RatatouilleLogger) -> Result<(), K2DbError> {
        let mut slot = self.inner.logger.write().map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Logger registry lock poisoned",
                Some("sys_mdb_logger_registry".to_owned()),
            )
        })?;
        *slot = Some(logger);
        Ok(())
    }

    /// Clears the currently attached Ratatouille logger.
    pub fn clear_logger(&self) -> Result<(), K2DbError> {
        let mut slot = self.inner.logger.write().map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Logger registry lock poisoned",
                Some("sys_mdb_logger_registry".to_owned()),
            )
        })?;
        *slot = None;
        Ok(())
    }

    pub fn set_schema(
        &self,
        collection_name: &str,
        schema: serde_json::Value,
        mode: SchemaMode,
    ) -> Result<(), K2DbError> {
        self.validate_collection_name(collection_name)?;
        let registered = RegisteredSchema::compile(schema, mode)?;
        let mut schemas = self.inner.schemas.write().map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Schema registry lock poisoned",
                Some("sys_mdb_schema_registry".to_owned()),
            )
        })?;
        schemas.insert(collection_name.to_owned(), registered);
        Ok(())
    }

    pub fn clear_schema(&self, collection_name: &str) -> Result<(), K2DbError> {
        self.validate_collection_name(collection_name)?;
        let mut schemas = self.inner.schemas.write().map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Schema registry lock poisoned",
                Some("sys_mdb_schema_registry".to_owned()),
            )
        })?;
        schemas.remove(collection_name);
        Ok(())
    }

    pub fn clear_schemas(&self) -> Result<(), K2DbError> {
        let mut schemas = self.inner.schemas.write().map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Schema registry lock poisoned",
                Some("sys_mdb_schema_registry".to_owned()),
            )
        })?;
        schemas.clear();
        Ok(())
    }

    pub async fn connect(config: DatabaseConfig) -> Result<Self, K2DbError> {
        let db = Self::new(config)?;
        db.init().await?;
        Ok(db)
    }

    pub fn with_scope(&self, scope: Scope) -> ScopedK2Db {
        ScopedK2Db {
            db: self.clone(),
            scope,
        }
    }

    pub async fn init(&self) -> Result<(), K2DbError> {
        {
            let state = self.inner.state.read().await;
            if state.database.is_some() {
                return Ok(());
            }
        }

        let (client_key, client) = client_pool::acquire(&self.inner.config).await?;
        let database = client.database(&self.inner.config.name);

        let mut state = self.inner.state.write().await;
        if state.database.is_none() {
            state.client_key = Some(client_key);
            state.database = Some(database);
            state.client = Some(client);
        }

        Ok(())
    }

    pub async fn release(&self) -> Result<(), K2DbError> {
        let key = {
            let mut state = self.inner.state.write().await;
            state.database = None;
            state.client = None;
            state.client_key.take()
        };

        if let Some(key) = key {
            client_pool::release(&key).await;
        }

        Ok(())
    }

    pub async fn is_healthy(&self) -> bool {
        self.database().await.is_ok()
    }

    pub fn config(&self) -> &DatabaseConfig {
        &self.inner.config
    }

    pub(crate) fn emit_ratatouille(&self, topic: &str, payload: &Value) {
        let Ok(slot) = self.inner.logger.read() else {
            return;
        };
        let Some(logger) = slot.as_ref() else {
            return;
        };
        let Ok(line) = serde_json::to_string(payload) else {
            return;
        };
        let _ = logger.log(topic, &line);
    }

    pub(crate) fn emit_db_error(&self, error: &K2DbError, meta: Value) {
        let mut payload = json!({
            "kind": "k2db:error",
            "at": Self::epoch_ms_now(),
            "k2": {
                "error": error.service_error,
                "key": error.key,
                "error_description": error.message,
                "context": error.context,
            },
            "sensitive": error.sensitive,
            "source": error.source().map(|source| source.to_string()),
        });

        if let (Value::Object(payload_obj), Value::Object(meta_obj)) = (&mut payload, meta) {
            for (key, value) in meta_obj {
                payload_obj.insert(key, value);
            }
        }

        self.emit_ratatouille("k2db:error", &payload);
    }

    pub(crate) async fn run_timed<T, F, Fut>(
        &self,
        op: &str,
        details: Value,
        action: F,
    ) -> Result<T, K2DbError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, K2DbError>>,
    {
        self.inner.config.hooks.emit_before_query(op, &details);
        let start = std::time::Instant::now();

        match action().await {
            Ok(value) => {
                let elapsed = start.elapsed();
                let duration_ms = elapsed.as_millis() as u64;
                let duration_ms = if duration_ms == 0 && !elapsed.is_zero() { 1 } else { duration_ms };
                let slow_ms = self.inner.config.slow_query_ms.unwrap_or(200);
                if duration_ms > slow_ms {
                    self.emit_ratatouille(
                        "k2db:debug",
                        &json!({
                            "kind": "k2db:slow_query",
                            "at": Self::epoch_ms_now(),
                            "op": op,
                            "duration_ms": duration_ms,
                            "details": details,
                        }),
                    );
                }

                let mut after = details.clone();
                if let Value::Object(obj) = &mut after {
                    obj.insert("ok".to_owned(), Value::Bool(true));
                }
                self.inner.config.hooks.emit_after_query(op, &after, duration_ms);
                Ok(value)
            }
            Err(error) => {
                let elapsed = start.elapsed();
                let duration_ms = elapsed.as_millis() as u64;
                let duration_ms = if duration_ms == 0 && !elapsed.is_zero() { 1 } else { duration_ms };
                let mut after = details.clone();
                if let Value::Object(obj) = &mut after {
                    obj.insert("ok".to_owned(), Value::Bool(false));
                    obj.insert("error".to_owned(), Value::String(error.message.clone()));
                }
                self.inner.config.hooks.emit_after_query(op, &after, duration_ms);
                Err(error)
            }
        }
    }

    pub fn validate_collection_name(&self, collection_name: &str) -> Result<(), K2DbError> {
        if collection_name.contains('\0') {
            return Err(K2DbError::new(
                ServiceError::BadRequest,
                "Collection name cannot contain null characters",
                Some("sys_mdb_invalid_collection_name".to_owned()),
            ));
        }

        if collection_name.starts_with("system.") {
            return Err(K2DbError::new(
                ServiceError::BadRequest,
                "Collection name cannot start with 'system.'",
                Some("sys_mdb_invalid_collection_name".to_owned()),
            ));
        }

        if collection_name.contains('$') {
            return Err(K2DbError::new(
                ServiceError::BadRequest,
                "Collection name cannot contain the '$' character",
                Some("sys_mdb_invalid_collection_name".to_owned()),
            ));
        }

        Ok(())
    }

    pub(crate) async fn database(&self) -> Result<Database, K2DbError> {
        self.init().await?;

        let state = self.inner.state.read().await;
        state.database.clone().ok_or_else(|| {
            K2DbError::new(
                ServiceError::SystemError,
                "Database handle is not initialized",
                Some("sys_mdb_init".to_owned()),
            )
        })
    }

    pub(crate) async fn collection(&self, collection_name: &str) -> Result<Collection<Document>, K2DbError> {
        let database = self.database().await?;
        Ok(database.collection::<Document>(collection_name))
    }

    async fn history_collection(&self, collection_name: &str) -> Result<Collection<Document>, K2DbError> {
        let database = self.database().await?;
        Ok(database.collection::<Document>(&self.history_name(collection_name)))
    }

    fn history_name(&self, collection_name: &str) -> String {
        format!("{collection_name}__history")
    }

    pub(crate) fn build_projection(&self, fields: Option<&[String]>) -> Result<Document, K2DbError> {
        let Some(fields) = fields else {
            return Ok(Document::new());
        };

        let mut projection = Document::new();
        for field in fields {
            let trimmed = field.trim();
            if trimmed.is_empty() {
                continue;
            }

            if !secure_projection_allowed(trimmed, &self.inner.config.secure_field_prefixes) {
                return Err(K2DbError::new(
                    ServiceError::BadRequest,
                    "Projection cannot include secure-prefixed field(s)",
                    Some("sys_mdb_projection_secure_field".to_owned()),
                ));
            }

            projection.insert(trimmed, 1);
        }

        Ok(projection)
    }

    pub(crate) fn build_find_projection(&self, mode: &ProjectionMode) -> Result<Document, K2DbError> {
        match mode {
            ProjectionMode::Default => Ok(doc! { "_id": 0 }),
            ProjectionMode::All => Ok(Document::new()),
            ProjectionMode::Include(fields) => {
                let mut projection = self.build_projection(Some(fields.as_slice()))?;
                projection.insert("_id", 0);
                Ok(projection)
            }
            ProjectionMode::Exclude(fields) => {
                let mut projection = doc! { "_id": 0 };
                for field in fields {
                    let trimmed = field.trim();
                    if !trimmed.is_empty() {
                        projection.insert(trimmed, 0);
                    }
                }
                Ok(projection)
            }
        }
    }

    pub(crate) fn apply_deleted_policy(&self, mut criteria: Document, options: &FindOptions) -> Document {
        if options.include_deleted || criteria.contains_key("_deleted") {
            return criteria;
        }

        if options.deleted_only {
            criteria.insert("_deleted", true);
            return criteria;
        }

        apply_default_non_deleted(criteria)
    }

    pub(crate) fn apply_schema(&self, collection_name: &str, document: Document, partial: bool) -> Result<Document, K2DbError> {
        let schemas = self.inner.schemas.read().map_err(|_| {
            K2DbError::new(
                ServiceError::SystemError,
                "Schema registry lock poisoned",
                Some("sys_mdb_schema_registry".to_owned()),
            )
        })?;
        let Some(schema) = schemas.get(collection_name) else {
            return Ok(document);
        };
        schema.apply(&document, partial)
    }

    pub(crate) fn sanitize_single_read(&self, collection_name: &str, document: Document) -> Result<Document, K2DbError> {
        let stripped = without_mongo_id(document);
        let decrypted = self.decrypt_document(collection_name, stripped)?;
        match decrypted {
            Bson::Document(document) => Ok(document),
            _ => unreachable!("document sanitation must remain a document"),
        }
    }

    pub(crate) fn sanitize_multi_read(&self, document: Document) -> Document {
        let stripped = without_mongo_id(document);
        match strip_secure_fields_deep(Bson::Document(stripped), &self.inner.config.secure_field_prefixes) {
            Bson::Document(document) => document,
            _ => unreachable!("document sanitation must remain a document"),
        }
    }

    pub(crate) fn encrypt_document(&self, collection_name: &str, id: &str, document: Document) -> Result<Document, K2DbError> {
        if !has_secure_encryption(self.inner.config.secure_field_encryption.as_ref()) {
            return Ok(document);
        }

        let aad_prefix = format!("k2db|{collection_name}|{id}");
        match encrypt_secure_fields_deep(
            Bson::Document(document),
            &self.inner.config.secure_field_prefixes,
            self.inner.config.secure_field_encryption.as_ref(),
            &aad_prefix,
        )? {
            Bson::Document(document) => Ok(document),
            _ => unreachable!("document encryption must remain a document"),
        }
    }

    pub(crate) fn encrypt_document_for_collection(&self, collection_name: &str, document: Document) -> Result<Document, K2DbError> {
        if !has_secure_encryption(self.inner.config.secure_field_encryption.as_ref()) {
            return Ok(document);
        }

        let aad_prefix = format!("k2db|{collection_name}");
        match encrypt_secure_fields_deep(
            Bson::Document(document),
            &self.inner.config.secure_field_prefixes,
            self.inner.config.secure_field_encryption.as_ref(),
            &aad_prefix,
        )? {
            Bson::Document(document) => Ok(document),
            _ => unreachable!("document encryption must remain a document"),
        }
    }

    fn decrypt_document(&self, collection_name: &str, document: Document) -> Result<Bson, K2DbError> {
        if !has_secure_encryption(self.inner.config.secure_field_encryption.as_ref()) {
            return Ok(Bson::Document(document));
        }

        let mut aad_prefixes = vec![format!("k2db|{collection_name}")];
        if let Ok(id) = document.get_str("_uuid") {
            aad_prefixes.insert(0, format!("k2db|{collection_name}|{id}"));
        }

        decrypt_secure_fields_deep(
            Bson::Document(document),
            &self.inner.config.secure_field_prefixes,
            self.inner.config.secure_field_encryption.as_ref(),
            &aad_prefixes,
        )
    }

    fn decrypt_snapshot(&self, collection_name: &str, id: &str, snapshot: Document) -> Result<Document, K2DbError> {
        if !has_secure_encryption(self.inner.config.secure_field_encryption.as_ref()) {
            return Ok(snapshot);
        }

        let aad_prefixes = vec![format!("k2db|{collection_name}|{id}"), format!("k2db|{collection_name}")];
        match decrypt_secure_fields_deep(
            Bson::Document(snapshot),
            &self.inner.config.secure_field_prefixes,
            self.inner.config.secure_field_encryption.as_ref(),
            &aad_prefixes,
        )? {
            Bson::Document(document) => Ok(document),
            _ => unreachable!("snapshot decryption must remain a document"),
        }
    }

    pub(crate) fn now_ms() -> Result<i64, K2DbError> {
        let duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_sav".to_owned()),
                    "System clock error",
                )
            })?;
        Ok(duration.as_millis() as i64)
    }

    fn epoch_ms_now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub(crate) fn is_duplicate_uuid_error(error: &mongodb::error::Error) -> bool {
        let message = error.to_string();
        message.contains("11000") && message.contains("_uuid")
    }

}

impl Drop for K2Db {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            let key = self
                .inner
                .state
                .try_read()
                .ok()
                .and_then(|state| state.client_key.clone());

            if let Some(key) = key {
                tokio::spawn(async move {
                    client_pool::release(&key).await;
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::K2Db;
    use crate::client_pool::ClientCacheKey;
    use crate::config::{AggregationMode, DatabaseConfig, HostConfig, OwnershipMode};
    use crate::observability::QueryHooks;

    fn config(name: &str) -> DatabaseConfig {
        DatabaseConfig {
            name: name.to_owned(),
            hosts: vec![HostConfig {
                host: "localhost".to_owned(),
                port: Some(27017),
            }],
            user: Some("user".to_owned()),
            password: Some("pass".to_owned()),
            auth_source: Some("admin".to_owned()),
            replica_set: Some("rs0".to_owned()),
            slow_query_ms: Some(200),
            ownership_mode: OwnershipMode::Lax,
            aggregation_mode: AggregationMode::Loose,
            secure_field_prefixes: Vec::new(),
            secure_field_encryption: None,
            hooks: QueryHooks::default(),
        }
    }

    #[test]
    fn pool_key_excludes_database_name() {
        let a = ClientCacheKey::build(&config("a"));
        let b = ClientCacheKey::build(&config("b"));
        assert_eq!(a, b);
    }

    #[test]
    fn constructor_validates_config() {
        let db = K2Db::new(config("k2db"));
        assert!(db.is_ok());
    }
}