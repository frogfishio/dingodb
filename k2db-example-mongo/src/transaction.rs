// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use futures_util::future::BoxFuture;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, Bson, Document};
use mongodb::ClientSession;

use crate::criteria::{apply_default_non_deleted, apply_scope, normalize_document_ids};
use crate::db::K2Db;
use crate::document::{generate_k2_id, normalize_id, require_owner, strip_reserved_fields};
use crate::error::K2DbError;
use crate::error::ServiceError;
use crate::options::FindOptions;
use crate::results::{CountResult, CreateResult, DeleteResult, RestoreResult, UpdateResult};
use crate::scope::Scope;

pub type TransactionFuture<'a, T> = BoxFuture<'a, Result<T, K2DbError>>;

/// Transaction-scoped accessors exposed inside `execute_transaction` closures.
///
/// ```no_run
/// use k2db::K2Db;
/// use mongodb::bson::doc;
///
/// # async fn demo(db: &K2Db) -> Result<(), k2db::K2DbError> {
/// db.execute_transaction(|mut tx| {
///     Box::pin(async move {
///         tx.create("widgets", "owner1", doc! { "name": "example" }).await?;
///         Ok::<(), k2db::K2DbError>(())
///     })
/// })
/// .await?;
/// # Ok(())
/// # }
/// ```
pub struct TransactionContext<'a> {
    db: &'a K2Db,
    session: &'a mut ClientSession,
    scope: Option<Scope>,
}

impl<'a> TransactionContext<'a> {
    pub(crate) fn new(
        db: &'a K2Db,
        session: &'a mut ClientSession,
        scope: Option<Scope>,
    ) -> Self {
        Self { db, session, scope }
    }

    pub fn session_mut(&mut self) -> &mut ClientSession {
        self.session
    }

    pub fn scope(&self) -> Option<&Scope> {
        self.scope.as_ref()
    }

    pub async fn create(
        &mut self,
        collection_name: &str,
        owner: &str,
        data: Document,
    ) -> Result<CreateResult, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let owner = require_owner(owner)?;
        let collection = self.db.collection(collection_name).await?;
        let uuid = generate_k2_id();
        let now = K2Db::now_ms()?;

        let mut document = strip_reserved_fields(&data);
        document = self.db.apply_schema(collection_name, document, false)?;
        document.insert("_uuid", uuid.clone());
        document.insert("_owner", owner);
        document.insert("_created", now);
        document.insert("_updated", now);
        let document = self.db.encrypt_document(collection_name, &uuid, document)?;

        collection
            .insert_one(document)
            .session(&mut *self.session)
            .await
            .map_err(|error| {
                if K2Db::is_duplicate_uuid_error(&error) {
                    return K2DbError::new(
                        ServiceError::AlreadyExists,
                        format!("A document with _uuid {uuid} already exists."),
                        Some("sys_mdb_crv3".to_owned()),
                    );
                }

                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_sav".to_owned()),
                    "Error saving object to database",
                )
            })?;

        Ok(CreateResult { id: uuid })
    }

    pub async fn get(
        &mut self,
        collection_name: &str,
        id: &str,
    ) -> Result<Document, K2DbError> {
        let id = normalize_id(id).to_ascii_uppercase();
        let criteria = doc! { "_uuid": id };

        self.find_one(collection_name, criteria, None)
            .await?
            .ok_or_else(|| {
                K2DbError::new(
                    ServiceError::NotFound,
                    "Document not found",
                    Some("sys_mdb_get_not_found".to_owned()),
                )
            })
    }

    pub async fn find_one(
        &mut self,
        collection_name: &str,
        criteria: Document,
        fields: Option<Vec<String>>,
    ) -> Result<Option<Document>, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let collection = self.db.collection(collection_name).await?;
        let mut query = normalize_document_ids(criteria);
        if !query.contains_key("_deleted") {
            query = apply_default_non_deleted(query);
        }
        query = apply_scope(query, self.scope.as_ref(), self.db.config().ownership_mode)?;

        let requested_uuid = fields
            .as_ref()
            .map(|items| items.iter().any(|field| field.trim() == "_uuid"))
            .unwrap_or(false);
        let mut projection = self.db.build_projection(fields.as_deref())?;
        if !projection.is_empty()
            && crate::secure_fields::has_secure_encryption(self.db.config().secure_field_encryption.as_ref())
        {
            projection.insert("_uuid", 1);
        }

        let mut action = collection.find_one(query);
        if !projection.is_empty() {
            action = action.projection(projection);
        }

        let found = action.session(&mut *self.session).await.map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::SystemError,
                Some("sys_mdb_fo".to_owned()),
                "Error finding document",
            )
        })?;

        found
            .map(|document| {
                let mut document = self.db.sanitize_single_read(collection_name, document)?;
                if fields.is_some()
                    && crate::secure_fields::has_secure_encryption(self.db.config().secure_field_encryption.as_ref())
                    && !requested_uuid
                {
                    document.remove("_uuid");
                }
                Ok(document)
            })
            .transpose()
    }

    pub async fn count(
        &mut self,
        collection_name: &str,
        criteria: Document,
    ) -> Result<CountResult, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let collection = self.db.collection(collection_name).await?;
        let mut query = normalize_document_ids(criteria);
        if !query.contains_key("_deleted") {
            query = apply_default_non_deleted(query);
        }
        query = apply_scope(query, self.scope.as_ref(), self.db.config().ownership_mode)?;

        let count = collection
            .count_documents(query)
            .session(&mut *self.session)
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_cn".to_owned()),
                    "Error counting objects with given criteria",
                )
            })?;

        Ok(CountResult { count })
    }

    pub async fn find(
        &mut self,
        collection_name: &str,
        criteria: Document,
        options: FindOptions,
    ) -> Result<Vec<Document>, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let collection = self.db.collection(collection_name).await?;

        let mut query = normalize_document_ids(criteria);
        query = self.db.apply_deleted_policy(query, &options);
        query = apply_scope(query, self.scope.as_ref(), self.db.config().ownership_mode)?;

        let projection = self.db.build_find_projection(&options.projection)?;
        let mut action = collection.find(query);
        if !projection.is_empty() {
            action = action.projection(projection);
        }
        if let Some(sort) = options.sort {
            action = action.sort(sort);
        }
        if options.skip > 0 {
            action = action.skip(options.skip);
        }
        if options.limit > 0 {
            action = action.limit(options.limit as i64);
        }

        let mut cursor = action.session(&mut *self.session).await.map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::SystemError,
                Some("sys_mdb_find_error".to_owned()),
                "Error executing find query",
            )
        })?;

        let documents: Vec<Document> = cursor
            .stream(&mut *self.session)
            .try_collect()
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_find_error".to_owned()),
                    "Error executing find query",
                )
            })?;

        Ok(documents
            .into_iter()
            .map(|document| self.db.sanitize_multi_read(document))
            .collect())
    }

    pub async fn update(
        &mut self,
        collection_name: &str,
        id: &str,
        data: Document,
        replace: bool,
    ) -> Result<UpdateResult, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let collection = self.db.collection(collection_name).await?;
        let normalized_id = normalize_id(id);
        let mut updates = strip_reserved_fields(&data);
        updates = self.db.apply_schema(collection_name, updates, !replace)?;
        updates.insert("_updated", K2Db::now_ms()?);
        let encrypted_updates = self
            .db
            .encrypt_document(collection_name, &normalized_id, updates.clone())?;

        let filter = apply_scope(
            doc! {
                "_uuid": normalized_id.clone(),
                "_deleted": { "$ne": true }
            },
            self.scope.as_ref(),
            self.db.config().ownership_mode,
        )?;

        let result = if replace {
            let original = self.get(collection_name, &normalized_id).await?;
            let mut replacement = encrypted_updates;
            for (key, value) in original.iter() {
                if key.starts_with('_') {
                    replacement.insert(key.clone(), value.clone());
                }
            }

            collection
                .replace_one(filter, replacement)
                .session(&mut *self.session)
                .await
                .map_err(|error| {
                    K2DbError::wrap(
                        error,
                        ServiceError::SystemError,
                        Some("sys_mdb_update_error".to_owned()),
                        format!("Error updating {collection_name}"),
                    )
                })?
        } else {
            collection
                .update_one(filter, doc! { "$set": encrypted_updates })
                .session(&mut *self.session)
                .await
                .map_err(|error| {
                    K2DbError::wrap(
                        error,
                        ServiceError::SystemError,
                        Some("sys_mdb_update_error".to_owned()),
                        format!("Error updating {collection_name}"),
                    )
                })?
        };

        if result.matched_count == 0 {
            return Err(K2DbError::new(
                ServiceError::NotFound,
                format!("Object in {collection_name} with UUID {normalized_id} not found"),
                Some("sys_mdb_update_not_found".to_owned()),
            ));
        }

        Ok(UpdateResult {
            updated: result.modified_count,
        })
    }

    pub async fn delete(
        &mut self,
        collection_name: &str,
        id: &str,
    ) -> Result<DeleteResult, K2DbError> {
        let result = self
            .delete_all(collection_name, doc! { "_uuid": normalize_id(id) })
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_remove_upd".to_owned()),
                    "Error removing object from collection",
                )
            })?;

        match result.deleted {
            1 => Ok(result),
            0 => Err(K2DbError::new(
                ServiceError::NotFound,
                "Document not found",
                Some("sys_mdb_remove_not_found".to_owned()),
            )),
            _ => Err(K2DbError::new(
                ServiceError::SystemError,
                "Multiple documents deleted when only one was expected",
                Some("sys_mdb_remove_multiple_deleted".to_owned()),
            )),
        }
    }

    pub async fn delete_all(
        &mut self,
        collection_name: &str,
        criteria: Document,
    ) -> Result<DeleteResult, K2DbError> {
        let result = self
            .update_all(collection_name, criteria, doc! { "_deleted": true })
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_deleteall_update".to_owned()),
                    format!("Error deleting from {collection_name}"),
                )
            })?;

        Ok(DeleteResult {
            deleted: result.updated,
        })
    }

    pub async fn update_all(
        &mut self,
        collection_name: &str,
        criteria: Document,
        values: Document,
    ) -> Result<UpdateResult, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let collection = self.db.collection(collection_name).await?;

        let deleted_flag = values.get("_deleted").cloned();
        let mut updates = strip_reserved_fields(&values);
        updates = self.db.apply_schema(collection_name, updates, true)?;
        if let Some(deleted_flag) = deleted_flag {
            updates.insert("_deleted", deleted_flag);
        }
        updates.insert("_updated", K2Db::now_ms()?);
        let updates = self
            .db
            .encrypt_document_for_collection(collection_name, updates)?;

        let mut query = normalize_document_ids(criteria);
        query = apply_scope(query, self.scope.as_ref(), self.db.config().ownership_mode)?;
        query.insert("_deleted", Bson::Document(doc! { "$ne": true }));

        let result = collection
            .update_many(query, doc! { "$set": updates })
            .session(&mut *self.session)
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_update1".to_owned()),
                    format!("Error updating {collection_name}"),
                )
            })?;

        Ok(UpdateResult {
            updated: result.modified_count,
        })
    }

    pub async fn restore(
        &mut self,
        collection_name: &str,
        criteria: Document,
    ) -> Result<RestoreResult, K2DbError> {
        self.db.validate_collection_name(collection_name)?;
        let collection = self.db.collection(collection_name).await?;
        let mut query = normalize_document_ids(criteria);
        query = apply_scope(query, self.scope.as_ref(), self.db.config().ownership_mode)?;
        query.insert("_deleted", true);

        let result = collection
            .update_many(
                query,
                doc! {
                    "$set": {
                        "_deleted": false,
                        "_updated": K2Db::now_ms()?
                    }
                },
            )
            .session(&mut *self.session)
            .await
            .map_err(|error| {
                K2DbError::wrap(
                    error,
                    ServiceError::SystemError,
                    Some("sys_mdb_pres".to_owned()),
                    "Error restoring a deleted item",
                )
            })?;

        Ok(RestoreResult {
            status: "restored".to_owned(),
            modified: result.modified_count,
        })
    }
}