// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use super::*;
use serde_json::json;

impl K2Db {
    pub async fn create(
        &self,
        collection_name: &str,
        owner: &str,
        data: Document,
    ) -> Result<CreateResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let owner = require_owner(owner)?;
            let collection = self.collection(collection_name).await?;
            let uuid = generate_k2_id();
            let now = Self::now_ms()?;

            let mut document = strip_reserved_fields(&data);
            document = self.apply_schema(collection_name, document, false)?;
            document.insert("_uuid", uuid.clone());
            document.insert("_owner", owner.clone());
            document.insert("_created", now);
            document.insert("_updated", now);
            let document = self.encrypt_document(collection_name, &uuid, document)?;

            self.run_timed(
                "insertOne",
                json!({ "collectionName": collection_name }),
                || async {
                    collection.insert_one(document.clone()).await.map_err(|error| {
                        if Self::is_duplicate_uuid_error(&error) {
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
                    Ok(CreateResult { id: uuid.clone() })
                },
            )
            .await
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "create", "collection": collection_name }),
            );
            error
        })
    }

    pub async fn create_with(&self, op: CreateOp) -> Result<CreateResult, K2DbError> {
        self.create(&op.collection, &op.owner, op.data).await
    }

    pub async fn get(
        &self,
        collection_name: &str,
        id: &str,
        scope: Option<&Scope>,
    ) -> Result<Document, K2DbError> {
        let id = normalize_id(id).to_ascii_uppercase();
        let criteria = doc! { "_uuid": id };

        self.find_one(collection_name, criteria, None, scope)
            .await?
            .ok_or_else(|| {
                K2DbError::new(
                    ServiceError::NotFound,
                    "Document not found",
                    Some("sys_mdb_get_not_found".to_owned()),
                )
            })
    }

    pub async fn get_with(&self, op: GetOp) -> Result<Document, K2DbError> {
        self.get(&op.collection, &op.id, op.scope.as_ref()).await
    }

    pub async fn find_one(
        &self,
        collection_name: &str,
        criteria: Document,
        fields: Option<Vec<String>>,
        scope: Option<&Scope>,
    ) -> Result<Option<Document>, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let mut query = normalize_document_ids(criteria);
            if !query.contains_key("_deleted") {
                query = apply_default_non_deleted(query);
            }
            query = apply_scope(query, scope, self.inner.config.ownership_mode)?;

            let requested_uuid = fields
                .as_ref()
                .map(|items| items.iter().any(|field| field.trim() == "_uuid"))
                .unwrap_or(false);
            let mut projection = self.build_projection(fields.as_deref())?;
            if !projection.is_empty() && has_secure_encryption(self.inner.config.secure_field_encryption.as_ref()) {
                projection.insert("_uuid", 1);
            }
            let mut action = collection.find_one(query.clone());
            if !projection.is_empty() {
                action = action.projection(projection.clone());
            }

            let found = self
                .run_timed(
                    "findOne",
                    json!({
                        "collectionName": collection_name,
                        "query": query,
                        "projection": projection,
                        "scope": scope.cloned(),
                    }),
                    || async {
                        action.await.map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_fo".to_owned()),
                                "Error finding document",
                            )
                        })
                    },
                )
                .await?;

            found
                .map(|document| {
                    let mut document = self.sanitize_single_read(collection_name, document)?;
                    if fields.is_some() && has_secure_encryption(self.inner.config.secure_field_encryption.as_ref()) && !requested_uuid {
                        document.remove("_uuid");
                    }
                    Ok(document)
                })
                .transpose()
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "findOne", "collection": collection_name, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn find_one_with(&self, op: FindOneOp) -> Result<Option<Document>, K2DbError> {
        self.find_one(&op.collection, op.criteria, op.fields, op.scope.as_ref())
            .await
    }

    pub async fn count(
        &self,
        collection_name: &str,
        criteria: Document,
        scope: Option<&Scope>,
    ) -> Result<CountResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let mut query = normalize_document_ids(criteria);
            if !query.contains_key("_deleted") {
                query = apply_default_non_deleted(query);
            }
            query = apply_scope(query, scope, self.inner.config.ownership_mode)?;

            let count = self
                .run_timed(
                    "countDocuments",
                    json!({ "collectionName": collection_name, "query": query.clone(), "scope": scope.cloned() }),
                    || async {
                        collection.count_documents(query).await.map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_cn".to_owned()),
                                "Error counting objects with given criteria",
                            )
                        })
                    },
                )
                .await?;

            Ok(CountResult { count })
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "countDocuments", "collection": collection_name, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn count_with(&self, op: CountOp) -> Result<CountResult, K2DbError> {
        self.count(&op.collection, op.criteria, op.scope.as_ref()).await
    }

    pub async fn find(
        &self,
        collection_name: &str,
        criteria: Document,
        options: FindOptions,
        scope: Option<&Scope>,
    ) -> Result<Vec<Document>, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;

            let mut query = normalize_document_ids(criteria);
            query = self.apply_deleted_policy(query, &options);
            query = apply_scope(query, scope, self.inner.config.ownership_mode)?;

            let projection = self.build_find_projection(&options.projection)?;
            let sort = options.sort.clone();
            let skip = options.skip;
            let limit = options.limit;

            let documents: Vec<Document> = self
                .run_timed(
                    "find",
                    json!({
                        "collectionName": collection_name,
                        "query": query.clone(),
                        "projection": projection.clone(),
                        "sort": sort.clone(),
                        "skip": skip,
                        "limit": limit,
                        "scope": scope.cloned(),
                    }),
                    || async {
                        let mut action = collection.find(query);
                        if !projection.is_empty() {
                            action = action.projection(projection);
                        }
                        if let Some(sort) = sort {
                            action = action.sort(sort);
                        }
                        if skip > 0 {
                            action = action.skip(skip);
                        }
                        if limit > 0 {
                            action = action.limit(limit as i64);
                        }

                        action.await.map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_find_error".to_owned()),
                                "Error executing find query",
                            )
                        })?
                        .try_collect()
                        .await
                        .map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_find_error".to_owned()),
                                "Error executing find query",
                            )
                        })
                    },
                )
                .await?;

            Ok(documents
                .into_iter()
                .map(|document| self.sanitize_multi_read(document))
                .collect())
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "find", "collection": collection_name, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn find_with(&self, op: FindOp) -> Result<Vec<Document>, K2DbError> {
        self.find(&op.collection, op.criteria, op.options, op.scope.as_ref())
            .await
    }

    pub async fn aggregate(
        &self,
        collection_name: &str,
        pipeline: Vec<Document>,
        skip: u64,
        limit: u64,
        scope: Option<&Scope>,
    ) -> Result<Vec<Document>, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            if pipeline.is_empty() {
                return Err(K2DbError::new(
                    ServiceError::SystemError,
                    "Aggregation criteria cannot be empty",
                    Some("sys_mdb_ag_empty".to_owned()),
                ));
            }

            assert_no_secure_field_refs_in_pipeline(&pipeline, &self.inner.config.secure_field_prefixes)?;
            validate_pipeline(self.inner.config.aggregation_mode, &pipeline, limit)?;

            let mut working = enforce_no_deleted_in_pipeline(&pipeline);
            working = enforce_scope_in_pipeline(&working, scope, self.inner.config.ownership_mode)?;
            if skip > 0 {
                working.push(doc! { "$skip": skip as i64 });
            }
            if limit > 0 {
                working.push(doc! { "$limit": limit as i64 });
            }
            let sanitized = sanitize_pipeline_matches(working);

            let collection = self.collection(collection_name).await?;
            let documents: Vec<Document> = self
                .run_timed(
                    "aggregate",
                    json!({
                        "collectionName": collection_name,
                        "pipeline": sanitized.clone(),
                        "skip": skip,
                        "limit": limit,
                        "scope": scope.cloned(),
                    }),
                    || async {
                        let mut action = collection.aggregate(sanitized);
                        if self.inner.config.aggregation_mode != crate::config::AggregationMode::Loose {
                            action = action.max_time(Duration::from_millis(2_000));
                        }

                        action
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_ag".to_owned()),
                                    "Aggregation failed",
                                )
                            })?
                            .try_collect()
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_ag".to_owned()),
                                    "Aggregation failed",
                                )
                            })
                    },
                )
                .await?;

            Ok(documents
                .into_iter()
                .map(|document| self.sanitize_multi_read(document))
                .collect())
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "aggregate", "collection": collection_name, "skip": skip, "limit": limit, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn aggregate_with(&self, op: AggregateOp) -> Result<Vec<Document>, K2DbError> {
        self.aggregate(&op.collection, op.pipeline, op.skip, op.limit, op.scope.as_ref())
            .await
    }

    pub async fn update_all(
        &self,
        collection_name: &str,
        criteria: Document,
        values: Document,
        scope: Option<&Scope>,
    ) -> Result<UpdateResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;

            let deleted_flag = values.get("_deleted").cloned();
            let mut updates = strip_reserved_fields(&values);
            updates = self.apply_schema(collection_name, updates, true)?;
            if let Some(deleted_flag) = deleted_flag {
                updates.insert("_deleted", deleted_flag);
            }
            updates.insert("_updated", Self::now_ms()?);
            let updates = self.encrypt_document_for_collection(collection_name, updates)?;

            let mut query = normalize_document_ids(criteria);
            query = apply_scope(query, scope, self.inner.config.ownership_mode)?;
            query.insert("_deleted", Bson::Document(doc! { "$ne": true }));

            let result = self
                .run_timed(
                    "updateMany",
                    json!({
                        "collectionName": collection_name,
                        "criteria": query.clone(),
                        "values": updates.clone(),
                        "scope": scope.cloned(),
                    }),
                    || async {
                        collection
                            .update_many(query, doc! { "$set": updates })
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_update1".to_owned()),
                                    format!("Error updating {collection_name}"),
                                )
                            })
                    },
                )
                .await?;

            Ok(UpdateResult {
                updated: result.modified_count,
            })
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "updateAll", "collection": collection_name, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn update_all_with(&self, op: UpdateManyOp) -> Result<UpdateResult, K2DbError> {
        self.update_all(&op.collection, op.criteria, op.values, op.scope.as_ref())
            .await
    }

    pub async fn update(
        &self,
        collection_name: &str,
        id: &str,
        data: Document,
        replace: bool,
        scope: Option<&Scope>,
    ) -> Result<UpdateResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let normalized_id = normalize_id(id);
            let mut updates = strip_reserved_fields(&data);
            updates = self.apply_schema(collection_name, updates, !replace)?;
            updates.insert("_updated", Self::now_ms()?);
            let encrypted_updates = self.encrypt_document(collection_name, &normalized_id, updates.clone())?;

            let filter = apply_scope(
                doc! {
                    "_uuid": normalized_id.clone(),
                    "_deleted": { "$ne": true }
                },
                scope,
                self.inner.config.ownership_mode,
            )?;

            let result = if replace {
                let original = self.get(collection_name, &normalized_id, scope).await?;
                let mut replacement = encrypted_updates;
                for (key, value) in original.iter() {
                    if key.starts_with('_') {
                        replacement.insert(key.clone(), value.clone());
                    }
                }

                self.run_timed(
                    "replaceOne",
                    json!({ "collectionName": collection_name, "_uuid": normalized_id.clone(), "scope": scope.cloned() }),
                    || async {
                        collection
                            .replace_one(filter, replacement)
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_update_error".to_owned()),
                                    format!("Error updating {collection_name}"),
                                )
                            })
                    },
                )
                .await?
            } else {
                self.run_timed(
                    "updateOne",
                    json!({ "collectionName": collection_name, "_uuid": normalized_id.clone(), "scope": scope.cloned() }),
                    || async {
                        collection
                            .update_one(filter, doc! { "$set": encrypted_updates })
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_update_error".to_owned()),
                                    format!("Error updating {collection_name}"),
                                )
                            })
                    },
                )
                .await?
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
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "update", "collection": collection_name, "uuid": id, "replace": replace, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn update_with(&self, op: UpdateOneOp) -> Result<UpdateResult, K2DbError> {
        self.update(&op.collection, &op.id, op.data, op.replace, op.scope.as_ref())
            .await
    }

    pub async fn delete_all(
        &self,
        collection_name: &str,
        criteria: Document,
        scope: Option<&Scope>,
    ) -> Result<DeleteResult, K2DbError> {
        let result = self
            .update_all(collection_name, criteria, doc! { "_deleted": true }, scope)
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

    pub async fn delete_all_with(&self, op: DeleteManyOp) -> Result<DeleteResult, K2DbError> {
        self.delete_all(&op.collection, op.criteria, op.scope.as_ref()).await
    }

    pub async fn delete(
        &self,
        collection_name: &str,
        id: &str,
        scope: Option<&Scope>,
    ) -> Result<DeleteResult, K2DbError> {
        let result = self
            .delete_all(collection_name, doc! { "_uuid": normalize_id(id) }, scope)
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

    pub async fn restore(
        &self,
        collection_name: &str,
        criteria: Document,
        scope: Option<&Scope>,
    ) -> Result<RestoreResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let mut query = normalize_document_ids(criteria);
            query = apply_scope(query, scope, self.inner.config.ownership_mode)?;
            query.insert("_deleted", true);

            let result = self
                .run_timed(
                    "updateMany",
                    json!({ "collectionName": collection_name, "query": query.clone(), "scope": scope.cloned() }),
                    || async {
                        collection
                            .update_many(
                                query,
                                doc! {
                                    "$set": {
                                        "_deleted": false,
                                        "_updated": Self::now_ms()?
                                    }
                                },
                            )
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_pres".to_owned()),
                                    "Error restoring a deleted item",
                                )
                            })
                    },
                )
                .await?;

            Ok(RestoreResult {
                status: "restored".to_owned(),
                modified: result.modified_count,
            })
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "restore", "collection": collection_name, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn restore_with(&self, op: RestoreOp) -> Result<RestoreResult, K2DbError> {
        self.restore(&op.collection, op.criteria, op.scope.as_ref()).await
    }

    pub async fn purge(
        &self,
        collection_name: &str,
        id: &str,
        scope: Option<&Scope>,
    ) -> Result<PurgeResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let normalized_id = normalize_id(id);

            let find_filter = apply_scope(
                doc! {
                    "_uuid": normalized_id.clone(),
                    "_deleted": true,
                },
                scope,
                self.inner.config.ownership_mode,
            )?;

            let existing = self
                .run_timed(
                    "findOne",
                    json!({ "collectionName": collection_name, "query": find_filter.clone(), "scope": scope.cloned() }),
                    || async {
                        collection.find_one(find_filter).await.map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_pg".to_owned()),
                                format!("Error purging item with id: {normalized_id}"),
                            )
                        })
                    },
                )
                .await?;

            if existing.is_none() {
                return Err(K2DbError::new(
                    ServiceError::SystemError,
                    "Cannot purge item that is not deleted",
                    Some("sys_mdb_gcol_pg2".to_owned()),
                ));
            }

            let delete_filter = apply_scope(
                doc! { "_uuid": normalized_id.clone() },
                scope,
                self.inner.config.ownership_mode,
            )?;

            self.run_timed(
                "deleteOne",
                json!({ "collectionName": collection_name, "query": delete_filter.clone(), "scope": scope.cloned() }),
                || async {
                    collection.delete_one(delete_filter).await.map_err(|error| {
                        K2DbError::wrap(
                            error,
                            ServiceError::SystemError,
                            Some("sys_mdb_pg".to_owned()),
                            format!("Error purging item with id: {normalized_id}"),
                        )
                    })?;
                    Ok(())
                },
            )
            .await?;

            Ok(PurgeResult { id: normalized_id })
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "purge", "collection": collection_name, "uuid": id, "scope": scope.cloned() }),
            );
            error
        })
    }

    pub async fn purge_deleted_older_than(
        &self,
        collection_name: &str,
        older_than_ms: u64,
        scope: Option<&Scope>,
    ) -> Result<PurgeManyResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let cutoff = Self::now_ms()? - older_than_ms as i64;

            let filter = apply_scope(
                doc! {
                    "_deleted": true,
                    "_updated": { "$lte": cutoff },
                },
                scope,
                self.inner.config.ownership_mode,
            )?;

            let result = self
                .run_timed(
                    "deleteMany",
                    json!({
                        "collectionName": collection_name,
                        "olderThanMs": older_than_ms,
                        "cutoff": cutoff,
                        "query": filter.clone(),
                        "scope": scope.cloned(),
                    }),
                    || async {
                        collection.delete_many(filter).await.map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_purge_older".to_owned()),
                                "Error purging deleted items by age",
                            )
                        })
                    },
                )
                .await?;

            Ok(PurgeManyResult {
                purged: result.deleted_count,
            })
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(
                &error,
                json!({ "op": "purgeDeletedOlderThan", "collection": collection_name, "olderThanMs": older_than_ms, "scope": scope.cloned() }),
            );
            error
        })
    }
}