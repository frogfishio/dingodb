//! Heap-scoped data façade.

use crate::adaptive_write::{AdaptiveWriteHandle, AdmissionResult};
use crate::durability::DurabilityMode;
use crate::error::{LocatorFault, LocatorFaultKind, StoreError};
use crate::history::SubjectHistory;
use crate::ids::random_id;
use crate::kernel::PhysicalStore;
use crate::layout::hex16;
use crate::secondary::SecondaryIndex;
use crate::store::WriteReceipt;
use residiuum_format::{decode_subject_v2, encode_subject_v2, SubjectObjectKind};
use residiuum_heap::{refresh_capability_or_terminate, HeapCap, Rights};
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};

/// Why a collection key could not contribute a complete body during heap scan.
///
/// Distinct failure modes (DEF-SCAN-001) — do not collapse into one bucket.
/// Locator kinds also carry optional [`LocatorFault`] context on the hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionScanHoleReason {
    /// Named segment media is absent.
    SegmentNotFound,
    /// Storage tier offline / unmounted.
    TierOffline,
    /// Multi-chunk reassembly incomplete.
    PayloadPartial,
    /// Conflicting chunk evidence.
    PayloadConflict,
    /// Locator offset past end of segment media.
    LocatorOffsetInvalid,
    /// Frame verify / checksum failed at locator.
    LocatorFrameVerifyFailed,
    /// Envelope segment id mismatches the index locator.
    LocatorSegmentIdMismatch,
}

impl CollectionScanHoleReason {
    /// Map a fail-closed resolve error into a scan hole reason, if it is a hole.
    pub fn from_store_error(e: &StoreError) -> Option<Self> {
        match e {
            StoreError::SegmentNotFound => Some(Self::SegmentNotFound),
            StoreError::TierOffline(_) => Some(Self::TierOffline),
            StoreError::PayloadPartial => Some(Self::PayloadPartial),
            StoreError::PayloadConflict => Some(Self::PayloadConflict),
            StoreError::LocatorFault(f) => Some(Self::from_fault_kind(f.kind)),
            _ => None,
        }
    }

    /// Map a structured locator fault kind into a scan hole reason.
    pub fn from_fault_kind(k: LocatorFaultKind) -> Self {
        match k {
            LocatorFaultKind::OffsetInvalid => Self::LocatorOffsetInvalid,
            LocatorFaultKind::FrameVerifyFailed => Self::LocatorFrameVerifyFailed,
            LocatorFaultKind::SegmentIdMismatch => Self::LocatorSegmentIdMismatch,
            LocatorFaultKind::SegmentNotFound => Self::SegmentNotFound,
        }
    }

    /// Stable snake_case label for logs / wire JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SegmentNotFound => "segment_not_found",
            Self::TierOffline => "tier_offline",
            Self::PayloadPartial => "payload_partial",
            Self::PayloadConflict => "payload_conflict",
            Self::LocatorOffsetInvalid => "locator_offset_invalid",
            Self::LocatorFrameVerifyFailed => "locator_frame_verify_failed",
            Self::LocatorSegmentIdMismatch => "locator_segment_id_mismatch",
        }
    }
}

/// One collection key that could not be fully resolved during a heap scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionScanHole {
    /// Application collection key (not SubjectV2).
    pub key: Vec<u8>,
    /// Distinct failure reason.
    pub reason: CollectionScanHoleReason,
    /// Structured locator diagnostics when the hole is a media/locator fault.
    pub locator: Option<LocatorFault>,
}

impl CollectionScanHole {
    /// Map a fail-closed resolve error into a scan/find hole, if it is a hole class.
    ///
    /// Used by collection scan and by secondary-index candidate materialization
    /// so incompleteness is tracked across every source (DEF-SCAN-001 blocker #5).
    pub fn from_error(key: Vec<u8>, e: &StoreError) -> Option<Self> {
        match e {
            StoreError::LocatorFault(f) => Some(Self {
                key,
                reason: CollectionScanHoleReason::from_fault_kind(f.kind),
                locator: Some((**f).clone()),
            }),
            StoreError::SegmentNotFound => Some(Self {
                key,
                reason: CollectionScanHoleReason::SegmentNotFound,
                locator: None,
            }),
            StoreError::TierOffline(_) => Some(Self {
                key,
                reason: CollectionScanHoleReason::TierOffline,
                locator: None,
            }),
            StoreError::PayloadPartial => Some(Self {
                key,
                reason: CollectionScanHoleReason::PayloadPartial,
                locator: None,
            }),
            StoreError::PayloadConflict => Some(Self {
                key,
                reason: CollectionScanHoleReason::PayloadConflict,
                locator: None,
            }),
            _ => None,
        }
    }
}

/// One page of a heap collection scan with **explicit holes** (DEF-SCAN-001).
///
/// Callers must not treat `entries.is_empty()` alone as "empty collection":
/// check [`Self::complete`] / [`Self::incomplete`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionScanPage {
    /// Fully resolved (key, body) pairs on this page.
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
    /// Keys examined that could not be fully resolved (distinct reasons).
    pub incomplete: Vec<CollectionScanHole>,
    /// Live subjects examined while filling this page (including holes).
    pub examined: usize,
    /// True only when `incomplete` is empty.
    pub complete: bool,
    /// More subjects may exist after this page (complete-row budget filled).
    pub has_more: bool,
    /// Last examined collection key (complete or hole), for continuation.
    pub last_key: Option<Vec<u8>>,
}

impl CollectionScanPage {
    /// True when no complete rows and no holes — an empty live set for this prefix.
    pub fn is_empty_live(&self) -> bool {
        self.entries.is_empty() && self.incomplete.is_empty()
    }
}

/// Capability-gated heap store. All methods re-check capability liveness.
pub struct HeapStore {
    physical: Arc<Mutex<PhysicalStore>>,
    cap: HeapCap,
    /// When present and lease-active, puts/deletes admit through AWO (AWO-3).
    adaptive: Option<AdaptiveWriteHandle>,
}

impl HeapStore {
    pub(super) fn from_host_with_adaptive(
        physical: Arc<Mutex<PhysicalStore>>,
        cap: HeapCap,
        adaptive: Option<AdaptiveWriteHandle>,
    ) -> Self {
        Self {
            physical,
            cap,
            adaptive,
        }
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
        self.put_if(subject, value, crate::WriteCondition::Unconditional)
    }

    /// Conditional put under the store mutex (APB-2 Key Atomic).
    pub fn put_if(
        &self,
        subject: &[u8],
        value: &[u8],
        condition: crate::WriteCondition,
    ) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.require_right(Rights::WRITE)?;
        self.require_subject_v2(subject, None, None)?;
        let receipt = {
            let mut guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            if let Some(awo) = self.adaptive.as_ref().filter(|h| h.lease_active()) {
                match awo.admit_put(
                    &mut guard,
                    subject,
                    value,
                    DurabilityMode::Durable,
                    condition,
                ) {
                    AdmissionResult::Admitted(c) => c
                        .wait()
                        .map_err(crate::adaptive_write::AdaptiveWriteHandle::to_store_error)?,
                    AdmissionResult::Rejected(e) => {
                        return Err(crate::adaptive_write::AdaptiveWriteHandle::to_store_error(e));
                    }
                }
            } else {
                guard.put_subject_bytes_if(subject, value, DurabilityMode::Durable, condition)?
            }
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

    /// Current store segment fingerprint (authoritative frontier candidate).
    ///
    /// Used by APB-6 read-view pins and index builds. Requires [`Rights::READ`].
    pub fn segment_fingerprint(&self) -> Result<[u8; 32], StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        let guard = self
            .physical
            .lock()
            .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
        guard.segment_fingerprint()
    }

    /// Delete by SubjectV2 key within the bound heap.
    pub fn delete(&self, subject: &[u8]) -> Result<WriteReceipt, StoreError> {
        self.delete_if(subject, crate::WriteCondition::Unconditional)
    }

    /// Conditional delete under the store mutex (APB-2 Key Atomic).
    pub fn delete_if(
        &self,
        subject: &[u8],
        condition: crate::WriteCondition,
    ) -> Result<WriteReceipt, StoreError> {
        self.gate()?;
        self.require_right(Rights::WRITE)?;
        self.require_subject_v2(subject, None, None)?;
        let receipt = {
            let mut guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            if let Some(awo) = self.adaptive.as_ref().filter(|h| h.lease_active()) {
                match awo.admit_delete(
                    &mut guard,
                    subject,
                    DurabilityMode::Durable,
                    condition,
                ) {
                    AdmissionResult::Admitted(c) => c
                        .wait()
                        .map_err(crate::adaptive_write::AdaptiveWriteHandle::to_store_error)?,
                    AdmissionResult::Rejected(e) => {
                        return Err(crate::adaptive_write::AdaptiveWriteHandle::to_store_error(e));
                    }
                }
            } else {
                guard.delete_subject_bytes_if(subject, DurabilityMode::Durable, condition)?
            }
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
        self.put_collection_if(collection_id, key, value, crate::WriteCondition::Unconditional)
    }

    /// Conditional collection put (APB-2 Key Atomic under store lock).
    pub fn put_collection_if(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
        value: &[u8],
        condition: crate::WriteCondition,
    ) -> Result<WriteReceipt, StoreError> {
        let subject = residiuum_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.put_if(&subject, value, condition)
    }

    /// Get a collection-scoped SubjectV2 value.
    pub fn get_collection(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let subject = residiuum_format::encode_subject_v2(
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
        self.delete_collection_if(collection_id, key, crate::WriteCondition::Unconditional)
    }

    /// Conditional collection delete (APB-2 Key Atomic under store lock).
    pub fn delete_collection_if(
        &self,
        collection_id: &[u8; 16],
        key: &[u8],
        condition: crate::WriteCondition,
    ) -> Result<WriteReceipt, StoreError> {
        let subject = residiuum_format::encode_subject_v2(
            self.cap.heap_id().as_bytes(),
            SubjectObjectKind::Collection,
            collection_id,
            key,
        )
        .map_err(|e| StoreError::HeapAdmit(format!("subject v2 encode: {e}")))?;
        self.delete_if(&subject, condition)
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
        let subject = residiuum_format::encode_subject_v2(
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
                residiuum_format::encode_subject_v2(
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
        // DEF-SCAN-001 blocker #5 (construction-time): incomplete locators during
        // the build walk must prevent Ready+complete_coverage. Otherwise document B
        // is omitted from postings, the index is Ready, and empty lookup falsely
        // proves absence (no candidate remains to surface a query-time hole).
        let saw_incomplete = self.fill_index_from_collection(collection_id, &mut idx)?;
        let fp_final = {
            let guard = self
                .physical
                .lock()
                .map_err(|_| StoreError::HeapCapability("store lock poisoned".into()))?;
            guard.segment_fingerprint()?
        };
        if saw_incomplete {
            idx.mark_partial(
                fp_final,
                "incomplete locators/payloads during index build; absence not proven",
            );
        } else if fp_final == idx.meta.source_frontier {
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

    /// Walk live collection subjects into `idx`. Returns `true` if any subject
    /// was incomplete (unresolved locator / partial payload / etc.) and was
    /// therefore **not** posted — caller must not mark Ready+complete_coverage.
    fn fill_index_from_collection(
        &self,
        collection_id: &[u8; 16],
        idx: &mut SecondaryIndex,
    ) -> Result<bool, StoreError> {
        let mut after: Option<Vec<u8>> = None;
        let mut saw_incomplete = false;
        loop {
            let page = self.scan_collection_page(collection_id, 4096, after.as_deref())?;
            if !page.incomplete.is_empty() {
                // Construction-time holes: do not invent field postings; record
                // that coverage is incomplete so Ready absence proofs are refused.
                saw_incomplete = true;
            }
            if page.entries.is_empty() && page.incomplete.is_empty() && !page.has_more {
                break;
            }
            for (key, body) in &page.entries {
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
            // Resume after last *examined* key (complete or hole), never only
            // the last complete row — same cursor honesty as scan pages.
            after = page.last_key.clone();
            if !page.has_more {
                break;
            }
        }
        Ok(saw_incomplete)
    }

    /// Scan live (key, body) pairs with **explicit holes** (DEF-SCAN-001).
    ///
    /// Bodies are raw store payloads (typed SDK tags when written via SDK).
    /// At most `limit` **complete** rows (clamped 1..=4096).
    ///
    /// Unresolved locators are **not** silently dropped into a plain `Vec` of
    /// survivors. The returned [`CollectionScanPage`] carries `entries` and
    /// `incomplete` with **distinct** reasons (offset invalid, frame verify,
    /// segment-id mismatch, segment not found, chunk partial/conflict, tier
    /// offline). `page.complete` is true only when `incomplete` is empty for
    /// the examined subjects.
    ///
    /// Physical [`crate::Store::scan_live_page`] uses the same incomplete-page
    /// posture. Point-get remains fail-closed for a single subject.
    ///
    /// Prefer this name over [`Self::scan_collection`] in new code; the shorter
    /// name remains a compatibility alias with the same page type.
    pub fn scan_collection_page(
        &self,
        collection_id: &[u8; 16],
        limit: usize,
        after_key: Option<&[u8]>,
    ) -> Result<CollectionScanPage, StoreError> {
        self.gate()?;
        self.require_right(Rights::READ)?;
        let limit = limit.clamp(1, 4096);
        let prefix = self.collection_subject_prefix(collection_id);
        let after_subject = match after_key {
            Some(k) => Some(
                residiuum_format::encode_subject_v2(
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
        let mut entries = Vec::new();
        let mut incomplete = Vec::new();
        let mut examined = 0usize;
        let mut last_key: Option<Vec<u8>> = None;
        let mut saw_more = false;
        // Bound total subjects examined (complete + holes). Prevents unbounded
        // walks over hole-only collections while still filling complete rows.
        let examine_budget = limit.saturating_mul(8).clamp(limit, 4096);
        for subject in subjects {
            if entries.len() >= limit || examined >= examine_budget {
                saw_more = true;
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
            examined += 1;
            last_key = Some(key.clone());
            match self.get(&subject) {
                Ok(Some(body)) => entries.push((key, body)),
                Ok(None) => {}
                Err(e) => {
                    if let Some(hole) = CollectionScanHole::from_error(key, &e) {
                        incomplete.push(hole);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        let complete = incomplete.is_empty();
        Ok(CollectionScanPage {
            entries,
            incomplete,
            examined,
            complete,
            has_more: saw_more,
            last_key,
        })
    }

    /// Compatibility alias for [`Self::scan_collection_page`].
    ///
    /// Returns the same incomplete-aware [`CollectionScanPage`] — **not** a
    /// plain survivor `Vec`. Call sites that need explicit page semantics
    /// should use `scan_collection_page`.
    #[inline]
    pub fn scan_collection(
        &self,
        collection_id: &[u8; 16],
        limit: usize,
        after_key: Option<&[u8]>,
    ) -> Result<CollectionScanPage, StoreError> {
        self.scan_collection_page(collection_id, limit, after_key)
    }

    /// Lookup candidate collection keys via a secondary index for equality filters.
    ///
    /// `equalities` is a list of (field path, JSON value) constraints (shallow AND
    /// of equalities). Returns:
    /// - `Ok(None)` — no exclusive index path; caller must scan.
    /// - `Ok(Some(keys))` — index supplies an **exclusive complete** candidate set
    ///   (Ready+complete_coverage only). Partial indexes are skipped even when
    ///   they have hits: a non-empty Partial list can omit peers never posted
    ///   during a damaged build (DEF-SCAN-001 blocker #5 residual).
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
            // Exclusive candidate sets only — Partial hits are incomplete.
            if !idx.meta.may_supply_exclusive_candidates() || idx.meta.fields.is_empty() {
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
            // Empty miss is authoritative: exclusive complete coverage only.
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