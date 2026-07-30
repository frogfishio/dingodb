// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use super::*;
use serde_json::json;

impl K2Db {
    pub async fn ensure_indexes(
        &self,
        collection_name: &str,
        options: EnsureIndexesOptions,
    ) -> Result<(), K2DbError> {
        self.validate_collection_name(collection_name)?;

        if options.uuid_partial_unique {
            self.create_index(
                collection_name,
                doc! { "_uuid": 1, "_deleted": 1 },
                Some(IndexOptions::builder().unique(Some(true)).build()),
            )
            .await?;
        } else if options.uuid_unique {
            self.create_index(
                collection_name,
                doc! { "_uuid": 1 },
                Some(IndexOptions::builder().unique(Some(true)).build()),
            )
            .await?;
        }

        if options.owner_index {
            self.create_index(collection_name, doc! { "_owner": 1 }, None)
                .await?;
        }

        if options.deleted_index {
            self.create_index(collection_name, doc! { "_deleted": 1 }, None)
                .await?;
        }

        Ok(())
    }

    pub async fn create_index(
        &self,
        collection_name: &str,
        index_spec: Document,
        options: Option<IndexOptions>,
    ) -> Result<(), K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;
            let collection = self.collection(collection_name).await?;
            let model = IndexModel::builder().keys(index_spec.clone()).options(options.clone()).build();
            self.run_timed(
                "createIndex",
                json!({
                    "collectionName": collection_name,
                    "indexSpec": index_spec,
                    "options": options,
                }),
                || async {
                    collection
                        .create_index(model)
                        .await
                        .map_err(|error| {
                            K2DbError::wrap(
                                error,
                                ServiceError::SystemError,
                                Some("sys_mdb_idx".to_owned()),
                                format!("Error creating index on {collection_name}"),
                            )
                        })?;
                    Ok(())
                },
            )
            .await
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "createIndex", "collection": collection_name }));
            error
        })
    }

    pub async fn drop_collection(
        &self,
        collection_name: &str,
        scope: Option<&Scope>,
    ) -> Result<DropResult, K2DbError> {
        let result = async {
            self.validate_collection_name(collection_name)?;

            match self.inner.config.ownership_mode {
                crate::config::OwnershipMode::Strict => {
                    if !matches!(scope, Some(Scope::All)) {
                        return Err(K2DbError::new(
                            ServiceError::BadRequest,
                            "Dropping a collection requires scope=\"*\" in strict ownership mode",
                            Some("sys_mdb_drop_scope_required".to_owned()),
                        ));
                    }
                }
                crate::config::OwnershipMode::Lax => {
                    if let Some(scope) = scope {
                        if !matches!(scope, Scope::All) {
                            return Err(K2DbError::new(
                                ServiceError::BadRequest,
                                "Dropping a collection only supports scope=\"*\"",
                                Some("sys_mdb_drop_scope_invalid".to_owned()),
                            ));
                        }
                    }
                }
            }

            let collection = self.collection(collection_name).await?;
            self.run_timed(
                "dropCollection",
                json!({ "collectionName": collection_name, "scope": scope.cloned() }),
                || async {
                    collection.drop().await.map_err(|error| {
                        K2DbError::wrap(
                            error,
                            ServiceError::SystemError,
                            Some("sys_mdb_drop".to_owned()),
                            "Error dropping collection",
                        )
                    })?;
                    Ok(DropResult {
                        status: "ok".to_owned(),
                    })
                },
            )
            .await
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "dropCollection", "collection": collection_name, "scope": scope.cloned() }));
            error
        })
    }

    pub async fn drop_database(&self) -> Result<(), K2DbError> {
        let result = async {
            let database = self.database().await?;
            self.run_timed(
                "dropDatabase",
                json!({ "database": self.config().name }),
                || async {
                    database.run_command(doc! { "dropDatabase": 1 }).await.map_err(|error| {
                        K2DbError::wrap(
                            error,
                            ServiceError::SystemError,
                            Some("sys_mdb_drop_db".to_owned()),
                            "Error dropping database",
                        )
                    })?;
                    Ok(())
                },
            )
            .await
        }
        .await;

        result.map_err(|error| {
            self.emit_db_error(&error, json!({ "op": "dropDatabase", "database": self.config().name }));
            error
        })
    }

    pub async fn execute_transaction<T, F>(&self, operations: F) -> Result<T, K2DbError>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> TransactionFuture<'a, T>,
    {
        let result = self.execute_transaction_scoped(None, operations).await;
        if let Err(error) = &result {
            self.emit_db_error(error, json!({ "op": "executeTransaction", "database": self.config().name }));
        }
        result
    }

    pub(crate) async fn execute_transaction_scoped<T, F>(
        &self,
        scope: Option<Scope>,
        operations: F,
    ) -> Result<T, K2DbError>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> TransactionFuture<'a, T>,
    {
        let client = {
            self.init().await?;
            let state = self.inner.state.read().await;
            state.client.clone().ok_or_else(|| {
                K2DbError::new(
                    ServiceError::SystemError,
                    "Database client is not initialized",
                    Some("sys_mdb_init".to_owned()),
                )
            })?
        };

        let mut session = client.start_session().await.map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::BadGateway,
                Some("sys_mdb_txn".to_owned()),
                "Transaction failed",
            )
        })?;

        session.start_transaction().await.map_err(|error| {
            K2DbError::wrap(
                error,
                ServiceError::BadGateway,
                Some("sys_mdb_txn".to_owned()),
                "Transaction failed",
            )
        })?;

        match operations(TransactionContext::new(self, &mut session, scope)).await {
            Ok(value) => {
                session.commit_transaction().await.map_err(|error| {
                    K2DbError::wrap(
                        error,
                        ServiceError::BadGateway,
                        Some("sys_mdb_txn".to_owned()),
                        "Transaction failed",
                    )
                })?;
                Ok(value)
            }
            Err(error) => {
                let _ = session.abort_transaction().await;
                Err(K2DbError::wrap(
                    error,
                    ServiceError::BadGateway,
                    Some("sys_mdb_txn".to_owned()),
                    "Transaction failed",
                ))
            }
        }
    }

    pub fn close(&self) {
        let db = self.clone();
        tokio::spawn(async move {
            let _ = db.release().await;
        });
    }
}