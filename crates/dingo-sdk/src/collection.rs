//! Named collection handle (DX_SPEC §5.2, §6, §7).

use crate::error::Error;
use crate::filter::{compare_field, Filter, QueryBuilder, QueryOptions};
use crate::receipt::{DeleteReceipt, PutOptions, WriteReceipt};
use crate::subject::{collection_prefix, decode_subject, encode_subject};
use crate::value::{decode_bytes, decode_json, encode_bytes, encode_json};
use dingo_store::Store;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::str;

/// A lazy handle to a named collection within a [`crate::Dingo`] database.
///
/// Merely constructing a collection does not touch disk. The first write
/// creates store-level subject entries (collection metadata catalog is Stage 6).
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
        Ok(DeleteReceipt::from_store(key.to_string(), removed, receipt))
    }

    /// Scan live keys in this collection (deterministic key order).
    ///
    /// Returns application keys only. Payload access is via get / get_bytes.
    /// Stage 4b: full collection scan into a bounded Vec. Streaming iterators
    /// for larger-than-memory sets are a later refinement.
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
        // live_entries is BTreeMap-ordered by subject; keys share the same prefix
        // so application key order matches subject order for fixed collection name.
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
    /// Yields `(key, value)` in stable key order without building a result `Vec`.
    /// The primary index may still hold live values in process memory (Stage 3);
    /// this path avoids a second full materialization for callers that process
    /// rows incrementally. Non-JSON payloads error on that entry.
    pub fn scan_json_iter(
        &self,
    ) -> Result<impl Iterator<Item = Result<(String, JsonValue), Error>> + '_, Error> {
        let prefix = collection_prefix(&self.name)?;
        let name = self.name.as_str();
        Ok(self
            .store
            .live_entries()
            .filter_map(move |(subject, body)| {
                if !subject.starts_with(&prefix) {
                    return None;
                }
                match decode_subject(subject) {
                    Some((coll, key)) if coll == name => {
                        Some(decode_json(body).map(|v| (key.to_string(), v)))
                    }
                    _ => None,
                }
            }))
    }

    /// Find JSON documents matching `filter` (DX_SPEC §7.1).
    ///
    /// Unbounded materialization in stable key order. Prefer
    /// [`Self::find_with`] with a limit for large collections.
    pub fn find(&self, filter: &Filter) -> Result<Vec<(String, JsonValue)>, Error> {
        self.find_with(filter, QueryOptions::default())
    }

    /// Find with limit / order options.
    pub fn find_with(
        &self,
        filter: &Filter,
        options: QueryOptions,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        self.scan_json_filtered(filter, &options)
    }

    /// Parse a DX/Mongo-style object filter and find matching documents.
    ///
    /// ```ignore
    /// users.find_json(&json!({
    ///     "status": "active",
    ///     "age": { "$gte": 18 }
    /// }))?;
    /// ```
    pub fn find_json(&self, filter_obj: &JsonValue) -> Result<Vec<(String, JsonValue)>, Error> {
        let filter = Filter::from_json(filter_obj)?;
        self.find(&filter)
    }

    /// Fluent query builder (DX_SPEC §7.2).
    pub fn query(&self) -> QueryBuilder<'_> {
        QueryBuilder::new(self)
    }

    fn scan_json_filtered(
        &self,
        filter: &Filter,
        options: &QueryOptions,
    ) -> Result<Vec<(String, JsonValue)>, Error> {
        let prefix = collection_prefix(&self.name)?;
        let mut out = Vec::new();
        for (subject, body) in self.store.live_entries() {
            if !subject.starts_with(&prefix) {
                continue;
            }
            let Some((coll, key)) = decode_subject(subject) else {
                continue;
            };
            if coll != self.name {
                continue;
            }
            let value = decode_json(body)?;
            if !filter.matches(&value) {
                continue;
            }
            out.push((key.to_string(), value));
        }

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
        }

        if let Some(limit) = options.limit {
            if out.len() > limit {
                out.truncate(limit);
            }
        }
        Ok(out)
    }
}

