// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use super::*;
use serde_json::json;

impl K2Db {
    pub async fn ensure_history_indexes(&self, collection_name: &str) -> Result<(), K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let history = self.history_collection(collection_name).await?;
            let history_name = self.history_name(collection_name);

            self.run_timed(
                "createIndex",
                json!({ "collectionName": history_name, "indexSpec": { "_uuid": 1, "_v": 1 } }),
                || async {
                    history
                        .create_index(
                            IndexModel::builder()
                                .keys(doc! { "_uuid": 1, "_v": 1 })
                                .options(IndexOptions::builder().unique(Some(true)).build())
                                .build(),
                        )
                        .await
                        .map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_idx".to_owned()),
                                format!("Error creating index on {}", history_name),
                            )
                        })?;
                    Ok(())
                },
            )
            .await?;

            self.run_timed(
                "createIndex",
                json!({ "collectionName": self.history_name(collection_name), "indexSpec": { "_uuid": 1, "_at": -1 } }),
                || async {
                    history
                        .create_index(IndexModel::builder().keys(doc! { "_uuid": 1, "_at": -1 }).build())
                        .await
                        .map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_idx".to_owned()),
                                format!("Error creating index on {}", self.history_name(collection_name)),
                            )
                        })?;
                    Ok(())
                },
            )
            .await
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "ensureHistoryIndexes", "collection": collection_name }));
            error
        })
    }

    pub async fn update_versioned(
        &self,
        collection_name: &str,
        id: &str,
        data: Document,
        replace: bool,
        max_versions: Option<u64>,
        scope: Option<&Scope>,
    ) -> Result<Vec<VersionedUpdateResult>, K2DbError> {
        let result = async {
            let normalized_id = normalize_id(id);
            let current = self.get(collection_name, &normalized_id, scope).await?;
            self.ensure_history_indexes(collection_name).await?;
            let version = self.snapshot_current(collection_name, &current).await?;

            let updated = self
                .update(collection_name, &normalized_id, data, replace, scope)
                .await?;

            if let Some(max_versions) = max_versions {
                self.prune_versions(collection_name, &normalized_id, max_versions)
                    .await?;
            }

            Ok(vec![VersionedUpdateResult {
                updated: updated.updated,
                version_saved: version,
            }])
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "updateVersioned", "collection": collection_name, "uuid": id, "replace": replace, "scope": scope.cloned() }));
            error
        })
    }

    pub async fn update_versioned_with(
        &self,
        op: VersionedUpdateOp,
    ) -> Result<Vec<VersionedUpdateResult>, K2DbError> {
        self.update_versioned(
            &op.collection,
            &op.id,
            op.data,
            op.replace,
            op.max_versions,
            op.scope.as_ref(),
        )
        .await
    }

    pub async fn list_versions(
        &self,
        collection_name: &str,
        id: &str,
        skip: u64,
        limit: u64,
        scope: Option<&Scope>,
    ) -> Result<Vec<VersionInfo>, K2DbError> {
        let result: Result<Vec<VersionInfo>, K2DbError> = async {
            let normalized_id = normalize_id(id);
            self.get(collection_name, &normalized_id, scope).await?;
            let history = self.history_collection(collection_name).await?;
            let rows: Vec<Document> = self
                .run_timed(
                    "find",
                    json!({
                        "collectionName": self.history_name(collection_name),
                        "query": { "_uuid": normalized_id.clone() },
                        "skip": skip,
                        "limit": limit,
                        "scope": scope.cloned(),
                    }),
                    || async {
                        history
                            .find(doc! { "_uuid": &normalized_id })
                            .projection(doc! { "_uuid": 1, "_v": 1, "_at": 1, "_id": 0 })
                            .sort(doc! { "_v": -1 })
                            .skip(skip)
                            .limit(limit as i64)
                            .await
                            .map_err(|error| {
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

            rows.into_iter()
                .map(|row| {
                    Ok(VersionInfo {
                        uuid: row.get_str("_uuid").map(str::to_owned).map_err(|error| {
                            K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Invalid version row")
                        })?,
                        version: row.get_i64("_v").map_err(|error| {
                            K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Invalid version row")
                        })? as u64,
                        at: row.get_i64("_at").map_err(|error| {
                            K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Invalid version row")
                        })? as u64,
                    })
                })
                .collect()
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "listVersions", "collection": collection_name, "uuid": id, "scope": scope.cloned() }));
            error
        })
    }

    pub async fn revert_to_version(
        &self,
        collection_name: &str,
        id: &str,
        version: u64,
        scope: Option<&Scope>,
    ) -> Result<UpdateResult, K2DbError> {
        let result = async {
            let normalized_id = normalize_id(id);
            self.get(collection_name, &normalized_id, scope).await?;
            let history = self.history_collection(collection_name).await?;
            let row = self
                .run_timed(
                    "findOne",
                    json!({
                        "collectionName": self.history_name(collection_name),
                        "query": { "_uuid": normalized_id.clone(), "_v": version as i64 },
                        "scope": scope.cloned(),
                    }),
                    || async {
                        history
                            .find_one(doc! { "_uuid": &normalized_id, "_v": version as i64 })
                            .await
                            .map_err(|error| {
                                K2DbError::wrap(
                                    error,
                                    ServiceError::SystemError,
                                    Some("sys_mdb_find_error".to_owned()),
                                    "Error finding document",
                                )
                            })
                    },
                )
                .await?;

            let Some(row) = row else {
                return Err(K2DbError::new(
                    ServiceError::NotFound,
                    format!("Version {version} for {normalized_id} not found"),
                    Some("sys_mdb_version_not_found".to_owned()),
                ));
            };

            let snapshot = row
                .get_document("snapshot")
                .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Invalid version snapshot"))?
                .clone();
            let snapshot = self.decrypt_snapshot(collection_name, &normalized_id, snapshot)?;

            let mut apply = Document::new();
            for (key, value) in snapshot {
                if !key.starts_with('_') {
                    apply.insert(key, value);
                }
            }

            self.update(collection_name, &normalized_id, apply, true, scope)
                .await
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "revertToVersion", "collection": collection_name, "uuid": id, "version": version, "scope": scope.cloned() }));
            error
        })
    }

    async fn next_version(&self, collection_name: &str, id: &str) -> Result<u64, K2DbError> {
        let history = self.history_collection(collection_name).await?;
        let last = history
            .find(doc! { "_uuid": id })
            .projection(doc! { "_v": 1, "_id": 0 })
            .sort(doc! { "_v": -1 })
            .limit(1)
            .await
            .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Error executing find query"))?
            .try_collect::<Vec<Document>>()
            .await
            .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Error executing find query"))?;

        Ok(last
            .first()
            .and_then(|row| row.get_i64("_v").ok())
            .map(|version| version as u64 + 1)
            .unwrap_or(1))
    }

    async fn snapshot_current(&self, collection_name: &str, current: &Document) -> Result<u64, K2DbError> {
        let id = current
            .get_str("_uuid")
            .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Current document missing _uuid"))?;
        let version = self.next_version(collection_name, id).await?;
        let history = self.history_collection(collection_name).await?;
        let snapshot = self.encrypt_document(collection_name, id, current.clone())?;
        self.run_timed(
            "insertOne",
            json!({ "collectionName": self.history_name(collection_name), "_uuid": id, "_v": version }),
            || async {
                history
                    .insert_one(doc! {
                        "_uuid": id,
                        "_v": version as i64,
                        "_at": Self::now_ms()?,
                        "snapshot": snapshot,
                    })
                    .await
                    .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_sav".to_owned()), "Error saving object to database"))?;
                Ok(())
            },
        )
        .await?;
        Ok(version)
    }

    async fn prune_versions(&self, collection_name: &str, id: &str, max_versions: u64) -> Result<(), K2DbError> {
        let history = self.history_collection(collection_name).await?;
        let count = self
            .run_timed(
                "countDocuments",
                json!({ "collectionName": self.history_name(collection_name), "query": { "_uuid": id } }),
                || async {
                    history
                        .count_documents(doc! { "_uuid": id })
                        .await
                        .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_cn".to_owned()), "Error counting objects with given criteria"))
                },
            )
            .await?;

        let overflow = count.saturating_sub(max_versions);
        if overflow == 0 {
            return Ok(());
        }

        let olds: Vec<Document> = self
            .run_timed(
                "find",
                json!({ "collectionName": self.history_name(collection_name), "query": { "_uuid": id }, "limit": overflow }),
                || async {
                    history
                        .find(doc! { "_uuid": id })
                        .projection(doc! { "_v": 1, "_id": 0 })
                        .sort(doc! { "_v": 1 })
                        .limit(overflow as i64)
                        .await
                        .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Error executing find query"))?
                        .try_collect()
                        .await
                        .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_find_error".to_owned()), "Error executing find query"))
                },
            )
            .await?;

        let versions = olds
            .into_iter()
            .filter_map(|row| row.get_i64("_v").ok())
            .collect::<Vec<_>>();
        if versions.is_empty() {
            return Ok(());
        }

        self.run_timed(
            "deleteMany",
            json!({ "collectionName": self.history_name(collection_name), "query": { "_uuid": id, "_v": { "$in": versions.clone() } } }),
            || async {
                history
                    .delete_many(doc! { "_uuid": id, "_v": { "$in": versions } })
                    .await
                    .map_err(|error| K2DbError::wrap(error, ServiceError::SystemError, Some("sys_mdb_purge_older".to_owned()), "Error purging deleted items by age"))?;
                Ok(())
            },
        )
        .await?;
        Ok(())
    }
}