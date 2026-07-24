//! Named collection handle (DX_SPEC §5.2, §6, §7, §8, §10).

use crate::error::Error;
use crate::filter::{compare_field, Filter, QueryBuilder, QueryOptions};
use crate::history::KeyHistory;
use crate::indexes::{try_index_lookup, IndexInfo, Indexes};
use crate::receipt::{DeleteReceipt, PutOptions, WriteReceipt};
use crate::subject::{collection_prefix, decode_subject, encode_subject};
use crate::value::{decode_bytes, decode_json, encode_bytes, encode_json};
use dingo_store::{PayloadResult, Store};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::str;

/// A lazy handle to a named collection within a [`crate::Dingo`] database.
///
/// Merely constructing a collection does not touch disk. The first write
/// creates store-level subject entries; the collection catalog is derived
/// (Stage 6) and rebuildable.
pub struct Collection<'a> {
    store: &'a mut Store,
    name: String,
}

impl<'a> Collection<'a> {
    pub(crate) fn new(store: &'a mut Store, name: String) -> Self {
        Self { store, name }
    }

    /// Collection name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Secondary index management (DX_SPEC §8).
    pub fn indexes(&mut self) -> Indexes<'_> {
        Indexes {
            store: self.store,
            collection: self.name.clone(),
        }
    }

    /// Put (create-or-replace) a JSON-serializable value under `key`.
    ///
    /// Default durability is [`dingo_store::DurabilityMode::Durable`].
    pub fn put<T: Serialize>(&mut self, key: &str, value: &T) -> Result<WriteReceipt, Error> {
        self.put_with(key, value, PutOptions::default())
    }

    /// Put with explicit options.
    pub fn put_with<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        let subject = encode_subject(&self.name, key)?;
        let json = serde_json::to_value(value)?;
        let body = encode_json(&json)?;
        // Stage 4 string keys → subject is always valid UTF-8 (version byte + UTF-8 parts).
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        let receipt = self.store.put(subject_str, &body, options.durability)?;
        // Best-effort: mark secondary indexes stale after writes.
        let _ = self.mark_indexes_stale();
        Ok(WriteReceipt::from_store(key.to_string(), receipt))
    }

    /// Get the current JSON value for `key`, if present.
    ///
    /// Returns `Ok(None)` only when absence is established for this key.
    pub fn get(&self, key: &str) -> Result<Option<JsonValue>, Error> {
        let subject = encode_subject(&self.name, key)?;
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        match self.store.get(subject_str)? {
            None => Ok(None),
            Some(body) => Ok(Some(decode_json(&body)?)),
        }
    }

    /// Get payload with explicit completeness (chunked values).
    pub fn get_payload(&self, key: &str) -> Result<Option<PayloadResult>, Error> {
        let subject = encode_subject(&self.name, key)?;
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        Ok(self.store.get_payload(subject_str)?)
    }

    /// Get and deserialize into `T`.
    pub fn get_as<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        match self.get(key)? {
            None => Ok(None),
            Some(v) => Ok(Some(serde_json::from_value(v)?)),
        }
    }

    /// Put opaque bytes under `key` (DX_SPEC §6.9).
    pub fn put_bytes(&mut self, key: &str, bytes: &[u8]) -> Result<WriteReceipt, Error> {
        self.put_bytes_with(key, bytes, PutOptions::default())
    }

    /// Put opaque bytes with explicit options.
    pub fn put_bytes_with(
        &mut self,
        key: &str,
        bytes: &[u8],
        options: PutOptions,
    ) -> Result<WriteReceipt, Error> {
        let subject = encode_subject(&self.name, key)?;
        let body = encode_bytes(bytes);
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        let receipt = self.store.put(subject_str, &body, options.durability)?;
        let _ = self.mark_indexes_stale();
        Ok(WriteReceipt::from_store(key.to_string(), receipt))
    }

    /// Get opaque bytes for `key`, if present.
    pub fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        let subject = encode_subject(&self.name, key)?;
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        match self.store.get(subject_str)? {
            None => Ok(None),
            Some(body) => Ok(Some(decode_bytes(&body)?)),
        }
    }

    /// Delete the current value for `key` (tombstone; history retained).
    ///
    /// Deleting an absent key is idempotent; [`DeleteReceipt::removed`] reports
    /// whether a visible value changed.
    pub fn delete(&mut self, key: &str) -> Result<DeleteReceipt, Error> {
        self.delete_with(key, PutOptions::default())
    }

    /// Delete with explicit durability options.
    pub fn delete_with(&mut self, key: &str, options: PutOptions) -> Result<DeleteReceipt, Error> {
        let subject = encode_subject(&self.name, key)?;
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        let removed = self.store.get(subject_str)?.is_some();
        let receipt = self.store.delete(subject_str, options.durability)?;
        let _ = self.mark_indexes_stale();
        Ok(DeleteReceipt::from_store(key.to_string(), removed, receipt))
    }

    /// Immutable event history for `key` (DX_SPEC §10.1).
    pub fn history(&self, key: &str) -> Result<KeyHistory, Error> {
        let subject = encode_subject(&self.name, key)?;
        let subject_str = str::from_utf8(&subject).expect("stage 4 subject is UTF-8");
        let hist = self.store.history(subject_str)?;
        KeyHistory::from_store(key.to_string(), hist)
    }

    /// Scan live keys in this collection (deterministic key order).
    ///
    /// Returns application keys only. Payload access is via get / get_bytes.
    pub fn scan_keys(&self) -> Result<Vec<String>, Error> {
        let prefix = collection_prefix(&self.name)?;
        let mut keys = Vec::new();
        for (subject, _body) in self.store.live_entries() {
            if !subject.starts_with(&prefix) {
                continue;
            }
            match decode_subject(subject) {
                Some((coll, key)) if coll == self.name => keys.push(key.to_string()),
                _ => continue,
            }
        }
        Ok(keys)
    }

    /// Scan live JSON entries `(key, value)` in this collection.
    ///
    /// Entries stored as bytes (not JSON) yield [`Error::TypeMismatch`] for that
    /// key and abort the scan. Prefer [`Self::scan_keys`] + typed gets when mixed.
    pub fn scan_json(&self) -> Result<Vec<(String, JsonValue)>, Error> {
        self.scan_json_filtered(&Filter::Always, &QueryOptions::default())
    }

    /// Stream live JSON entries one at a time (DX journey 6 / Stage 4b).
    ///
    /// Yields `(key, value)` in stable key order. Incomplete chunked payloads are
    /// skipped (use [`Self::get_payload`] for partial maps).
    pub fn scan_json_iter(
        &self,
    ) -> Result<impl Iterator<Item = Result<(String, JsonValue), Error>> + '_, Error> {
        let prefix = collection_prefix(&self.name)?;
        let name = self.name.clone();
        let logical = self.store.live_logical_entries()?;
        Ok(logical.into_iter().filter_map(move |(subject, body)| {
            if !subject.starts_with(&prefix) {
                return None;
            }
            match decode_subject(&subject) {
                Some((coll, key)) if coll == name => {
                    Some(decode_json(&body).map(|v| (key.to_string(), v)))
                }
                _ => None,
            }
        }))
    }

    /// Find JSON documents matching `filter` (DX_SPEC §7.1).
    ///
    /// Uses a secondary index when one is ready and applicable; otherwise scans.
    /// Unbounded materialization in stable key order. Prefer
    /// [`Self::find_with`] with a limit for large collections.
    pub fn find(&self, filter: &Filter) -> Result<Vec<(String, JsonValue)>, Error> {
        self.find_with(filter, QueryOptions::default())
    }

    /// Find with limit / order / budget options.
    pub fn find_with(
        &self,
        filter: &Filter,
        options: QueryOptions,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        self.scan_json_filtered(filter, &options)
    }

    /// Parse a DX/Mongo-style object filter and find matching documents.
    pub fn find_json(&self, filter_obj: &JsonValue) -> Result<Vec<(String, JsonValue)>, Error> {
        let filter = Filter::from_json(filter_obj)?;
        self.find(&filter)
    }

    /// Fluent query builder (DX_SPEC §7.2).
    pub fn query(&self) -> QueryBuilder<'_> {
        QueryBuilder::new(self)
    }

    /// List ready indexes (convenience).
    pub fn list_indexes(&self) -> Result<Vec<IndexInfo>, Error> {
        Ok(self
            .store
            .list_secondary_indexes(&self.name)?
            .iter()
            .map(|i| IndexInfo {
                name: i.meta.name.clone(),
                collection: i.meta.collection.clone(),
                fields: i.meta.fields.clone(),
                state: i.meta.state,
                entry_count: i.meta.entry_count,
                complete_coverage: i.meta.complete_coverage,
            })
            .collect())
    }

    fn mark_indexes_stale(&mut self) -> Result<(), Error> {
        let indexes = self.store.list_secondary_indexes(&self.name)?;
        for mut idx in indexes {
            if idx.meta.state.usable() || idx.meta.state == dingo_store::IndexState::Ready {
                idx.mark_stale();
                self.store.write_secondary_index(&idx)?;
            }
        }
        Ok(())
    }

    fn scan_json_filtered(
        &self,
        filter: &Filter,
        options: &QueryOptions,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        // Try index acceleration when not force-scanning.
        if !options.force_scan {
            if let Some((info, subjects)) = try_index_lookup(self.store, &self.name, filter)? {
                // Only use when ready with complete coverage, or partial with disclosure
                // (we still re-filter docs to ensure correctness).
                if info.state.usable() {
                    return self.collect_from_subjects(subjects, filter, options, Some(&info));
                }
            }
        }

        // Scan path with optional budget.
        let prefix = collection_prefix(&self.name)?;
        let logical = self.store.live_logical_entries()?;
        let mut scanned = 0usize;
        let mut out = Vec::new();
        for (subject, body) in logical {
            if !subject.starts_with(&prefix) {
                continue;
            }
            let Some((coll, key)) = decode_subject(&subject) else {
                continue;
            };
            if coll != self.name {
                continue;
            }
            scanned += 1;
            if let Some(budget) = &options.budget {
                if let Some(max) = budget.max_docs_scanned {
                    if scanned > max {
                        return Err(Error::QueryBudgetRequired(format!(
                            "scan examined more than {max} documents without a usable index; \
                             raise budget or create an index"
                        )));
                    }
                }
            }
            let value = decode_json(&body)?;
            if !filter.matches(&value) {
                continue;
            }
            out.push((key.to_string(), value));
        }

        self.finish_query(out, options)
    }

    fn collect_from_subjects(
        &self,
        subjects: Vec<Vec<u8>>,
        filter: &Filter,
        options: &QueryOptions,
        _info: Option<&IndexInfo>,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        let mut out = Vec::new();
        let mut scanned = 0usize;
        for subject in subjects {
            scanned += 1;
            if let Some(budget) = &options.budget {
                if let Some(max) = budget.max_docs_scanned {
                    if scanned > max {
                        return Err(Error::QueryBudgetRequired(format!(
                            "index probe exceeded budget of {max} documents"
                        )));
                    }
                }
            }
            let subject_str = match str::from_utf8(&subject) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let Some(body) = self.store.get(subject_str)? else {
                continue;
            };
            let value = decode_json(&body)?;
            if !filter.matches(&value) {
                continue;
            }
            let Some((coll, key)) = decode_subject(&subject) else {
                continue;
            };
            if coll != self.name {
                continue;
            }
            out.push((key.to_string(), value));
        }
        self.finish_query(out, options)
    }

    fn finish_query(
        &self,
        mut out: Vec<(String, JsonValue)>,
        options: &QueryOptions,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        if let Some((ref field, order)) = options.order_by {
            let order = order;
            out.sort_by(|a, b| {
                let cmp = compare_field(&a.1, &b.1, field, order);
                if cmp == std::cmp::Ordering::Equal {
                    a.0.cmp(&b.0)
                } else {
                    cmp
                }
            });
        } else {
            out.sort_by(|a, b| a.0.cmp(&b.0));
        }

        if let Some(limit) = options.limit {
            if out.len() > limit {
                out.truncate(limit);
            }
        }
        Ok(out)
    }
}
