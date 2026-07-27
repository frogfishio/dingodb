//! Filesystem-backed append store (OVERVIEW §§6–7, §9; Stages 3, 6, 9).

use crate::catalog::{
    collections_catalog_path, try_load_collection_catalog, write_collection_catalog,
    CollectionCatalog,
};
use crate::chunk_payload::{
    decode_chunk_manifest, decode_piece_body, encode_chunk_manifest, encode_piece_body,
    is_chunk_manifest, manifest_from_pieces, reassemble_with_manifest, split_into_pieces,
    PayloadResult, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_THRESHOLD,
};
use crate::compact::{
    estimate_compact_bytes, new_planned_job, reclaim_source_segments, reclaimable_source_ids,
    report_from_job, try_load_checkpoint, try_load_compact_job, verify_live_segment,
    write_checkpoint, write_compact_job, write_live_segment, CheckpointMeta, CompactJob,
    CompactOptions, CompactPhase, CompactReport,
};

use crate::durability::DurabilityMode;
use crate::envelope::{
    decode_item_envelope, encode_item_envelope, EventKind, ItemEnvelope, MAX_SUBJECT_LEN,
};
use crate::error::StoreError;
use crate::history::{subject_history_tiered, SubjectHistory};
use crate::ids::{mint_sortable_segment_id, random_id, segment_seq_from_id, subject_item_id};
use crate::index::PrimaryIndex;
use crate::index_cache::{
    primary_cache_path, segment_fingerprint, try_load_primary_index,
    try_load_primary_index_frontier, write_primary_index_frontier, IndexFrontier,
};
use crate::layout::{list_dingo_files, StorePaths};
use crate::secondary::{
    delete_secondary_index, list_secondary_index_paths, secondary_index_path,
    try_load_secondary_index, write_secondary_index, SecondaryIndex,
};
use crate::segment_catalog::{
    rebuild_segment_catalog, segment_catalog_path, try_load_segment_catalog, write_segment_catalog,
    SegmentCatalog, SegmentSummary,
};
use crate::tier::{
    classify_segment_bytes, discover_placements, load_tier_roots_file, register_hot_segment,
    tier_placement_path, transfer_segment, try_load_placement, write_placement,
    write_tier_roots_file, FormatClassification, MigrationEvidence, TierAwareGet, TierClass,
    TierCoverage, TierMoveMode, TierPlacement,
};
use crate::write_dedup::{
    load_write_dedup, save_write_dedup, write_dedup_path, DedupRecord, WriteDedupTable,
};
use crate::writer_lock::WriterLock;
use dingo_format::{
    decode_store_descriptor_body, encode_store_descriptor_frame, scan_forward, ActiveSegment,
    FrameFlags, FrameHeader, FrameKind, FrameParts, SafetyLimits, SegmentId,
};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Draft meta format version written under `store-info/meta`.
const META_VERSION: &str = "dingo-store-9\n";

/// Soft max size of the active segment before auto-seal (bytes).
const DEFAULT_SEAL_THRESHOLD: u64 = 4 * 1024 * 1024;

/// How many buffered/durable writes may land before a full index-cache checkpoint
/// is forced (DEF-023 rate limit). Catalog refresh is cheap and not limited.
const DERIVED_CHECKPOINT_EVERY_OPS: u64 = 32;

/// Why a live subject could not contribute a complete logical body (DEF-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompleteReason {
    /// Chunked payload is only partially available.
    PayloadPartial,
    /// No surviving chunk bodies for a declared manifest.
    PayloadUnavailable,
    /// Conflicting chunk content at a manifest position.
    PayloadConflict,
    /// Subject bytes are not valid UTF-8 (cannot be addressed by string APIs).
    NonUtf8Subject,
}

/// One live subject that could not be fully read during a logical scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveIncomplete {
    /// Subject key bytes.
    pub subject: Vec<u8>,
    /// Why reassembly failed.
    pub reason: IncompleteReason,
}

/// Result of scanning live logical payloads with coverage honesty (DEF-012).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveLogicalScan {
    /// Fully reassembled live entries.
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
    /// Live subjects that are not fully readable.
    pub incomplete: Vec<LiveIncomplete>,
    /// True only when `incomplete` is empty **and** tier coverage is complete.
    pub complete: bool,
    /// Offline / unmounted tiers or unavailable segments prevent proven completeness.
    pub tier_coverage_incomplete: bool,
}

/// One unfenced page of live bodies for secondary index construction (DEF-027).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexBuildPage {
    /// Complete (subject, body) pairs on this page.
    pub entries: Vec<(Vec<u8>, Vec<u8>)>,
    /// Subjects skipped because payload reassembly was incomplete.
    pub incomplete: Vec<Vec<u8>>,
    /// More live subjects remain after `after`.
    pub has_more: bool,
    /// Exclusive resume point (last examined subject), when any work ran.
    pub after: Option<Vec<u8>>,
    /// Subjects examined (complete + incomplete).
    pub examined: usize,
}

/// Receipt returned after an acknowledged write (OVERVIEW §7.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReceipt {
    /// Store identifier.
    pub store_id: [u8; 16],
    /// Segment that received the frame.
    pub segment_id: [u8; 16],
    /// Item lineage identifier.
    pub item_id: [u8; 16],
    /// Unique event identifier for this write.
    pub event_id: [u8; 16],
    /// Event kind that was recorded.
    pub event_kind: EventKind,
    /// Durability mode that was actually applied.
    pub durability: DurabilityMode,
    /// Byte offset of the frame within the segment file.
    pub offset: u64,
}

/// Summary of a catalog-free salvage pass over all segment files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SalvageReport {
    /// Files scanned.
    pub files_scanned: usize,
    /// Structurally verified frames (all kinds).
    pub verified_frames: u64,
    /// Verified item events with decodable draft envelopes.
    pub item_events: u64,
    /// Explicit holes found across files.
    pub holes: u64,
    /// Live subjects after applying events in file order.
    pub live_subjects: usize,
}

/// Result of non-destructive salvage / export into a new store path (Stage 7 + DEF-011).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SalvageCopyReport {
    /// Scan summary of the **source** store (source is never mutated).
    pub source: SalvageReport,
    /// Destination store root that received recovered evidence or live state.
    pub destination: PathBuf,
    /// Recovery mode used for this copy.
    pub mode: crate::recovery::SalvageMode,
    /// Live subjects present in the destination after recovery.
    pub subjects_copied: usize,
    /// Verified frames byte-copied (evidence mode); zero for live-state export.
    pub frames_copied: u64,
    /// Holes recorded in the recovery manifest (evidence mode).
    pub holes_recorded: u64,
    /// Path of the recovery manifest when written (evidence mode).
    pub manifest_path: Option<PathBuf>,
}

/// Open single-node store handle.
pub struct Store {
    paths: StorePaths,
    store_id: [u8; 16],
    limits: SafetyLimits,
    /// Visibility index (includes memory-mode publishes).
    index: PrimaryIndex,
    /// Segment-derived durable projection only (DEF-013 / DEF-023).
    ///
    /// Updated only after buffered/durable append succeeds. Never includes
    /// memory-mode visibility. Used for index-cache and on-disk catalog writes
    /// so the write path never rescans sealed segment bytes.
    durable_index: PrimaryIndex,
    /// Buffered/durable ops since the last full index-cache checkpoint (DEF-023).
    derived_ops_since_checkpoint: u64,
    /// Active in-memory segment + file, if any.
    active: Option<ActiveWriter>,
    /// Counter used to mint sortable segment ids (recovered from on-disk max).
    segment_seq: u64,
    /// Seal active segment when it reaches this many bytes.
    seal_threshold: u64,
    /// Bodies larger than this are written as chunked payloads (Stage 6).
    chunk_threshold: usize,
    /// Max logical bytes per payload-chunk frame.
    chunk_size: usize,
    /// Derived collection catalog (rebuildable).
    collection_catalog: CollectionCatalog,
    /// Segment placement across storage tiers (Stage 9, derived).
    tier_placement: TierPlacement,
    /// Hierarchical segment summary catalog (Stage 9, derived).
    segment_catalog: SegmentCatalog,
    /// Exclusive writer ownership (DEF-020). `None` for inspect/read-only opens.
    writer_lock: Option<WriterLock>,
    /// Client operation dedup table (DEF-010); empty when unused.
    write_dedup: WriteDedupTable,
}

struct ActiveWriter {
    segment_id: [u8; 16],
    segment: ActiveSegment,
    file: File,
    /// Bytes known durable on disk for this file (complete frames only).
    durable_len: u64,
}

impl Store {
    /// Create a new store at `path` (directory). Fails if a store already exists.
    pub fn create(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let paths = StorePaths::new(path.as_ref());
        if paths.looks_like_store() {
            return Err(StoreError::AlreadyExists(paths.root.clone()));
        }
        if paths.root.exists() {
            if !paths.root.is_dir() {
                return Err(StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "path exists and is not a directory",
                )));
            }
            // Allow empty directory only.
            if fs::read_dir(&paths.root)?.next().is_some() {
                return Err(StoreError::AlreadyExists(paths.root.clone()));
            }
        }
        paths.create_dirs()?;
        // Exclusive ownership before any authoritative write (DEF-020).
        let writer_lock = WriterLock::acquire(&paths)?;
        let store_id = random_id()?;
        let created_ns = now_ns();
        crate::atomic_file::write_atomic(&paths.store_id_file(), &store_id)?;
        crate::atomic_file::write_atomic(&paths.meta_file(), META_VERSION.as_bytes())?;
        crate::failpoint::hit("store.create.after_meta")?;
        write_store_descriptor_file(&paths, store_id, created_ns)?;
        // Ensure parent dir entry is durable for create.
        sync_dir(&paths.root)?;

        let mut store = Self {
            paths,
            store_id,
            limits: SafetyLimits::default(),
            index: PrimaryIndex::new(),
            durable_index: PrimaryIndex::new(),
            derived_ops_since_checkpoint: 0,
            active: None,
            segment_seq: 0,
            seal_threshold: DEFAULT_SEAL_THRESHOLD,
            chunk_threshold: DEFAULT_CHUNK_THRESHOLD,
            chunk_size: DEFAULT_CHUNK_SIZE,
            collection_catalog: CollectionCatalog::new(),
            tier_placement: TierPlacement::new(),
            segment_catalog: SegmentCatalog::new(),
            writer_lock: Some(writer_lock),
            write_dedup: WriteDedupTable::new(),
        };
        store.start_active_segment()?;
        store.persist_active(DurabilityMode::Durable)?;
        crate::failpoint::hit("store.create.after_active_header")?;
        store.persist_index_cache()?;
        store.refresh_collection_catalog()?;
        store.refresh_tier_state()?;
        Ok(store)
    }

    /// Open an existing store, or create if the path does not exist yet.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let root = path.as_ref();
        let paths = StorePaths::new(root);
        if !paths.looks_like_store() {
            if root.exists() {
                // Empty directory → create; non-empty without store-info → error.
                if root.is_dir() {
                    let empty = fs::read_dir(root)?.next().is_none();
                    if empty {
                        return Self::create(root);
                    }
                }
                return Err(StoreError::NotAStore(root.to_path_buf()));
            }
            return Self::create(root);
        }

        // Exclusive ownership before opening the active segment (DEF-020).
        let writer_lock = WriterLock::acquire(&paths)?;
        let store_id = read_store_id(&paths)?;
        let meta = fs::read_to_string(paths.meta_file()).unwrap_or_default();
        if !meta.starts_with("dingo-store-") {
            return Err(StoreError::CorruptMeta("unexpected meta version"));
        }
        // Store descriptor is framed evidence, not the sole identity map.
        // Mismatch with store_id is corrupt; absence is tolerated for older trees.
        verify_store_descriptor_if_present(&paths, store_id)?;

        let mut store = Self {
            paths,
            store_id,
            limits: SafetyLimits::default(),
            index: PrimaryIndex::new(),
            durable_index: PrimaryIndex::new(),
            derived_ops_since_checkpoint: 0,
            active: None,
            segment_seq: 0,
            seal_threshold: DEFAULT_SEAL_THRESHOLD,
            chunk_threshold: DEFAULT_CHUNK_THRESHOLD,
            chunk_size: DEFAULT_CHUNK_SIZE,
            collection_catalog: CollectionCatalog::new(),
            tier_placement: TierPlacement::new(),
            segment_catalog: SegmentCatalog::new(),
            writer_lock: Some(writer_lock),
            write_dedup: WriteDedupTable::new(),
        };
        store.load_tier_state()?;
        store.load_or_rebuild_index()?;
        store.load_or_rebuild_catalog()?;
        store.write_dedup = load_write_dedup(&write_dedup_path(&store.paths))?;
        store.resume_or_start_active()?;
        // Finish or cancel incomplete compaction jobs (DEF-024).
        let _ = store.recover_compact_jobs()?;
        Ok(store)
    }

    /// Open an **existing** store for read-only inspection (Stage 7 doctor).
    ///
    /// Never creates a store, never opens the active segment for append, and
    /// never persists derived catalogs/indexes. Primary index and collection
    /// catalog are rebuilt in memory from authoritative segment bytes when
    /// needed. Suitable for `dingo doctor` (DX_SPEC §13.3). Does **not** take
    /// the exclusive writer lock, so it can run while a writer holds the store.
    pub fn open_inspect(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let root = path.as_ref();
        let paths = StorePaths::new(root);
        if !paths.looks_like_store() {
            return Err(StoreError::NotAStore(root.to_path_buf()));
        }

        let store_id = read_store_id(&paths)?;
        let meta = fs::read_to_string(paths.meta_file()).unwrap_or_default();
        if !meta.starts_with("dingo-store-") {
            return Err(StoreError::CorruptMeta("unexpected meta version"));
        }
        verify_store_descriptor_if_present(&paths, store_id)?;

        let mut store = Self {
            paths,
            store_id,
            limits: SafetyLimits::default(),
            index: PrimaryIndex::new(),
            durable_index: PrimaryIndex::new(),
            derived_ops_since_checkpoint: 0,
            active: None,
            segment_seq: 0,
            seal_threshold: DEFAULT_SEAL_THRESHOLD,
            chunk_threshold: DEFAULT_CHUNK_THRESHOLD,
            chunk_size: DEFAULT_CHUNK_SIZE,
            collection_catalog: CollectionCatalog::new(),
            tier_placement: TierPlacement::new(),
            segment_catalog: SegmentCatalog::new(),
            writer_lock: None,
            write_dedup: WriteDedupTable::new(),
        };
        store.load_tier_state_readonly()?;
        // Memory-only index: prefer frontier/v1 cache, else rebuild without writing.
        store.load_or_rebuild_index_readonly()?;
        let seg_paths = all_segment_paths(&store.paths, Some(&store.tier_placement))?;
        let fp = segment_fingerprint(&seg_paths)?;
        // Catalog: load if valid, else rebuild in memory only (no write).
        let cat_path = crate::catalog::collections_catalog_path(&store.paths.catalogs_dir());
        if let Some(cat) = try_load_collection_catalog(&cat_path, store.store_id, fp)? {
            store.collection_catalog = cat;
        } else {
            store.collection_catalog = CollectionCatalog::from_index(&store.index);
        }
        // Intentionally no resume_or_start_active — no writer handle.
        Ok(store)
    }

    /// Store root path.
    pub fn path(&self) -> &Path {
        &self.paths.root
    }

    /// Store identifier.
    pub fn store_id(&self) -> [u8; 16] {
        self.store_id
    }

    /// Number of live (non-deleted) subjects in the primary index.
    pub fn live_count(&self) -> usize {
        self.index.live_len()
    }

    /// Number of subjects with any recorded state (including delete tombstones).
    pub fn tracked_count(&self) -> usize {
        self.index.len()
    }

    /// Iterate live subjects and **stored** bodies (derived primary index).
    ///
    /// Chunked values yield the chunk **manifest**, not the logical payload.
    /// Prefer [`Self::live_logical_entries`] for application-level scans.
    pub fn live_entries(&self) -> impl Iterator<Item = (&[u8], &[u8])> + '_ {
        self.index
            .live_entries()
            .map(|(k, v)| (k.as_slice(), v.body.as_slice()))
    }

    /// Live subjects with logical payloads fully reassembled when chunked.
    ///
    /// **Fail-closed (DEF-012):** if any live subject has a partial, conflicting,
    /// or unavailable payload, returns [`StoreError::CoverageIncomplete`] rather
    /// than silently omitting those subjects. Use [`Self::scan_live_logical`] for
    /// an explicit partial-aware envelope, or [`Self::get_payload`] for one key.
    pub fn live_logical_entries(&self) -> Result<crate::compact::CheckpointPairs, StoreError> {
        let scan = self.scan_live_logical()?;
        if !scan.complete {
            let mut reasons = Vec::new();
            if !scan.incomplete.is_empty() {
                reasons.push(format!(
                    "{} live subject(s) have incomplete payloads",
                    scan.incomplete.len()
                ));
            }
            if scan.tier_coverage_incomplete {
                reasons.push("offline or unavailable storage tier(s)".into());
            }
            return Err(StoreError::CoverageIncomplete(format!(
                "{}; use scan_live_logical or get_payload for partial maps",
                reasons.join("; ")
            )));
        }
        Ok(scan.entries)
    }

    /// Scan live logical payloads with explicit incompleteness (DEF-012).
    ///
    /// Always returns every complete reassembly and lists incomplete subjects.
    /// `complete` is true only when every live subject produced a full body
    /// **and** tier coverage has no offline/unavailable segments.
    ///
    /// **Memory:** materializes the full live set. Prefer
    /// [`Self::scan_live_page`] for bounded-memory scans (DEF-026).
    pub fn scan_live_logical(&self) -> Result<LiveLogicalScan, StoreError> {
        let mut opts = crate::cursor::LiveScanPageOptions::new(crate::cursor::MAX_PAGE_SIZE);
        // Drain all pages without holding a giant intermediate only for subjects —
        // still assembles the full result (legacy API contract).
        let mut entries = Vec::new();
        let mut incomplete = Vec::new();
        let mut tier_coverage_incomplete = false;
        let mut cont: Option<Vec<u8>> = None;
        loop {
            opts.continuation = cont.take();
            let page = self.scan_live_page(&opts)?;
            entries.extend(page.entries);
            incomplete.extend(page.incomplete);
            tier_coverage_incomplete |= page.tier_coverage_incomplete;
            if !page.has_more {
                break;
            }
            cont = page.continuation;
            // Keep prefix if set (none for full scan).
            opts.page_size = crate::cursor::MAX_PAGE_SIZE;
        }
        let complete = incomplete.is_empty() && !tier_coverage_incomplete;
        Ok(LiveLogicalScan {
            entries,
            incomplete,
            complete,
            tier_coverage_incomplete,
        })
    }

    /// One bounded page of live logical payloads (DEF-026).
    ///
    /// Reads at most `options.page_size` complete bodies. Subject order is
    /// ascending. Pass the returned `continuation` token to resume; tokens are
    /// MAC-authenticated to this store and fenced by scan generation.
    ///
    /// Incomplete subjects encountered on the page are reported in
    /// `incomplete` and still advance the cursor so scans make forward progress.
    pub fn scan_live_page(
        &self,
        options: &crate::cursor::LiveScanPageOptions,
    ) -> Result<crate::cursor::LiveScanPage, StoreError> {
        use crate::cursor::{
            decode_token, encode_token, incomplete, scan_generation, CursorState, LiveScanPage,
            DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE,
        };

        let page_size = if options.page_size == 0 {
            DEFAULT_PAGE_SIZE
        } else {
            options.page_size.min(MAX_PAGE_SIZE)
        };

        let seg_fp = self.segment_fingerprint()?;
        let live_count = self.index.live_len() as u64;
        let generation = scan_generation(&self.store_id, &seg_fp, live_count);

        let (prefix, after, token_page_size) = if let Some(ref tok) = options.continuation {
            let state = decode_token(&self.store_id, tok)?;
            if state.generation != generation {
                return Err(StoreError::CursorStale(
                    "scan generation changed (live set or segment fingerprint); restart scan"
                        .into(),
                ));
            }
            // Prefix is fixed for the lifetime of the cursor.
            if let (Some(ref want), Some(ref got)) = (&options.prefix, &state.prefix) {
                if want != got {
                    return Err(StoreError::CursorInvalid(
                        "continuation prefix does not match request".into(),
                    ));
                }
            }
            let prefix = state.prefix.clone().or_else(|| options.prefix.clone());
            (prefix, state.after, state.page_size.clamp(1, MAX_PAGE_SIZE))
        } else {
            (options.prefix.clone(), None, page_size)
        };

        // Prefer token page size on resume so clients cannot silently widen.
        let page_size = if options.continuation.is_some() {
            token_page_size
        } else {
            page_size
        };

        let mut entries = Vec::new();
        let mut incomplete_list = Vec::new();
        let mut examined = 0usize;
        let mut last_subject: Option<Vec<u8>> = after.clone();
        let mut saw_more = false;

        // Bound work per page: page_size complete bodies, or a cap of examined
        // subjects when many are incomplete (forward progress without O(n) bodies).
        let max_examine = page_size.saturating_mul(8).max(page_size);
        let mut iter = self
            .index
            .live_entries_after(after.as_deref(), prefix.as_deref());

        loop {
            if entries.len() >= page_size || examined >= max_examine {
                saw_more = iter.next().is_some();
                break;
            }
            let Some((subject_ref, _)) = iter.next() else {
                break;
            };
            let subject = subject_ref.clone();
            examined += 1;
            last_subject = Some(subject.clone());
            let subject_str = match std::str::from_utf8(&subject) {
                Ok(s) => s,
                Err(_) => {
                    incomplete_list.push(incomplete(subject, IncompleteReason::NonUtf8Subject));
                    continue;
                }
            };
            match self.get_payload(subject_str)? {
                None => {}
                Some(PayloadResult::Complete { body }) => entries.push((subject, body)),
                Some(PayloadResult::Partial { .. }) => {
                    incomplete_list.push(incomplete(subject, IncompleteReason::PayloadPartial));
                }
                Some(PayloadResult::Unavailable { .. }) => {
                    incomplete_list
                        .push(incomplete(subject, IncompleteReason::PayloadUnavailable));
                }
                Some(PayloadResult::Conflicting { .. }) => {
                    incomplete_list.push(incomplete(subject, IncompleteReason::PayloadConflict));
                }
            }
        }

        let tier_coverage_incomplete = self.tier_coverage().is_incomplete();
        let continuation = if saw_more {
            let state = CursorState {
                generation,
                prefix: prefix.clone(),
                after: last_subject,
                page_size,
            };
            Some(encode_token(&self.store_id, &state)?)
        } else {
            None
        };

        let complete =
            !saw_more && incomplete_list.is_empty() && !tier_coverage_incomplete;
        Ok(LiveScanPage {
            entries,
            incomplete: incomplete_list,
            complete,
            tier_coverage_incomplete,
            has_more: saw_more,
            continuation,
            examined,
        })
    }

    /// Whether this handle holds exclusive writer ownership (DEF-020).
    pub fn holds_writer_lock(&self) -> bool {
        self.writer_lock.is_some()
    }

    /// Put opaque bytes under `subject` (OVERVIEW put event).
    ///
    /// Bodies larger than the chunk threshold are stored as chunked payloads
    /// (FORMAT_SPEC §8). The primary index retains the chunk manifest; get
    /// reassembles surviving chunks.
    pub fn put(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
    ) -> Result<WriteReceipt, StoreError> {
        // Memory mode is visibility-only: keep the full body in the index and
        // never append frames (avoids later durable flushes contaminating disk).
        if mode == DurabilityMode::Memory {
            return self.write_event(subject, EventKind::Put, value, mode);
        }
        if value.len() > self.chunk_threshold {
            self.write_chunked_put(subject, value, mode)
        } else {
            self.write_event(subject, EventKind::Put, value, mode)
        }
    }

    /// Resolve a client operation id for idempotent remote writes (DEF-010).
    ///
    /// - `Ok(Some(receipt))` — exact retry; return the original receipt
    /// - `Ok(None)` — new operation; caller should perform the write then
    ///   [`Self::record_write_dedup`]
    /// - `Err(ConsistencyViolation)` — id reused with different content
    pub fn resolve_write_dedup(
        &self,
        operation_id: &[u8; 16],
        content_hash: &[u8; 32],
    ) -> Result<Option<WriteReceipt>, StoreError> {
        match self.write_dedup.get(operation_id) {
            None => Ok(None),
            Some(rec) if &rec.content_hash == content_hash => Ok(Some(WriteReceipt {
                store_id: rec.store_id,
                segment_id: rec.segment_id,
                item_id: rec.item_id,
                event_id: rec.event_id,
                event_kind: rec.event_kind,
                durability: rec.durability,
                offset: rec.offset,
            })),
            Some(_) => Err(StoreError::ConsistencyViolation(
                "operation_id reused with different content identity".into(),
            )),
        }
    }

    /// Persist a successful mutation under `operation_id` (DEF-010).
    ///
    /// Called after the authoritative append so restart recovers the receipt.
    pub fn record_write_dedup(
        &mut self,
        operation_id: [u8; 16],
        content_hash: [u8; 32],
        receipt: &WriteReceipt,
    ) -> Result<(), StoreError> {
        self.write_dedup.insert(
            operation_id,
            DedupRecord {
                content_hash,
                store_id: receipt.store_id,
                segment_id: receipt.segment_id,
                item_id: receipt.item_id,
                event_id: receipt.event_id,
                event_kind: receipt.event_kind,
                durability: receipt.durability,
                offset: receipt.offset,
            },
        );
        save_write_dedup(&write_dedup_path(&self.paths), &self.write_dedup)
    }

    /// Get current live value for `subject`, if any.
    ///
    /// For chunked values this reassembles chunks and returns the complete body
    /// only when every required chunk is present. Use [`Self::get_payload`] for
    /// partial maps.
    pub fn get(&self, subject: &str) -> Result<Option<Vec<u8>>, StoreError> {
        match self.get_payload(subject)? {
            None => Ok(None),
            Some(PayloadResult::Complete { body }) => Ok(Some(body)),
            Some(PayloadResult::Partial { .. })
            | Some(PayloadResult::Unavailable { .. })
            | Some(PayloadResult::Conflicting { .. }) => {
                // Do not silently return incomplete data as a full get.
                Err(StoreError::PayloadPartial)
            }
        }
    }

    /// Get the current payload with explicit completeness (Stage 6 chunks).
    ///
    /// Returns `Ok(None)` when the subject has no live value. Inline (non-chunked)
    /// bodies always yield [`PayloadResult::Complete`].
    pub fn get_payload(&self, subject: &str) -> Result<Option<PayloadResult>, StoreError> {
        let key = subject.as_bytes();
        let Some(body) = self.index.get_live(key) else {
            return Ok(None);
        };
        if !is_chunk_manifest(body) {
            return Ok(Some(PayloadResult::Complete {
                body: body.to_vec(),
            }));
        }
        let Some(manifest) = decode_chunk_manifest(body) else {
            return Err(StoreError::CorruptMeta("invalid chunk manifest"));
        };
        let item_id = self
            .index
            .get(key)
            .map(|e| e.item_id())
            .unwrap_or([0u8; 16]);
        let pieces = self.collect_chunk_pieces(item_id)?;
        Ok(Some(reassemble_with_manifest(&manifest, &pieces)))
    }

    /// Record a logical delete for `subject`.
    pub fn delete(
        &mut self,
        subject: &str,
        mode: DurabilityMode,
    ) -> Result<WriteReceipt, StoreError> {
        self.write_event(subject, EventKind::Delete, &[], mode)
    }

    /// Event history for a subject key (oldest first; DX_SPEC §10.1).
    pub fn history(&self, subject: &str) -> Result<SubjectHistory, StoreError> {
        subject_history_tiered(
            &self.paths,
            self.limits,
            subject.as_bytes(),
            Some(&self.tier_placement),
        )
    }

    /// Rebuild the primary index by scanning all segment files (no catalog trust).
    ///
    /// Also refreshes the optional on-disk index cache and collection catalog.
    pub fn rebuild_index(&mut self) -> Result<(), StoreError> {
        self.rebuild_index_from_segments()?;
        // Best-effort cache refresh; failure to write cache must not fail rebuild.
        let _ = self.persist_index_cache();
        let _ = self.refresh_collection_catalog();
        Ok(())
    }

    /// Rebuild derived catalogs from the primary index / segments.
    pub fn rebuild_catalogs(&mut self) -> Result<(), StoreError> {
        self.refresh_collection_catalog()
    }

    /// Collection names known from the derived catalog (sorted).
    pub fn list_collections(&self) -> Vec<String> {
        self.collection_catalog
            .names()
            .map(|s| s.to_string())
            .collect()
    }

    /// Override the chunk size threshold (primarily for tests).
    pub fn set_chunk_threshold(&mut self, threshold: usize) {
        self.chunk_threshold = threshold;
    }

    /// Override per-chunk payload size (primarily for tests).
    pub fn set_chunk_size(&mut self, size: usize) {
        if size > 0 {
            self.chunk_size = size;
        }
    }

    /// Compact live state into a new sealed segment (sources retained).
    ///
    /// Runs the DEF-024 phase pipeline through **activate** and leaves sources
    /// on disk. Use [`Self::compact_live_with`] to reclaim after activate.
    pub fn compact_live(&mut self) -> Result<CompactReport, StoreError> {
        self.compact_live_with(CompactOptions::default())
    }

    /// Compact live state with explicit reclaim / horizon options (DEF-024).
    ///
    /// Phases: plan → create → verify → activate → optional reclaim.
    /// Reclaim of live-projection sources requires `allow_history_loss`.
    pub fn compact_live_with(
        &mut self,
        options: CompactOptions,
    ) -> Result<CompactReport, StoreError> {
        if options.reclaim_sources && !options.allow_history_loss {
            return Err(StoreError::ConsistencyViolation(
                "compact reclaim requires allow_history_loss for live-projection coverage".into(),
            ));
        }

        self.seal_active()?;
        let source_paths = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let sources: Vec<String> = source_paths
            .iter()
            .map(|p| examination_source_name(&self.paths.root, p))
            .collect();
        let source_ids = reclaimable_source_ids(&self.paths, &sources);
        let live_planned = self.index.live_entries().count();
        let (est_read, est_write) = estimate_compact_bytes(&self.paths, &sources, &self.index);
        let segment_id = self.next_segment_id();
        let job_id = random_id()?;
        let created_ns = now_ns();
        let recovery_generation = next_compact_recovery_generation(&self.paths)?;

        let mut job = new_planned_job(
            self.store_id,
            job_id,
            segment_id,
            sources.clone(),
            source_ids,
            live_planned,
            est_read,
            est_write,
            recovery_generation,
            &options,
            created_ns,
        );
        write_compact_job(&self.paths, &job)?;
        crate::failpoint::hit("store.compact.after_plan")?;

        if job.cancel_requested {
            job.phase = CompactPhase::Cancelled;
            job.detail = Some("cancelled before create".into());
            job.updated_ns = now_ns();
            write_compact_job(&self.paths, &job)?;
            return report_from_job(&job);
        }

        // --- create ---
        // Event ids are pure CSPRNG identities (DEF-025); ordering is writer_seq.
        let store_id = self.store_id;
        let mut mint = || random_id();
        let create_result = write_live_segment(
            &self.paths,
            store_id,
            self.limits,
            &self.index,
            segment_id,
            &mut mint,
            created_ns,
        );
        let (written, bytes_written) = match create_result {
            Ok(v) => v,
            Err(e) => {
                job.phase = CompactPhase::Failed;
                job.detail = Some(format!("create failed: {e}"));
                job.updated_ns = now_ns();
                let _ = write_compact_job(&self.paths, &job);
                return Err(e);
            }
        };
        job.phase = CompactPhase::Created;
        job.live_subjects_written = written;
        job.bytes_written = bytes_written;
        job.bytes_read = est_read;
        job.updated_ns = now_ns();
        write_compact_job(&self.paths, &job)?;
        crate::failpoint::hit("store.compact.after_create")?;

        // --- verify ---
        if let Err(e) = verify_live_segment(
            &self.paths,
            self.limits,
            &self.index,
            &segment_id,
            written,
        ) {
            job.phase = CompactPhase::Failed;
            job.detail = Some(format!("verify failed: {e}"));
            job.updated_ns = now_ns();
            let _ = write_compact_job(&self.paths, &job);
            return Err(e);
        }
        job.phase = CompactPhase::Verified;
        job.updated_ns = now_ns();
        write_compact_job(&self.paths, &job)?;

        // --- activate ---
        let _ = register_hot_segment(&self.paths, &mut self.tier_placement, segment_id);
        let _ = self.persist_tier_state();
        let _ = self.refresh_segment_catalog();
        let _ = self.persist_index_cache();
        crate::failpoint::hit("store.compact.after_activate")?;
        job.phase = CompactPhase::Activated;
        job.updated_ns = now_ns();
        write_compact_job(&self.paths, &job)?;

        // --- optional reclaim ---
        if options.reclaim_sources {
            job.phase = CompactPhase::RetentionHold;
            job.updated_ns = now_ns();
            write_compact_job(&self.paths, &job)?;
            self.reclaim_compact_job_inner(&mut job)?;
        }

        report_from_job(&job)
    }

    /// Explicitly reclaim sources for an activated compact job (DEF-024).
    ///
    /// Requires the job to have `allow_history_loss` (set at plan time or via
    /// this call's force flag when the job already recorded it).
    pub fn reclaim_compact_job(&mut self, job_id: &[u8; 16]) -> Result<CompactReport, StoreError> {
        let mut job = try_load_compact_job(&self.paths, job_id)?
            .ok_or(StoreError::CorruptMeta("compact job not found"))?;
        if !job.allow_history_loss {
            return Err(StoreError::ConsistencyViolation(
                "compact reclaim refused: job does not allow history loss".into(),
            ));
        }
        if matches!(job.phase, CompactPhase::Activated) {
            job.phase = CompactPhase::RetentionHold;
            job.updated_ns = now_ns();
            write_compact_job(&self.paths, &job)?;
        }
        self.reclaim_compact_job_inner(&mut job)?;
        report_from_job(&job)
    }

    /// Cancel an in-flight compact job that has not yet activated.
    ///
    /// Activated/reclaimed jobs cannot be cancelled (output is already live).
    pub fn cancel_compact_job(&mut self, job_id: &[u8; 16]) -> Result<CompactJob, StoreError> {
        let mut job = try_load_compact_job(&self.paths, job_id)?
            .ok_or(StoreError::CorruptMeta("compact job not found"))?;
        if matches!(
            job.phase,
            CompactPhase::Activated
                | CompactPhase::RetentionHold
                | CompactPhase::Reclaimed
                | CompactPhase::Cancelled
                | CompactPhase::Failed
        ) {
            return Err(StoreError::ConsistencyViolation(format!(
                "cannot cancel compact job in phase {}",
                job.phase.as_str()
            )));
        }
        job.cancel_requested = true;
        job.phase = CompactPhase::Cancelled;
        job.detail = Some("operator cancel".into());
        job.updated_ns = now_ns();
        // Best-effort: remove unactivated output segment so it does not linger.
        if let Some(out_id) = job.output_segment_bytes() {
            let p = self.paths.sealed_segment(&out_id);
            if p.is_file() && matches!(job.phase, CompactPhase::Cancelled) {
                // Only delete if we never activated (still true here).
                let _ = fs::remove_file(&p);
            }
        }
        write_compact_job(&self.paths, &job)?;
        Ok(job)
    }

    /// Load a compaction job record if present.
    pub fn load_compact_job(&self, job_id: &[u8; 16]) -> Result<Option<CompactJob>, StoreError> {
        try_load_compact_job(&self.paths, job_id)
    }

    /// List durable compaction job records.
    pub fn list_compact_jobs(&self) -> Result<Vec<CompactJob>, StoreError> {
        crate::compact::list_compact_jobs(&self.paths)
    }

    /// Resume incomplete compact jobs after open (DEF-024 recovery).
    ///
    /// - `planned`: cancel (no durable output yet, or incomplete create)
    /// - `created` / `verified`: finish verify+activate (sources retained)
    /// - `activated` / `retention_hold` / terminal: leave for operator
    pub fn recover_compact_jobs(&mut self) -> Result<Vec<CompactJob>, StoreError> {
        let jobs = crate::compact::list_compact_jobs(&self.paths)?;
        let mut out = Vec::new();
        for mut job in jobs {
            match job.phase {
                CompactPhase::Planned => {
                    job.phase = CompactPhase::Cancelled;
                    job.detail = Some("cancelled on recover: incomplete plan".into());
                    job.updated_ns = now_ns();
                    if let Some(id) = job.output_segment_bytes() {
                        let p = self.paths.sealed_segment(&id);
                        if p.is_file() {
                            // Incomplete create may have left a partial file;
                            // only remove if not registered as activated output.
                            let _ = fs::remove_file(&p);
                        }
                    }
                    write_compact_job(&self.paths, &job)?;
                }
                CompactPhase::Created | CompactPhase::Verified => {
                    if let Err(e) = self.finish_compact_job_after_create(&mut job) {
                        job.phase = CompactPhase::Failed;
                        job.detail = Some(format!("recover failed: {e}"));
                        job.updated_ns = now_ns();
                        let _ = write_compact_job(&self.paths, &job);
                    }
                }
                CompactPhase::Activated
                | CompactPhase::RetentionHold
                | CompactPhase::Reclaimed
                | CompactPhase::Cancelled
                | CompactPhase::Failed => {}
            }
            out.push(job);
        }
        Ok(out)
    }

    fn finish_compact_job_after_create(&mut self, job: &mut CompactJob) -> Result<(), StoreError> {
        let segment_id = job
            .output_segment_bytes()
            .ok_or(StoreError::CorruptMeta("compact output segment id"))?;
        let expected = job.live_subjects_written.max(job.live_subjects_planned);
        if job.phase == CompactPhase::Created {
            verify_live_segment(
                &self.paths,
                self.limits,
                &self.index,
                &segment_id,
                expected,
            )?;
            job.phase = CompactPhase::Verified;
            job.updated_ns = now_ns();
            write_compact_job(&self.paths, job)?;
        }
        let _ = register_hot_segment(&self.paths, &mut self.tier_placement, segment_id);
        let _ = self.persist_tier_state();
        let _ = self.refresh_segment_catalog();
        let _ = self.persist_index_cache();
        job.phase = CompactPhase::Activated;
        job.updated_ns = now_ns();
        write_compact_job(&self.paths, job)?;
        Ok(())
    }

    fn reclaim_compact_job_inner(&mut self, job: &mut CompactJob) -> Result<(), StoreError> {
        let (reclaimed, retained, deleted_ids) = reclaim_source_segments(&self.paths, job)?;
        for id in &deleted_ids {
            self.tier_placement.remove(id);
        }
        let _ = self.persist_tier_state();
        let _ = self.refresh_segment_catalog();
        // Live index still valid; rebuild so segment pointers prefer survivors.
        let _ = self.rebuild_index_from_segments();
        let _ = self.persist_index_cache();
        job.bytes_reclaimed = job.bytes_reclaimed.saturating_add(reclaimed);
        job.bytes_retained = retained;
        job.sources_retained = retained > 0
            || job
                .source_segment_ids
                .iter()
                .filter_map(|h| crate::layout::unhex16(h))
                .any(|id| self.paths.sealed_segment(&id).is_file());
        // After reclaim of all listed sources, sources_retained is false.
        if deleted_ids.len() == job.source_segment_ids.len()
            || job
                .source_segment_ids
                .iter()
                .filter_map(|h| crate::layout::unhex16(h))
                .all(|id| !self.paths.sealed_segment(&id).is_file())
        {
            job.sources_retained = false;
        }
        job.phase = CompactPhase::Reclaimed;
        job.updated_ns = now_ns();
        write_compact_job(&self.paths, job)?;
        Ok(())
    }

    /// Write a derived checkpoint under `snapshots/` with declared coverage.
    pub fn checkpoint(&self, coverage: &str) -> Result<(CheckpointMeta, PathBuf), StoreError> {
        let paths_list = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let fp = segment_fingerprint(&paths_list)?;
        let live: Vec<(Vec<u8>, Vec<u8>)> = self
            .index
            .live_entries()
            .map(|(k, v)| (k.clone(), v.body.clone()))
            .collect();
        let pairs: Vec<(&[u8], &[u8])> = live
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();
        let meta = CheckpointMeta {
            checkpoint_id: random_id()?,
            live_subjects: live.len(),
            segment_fingerprint: fp,
            coverage: coverage.to_string(),
            projection: "primary-live-v1".into(),
            created_ns: now_ns(),
        };
        let path = write_checkpoint(&self.paths, self.store_id, &meta, &pairs)?;
        Ok((meta, path))
    }

    /// Load a checkpoint file if it belongs to this store.
    pub fn load_checkpoint(
        &self,
        path: &Path,
    ) -> Result<Option<(CheckpointMeta, crate::compact::CheckpointPairs)>, StoreError> {
        try_load_checkpoint(path, self.store_id)
    }

    /// One page of live complete bodies for secondary index builds (DEF-027).
    ///
    /// Unlike [`Self::scan_live_page`], this walk is **not** generation-fenced:
    /// concurrent writes may extend the live set while the build runs; callers
    /// reconcile via snapshot fingerprint + catch-up before marking Ready.
    /// Incomplete payloads are skipped (listed separately) so builds make
    /// forward progress without blocking writes.
    pub fn scan_live_bodies_for_build(
        &self,
        prefix: Option<&[u8]>,
        after: Option<&[u8]>,
        page_size: usize,
    ) -> Result<IndexBuildPage, StoreError> {
        let page_size = page_size.clamp(1, crate::cursor::MAX_PAGE_SIZE);
        let max_examine = page_size.saturating_mul(8).max(page_size);
        let mut entries = Vec::new();
        let mut incomplete = Vec::new();
        let mut examined = 0usize;
        let mut last_subject: Option<Vec<u8>> = after.map(|a| a.to_vec());
        let mut has_more = false;
        let mut iter = self.index.live_entries_after(after, prefix);
        loop {
            if entries.len() >= page_size || examined >= max_examine {
                has_more = iter.next().is_some();
                break;
            }
            let Some((subject_ref, _)) = iter.next() else {
                break;
            };
            let subject = subject_ref.clone();
            examined += 1;
            last_subject = Some(subject.clone());
            let subject_str = match std::str::from_utf8(&subject) {
                Ok(s) => s,
                Err(_) => {
                    incomplete.push(subject);
                    continue;
                }
            };
            match self.get_payload(subject_str)? {
                None => {}
                Some(PayloadResult::Complete { body }) => entries.push((subject, body)),
                Some(PayloadResult::Partial { .. })
                | Some(PayloadResult::Unavailable { .. })
                | Some(PayloadResult::Conflicting { .. }) => incomplete.push(subject),
            }
        }
        Ok(IndexBuildPage {
            entries,
            incomplete,
            has_more,
            after: last_subject,
            examined,
        })
    }

    /// Persist a secondary index file (derived only).
    pub fn write_secondary_index(&self, index: &SecondaryIndex) -> Result<PathBuf, StoreError> {
        let path = secondary_index_path(&self.paths, &index.meta.collection, &index.meta.name);
        write_secondary_index(&path, self.store_id, index)?;
        Ok(path)
    }

    /// Load a secondary index by collection + name.
    pub fn load_secondary_index(
        &self,
        collection: &str,
        name: &str,
    ) -> Result<Option<SecondaryIndex>, StoreError> {
        let path = secondary_index_path(&self.paths, collection, name);
        try_load_secondary_index(&path, self.store_id)
    }

    /// List secondary indexes for a collection.
    pub fn list_secondary_indexes(
        &self,
        collection: &str,
    ) -> Result<Vec<SecondaryIndex>, StoreError> {
        let mut out = Vec::new();
        for path in list_secondary_index_paths(&self.paths, collection)? {
            if let Some(idx) = try_load_secondary_index(&path, self.store_id)? {
                out.push(idx);
            }
        }
        Ok(out)
    }

    /// Delete a secondary index file (never touches segments).
    pub fn delete_secondary_index(&self, collection: &str, name: &str) -> Result<(), StoreError> {
        let path = secondary_index_path(&self.paths, collection, name);
        delete_secondary_index(&path)
    }

    /// Current segment fingerprint (for index build coverage).
    pub fn segment_fingerprint(&self) -> Result<[u8; 32], StoreError> {
        let paths = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        segment_fingerprint(&paths)
    }

    /// Store layout paths (derived dirs safe to wipe for salvage tests).
    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    /// Load optional index cache via frontier (DEF-023) or v1 fingerprint; else rebuild.
    fn load_or_rebuild_index(&mut self) -> Result<(), StoreError> {
        if self.try_load_index_from_cache()? {
            return Ok(());
        }
        self.rebuild_index()
    }

    /// Read-only open path: load cache or rebuild without writing derived files.
    fn load_or_rebuild_index_readonly(&mut self) -> Result<(), StoreError> {
        if self.try_load_index_from_cache()? {
            return Ok(());
        }
        self.rebuild_index_from_segments()
    }

    /// Attempt frontier v2 or legacy v1 cache load. Returns true when applied.
    fn try_load_index_from_cache(&mut self) -> Result<bool, StoreError> {
        let sealed_paths = sealed_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let sealed_fp = segment_fingerprint(&sealed_paths)?;
        let all_paths = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let cache_path = primary_cache_path(&self.paths.indexes_dir());

        // DEF-023: v2 checkpoint + active-tail delta (O(changed bytes), not full rescan).
        if let Some((mut index, frontier)) =
            try_load_primary_index_frontier(&cache_path, self.store_id)?
        {
            if frontier.sealed_fingerprint == sealed_fp {
                let active_path = self.paths.active_segment();
                let active_ok = match (
                    active_path.is_file(),
                    frontier.active_segment_id != [0u8; 16],
                ) {
                    (false, false) => true,
                    (false, true) => {
                        // Cache expected an active segment that is gone — treat as miss
                        // only when covered_len was non-zero (empty active is fine).
                        frontier.active_covered_len == 0
                    }
                    (true, _) => {
                        let meta_len = fs::metadata(&active_path).map(|m| m.len()).unwrap_or(0);
                        if meta_len < frontier.active_covered_len {
                            false
                        } else {
                            apply_active_tail(
                                &mut index,
                                &active_path,
                                frontier.active_covered_len,
                                self.limits,
                            )?;
                            true
                        }
                    }
                };
                if active_ok {
                    self.install_loaded_index(index, &all_paths)?;
                    return Ok(true);
                }
            }
        }

        // Legacy v1: exact full fingerprint match (sealed + active lengths).
        let fp = segment_fingerprint(&all_paths)?;
        if let Some(index) = try_load_primary_index(&cache_path, self.store_id, fp)? {
            self.install_loaded_index(index, &all_paths)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn install_loaded_index(
        &mut self,
        index: PrimaryIndex,
        all_paths: &[PathBuf],
    ) -> Result<(), StoreError> {
        self.index = index.clone();
        self.durable_index = index;
        let sealed = list_dingo_files(&self.paths.segments_dir())?;
        self.segment_seq = max_segment_seq_from_paths(all_paths).max(sealed.len() as u64);
        self.derived_ops_since_checkpoint = 0;
        Ok(())
    }

    fn rebuild_index_from_segments(&mut self) -> Result<(), StoreError> {
        self.index = index_from_segments(&self.paths, self.limits, Some(&self.tier_placement))?;
        self.durable_index = self.index.clone();
        let sealed = list_dingo_files(&self.paths.segments_dir())?;
        let paths = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        self.segment_seq = max_segment_seq_from_paths(&paths).max(sealed.len() as u64);
        self.derived_ops_since_checkpoint = 0;
        Ok(())
    }

    /// Write the optional primary index cache under `indexes/` (Stage 3c / DEF-023).
    ///
    /// Checkpoint is built from the in-memory **durable** projection (no full
    /// segment rescan). Memory-mode publishes are never persisted. Safe to
    /// delete: open/rebuild recovers from segments (full scan) or from a prior
    /// frontier checkpoint plus the active tail.
    pub fn persist_index_cache(&mut self) -> Result<(), StoreError> {
        let frontier = self.current_index_frontier()?;
        write_primary_index_frontier(
            &primary_cache_path(&self.paths.indexes_dir()),
            self.store_id,
            &frontier,
            &self.durable_index,
        )?;
        self.derived_ops_since_checkpoint = 0;
        Ok(())
    }

    /// Sealed-set fingerprint + active covered length for the durable index.
    fn current_index_frontier(&self) -> Result<IndexFrontier, StoreError> {
        let sealed = sealed_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let sealed_fingerprint = segment_fingerprint(&sealed)?;
        let (active_segment_id, active_covered_len) = match &self.active {
            Some(w) => (w.segment_id, w.durable_len),
            None => {
                // No writer handle (inspect) or inactive: use on-disk active metadata.
                let path = self.paths.active_segment();
                if path.is_file() {
                    let len = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    // Segment id unknown without scanning; zeros means "any/unknown".
                    // Callers that only persist with an active writer always set id.
                    ([0u8; 16], len)
                } else {
                    ([0u8; 16], 0)
                }
            }
        };
        Ok(IndexFrontier {
            sealed_fingerprint,
            active_segment_id,
            active_covered_len,
        })
    }

    /// After a durable append: update derived state without scanning sealed data.
    ///
    /// Catalog refresh is always applied (small). Full index-cache checkpoints
    /// are rate-limited so amortized write work stays independent of retained
    /// segment volume (DEF-023).
    fn note_durable_derived(&mut self) -> Result<(), StoreError> {
        let _ = self.refresh_collection_catalog();
        self.derived_ops_since_checkpoint = self.derived_ops_since_checkpoint.saturating_add(1);
        if self.derived_ops_since_checkpoint >= DERIVED_CHECKPOINT_EVERY_OPS {
            let _ = self.persist_index_cache();
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // mirrors index::apply_event event fields
    fn apply_durable_event(
        &mut self,
        subject: Vec<u8>,
        kind: EventKind,
        body: Vec<u8>,
        item_id: [u8; 16],
        event_id: [u8; 16],
        segment_id: [u8; 16],
        writer_sequence: u64,
    ) {
        self.index.apply_event(
            subject.clone(),
            kind,
            body.clone(),
            item_id,
            event_id,
            segment_id,
            writer_sequence,
        );
        self.durable_index.apply_event(
            subject,
            kind,
            body,
            item_id,
            event_id,
            segment_id,
            writer_sequence,
        );
    }

    /// Path of the optional primary index cache file.
    pub fn index_cache_path(&self) -> PathBuf {
        primary_cache_path(&self.paths.indexes_dir())
    }

    /// Path of the framed store descriptor under `store-info/`.
    pub fn store_descriptor_path(&self) -> PathBuf {
        self.paths.store_descriptor_file()
    }

    /// Catalog-free salvage: scan every segment file and report counts.
    ///
    /// Does not mutate on-disk authoritative bytes. Live-subject projection
    /// uses the same recovery order and `event_id` dedup as index rebuild.
    pub fn salvage(&self) -> Result<SalvageReport, StoreError> {
        let mut files_scanned = 0usize;
        let mut verified_frames = 0u64;
        let mut item_events = 0u64;
        let mut holes = 0u64;

        for path in all_segment_paths(&self.paths, Some(&self.tier_placement))? {
            let bytes = fs::read(&path)?;
            files_scanned += 1;
            let report = scan_forward(&bytes, self.limits);
            verified_frames += report.verified_count() as u64;
            holes += report.holes().count() as u64;
            for (_offset, frame) in report.verified_frames() {
                if frame.header.known_kind() != Some(FrameKind::ItemEvent) {
                    continue;
                }
                if decode_item_envelope(&frame.envelope).is_some() {
                    item_events += 1;
                }
            }
        }

        let temp_index = index_from_segments(&self.paths, self.limits, Some(&self.tier_placement))?;
        Ok(SalvageReport {
            files_scanned,
            verified_frames,
            item_events,
            holes,
            live_subjects: temp_index.live_entries().count(),
        })
    }

    /// Create a full, content-hashed backup package (DEF-050).
    ///
    /// Distinct from [`Self::salvage_to`] (damage recovery) and
    /// [`Self::export_live_state`] (new lineage). Authoritative trees
    /// (`store-info`, `active`, `segments`, `chunks`, `recovery`, `tiers`) are
    /// copied into `package/store/` with blake3 per file; derived catalogs are
    /// omitted and rebuilt on restore.
    ///
    /// When this handle holds the exclusive writer lock, the active segment is
    /// flushed durable first ([`BackupConsistency::FlushedExclusive`]).
    /// Inspect-only opens copy on-disk files without a flush
    /// ([`BackupConsistency::OnDiskInspect`]).
    ///
    /// `package` must not already exist (or must be empty).
    pub fn backup_to(
        &mut self,
        package: impl AsRef<Path>,
    ) -> Result<crate::backup::BackupReport, StoreError> {
        let consistency = if self.writer_lock.is_some() {
            self.persist_active(DurabilityMode::Durable)?;
            crate::backup::BackupConsistency::FlushedExclusive
        } else {
            crate::backup::BackupConsistency::OnDiskInspect
        };
        crate::backup::write_full_backup(
            &self.paths.root,
            self.store_id,
            package.as_ref(),
            consistency,
        )
    }

    /// Run a bounded integrity scrub step (DEF-051).
    ///
    /// Verifies sealed segments (and optionally active/chunks) with full-file
    /// BLAKE3 and forward frame scan. Compares against placement `content_hash`
    /// when known. Findings are persisted under `recovery/scrub/`; corrupt
    /// evidence may be copied to quarantine without removing the original.
    ///
    /// Work is bounded by [`crate::ScrubOptions::max_files`] /
    /// [`crate::ScrubOptions::max_bytes`] so scrub never starves foreground
    /// callers that schedule multiple steps.
    pub fn scrub_once(
        &self,
        opts: crate::ScrubOptions,
    ) -> Result<crate::ScrubReport, StoreError> {
        crate::scrub::scrub_once(&self.paths, self.store_id, &self.tier_placement, &opts)
    }

    /// Scrub status: age, coverage, bytes verified, failures, pause flag.
    pub fn scrub_status(&self) -> Result<crate::ScrubStatus, StoreError> {
        crate::scrub::scrub_status(&self.paths, self.store_id)
    }

    /// Pause scrub so subsequent [`Self::scrub_once`] calls no-op until resume.
    pub fn pause_scrub(&self) -> Result<crate::ScrubStatus, StoreError> {
        crate::scrub::pause_scrub(&self.paths, self.store_id)
    }

    /// Resume a paused scrub.
    pub fn resume_scrub(&self) -> Result<crate::ScrubStatus, StoreError> {
        crate::scrub::resume_scrub(&self.paths, self.store_id)
    }

    /// Open scrub findings (hash mismatch, holes, missing media).
    pub fn list_scrub_findings(&self) -> Result<Vec<crate::ScrubFinding>, StoreError> {
        crate::scrub::list_scrub_findings(&self.paths, self.store_id)
    }

    /// Run scrub to completion under the given per-step bounds (loop).
    ///
    /// Stops early if paused or if a single step makes no progress while
    /// targets remain (safety).
    pub fn scrub_to_completion(
        &self,
        opts: crate::ScrubOptions,
    ) -> Result<crate::ScrubReport, StoreError> {
        let mut last = self.scrub_once(opts.clone())?;
        let mut guard = 0u32;
        while !last.cycle_completed && !last.paused && guard < 10_000 {
            guard += 1;
            let next = self.scrub_once(opts.clone())?;
            if next.targets_processed == 0 && !next.cycle_completed {
                break;
            }
            last = next;
        }
        Ok(last)
    }

    /// Format migration preflight (DEF-052): version matrix + segment classification.
    ///
    /// Does not write a durable job. Destination must be empty / absent.
    pub fn migrate_preflight(
        &self,
        dest: impl AsRef<Path>,
    ) -> Result<crate::MigratePreflight, StoreError> {
        crate::migrate::migrate_preflight(&self.paths.root, dest.as_ref(), self.store_id)
    }

    /// Phased format migration into a new store directory (DEF-052).
    ///
    /// Never rewrites the source in place. Copies authoritative trees with
    /// per-file blake3, preserves unsupported / unreadable segment bytes as
    /// opaque evidence, and only marks success after open+verify of the
    /// destination. Durable job under `recovery/migration/job.v1.json`.
    ///
    /// When this handle holds the exclusive writer lock, the active segment is
    /// flushed durable first so the migration boundary is crash-consistent.
    pub fn migrate_to(
        &mut self,
        dest: impl AsRef<Path>,
        opts: crate::MigrateOptions,
    ) -> Result<crate::MigrateReport, StoreError> {
        if self.writer_lock.is_some() {
            self.persist_active(DurabilityMode::Durable)?;
        }
        crate::migrate::migrate_store(&self.paths.root, dest.as_ref(), self.store_id, opts)
    }

    /// Load the durable migration job from this store's recovery directory.
    pub fn load_migration_job(&self) -> Result<Option<crate::MigrationJob>, StoreError> {
        crate::migrate::load_migration_job(&self.paths.root)
    }

    /// Evidence-preserving salvage into a **new** store directory (DX_SPEC §13.4, DEF-011).
    ///
    /// The source store is never mutated. Destination must not already be a
    /// store (same rules as [`Store::create`]). Verified frames are copied
    /// **byte-identical** into destination sealed segments; holes and scan
    /// parameters are recorded under `recovery/salvage-manifest.v1.json`.
    /// Event, item, and frame identities inside those frames are preserved.
    ///
    /// For a clean current-state database (re-put live values, new lineage),
    /// use [`Self::export_live_state`] instead.
    pub fn salvage_to(&self, dest: impl AsRef<Path>) -> Result<SalvageCopyReport, StoreError> {
        let dest = dest.as_ref();
        let source = self.salvage()?;

        // Skeleton destination: empty active segment + store identity. Recovered
        // frames go only into `segments/` so open does not re-encode them.
        let dest_store = Store::create(dest)?;
        let dest_store_id = dest_store.store_id;
        let dest_paths = dest_store.paths.clone();
        drop(dest_store);

        let mut source_files = Vec::new();
        for path in all_segment_paths(&self.paths, Some(&self.tier_placement))? {
            let rel = examination_source_name(&self.paths.root, &path);
            source_files.push((rel, path));
        }

        let (mut manifest, frames_copied, holes_recorded) = crate::recovery::copy_verified_frames(
            &self.paths.root,
            self.store_id,
            &dest_paths,
            dest_store_id,
            &source_files,
            self.limits,
        )?;

        // Rebuild derived state from the copied frames (does not rewrite them).
        let mut dest_open = Store::open(dest)?;
        let live_subjects = dest_open.index.live_entries().count();
        manifest.live_subjects = live_subjects;
        // Re-hash after filling live_subjects.
        manifest.content_hash_hex = {
            let mut for_hash = manifest.clone();
            for_hash.content_hash_hex.clear();
            let body = serde_json::to_vec(&for_hash).map_err(|e| {
                StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("serialize recovery manifest for hash: {e}"),
                ))
            })?;
            let h = blake3::hash(&body);
            let mut s = String::with_capacity(64);
            for b in h.as_bytes() {
                s.push_str(&format!("{b:02x}"));
            }
            s
        };
        let manifest_path = crate::recovery::write_recovery_manifest(&dest_paths, &manifest)?;
        let _ = dest_open.persist_index_cache();
        let _ = dest_open.rebuild_catalogs();
        drop(dest_open);

        Ok(SalvageCopyReport {
            source,
            destination: dest.to_path_buf(),
            mode: crate::recovery::SalvageMode::Evidence,
            subjects_copied: live_subjects,
            frames_copied,
            holes_recorded,
            manifest_path: Some(manifest_path),
        })
    }

    /// Materialize **live logical state** into a new store (DEF-011 export path).
    ///
    /// Unlike [`Self::salvage_to`], this re-appends complete live payloads as
    /// durable puts with **new** store/event lineage. History, tombstones,
    /// partials, and holes are **not** preserved. Prefer `salvage_to` when
    /// examination evidence must survive.
    pub fn export_live_state(&self, dest: impl AsRef<Path>) -> Result<SalvageCopyReport, StoreError> {
        let dest = dest.as_ref();
        let source = self.salvage()?;
        let live = self.live_logical_entries()?;
        let mut dest_store = Store::create(dest)?;
        let mut subjects_copied = 0usize;
        for (subject, body) in live {
            let subject_str = std::str::from_utf8(&subject).map_err(|_| {
                StoreError::BadEnvelope("non-utf8 subject cannot be materialised via put")
            })?;
            dest_store.put(subject_str, &body, DurabilityMode::Durable)?;
            subjects_copied += 1;
        }
        // Best-effort seal so destination is fully self-describing on disk.
        let _ = dest_store.seal_active();
        let _ = dest_store.persist_index_cache();
        let _ = dest_store.rebuild_catalogs();
        Ok(SalvageCopyReport {
            source,
            destination: dest.to_path_buf(),
            mode: crate::recovery::SalvageMode::LiveStateExport,
            subjects_copied,
            frames_copied: 0,
            holes_recorded: 0,
            manifest_path: None,
        })
    }

    /// Stable scan-report names and raw bytes for every authoritative segment
    /// object (sealed + active), ordered for deterministic examination.
    ///
    /// Source strings are relative to the store root (`segments/….dingo`,
    /// `active/active.dingo`). Does not mutate disk. Used by Stage 5
    /// (`dingo-examine`) to project [`dingo_format`] salvage regions into
    /// examination units without depending on catalogs or indexes.
    pub fn examination_sources(&self) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
        let mut out = Vec::new();
        for path in all_segment_paths(&self.paths, Some(&self.tier_placement))? {
            let source = examination_source_name(&self.paths.root, &path);
            let bytes = fs::read(&path)?;
            out.push((source, bytes));
        }
        Ok(out)
    }

    /// Safety limits applied to frame verification and salvage scans.
    pub fn safety_limits(&self) -> SafetyLimits {
        self.limits
    }

    /// Seal the active segment if present, moving it to `segments/`.
    pub fn seal_active(&mut self) -> Result<(), StoreError> {
        let Some(mut writer) = self.active.take() else {
            return Ok(());
        };
        // Ensure all buffered bytes are on the file before seal rewrite.
        self.flush_active_file(&mut writer, DurabilityMode::Durable)?;

        let sealed = writer.segment.seal()?;
        let bytes = sealed.as_bytes();
        let dest = self.paths.sealed_segment(&writer.segment_id);

        crate::failpoint::hit("store.seal.before_dest_write")?;

        // Write sealed image to destination, then remove active file.
        {
            let mut out = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&dest)?;
            out.write_all(bytes)?;
            out.sync_all()?;
        }
        crate::failpoint::hit("store.seal.after_dest_sync")?;
        sync_dir(&self.paths.segments_dir())?;

        // Truncate/remove active.
        let sealed_id = writer.segment_id;
        drop(writer.file);
        let active_path = self.paths.active_segment();
        if active_path.exists() {
            fs::remove_file(&active_path)?;
        }
        crate::failpoint::hit("store.seal.after_active_remove")?;
        sync_dir(&self.paths.active_dir())?;

        // Stage 9: register sealed segment on hot tier.
        let _ = register_hot_segment(&self.paths, &mut self.tier_placement, sealed_id);
        let _ = self.persist_tier_state();
        let _ = self.refresh_segment_catalog();

        self.segment_seq = self.segment_seq.saturating_add(1);
        self.start_active_segment()?;
        self.persist_active(DurabilityMode::Durable)?;
        let _ = self.persist_index_cache();
        Ok(())
    }

    /// Paths used for derived state (safe to delete for salvage tests).
    ///
    /// Tier **media** under `tiers/warm|cold|archive` is authoritative when
    /// segments live there; only `catalogs/` placement/summary files are derived.
    pub fn derived_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.paths.catalogs_dir(),
            self.paths.indexes_dir(),
            self.paths.snapshots_dir(),
        ]
    }

    // --- Stage 9: tiers / archive ---

    /// Current tier placement map (segment id → media).
    pub fn tier_placement(&self) -> &TierPlacement {
        &self.tier_placement
    }

    /// Hierarchical segment summary catalog (cold-search accelerator).
    pub fn segment_catalog(&self) -> &SegmentCatalog {
        &self.segment_catalog
    }

    /// Tier coverage for the current open state (offline media → incomplete).
    pub fn tier_coverage(&self) -> TierCoverage {
        self.tier_placement.coverage()
    }

    /// Mark a storage tier online or offline without deleting media.
    ///
    /// Offline tiers create coverage holes; they must not be reported as empty
    /// successful absence (OVERVIEW §9.2).
    pub fn set_tier_available(
        &mut self,
        tier: TierClass,
        available: bool,
    ) -> Result<(), StoreError> {
        self.tier_placement.set_tier_available(tier, available);
        self.persist_tier_state()?;
        // Rebuild index from remaining available segments only.
        self.rebuild_index_from_segments()?;
        let _ = self.persist_index_cache();
        let _ = self.refresh_collection_catalog();
        self.refresh_segment_catalog()?;
        Ok(())
    }

    /// Copy or move a sealed segment to another tier (stable segment identity).
    pub fn transfer_segment_to_tier(
        &mut self,
        segment_id: [u8; 16],
        to_tier: TierClass,
        mode: TierMoveMode,
    ) -> Result<MigrationEvidence, StoreError> {
        // Ensure placement knows about hot sealed segments.
        discover_placements(&self.paths, &mut self.tier_placement)?;
        let evidence = transfer_segment(
            &self.paths,
            &mut self.tier_placement,
            segment_id,
            to_tier,
            mode,
        )?;
        self.persist_tier_state()?;
        self.refresh_segment_catalog()?;
        // Fingerprint changed; refresh derived caches.
        let _ = self.persist_index_cache();
        Ok(evidence)
    }

    /// List sealed segment ids currently registered (any tier).
    pub fn list_segment_ids(&self) -> Vec<[u8; 16]> {
        self.tier_placement
            .entries()
            .map(|p| p.segment_id)
            .collect()
    }

    /// Segment summaries for cold search (hierarchical catalog).
    pub fn list_segment_summaries(&self) -> Vec<SegmentSummary> {
        self.segment_catalog.summaries().cloned().collect()
    }

    /// Rebuild hierarchical segment catalog from available media.
    ///
    /// After catalog loss, offline segments retained in placement keep last-known
    /// metadata when possible; available segments are re-scanned.
    pub fn rebuild_segment_catalog(&mut self) -> Result<(), StoreError> {
        discover_placements(&self.paths, &mut self.tier_placement)?;
        self.refresh_segment_catalog()?;
        self.persist_tier_state()?;
        Ok(())
    }

    /// Get with explicit tier coverage (absence only proven when coverage complete).
    pub fn get_with_tier_coverage(&self, subject: &str) -> Result<TierAwareGet, StoreError> {
        let coverage = self.tier_coverage();
        let value = self.get(subject)?;
        let absence_proven = value.is_none() && coverage.is_complete();
        Ok(TierAwareGet {
            value,
            coverage,
            absence_proven,
        })
    }

    /// Classify a sealed segment file without rewriting bytes (multi-gen readers).
    pub fn classify_segment(
        &self,
        segment_id: &[u8; 16],
    ) -> Result<FormatClassification, StoreError> {
        let path = if let Some(p) = self.tier_placement.get(segment_id) {
            crate::tier::resolve_placement_path(&self.paths, p)?
        } else {
            self.paths.sealed_segment(segment_id)
        };
        if !path.is_file() {
            return Err(StoreError::SegmentNotFound);
        }
        let bytes = fs::read(&path)?;
        Ok(classify_segment_bytes(&bytes))
    }

    /// Soft seal threshold override (tests / operators).
    pub fn set_seal_threshold(&mut self, bytes: u64) {
        if bytes > 0 {
            self.seal_threshold = bytes;
        }
    }

    // --- internals ---

    fn load_tier_state(&mut self) -> Result<(), StoreError> {
        load_tier_roots_file(&self.paths, &mut self.tier_placement);
        let path = tier_placement_path(&self.paths.catalogs_dir());
        if let Some(p) = try_load_placement(&path, self.store_id)? {
            // Preserve offline flags from roots after load.
            let roots_avail: Vec<_> = [
                TierClass::Hot,
                TierClass::Warm,
                TierClass::Cold,
                TierClass::Archive,
            ]
            .into_iter()
            .map(|t| (t, self.tier_placement.is_tier_available(t)))
            .collect();
            self.tier_placement = p;
            for (t, a) in roots_avail {
                // roots.txt is operator source of truth for online/offline.
                if !a {
                    self.tier_placement.set_tier_available(t, false);
                }
            }
            load_tier_roots_file(&self.paths, &mut self.tier_placement);
        }
        discover_placements(&self.paths, &mut self.tier_placement)?;
        let prior = try_load_segment_catalog(
            &segment_catalog_path(&self.paths.catalogs_dir()),
            self.store_id,
        )?;
        self.segment_catalog = rebuild_segment_catalog(
            &self.paths,
            &self.tier_placement,
            prior.as_ref(),
            self.limits,
        )?;
        let _ = write_segment_catalog(
            &segment_catalog_path(&self.paths.catalogs_dir()),
            self.store_id,
            &self.segment_catalog,
        );
        let _ = write_placement(&path, self.store_id, &self.tier_placement);
        let _ = write_tier_roots_file(&self.paths, &self.tier_placement);
        Ok(())
    }

    fn load_tier_state_readonly(&mut self) -> Result<(), StoreError> {
        load_tier_roots_file(&self.paths, &mut self.tier_placement);
        let path = tier_placement_path(&self.paths.catalogs_dir());
        if let Some(p) = try_load_placement(&path, self.store_id)? {
            self.tier_placement = p;
            load_tier_roots_file(&self.paths, &mut self.tier_placement);
        }
        discover_placements(&self.paths, &mut self.tier_placement)?;
        let prior = try_load_segment_catalog(
            &segment_catalog_path(&self.paths.catalogs_dir()),
            self.store_id,
        )?;
        self.segment_catalog = rebuild_segment_catalog(
            &self.paths,
            &self.tier_placement,
            prior.as_ref(),
            self.limits,
        )?;
        Ok(())
    }

    fn refresh_tier_state(&mut self) -> Result<(), StoreError> {
        discover_placements(&self.paths, &mut self.tier_placement)?;
        self.refresh_segment_catalog()?;
        self.persist_tier_state()?;
        Ok(())
    }

    fn persist_tier_state(&self) -> Result<(), StoreError> {
        let path = tier_placement_path(&self.paths.catalogs_dir());
        write_placement(&path, self.store_id, &self.tier_placement)?;
        write_tier_roots_file(&self.paths, &self.tier_placement)?;
        Ok(())
    }

    fn refresh_segment_catalog(&mut self) -> Result<(), StoreError> {
        let prior = self.segment_catalog.clone();
        self.segment_catalog =
            rebuild_segment_catalog(&self.paths, &self.tier_placement, Some(&prior), self.limits)?;
        let _ = write_segment_catalog(
            &segment_catalog_path(&self.paths.catalogs_dir()),
            self.store_id,
            &self.segment_catalog,
        );
        Ok(())
    }

    fn load_or_rebuild_catalog(&mut self) -> Result<(), StoreError> {
        let paths = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let fp = segment_fingerprint(&paths)?;
        let cat_path = crate::catalog::collections_catalog_path(&self.paths.catalogs_dir());
        if let Some(cat) = try_load_collection_catalog(&cat_path, self.store_id, fp)? {
            self.collection_catalog = cat;
            return Ok(());
        }
        self.refresh_collection_catalog()
    }

    fn refresh_collection_catalog(&mut self) -> Result<(), StoreError> {
        let paths = all_segment_paths(&self.paths, Some(&self.tier_placement))?;
        let fp = segment_fingerprint(&paths)?;
        // DEF-013 / DEF-023: persist only the durable projection (no segment rescan).
        // Memory-mode publishes live in `self.index` but must not contaminate
        // on-disk catalogs, index caches, or checkpoints.
        let durable_cat = CollectionCatalog::from_index(&self.durable_index);
        let cat_path = collections_catalog_path(&self.paths.catalogs_dir());
        write_collection_catalog(&cat_path, self.store_id, fp, &durable_cat)?;
        // In-process list_collections reflects visibility (includes memory).
        self.collection_catalog = CollectionCatalog::from_index(&self.index);
        Ok(())
    }

    fn write_chunked_put(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
    ) -> Result<WriteReceipt, StoreError> {
        let subject_bytes = subject.as_bytes();
        if subject_bytes.len() > MAX_SUBJECT_LEN {
            return Err(StoreError::SubjectTooLong {
                max: MAX_SUBJECT_LEN,
            });
        }
        if self.active.is_none() {
            self.start_active_segment()?;
        }
        if let Some(w) = &self.active {
            if w.segment.len() >= self.seal_threshold {
                self.seal_active()?;
            }
        }

        let segment_id = self
            .active
            .as_ref()
            .map(|w| w.segment_id)
            .expect("active segment");
        let item_id = match self.index.get(subject_bytes) {
            Some(entry) => entry.item_id(),
            None => subject_item_id(subject_bytes),
        };

        let pieces = split_into_pieces(item_id, value, self.chunk_size)?;
        // Pre-mint event ids so we do not hold &mut active across next_event_id.
        let mut chunk_event_ids: Vec<[u8; 16]> = Vec::with_capacity(pieces.len());
        for _ in 0..pieces.len() {
            chunk_event_ids.push(self.next_event_id()?);
        }
        let event_id = self.next_event_id()?;
        let created_ns = now_ns();

        let chunk_envelopes: Result<Vec<_>, _> = pieces
            .iter()
            .map(|_| {
                encode_item_envelope(&ItemEnvelope {
                    store_id: self.store_id,
                    segment_id,
                    item_id,
                    event_kind: EventKind::Put,
                    created_ns,
                    subject: subject_bytes.to_vec(),
                })
                .map_err(StoreError::BadEnvelope)
            })
            .collect();
        let chunk_envelopes = chunk_envelopes?;

        {
            let writer = self.active.as_mut().expect("active segment");
            for (piece, (chunk_event_id, envelope)) in pieces
                .iter()
                .zip(chunk_event_ids.iter().zip(chunk_envelopes.iter()))
            {
                let body = encode_piece_body(piece);
                let header = FrameHeader {
                    wire_major: dingo_format::WIRE_MAJOR,
                    wire_minor: dingo_format::WIRE_MINOR,
                    frame_kind: FrameKind::PayloadChunk.as_u8(),
                    flags: FrameFlags::new(FrameFlags::CHUNKED),
                    envelope_len: envelope.len() as u32,
                    body_len: body.len() as u64,
                    logical_len: piece.logical_len,
                    writer_sequence: 0,
                    event_id: *chunk_event_id,
                };
                writer.segment.append_parts(&FrameParts {
                    header,
                    envelope: envelope.clone(),
                    body,
                })?;
            }
        }

        let manifest = manifest_from_pieces(&pieces, &chunk_event_ids, value)?;
        let manifest_body = encode_chunk_manifest(&manifest);
        let item_envelope = encode_item_envelope(&ItemEnvelope {
            store_id: self.store_id,
            segment_id,
            item_id,
            event_kind: EventKind::Put,
            created_ns,
            subject: subject_bytes.to_vec(),
        })
        .map_err(StoreError::BadEnvelope)?;

        let offset = {
            let writer = self.active.as_mut().expect("active segment");
            let header = FrameHeader {
                wire_major: dingo_format::WIRE_MAJOR,
                wire_minor: dingo_format::WIRE_MINOR,
                frame_kind: FrameKind::ItemEvent.as_u8(),
                flags: FrameFlags::new(FrameFlags::CHUNKED),
                envelope_len: item_envelope.len() as u32,
                body_len: manifest_body.len() as u64,
                logical_len: value.len() as u64,
                writer_sequence: 0,
                event_id,
            };
            let offset = writer.segment.append_parts(&FrameParts {
                header,
                envelope: item_envelope,
                body: manifest_body.clone(),
            })?;
            match mode {
                DurabilityMode::Memory => {}
                DurabilityMode::Buffered | DurabilityMode::Durable => {
                    Self::write_segment_tail(writer, mode)?;
                }
            }
            offset
        };

        // Publish visibility only after authoritative append succeeded (DEF-023).
        self.apply_durable_event(
            subject_bytes.to_vec(),
            EventKind::Put,
            manifest_body,
            item_id,
            event_id,
            segment_id,
            0,
        );

        if mode != DurabilityMode::Memory {
            let _ = self.note_durable_derived();
        }

        Ok(WriteReceipt {
            store_id: self.store_id,
            segment_id,
            item_id,
            event_id,
            event_kind: EventKind::Put,
            durability: mode,
            offset,
        })
    }

    fn collect_chunk_pieces(
        &self,
        item_id: [u8; 16],
    ) -> Result<Vec<dingo_format::ChunkPiece>, StoreError> {
        let mut pieces = Vec::new();
        let mut seen_hashes: HashSet<([u8; 16], u32, [u8; 32])> = HashSet::new();
        for path in all_segment_paths(&self.paths, Some(&self.tier_placement))? {
            let bytes = fs::read(&path)?;
            let report = scan_forward(&bytes, self.limits);
            for (_offset, frame) in report.verified_frames() {
                if frame.header.known_kind() != Some(FrameKind::PayloadChunk) {
                    continue;
                }
                let Some(piece) = decode_piece_body(&frame.body) else {
                    continue;
                };
                if piece.item_id != item_id {
                    continue;
                }
                let h = *blake3::hash(&piece.body).as_bytes();
                if seen_hashes.insert((piece.item_id, piece.index, h)) {
                    pieces.push(piece);
                }
            }
        }
        Ok(pieces)
    }

    fn write_event(
        &mut self,
        subject: &str,
        kind: EventKind,
        body: &[u8],
        mode: DurabilityMode,
    ) -> Result<WriteReceipt, StoreError> {
        let subject_bytes = subject.as_bytes();
        if subject_bytes.len() > MAX_SUBJECT_LEN {
            return Err(StoreError::SubjectTooLong {
                max: MAX_SUBJECT_LEN,
            });
        }
        if !self.limits.accepts_lengths(0, body.len() as u64) {
            // envelope is non-zero; re-check after encode
        }
        if body.len() as u64 > self.limits.max_body_len {
            return Err(StoreError::PayloadTooLarge);
        }

        // DEF-013: memory mode is visibility-only — never append frames that a
        // later durable write would flush via write_segment_tail.
        if mode == DurabilityMode::Memory {
            if self.active.is_none() {
                self.start_active_segment()?;
            }
            let segment_id = self
                .active
                .as_ref()
                .map(|w| w.segment_id)
                .expect("active segment");
            let item_id = match self.index.get(subject_bytes) {
                Some(entry) => entry.item_id(),
                None => subject_item_id(subject_bytes),
            };
            let event_id = self.next_event_id()?;
            self.index.apply_event(
                subject_bytes.to_vec(),
                kind,
                body.to_vec(),
                item_id,
                event_id,
                segment_id,
                0,
            );
            // Visibility catalog only (not persisted).
            if let Some(name) = crate::catalog::collection_name_from_subject(subject_bytes) {
                self.collection_catalog.insert(name);
            }
            return Ok(WriteReceipt {
                store_id: self.store_id,
                segment_id,
                item_id,
                event_id,
                event_kind: kind,
                durability: DurabilityMode::Memory,
                offset: 0,
            });
        }

        if self.active.is_none() {
            self.start_active_segment()?;
        }

        // Maybe seal first if oversized.
        if let Some(w) = &self.active {
            if w.segment.len() >= self.seal_threshold {
                self.seal_active()?;
            }
        }

        let segment_id = self
            .active
            .as_ref()
            .map(|w| w.segment_id)
            .expect("active segment");

        let item_id = match self.index.get(subject_bytes) {
            Some(entry) => entry.item_id(),
            None => subject_item_id(subject_bytes),
        };
        let event_id = self.next_event_id()?;
        let created_ns = now_ns();

        let env = ItemEnvelope {
            store_id: self.store_id,
            segment_id,
            item_id,
            event_kind: kind,
            created_ns,
            subject: subject_bytes.to_vec(),
        };
        let envelope = encode_item_envelope(&env).map_err(StoreError::BadEnvelope)?;
        if !self
            .limits
            .accepts_lengths(envelope.len() as u32, body.len() as u64)
        {
            return Err(StoreError::PayloadTooLarge);
        }

        let writer = self.active.as_mut().expect("active segment");
        let offset = writer
            .segment
            .append(FrameKind::ItemEvent, &envelope, body, event_id)?;

        Self::write_segment_tail(writer, mode)?;

        // Publish visibility only after authoritative append succeeded (DEF-023).
        // Durable projection updated incrementally — no full-store segment rescan.
        self.apply_durable_event(
            subject_bytes.to_vec(),
            kind,
            body.to_vec(),
            item_id,
            event_id,
            segment_id,
            0, // writer_sequence already inside frame; not required for index
        );

        let _ = self.note_durable_derived();

        Ok(WriteReceipt {
            store_id: self.store_id,
            segment_id,
            item_id,
            event_id,
            event_kind: kind,
            durability: mode,
            offset,
        })
    }

    fn write_segment_tail(
        writer: &mut ActiveWriter,
        mode: DurabilityMode,
    ) -> Result<(), StoreError> {
        crate::failpoint::hit("store.active.write_tail.before")?;
        let bytes = writer.segment.as_bytes();
        let start = writer.durable_len as usize;
        if start > bytes.len() {
            return Err(StoreError::CorruptMeta("durable_len past segment"));
        }
        if start < bytes.len() {
            let pending = &bytes[start..];
            writer.file.seek(SeekFrom::Start(writer.durable_len))?;
            // DEF-022: optional short-write injection mid-append.
            if crate::failpoint::consume_short_write("store.active.write_tail.short_write") {
                let n = crate::failpoint::short_write_len(pending.len());
                if n > 0 {
                    writer.file.write_all(&pending[..n])?;
                    // Do not advance durable_len past the short write so a
                    // later retry could rewrite; crash/drop leaves torn bytes.
                    writer.durable_len += n as u64;
                }
                return Err(StoreError::Io(std::io::Error::new(
                    std::io::ErrorKind::WriteZero,
                    "failpoint short write: store.active.write_tail.short_write",
                )));
            }
            writer.file.write_all(pending)?;
            writer.durable_len = bytes.len() as u64;
        }
        crate::failpoint::hit("store.active.write_tail.after_write")?;
        if mode == DurabilityMode::Durable {
            writer.file.sync_all()?;
            crate::failpoint::hit("store.active.write_tail.after_sync")?;
        }
        Ok(())
    }

    fn flush_active_file(
        &self,
        writer: &mut ActiveWriter,
        mode: DurabilityMode,
    ) -> Result<(), StoreError> {
        Self::write_segment_tail(writer, mode)
    }

    fn persist_active(&mut self, mode: DurabilityMode) -> Result<(), StoreError> {
        if let Some(writer) = self.active.as_mut() {
            Self::write_segment_tail(writer, mode)?;
            if mode == DurabilityMode::Durable {
                crate::failpoint::hit("store.active.dir_sync")?;
                sync_dir(&self.paths.active_dir())?;
            }
        }
        Ok(())
    }

    fn start_active_segment(&mut self) -> Result<(), StoreError> {
        let segment_id = self.next_segment_id();
        let ids = SegmentId::new(self.store_id, segment_id);
        let segment = ActiveSegment::create(ids, self.limits, now_ns())?;
        let path = self.paths.active_segment();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(segment.as_bytes())?;
        let durable_len = segment.len();
        file.sync_all()?;
        self.active = Some(ActiveWriter {
            segment_id,
            segment,
            file,
            durable_len,
        });
        Ok(())
    }

    fn resume_or_start_active(&mut self) -> Result<(), StoreError> {
        let path = self.paths.active_segment();
        if !path.exists() {
            self.start_active_segment()?;
            self.persist_active(DurabilityMode::Durable)?;
            return Ok(());
        }

        let mut file = OpenOptions::new().read(true).write(true).open(&path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        // Truncate incomplete tail: keep only verified contiguous prefix from offset 0.
        let (kept, segment_id) = recover_active_bytes(&bytes, self.store_id, self.limits)?;
        if kept.len() != bytes.len() {
            file.set_len(kept.len() as u64)?;
            file.seek(SeekFrom::Start(kept.len() as u64))?;
            file.sync_all()?;
        }

        // Rebuild ActiveSegment by re-appending recovered item events.
        let rebuilt = rebuild_active_from_bytes(&kept, self.store_id, segment_id, self.limits)?;
        let durable_len = rebuilt.len();
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(rebuilt.as_bytes())?;
        file.sync_all()?;

        self.active = Some(ActiveWriter {
            segment_id,
            segment: rebuilt,
            file,
            durable_len,
        });
        Ok(())
    }

    fn next_segment_id(&mut self) -> [u8; 16] {
        self.segment_seq = self.segment_seq.saturating_add(1);
        // Sortable identity: monotonic seq recovered from disk on open (DEF-025).
        mint_sortable_segment_id(self.segment_seq, &self.store_id)
    }

    /// Pure CSPRNG event identity (not sortable; order uses writer_sequence).
    fn next_event_id(&mut self) -> Result<[u8; 16], StoreError> {
        random_id()
    }
}

/// One verified item event discovered on disk (shared with history module).
#[derive(Debug, Clone)]
pub(crate) struct DiskEventPub {
    pub(crate) file: PathBuf,
    pub(crate) offset: u64,
    pub(crate) writer_sequence: u64,
    pub(crate) subject: Vec<u8>,
    pub(crate) kind: EventKind,
    pub(crate) body: Vec<u8>,
    pub(crate) item_id: [u8; 16],
    pub(crate) event_id: [u8; 16],
    pub(crate) segment_id: [u8; 16],
}

/// Compare recovery order for item events (segment mint order, then sequence).
pub(crate) fn cmp_disk_events_pub(a: &DiskEventPub, b: &DiskEventPub) -> Ordering {
    segment_seq_key(&a.segment_id)
        .cmp(&segment_seq_key(&b.segment_id))
        .then(a.writer_sequence.cmp(&b.writer_sequence))
        .then(a.offset.cmp(&b.offset))
        .then(a.file.cmp(&b.file))
        .then(a.event_id.cmp(&b.event_id))
}

/// Collect verified item events from all segment files; also reports holes.
pub(crate) fn collect_item_events_for_history(
    paths: &StorePaths,
    limits: SafetyLimits,
    placement: Option<&TierPlacement>,
) -> Result<(Vec<DiskEventPub>, bool), StoreError> {
    collect_item_events_tiered(paths, limits, placement)
}

fn collect_item_events_tiered(
    paths: &StorePaths,
    limits: SafetyLimits,
    placement: Option<&TierPlacement>,
) -> Result<(Vec<DiskEventPub>, bool), StoreError> {
    let mut events = Vec::new();
    let mut has_holes = false;
    for path in all_segment_paths(paths, placement)? {
        let bytes = fs::read(&path)?;
        let report = scan_forward(&bytes, limits);
        if report.holes().next().is_some() {
            has_holes = true;
        }
        for (offset, frame) in report.verified_frames() {
            if frame.header.known_kind() != Some(FrameKind::ItemEvent) {
                continue;
            }
            let Some(env) = decode_item_envelope(&frame.envelope) else {
                continue;
            };
            events.push(DiskEventPub {
                file: path.clone(),
                offset,
                writer_sequence: frame.header.writer_sequence,
                subject: env.subject,
                kind: env.event_kind,
                body: frame.body.clone(),
                item_id: env.item_id,
                event_id: frame.header.event_id,
                segment_id: env.segment_id,
            });
        }
    }
    Ok((events, has_holes))
}

type DiskEvent = DiskEventPub;

/// Compare recovery order for item events (segment mint order, then sequence).
fn cmp_disk_events(a: &DiskEvent, b: &DiskEvent) -> Ordering {
    cmp_disk_events_pub(a, b)
}

/// Next recovery generation for a compact job (max existing + 1).
fn next_compact_recovery_generation(paths: &StorePaths) -> Result<u64, StoreError> {
    let jobs = crate::compact::list_compact_jobs(paths)?;
    let max = jobs
        .iter()
        .map(|j| j.recovery_generation)
        .max()
        .unwrap_or(0);
    Ok(max.saturating_add(1))
}

/// First 8 LE bytes of segment_id are the mint counter (see `next_segment_id`).
fn segment_seq_key(segment_id: &[u8; 16]) -> u64 {
    segment_seq_from_id(segment_id)
}

fn max_segment_seq_from_paths(paths: &[PathBuf]) -> u64 {
    let mut max = 0u64;
    for path in paths {
        if let Some(id) = crate::layout::segment_id_from_filename(path) {
            max = max.max(segment_seq_key(&id));
        }
    }
    max
}

fn write_store_descriptor_file(
    paths: &StorePaths,
    store_id: [u8; 16],
    created_ns: u64,
) -> Result<(), StoreError> {
    let frame = encode_store_descriptor_frame(store_id, created_ns)?;
    let path = paths.store_descriptor_file();
    crate::atomic_file::write_atomic(&path, &frame)?;
    Ok(())
}

fn verify_store_descriptor_if_present(
    paths: &StorePaths,
    store_id: [u8; 16],
) -> Result<(), StoreError> {
    let path = paths.store_descriptor_file();
    if !path.is_file() {
        return Ok(());
    }
    let bytes = fs::read(&path)?;
    let report = scan_forward(&bytes, SafetyLimits::default());
    let mut found = false;
    for (_off, frame) in report.verified_frames() {
        if frame.header.known_kind() != Some(FrameKind::StoreDescriptor) {
            continue;
        }
        let Some((id, _ns, _tag)) = decode_store_descriptor_body(&frame.body) else {
            return Err(StoreError::CorruptMeta("store descriptor body invalid"));
        };
        if id != store_id {
            return Err(StoreError::CorruptMeta(
                "store descriptor store_id mismatch",
            ));
        }
        found = true;
    }
    if !found {
        // File present but no verified store descriptor — tolerate for salvage;
        // identity still comes from store_id file.
        return Ok(());
    }
    Ok(())
}

fn sealed_segment_paths(
    paths: &StorePaths,
    placement: Option<&TierPlacement>,
) -> Result<Vec<PathBuf>, StoreError> {
    if let Some(p) = placement {
        crate::tier::available_sealed_paths(paths, p)
    } else {
        // Hot sealed only (legacy callers without placement).
        Ok(list_dingo_files(&paths.segments_dir())?)
    }
}

fn all_segment_paths(
    paths: &StorePaths,
    placement: Option<&TierPlacement>,
) -> Result<Vec<PathBuf>, StoreError> {
    let mut out = sealed_segment_paths(paths, placement)?;
    let active = paths.active_segment();
    if active.is_file() {
        out.push(active);
    }
    // Sealed first (sorted), then active last.
    Ok(out)
}

/// Apply item events from the active segment starting at byte offset `from_offset`.
///
/// Used with a frontier checkpoint so open cost is O(active tail), not O(all data).
fn apply_active_tail(
    index: &mut PrimaryIndex,
    active_path: &Path,
    from_offset: u64,
    limits: SafetyLimits,
) -> Result<(), StoreError> {
    let bytes = fs::read(active_path)?;
    if from_offset as usize > bytes.len() {
        return Err(StoreError::CorruptMeta("active frontier past file end"));
    }
    if from_offset as usize == bytes.len() {
        return Ok(());
    }
    let report = scan_forward(&bytes, limits);
    for (offset, frame) in report.verified_frames() {
        if offset < from_offset {
            continue;
        }
        if frame.header.known_kind() != Some(FrameKind::ItemEvent) {
            continue;
        }
        let Some(env) = decode_item_envelope(&frame.envelope) else {
            continue;
        };
        index.apply_event(
            env.subject,
            env.event_kind,
            frame.body.clone(),
            env.item_id,
            frame.header.event_id,
            env.segment_id,
            frame.header.writer_sequence,
        );
    }
    Ok(())
}

/// Relative scan-report name for a segment path under the store root.
fn examination_source_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown.dingo".into())
        })
}

fn collect_item_events(
    paths: &StorePaths,
    limits: SafetyLimits,
    placement: Option<&TierPlacement>,
) -> Result<Vec<DiskEvent>, StoreError> {
    let (events, _holes) = collect_item_events_tiered(paths, limits, placement)?;
    Ok(events)
}

/// Rebuild a primary index solely from segment bytes (ignores in-memory state).
///
/// Recovery order is content-based so renames / reordering of segment files
/// (OVERVIEW §16.10) do not scramble put/delete application: segment mint
/// order (LE u64 in `segment_id`) → `writer_sequence` → offset. Duplicate
/// segment copies are ignored via `event_id` dedup (first occurrence wins).
///
/// When `placement` is set, only **available** tier media are scanned; offline
/// segments are omitted and must be reported via [`TierCoverage`].
fn index_from_segments(
    paths: &StorePaths,
    limits: SafetyLimits,
    placement: Option<&TierPlacement>,
) -> Result<PrimaryIndex, StoreError> {
    let mut events = collect_item_events(paths, limits, placement)?;
    events.sort_by(cmp_disk_events);
    let mut index = PrimaryIndex::new();
    let mut seen_events: HashSet<[u8; 16]> = HashSet::new();
    for ev in events {
        if !seen_events.insert(ev.event_id) {
            continue;
        }
        index.apply_event(
            ev.subject,
            ev.kind,
            ev.body,
            ev.item_id,
            ev.event_id,
            ev.segment_id,
            ev.writer_sequence,
        );
    }
    Ok(index)
}

/// Keep longest prefix of complete verified frames from the start of the buffer.
/// Incomplete tail is dropped (OVERVIEW §6.2 / §7.3).
fn recover_active_bytes(
    bytes: &[u8],
    store_id: [u8; 16],
    limits: SafetyLimits,
) -> Result<(Vec<u8>, [u8; 16]), StoreError> {
    if bytes.is_empty() {
        return Ok((Vec::new(), random_id()?));
    }
    let report = scan_forward(bytes, limits);
    let mut end = 0u64;
    let mut segment_id = None;
    for region in &report.regions {
        match region {
            dingo_format::ScanRegion::VerifiedFrame { range, frame } => {
                // Only accept frames that form a contiguous prefix.
                if range.start != end {
                    break;
                }
                end = range.end;
                if frame.header.known_kind() == Some(FrameKind::SegmentDescriptor) {
                    if let Some((ids, _, _)) = dingo_format::decode_descriptor_body(&frame.body) {
                        if ids.store_id == store_id {
                            segment_id = Some(ids.segment_id);
                        }
                    }
                }
            }
            dingo_format::ScanRegion::Hole { .. } => {
                // Stop at first hole after contiguous verified prefix.
                break;
            }
        }
    }
    let kept = bytes[..end as usize].to_vec();
    let sid = match segment_id {
        Some(id) => id,
        None => random_id()?,
    };
    Ok((kept, sid))
}

/// Rebuild an ActiveSegment that matches recovered complete frames.
///
/// Strategy: create a new segment with the same ids and re-append item events
/// found in `kept` (descriptor is recreated; summary is not present in active).
fn rebuild_active_from_bytes(
    kept: &[u8],
    store_id: [u8; 16],
    segment_id: [u8; 16],
    limits: SafetyLimits,
) -> Result<ActiveSegment, StoreError> {
    let ids = SegmentId::new(store_id, segment_id);
    let mut created_ns = now_ns();
    if !kept.is_empty() {
        let report = scan_forward(kept, limits);
        for (_r, frame) in report.verified_frames() {
            if frame.header.known_kind() == Some(FrameKind::SegmentDescriptor) {
                if let Some((_, ns, _)) = dingo_format::decode_descriptor_body(&frame.body) {
                    created_ns = ns;
                }
            }
        }
    }

    let mut seg = ActiveSegment::create(ids, limits, created_ns)?;
    if kept.is_empty() {
        return Ok(seg);
    }

    let report = scan_forward(kept, limits);
    for (_offset, frame) in report.verified_frames() {
        // Re-append application content frames (items + payload chunks).
        // Preserve flags/kind via append_parts so chunked puts survive reopen.
        match frame.header.known_kind() {
            Some(FrameKind::ItemEvent) | Some(FrameKind::PayloadChunk) => {
                let mut header = frame.header.clone();
                // writer_sequence is reassigned by append_parts.
                header.writer_sequence = 0;
                seg.append_parts(&FrameParts {
                    header,
                    envelope: frame.envelope.clone(),
                    body: frame.body.clone(),
                })?;
            }
            _ => {}
        }
    }
    Ok(seg)
}

fn read_store_id(paths: &StorePaths) -> Result<[u8; 16], StoreError> {
    let raw = fs::read(paths.store_id_file())?;
    if raw.len() != 16 {
        return Err(StoreError::CorruptMeta("store_id must be 16 bytes"));
    }
    let mut id = [0u8; 16];
    id.copy_from_slice(&raw);
    Ok(id)
}

fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn sync_dir(path: &Path) -> Result<(), StoreError> {
    // Directory fsync is best-effort on platforms that support it.
    #[cfg(unix)]
    {
        let dir = File::open(path)?;
        dir.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_put_get_delete() {
        let dir = tempdir().unwrap();
        let mut store = Store::create(dir.path()).unwrap();
        let receipt = store
            .put("user-42", b"alice", DurabilityMode::Durable)
            .unwrap();
        assert_eq!(receipt.durability, DurabilityMode::Durable);
        assert_eq!(receipt.event_kind, EventKind::Put);
        assert_eq!(
            store.get("user-42").unwrap().as_deref(),
            Some(b"alice".as_slice())
        );
        store.delete("user-42", DurabilityMode::Durable).unwrap();
        assert!(store.get("user-42").unwrap().is_none());
    }

    #[test]
    fn reopen_recovers_state() {
        let dir = tempdir().unwrap();
        {
            let mut store = Store::create(dir.path()).unwrap();
            store.put("a", b"1", DurabilityMode::Durable).unwrap();
            store.put("b", b"2", DurabilityMode::Buffered).unwrap();
        }
        let store = Store::open(dir.path()).unwrap();
        assert_eq!(store.get("a").unwrap().as_deref(), Some(b"1".as_slice()));
        assert_eq!(store.get("b").unwrap().as_deref(), Some(b"2".as_slice()));
    }
}
