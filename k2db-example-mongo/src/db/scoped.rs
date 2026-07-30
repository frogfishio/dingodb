// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use super::*;

impl ScopedK2Db {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn db(&self) -> &K2Db {
        &self.db
    }

    pub fn set_schema(&self, collection_name: &str, schema: serde_json::Value, mode: SchemaMode) -> Result<(), K2DbError> {
        self.db.set_schema(collection_name, schema, mode)
    }

    pub fn clear_schema(&self, collection_name: &str) -> Result<(), K2DbError> {
        self.db.clear_schema(collection_name)
    }

    pub fn clear_schemas(&self) -> Result<(), K2DbError> {
        self.db.clear_schemas()
    }

    pub async fn get(&self, collection_name: &str, id: &str) -> Result<Document, K2DbError> {
        self.db.get(collection_name, id, Some(&self.scope)).await
    }

    pub async fn get_with(&self, mut op: GetOp) -> Result<Document, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.get_with(op).await
    }

    pub async fn find_one(
        &self,
        collection_name: &str,
        criteria: Document,
        fields: Option<Vec<String>>,
    ) -> Result<Option<Document>, K2DbError> {
        self.db.find_one(collection_name, criteria, fields, Some(&self.scope)).await
    }

    pub async fn find_one_with(&self, mut op: FindOneOp) -> Result<Option<Document>, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.find_one_with(op).await
    }

    pub async fn count(&self, collection_name: &str, criteria: Document) -> Result<CountResult, K2DbError> {
        self.db.count(collection_name, criteria, Some(&self.scope)).await
    }

    pub async fn count_with(&self, mut op: CountOp) -> Result<CountResult, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.count_with(op).await
    }

    pub async fn find(
        &self,
        collection_name: &str,
        criteria: Document,
        options: FindOptions,
    ) -> Result<Vec<Document>, K2DbError> {
        self.db.find(collection_name, criteria, options, Some(&self.scope)).await
    }

    pub async fn find_with(&self, mut op: FindOp) -> Result<Vec<Document>, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.find_with(op).await
    }

    pub async fn update(
        &self,
        collection_name: &str,
        id: &str,
        data: Document,
        replace: bool,
    ) -> Result<UpdateResult, K2DbError> {
        self.db.update(collection_name, id, data, replace, Some(&self.scope)).await
    }

    pub async fn update_with(&self, mut op: UpdateOneOp) -> Result<UpdateResult, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.update_with(op).await
    }

    pub async fn update_all(
        &self,
        collection_name: &str,
        criteria: Document,
        values: Document,
    ) -> Result<UpdateResult, K2DbError> {
        self.db.update_all(collection_name, criteria, values, Some(&self.scope)).await
    }

    pub async fn update_all_with(&self, mut op: UpdateManyOp) -> Result<UpdateResult, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.update_all_with(op).await
    }

    pub async fn delete(&self, collection_name: &str, id: &str) -> Result<DeleteResult, K2DbError> {
        self.db.delete(collection_name, id, Some(&self.scope)).await
    }

    pub async fn aggregate(
        &self,
        collection_name: &str,
        pipeline: Vec<Document>,
        skip: u64,
        limit: u64,
    ) -> Result<Vec<Document>, K2DbError> {
        self.db
            .aggregate(collection_name, pipeline, skip, limit, Some(&self.scope))
            .await
    }

    pub async fn aggregate_with(&self, mut op: AggregateOp) -> Result<Vec<Document>, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.aggregate_with(op).await
    }

    pub async fn delete_all(&self, collection_name: &str, criteria: Document) -> Result<DeleteResult, K2DbError> {
        self.db.delete_all(collection_name, criteria, Some(&self.scope)).await
    }

    pub async fn delete_all_with(&self, mut op: DeleteManyOp) -> Result<DeleteResult, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.delete_all_with(op).await
    }

    pub async fn restore(&self, collection_name: &str, criteria: Document) -> Result<RestoreResult, K2DbError> {
        self.db.restore(collection_name, criteria, Some(&self.scope)).await
    }

    pub async fn restore_with(&self, mut op: RestoreOp) -> Result<RestoreResult, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.restore_with(op).await
    }

    pub async fn purge(&self, collection_name: &str, id: &str) -> Result<PurgeResult, K2DbError> {
        self.db.purge(collection_name, id, Some(&self.scope)).await
    }

    pub async fn purge_deleted_older_than(
        &self,
        collection_name: &str,
        older_than_ms: u64,
    ) -> Result<PurgeManyResult, K2DbError> {
        self.db
            .purge_deleted_older_than(collection_name, older_than_ms, Some(&self.scope))
            .await
    }

    pub async fn drop_collection(&self, collection_name: &str) -> Result<DropResult, K2DbError> {
        self.db.drop_collection(collection_name, Some(&self.scope)).await
    }

    pub async fn execute_transaction<T, F>(&self, operations: F) -> Result<T, K2DbError>
    where
        F: for<'a> FnOnce(TransactionContext<'a>) -> TransactionFuture<'a, T>,
    {
        self.db
            .execute_transaction_scoped(Some(self.scope.clone()), operations)
            .await
    }

    pub async fn ensure_history_indexes(&self, collection_name: &str) -> Result<(), K2DbError> {
        self.db.ensure_history_indexes(collection_name).await
    }

    pub async fn update_versioned(
        &self,
        collection_name: &str,
        id: &str,
        data: Document,
        replace: bool,
        max_versions: Option<u64>,
    ) -> Result<Vec<VersionedUpdateResult>, K2DbError> {
        self.db
            .update_versioned(collection_name, id, data, replace, max_versions, Some(&self.scope))
            .await
    }

    pub async fn update_versioned_with(
        &self,
        mut op: VersionedUpdateOp,
    ) -> Result<Vec<VersionedUpdateResult>, K2DbError> {
        op.scope = Some(self.scope.clone());
        self.db.update_versioned_with(op).await
    }

    pub async fn list_versions(
        &self,
        collection_name: &str,
        id: &str,
        skip: u64,
        limit: u64,
    ) -> Result<Vec<VersionInfo>, K2DbError> {
        self.db
            .list_versions(collection_name, id, skip, limit, Some(&self.scope))
            .await
    }

    pub async fn revert_to_version(
        &self,
        collection_name: &str,
        id: &str,
        version: u64,
    ) -> Result<UpdateResult, K2DbError> {
        self.db
            .revert_to_version(collection_name, id, version, Some(&self.scope))
            .await
    }
}