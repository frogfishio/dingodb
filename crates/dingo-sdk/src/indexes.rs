//! Secondary indexes (DX_SPEC §8, Stage 6).
//!
//! Indexes are derived, online, resumable, and deletable. Queries remain
//! correct without them via scan (+ optional budget). Work over both embedded
//! stores and remote `dingo serve` connections (Stage 7 remote parity).

use crate::dingo::Backend;
use crate::error::Error;
use crate::filter::{resolve_path_value, Filter, Pred};
use crate::subject::{decode_subject, encode_subject};
use crate::value::decode_json;
use dingo_store::{IndexState, SecondaryIndex, Store};
use serde_json::Value as JsonValue;

/// Public view of a secondary index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexInfo {
    /// Index name within the collection.
    pub name: String,
    /// Collection name.
    pub collection: String,
    /// Indexed field paths.
    pub fields: Vec<String>,
    /// Lifecycle state.
    pub state: IndexState,
    /// Number of postings.
    pub entry_count: u64,
    /// Whether the index claims complete coverage of live keys.
    pub complete_coverage: bool,
}

impl IndexInfo {
    pub(crate) fn from_store(idx: &SecondaryIndex) -> Self {
        Self {
            name: idx.meta.name.clone(),
            collection: idx.meta.collection.clone(),
            fields: idx.meta.fields.clone(),
            state: idx.meta.state,
            entry_count: idx.meta.entry_count,
            complete_coverage: idx.meta.complete_coverage,
        }
    }
}

/// Handle for managing indexes on a collection (embedded or remote).
pub struct Indexes<'a> {
    pub(crate) backend: &'a mut Backend,
    pub(crate) collection: String,
}

impl<'a> Indexes<'a> {
    /// Create (or rebuild) a field index. Online: builds from current live docs.
    pub fn create(&mut self, name: &str, fields: &[&str]) -> Result<IndexInfo, Error> {
        match self.backend {
            Backend::Local(store) => create_index_on_store(store, &self.collection, name, fields),
            Backend::Remote(client) => client.index_create(&self.collection, name, fields),
            Backend::Cluster(_) => Err(Error::RemoteUnsupported(
                "secondary indexes on cluster (Stage 8e+)",
            )),
        }
    }

    /// List indexes on this collection.
    pub fn list(&mut self) -> Result<Vec<IndexInfo>, Error> {
        match self.backend {
            Backend::Local(store) => Ok(store
                .list_secondary_indexes(&self.collection)?
                .iter()
                .map(IndexInfo::from_store)
                .collect()),
            Backend::Remote(client) => client.index_list(&self.collection),
            Backend::Cluster(_) => Ok(Vec::new()),
        }
    }

    /// Get one index by name.
    pub fn get(&mut self, name: &str) -> Result<Option<IndexInfo>, Error> {
        match self.backend {
            Backend::Local(store) => Ok(store
                .load_secondary_index(&self.collection, name)?
                .map(|i| IndexInfo::from_store(&i))),
            Backend::Remote(client) => {
                let all = client.index_list(&self.collection)?;
                Ok(all.into_iter().find(|i| i.name == name))
            }
            Backend::Cluster(_) => Ok(None),
        }
    }

    /// Delete an index (never deletes authoritative data).
    pub fn drop(&mut self, name: &str) -> Result<(), Error> {
        match self.backend {
            Backend::Local(store) => {
                store.delete_secondary_index(&self.collection, name)?;
                Ok(())
            }
            Backend::Remote(client) => client.index_drop(&self.collection, name),
            Backend::Cluster(_) => Err(Error::RemoteUnsupported(
                "secondary indexes on cluster (Stage 8e+)",
            )),
        }
    }

    /// Rebuild an existing index definition from live data.
    pub fn rebuild(&mut self, name: &str) -> Result<IndexInfo, Error> {
        match self.backend {
            Backend::Local(store) => {
                let existing = store
                    .load_secondary_index(&self.collection, name)?
                    .ok_or_else(|| Error::QueryInvalid(format!("index not found: {name}")))?;
                let fields: Vec<&str> = existing.meta.fields.iter().map(|s| s.as_str()).collect();
                create_index_on_store(store, &self.collection, name, &fields)
            }
            Backend::Remote(client) => client.index_rebuild(&self.collection, name),
            Backend::Cluster(_) => Err(Error::RemoteUnsupported(
                "secondary indexes on cluster (Stage 8e+)",
            )),
        }
    }
}

/// Build (or rebuild) a secondary field index on an open store.
pub(crate) fn create_index_on_store(
    store: &mut Store,
    collection: &str,
    name: &str,
    fields: &[&str],
) -> Result<IndexInfo, Error> {
    if name.is_empty() {
        return Err(Error::InvalidKey("index name empty"));
    }
    if fields.is_empty() {
        return Err(Error::QueryInvalid(
            "index requires at least one field".into(),
        ));
    }
    let field_owned: Vec<String> = fields.iter().map(|s| (*s).to_string()).collect();
    let mut idx = SecondaryIndex::new_building(collection, name, field_owned.clone());

    // Full build from live logical entries.
    let fp = store.segment_fingerprint()?;
    let live = store.live_logical_entries()?;
    for (subject, body) in live {
        let Some((coll, _key)) = decode_subject(&subject) else {
            continue;
        };
        if coll != collection {
            continue;
        }
        let Ok(doc) = decode_json(&body) else {
            continue;
        };
        if let Some(ikey) = index_key_for_doc(&doc, &field_owned) {
            idx.insert(ikey, subject);
        }
    }
    idx.mark_ready(fp);
    store.write_secondary_index(&idx)?;
    Ok(IndexInfo::from_store(&idx))
}

/// Mark usable secondary indexes on `collection` as stale after a write.
pub(crate) fn mark_indexes_stale(store: &mut Store, collection: &str) -> Result<(), Error> {
    let indexes = store.list_secondary_indexes(collection)?;
    for mut idx in indexes {
        if idx.meta.state.usable() || idx.meta.state == IndexState::Ready {
            idx.mark_stale();
            store.write_secondary_index(&idx)?;
        }
    }
    Ok(())
}

/// Build opaque index key bytes from ordered field values (JSON text).
pub(crate) fn index_key_for_doc(doc: &JsonValue, fields: &[String]) -> Option<Vec<u8>> {
    let mut parts = Vec::new();
    for f in fields {
        let v = resolve_path_value(doc, f)?;
        // Stable JSON encoding for the field value.
        let enc = serde_json::to_vec(v).ok()?;
        parts.push(enc);
    }
    // Join with 0x1f unit separator so multi-field keys stay unambiguous.
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            out.push(0x1f);
        }
        out.extend_from_slice(p);
    }
    Some(out)
}

/// If the filter is a simple equality (or AND of equalities) on fields that
/// match an index prefix, return candidate subjects from that index.
pub(crate) fn try_index_lookup(
    store: &Store,
    collection: &str,
    filter: &Filter,
) -> Result<Option<(IndexInfo, Vec<Vec<u8>>)>, Error> {
    let eqs = equality_fields(filter);
    if eqs.is_empty() {
        return Ok(None);
    }
    let indexes = store.list_secondary_indexes(collection)?;
    // Prefer a ready index whose fields are a prefix of the equality set order.
    for idx in indexes {
        if !idx.meta.state.usable() {
            continue;
        }
        if idx.meta.fields.is_empty() {
            continue;
        }
        // All index fields must appear as equalities.
        let mut values = Vec::new();
        let mut ok = true;
        for f in &idx.meta.fields {
            match eqs.iter().find(|(path, _)| path == f) {
                Some((_, v)) => values.push((*v).clone()),
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            continue;
        }
        let key = {
            let mut out = Vec::new();
            for (i, v) in values.iter().enumerate() {
                if i > 0 {
                    out.push(0x1f);
                }
                out.extend_from_slice(&serde_json::to_vec(v)?);
            }
            out
        };
        let subjects = idx.lookup(&key).to_vec();
        return Ok(Some((IndexInfo::from_store(&idx), subjects)));
    }
    Ok(None)
}

/// Extract field equality constraints from a filter (shallow AND of Eq only).
fn equality_fields(filter: &Filter) -> Vec<(String, JsonValue)> {
    match filter {
        Filter::Field {
            path,
            pred: Pred::Eq(v),
        } => vec![(path.clone(), v.clone())],
        Filter::And(parts) => {
            let mut out = Vec::new();
            for p in parts {
                out.extend(equality_fields(p));
            }
            out
        }
        Filter::Always => Vec::new(),
        _ => Vec::new(),
    }
}

/// Encode a collection key into a store subject (helper for tests/SDK).
#[allow(dead_code)]
pub(crate) fn subject_for(collection: &str, key: &str) -> Result<Vec<u8>, Error> {
    encode_subject(collection, key)
}
