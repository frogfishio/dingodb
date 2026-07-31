//! Heap-scoped data façade.

use crate::durability::DurabilityMode;
use crate::error::StoreError;
use crate::history::SubjectHistory;
use crate::ids::random_id;
use crate::kernel::PhysicalStore;
use crate::layout::hex16;
use crate::secondary::SecondaryIndex;
use crate::store::WriteReceipt;
use residuum_format::{decode_subject_v2, encode_subject_v2, SubjectObjectKind};
use residuum_heap::{refresh_capability_or_terminate, HeapCap, Rights};
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};

/// Capability-gated heap store. All methods re-check capability liveness.
pub struct HeapStore {
    physical: Arc<Mutex<PhysicalStore>>,
    cap: HeapCap,
}

impl HeapStore {
    pub(super) fn from_host(physical: Arc<Mutex<PhysicalStore>>, cap: HeapCap) -> Self {
        Self { physical, cap }
    }

    /// Bound heap capability.
    pub fn capability(&self) -> &HeapCap {
        &self.cap
    }

    fn gate(&self) -> Result<(), StoreError> {
        refresh_capability_or_terminate(&self.cap)
            .map_err(|e| StoreError::HeapCapability(e.to_string()))
    }

    fn require_right(&self, required: Rights) -> Result<(), StoreError> {
        if self.cap.rights().contains(required) {
            Ok(())
        } else {
            Err(StoreError::HeapCapability(format!(
                "missing right {}",
                required.bits()
            )))
        }
    }

    /// Decode and validate a SubjectV2 buffer for this bound heap.
    ///
    /// Qualified heap data paths require SubjectV2 (version byte `0x02`). Legacy
    /// v1 string subjects are rejected so foreign-heap names cannot ride the
    /// flat keyspace.
    fn require_subject_v2(
        &self,
        subject: &[u8],
        expect_kind: Option<SubjectObjectKind>,
        expect_object_id: Option<&[u8; 16]>,
    ) -> Result<(), StoreError> {
        let sv2 = decode_subject_v2(subject)
            .map_err(|e| StoreError::HeapAdmit(format!("subject v2: {e}")))?;
        if sv2.heap_id != self.cap.heap_id().as_bytes() {
            return Err(StoreError::HeapAdmit("subject heap mismatch".into()));
        }
        if let Some(kind) = expect_kind {
            if sv2.object_kind != kind {
                return Err(StoreError::HeapAdmit("subject object kind mismatch".into()));
            }
        }
        if let Some(oid) = expect_object_id {
            if sv2.object_id != oid {
                return Err(StoreError::HeapAdmit("subject object id mismatch".into()));
            }
        }
        Ok(())
    }

    /// Put under a SubjectV2 key within the bound heap.
    pub fn put(&self, subject: &[u8], value: &[u8]) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.require_right(Rights::WRITE)?;
        self.require_subject_v2(subject, None, None)?;
        let receipt = {
            let mut guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.put_subject_bytes(subject, value, DurabilityMode::Durable)?
        };
        // Collection writes invalidate derived secondary indexes (DEF-027).
        if let Ok(sv2) = decode_subject_v2(subject) {
            if sv2.object_kind == SubjectObjectKind::Collection {
                self.mark_indexes_stale(sv2.object_id)?;
            }
        }
        Ok(receipt)
    }

    /// Get by SubjectV2 key within the bound heap.
    pub fn get(&self, subject: &[u8]) -> Result<Option<Vec<u8>>, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        self.require_subject_v2(subject, None, None)?;
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.get_subject_bytes(subject)
    }

    /// Delete by SubjectV2 key within the bound heap.
    pub fn delete(&self, subject: &[u8]) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.require_right(Rights::WRITE)?;
        self.require_subject_v2(subject, None, None)?;
        let receipt = {
            let mut guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.delete_subject_bytes(subject, DurabilityMode::Durable)?
        };
        if let Ok(sv2) = decode_subject_v2(subject) {
            if sv2.object_kind == SubjectObjectKind::Collection {
                self.mark_indexes_stale(sv2.object_id)?;
            }
        }
        Ok(receipt)
    }

    /// Put under a collection-scoped SubjectV2 (object id + key must match).
    pub fn put_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
        value: &[u8],
    ) -> Result<WriteReceipt, StoreError> {
        let subject = residuum_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.put(&subject, value)
    }

    /// Get a collection-scoped SubjectV2 value.
    pub fn get_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let subject = residuum_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.get(&subject)
    }

    /// Delete a collection-scoped SubjectV2 value.
    pub fn delete_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
    ) -> Result<WriteReceipt, StoreError> {
        let subject = residuum_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.delete(&subject)
    }

    /// Mark usable secondary indexes for a collection as stale after a write.
    ///
    /// Ready/Partial → Stale (absence proofs disabled). Building/Rebuilding keep
    /// their state but lose complete_coverage. Failures are returned (DEF-027).
    pub fn mark_indexes_stale(&self, collection_id: &[u8; 16]) -> Result<(), StoreError> {
        self.gate()?;
        // Write right already required by caller put/delete; re-check for direct calls.
        self.require_right(Rights::WRITE)?;
        let scope = self.index_scope_key(collection_id);
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        let indexes = guard.list_secondary_indexes(&scope)?;
        for mut idx in indexes {
            let before_state = idx.meta.state;
            let before_cov = idx.meta.complete_coverage;
            idx.mark_stale();
            if idx.meta.state != before_state || idx.meta.complete_coverage != before_cov {
                guard.write_secondary_index(&idx)?;
            }
        }
        Ok(())
    }

    /// Event history for a collection key (SubjectV2), oldest first.
    pub fn history_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
    ) -> Result<SubjectHistory, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        // Prefer ReadHistory when present; Read alone is accepted for the first cut.
        let subject = residuum_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.require_subject_v2(&subject, Some(SubjectObjectKind::Collection), Some(collection_id))?;
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.history_subject_bytes(&subject)
    }

    /// Subject-byte prefix for all keys in one collection under this heap.
    ///
    /// Layout: `0x02 || heap_id || 0x01 || collection_id` (without key length/key).
    pub fn collection_subject_prefix(&self, collection_id: &[u8; 16]) -> Vec<u8> {
        let mut p = Vec::with_capacity(1 + 16 + 1 + 16);
        p.push(0x02);
        p.extend_from_slice(self.cap.heap_id().as_bytes());
        p.push(SubjectObjectKind::Collection as u8);
        p.extend_from_slice(collection_id);
        p
    }

    /// List application keys in a collection (SubjectV2), ordered by subject.
    ///
    /// `after_key` resumes after that application key (not a continuation token).
    /// At most `limit` keys are returned (clamped 1..=4096).
    pub fn list_collection_keys(
        &self,
        collection_id: &[u8; 16],
        limit: usize,
        after_key: Option<&[u8]>,
    ) -> Result<Vec<Vec<u8>>, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        let limit = limit.clamp(1, 4096);
        let prefix = self.collection_subject_prefix(collection_id);
        let after_subject = match after_key {
            Some(k) => Some(
                residuum_format::encode_subject_v2(
                    self.cap.heap_id().as_bytes(),
                    SubjectObjectKind::Collection,
                    collection_id,
                    k,
                )
                .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?,
            ),
            None => None,
        };
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        let subjects = guard.index_live_after(after_subject.as_deref(), Some(&prefix));
        drop(guard);
        let mut out = Vec::new();
        for subject in subjects {
            if out.len() >= limit {
                break;
            }
            match decode_subject_v2(&subject) {
                Ok(sv2)
                    if sv2.heap_id == self.cap.heap_id().as_bytes()
                        && sv2.object_kind == SubjectObjectKind::Collection
                        && sv2.object_id == collection_id =>
                {
                    out.push(sv2.key.to_vec());
                }
                _ => continue,
            }
        }
        Ok(out)
    }

    /// Stable secondary-index path key: unique per heap + collection id.
    ///
    /// Avoids cross-heap collision when two heaps share a human collection name.
    pub fn index_scope_key(&self, collection_id: &[u8; 16]) -> String {
        format!(
            "h{}-c{}",
            hex16(self.cap.heap_id().as_bytes()),
            hex16(collection_id)
        )
    }

    /// List secondary indexes for a collection (metadata only).
    pub fn list_indexes(
        &self,
        collection_id: &[u8; 16],
    ) -> Result<Vec<SecondaryIndex>, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        let scope = self.index_scope_key(collection_id);
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.list_secondary_indexes(&scope)
    }

    /// Create (or rebuild definition) a field index over JSON documents.
    ///
    /// First cut: full rebuild from a SubjectV2 collection scan (no resume).
    /// Requires [`Rights::INDEX_ADMIN`]. Build scan also needs Read on the cap.
    pub fn create_index(
        &self,
        collection_id: &[u8; 16],
        name: &str,
        fields: &[&str],
    ) -> Result<SecondaryIndex, StoreError> {
        self.gate()?;
        self.require_right(Rights::INDEX_ADMIN)?;
        if name.is_empty() || name.len() > 256 {
            return Err(StoreError::HeapAdmit("index name invalid".into()));
        }
        if fields.is_empty() || fields.len() > 16 {
            return Err(StoreError::HeapAdmit("index fields invalid".into()));
        }
        let field_owned: Vec<String> = fields.iter().map(|s| (*s).to_string()).collect();
        let scope = self.index_scope_key(collection_id);
        let mut idx = SecondaryIndex::new_building(&scope, name, field_owned);
        let build_id = random_id()?;
        let fp = {
            let guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.segment_fingerprint()?
        };
        idx.begin_build(build_id, fp, false);
        self.fill_index_from_collection(collection_id, &mut idx)?;
        let fp_final = {
            let guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.segment_fingerprint()?
        };
        if fp_final == idx.meta.source_frontier {
            idx.mark_ready(fp_final);
        } else {
            idx.mark_partial(fp_final, "source frontier drifted during build");
        }
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.write_secondary_index(&idx)?;
        Ok(idx)
    }

    /// Drop a secondary index by name.
    pub fn drop_index(&self, collection_id: &[u8; 16], name: &str) -> Result<(), StoreError> {
        self.gate()?;
        self.require_right(Rights::INDEX_ADMIN)?;
        let scope = self.index_scope_key(collection_id);
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.delete_secondary_index(&scope, name)
    }

    /// Rebuild an existing index definition from a full collection scan.
    pub fn rebuild_index(
        &self,
        collection_id: &[u8; 16],
        name: &str,
    ) -> Result<SecondaryIndex, StoreError> {
        self.gate()?;
        self.require_right(Rights::INDEX_ADMIN)?;
        let scope = self.index_scope_key(collection_id);
        let existing = {
            let guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.load_secondary_index(&scope, name)?
        }
        .ok_or_else(|| StoreError::HeapAdmit("index not found".into()))?;
        let fields: Vec<String> = existing.meta.fields.clone();
        let field_refs: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
        // Drop then recreate with same fields.
        self.drop_index(collection_id, name)?;
        self.create_index(collection_id, name, &field_refs)
    }

    fn fill_index_from_collection(
        &self,
        collection_id: &[u8; 16],
        idx: &mut SecondaryIndex,
    ) -> Result<(), StoreError> {
        let mut after: Option<Vec<u8>> = None;
        loop {
            let page = self.scan_collection(collection_id, 4096, after.as_deref())?;
            if page.is_empty() {
                break;
            }
            for (key, body) in &page {
                let subject = encode_subject_v2(
                    self.cap.heap_id().as_bytes(),
                    SubjectObjectKind::Collection,
                    collection_id,
                    key,
                )
                .map_err(|e| StoreError::HeapAdmit(format!("subject v2: {e}")))?;
                if body.first() != Some(&0x01) {
                    continue;
                }
                let Ok(doc) = serde_json::from_slice::<JsonValue>(&body[1..]) else {
                    continue;
                };
                if let Some(ik) = index_key_from_doc(&doc, &idx.meta.fields) {
                    idx.insert(ik, subject);
                }
            }
            after = page.last().map(|(k, _)| k.clone());
            if page.len() < 4096 {
                break;
            }
        }
        Ok(())
    }

    /// Scan live (key, body) pairs in a collection under this heap.
    ///
    /// Bodies are raw store payloads (typed SDK tags when written via SDK).
    /// At most `limit` complete rows (clamped 1..=4096).
    pub fn scan_collection(
        &self,
        collection_id: &[u8; 16],
        limit: usize,
        after_key: Option<&[u8]>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        let limit = limit.clamp(1, 4096);
        let prefix = self.collection_subject_prefix(collection_id);
        let after_subject = match after_key {
            Some(k) => Some(
                residuum_format::encode_subject_v2(
                    self.cap.heap_id().as_bytes(),
                    SubjectObjectKind::Collection,
                    collection_id,
                    k,
                )
                .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?,
            ),
            None => None,
        };
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        let subjects = guard.index_live_after(after_subject.as_deref(), Some(&prefix));
        drop(guard);
        let mut out = Vec::new();
        for subject in subjects {
            if out.len() >= limit {
                break;
            }
            let sv2 = match decode_subject_v2(&subject) {
                Ok(s)
                    if s.heap_id == self.cap.heap_id().as_bytes()
                        && s.object_kind == SubjectObjectKind::Collection
                        && s.object_id == collection_id =>
                {
                    s
                }
                _ => continue,
            };
            let key = sv2.key.to_vec();
            match self.get(&subject)? {
                Some(body) => out.push((key, body)),
                None => continue,
            }
        }
        Ok(out)
    }

    /// Lookup candidate collection keys via a secondary index for equality filters.
    ///
    /// `equalities` is a list of (field path, JSON value) constraints (shallow AND
    /// of equalities). Returns:
    /// - `Ok(None)` — no usable index matches; caller must scan.
    /// - `Ok(Some(keys))` — index path used; keys are application collection keys
    ///   (may be empty when Ready+complete_coverage proves absence).
    pub fn lookup_index_keys(
        &self,
        collection_id: &[u8; 16],
        equalities: &[(String, JsonValue)],
    ) -> Result<Option<Vec<Vec<u8>>>, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        if equalities.is_empty() {
            return Ok(None);
        }
        let scope = self.index_scope_key(collection_id);
        let indexes = {
            let guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.list_secondary_indexes(&scope)?
        };
        for idx in indexes {
            if !idx.meta.may_accelerate_hits() || idx.meta.fields.is_empty() {
                continue;
            }
            // All index fields must appear as equalities.
            let mut values = Vec::new();
            let mut ok = true;
            for f in &idx.meta.fields {
                match equalities.iter().find(|(path, _)| path == f) {
                    Some((_, v)) => values.push(v.clone()),
                    None => {
                        ok = false;
                        break;
                    }
                }
            }
            if !ok {
                continue;
            }
            let key = match index_key_from_values(&values) {
                Some(k) => k,
                None => continue,
            };
            let subjects = idx.lookup(&key).to_vec();
            // Empty miss is authoritative only when Ready + complete_coverage.
            if subjects.is_empty() && !idx.meta.may_prove_absence() {
                continue;
            }
            let mut keys = Vec::new();
            for subject in subjects {
                let sv2 = match decode_subject_v2(&subject) {
                    Ok(s)
                        if s.heap_id == self.cap.heap_id().as_bytes()
                            && s.object_kind == SubjectObjectKind::Collection
                            && s.object_id == collection_id =>
                    {
                        s
                    }
                    _ => continue,
                };
                keys.push(sv2.key.to_vec());
            }
            return Ok(Some(keys));
        }
        Ok(None)
    }
}

/// Build opaque index key from ordered JSON field values (same encoding as build).
fn index_key_from_values(values: &[JsonValue]) -> Option<Vec<u8>> {
    let mut parts = Vec::new();
    for v in values {
        parts.push(serde_json::to_vec(v).ok()?);
    }
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            out.push(0x1f);
        }
        out.extend_from_slice(p);
    }
    Some(out)
}

/// Build opaque index key bytes from ordered field values (JSON text).
fn index_key_from_doc(doc: &JsonValue, fields: &[String]) -> Option<Vec<u8>> {
    let mut parts = Vec::new();
    for f in fields {
        let v = resolve_json_path(doc, f)?;
        let enc = serde_json::to_vec(v).ok()?;
        parts.push(enc);
    }
    let mut out = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            out.push(0x1f);
        }
        out.extend_from_slice(p);
    }
    Some(out)
}

fn resolve_json_path<'a>(doc: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let mut cur = doc;
    for seg in path.split('.') {
        cur = cur.as_object()?.get(seg)?;
    }
    Some(cur)
}