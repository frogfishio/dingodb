//! Filesystem-backed append store (OVERVIEW §§6–7, Stage 6).

use crate::catalog::{
    rebuild_collection_catalog, try_load_collection_catalog, CollectionCatalog,
};
use crate::chunk_payload::{
    decode_chunk_manifest, decode_piece_body, encode_chunk_manifest, encode_piece_body,
    is_chunk_manifest, manifest_from_pieces, reassemble_with_manifest, split_into_pieces,
    PayloadResult, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_THRESHOLD,
};
use crate::compact::{
    compact_live_to_new_segment, try_load_checkpoint, write_checkpoint, CheckpointMeta,
    CompactReport,
};
use crate::durability::DurabilityMode;
use crate::envelope::{
    decode_item_envelope, encode_item_envelope, EventKind, ItemEnvelope, MAX_SUBJECT_LEN,
};
use crate::error::StoreError;
use crate::history::{subject_history, SubjectHistory};
use crate::index::PrimaryIndex;
use crate::index_cache::{
    primary_cache_path, segment_fingerprint, try_load_primary_index, write_primary_index,
};
use crate::layout::{list_dingo_files, StorePaths};
use crate::secondary::{
    delete_secondary_index, list_secondary_index_paths, secondary_index_path,
    try_load_secondary_index, write_secondary_index, SecondaryIndex,
};
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
const META_VERSION: &str = "dingo-store-6\n";

/// Soft max size of the active segment before auto-seal (bytes).
const DEFAULT_SEAL_THRESHOLD: u64 = 4 * 1024 * 1024;

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

/// Open single-node store handle.
pub struct Store {
    paths: StorePaths,
    store_id: [u8; 16],
    limits: SafetyLimits,
    index: PrimaryIndex,
    /// Active in-memory segment + file, if any.
    active: Option<ActiveWriter>,
    /// Counter used to mint segment ids (monotonic diagnostic).
    segment_seq: u64,
    /// Seal active segment when it reaches this many bytes.
    seal_threshold: u64,
    /// Counter for event_id generation within process.
    event_counter: u64,
    /// Bodies larger than this are written as chunked payloads (Stage 6).
    chunk_threshold: usize,
    /// Max logical bytes per payload-chunk frame.
    chunk_size: usize,
    /// Derived collection catalog (rebuildable).
    collection_catalog: CollectionCatalog,
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
        let store_id = random_id();
        let created_ns = now_ns();
        fs::write(paths.store_id_file(), store_id)?;
        fs::write(paths.meta_file(), META_VERSION)?;
        write_store_descriptor_file(&paths, store_id, created_ns)?;
        // Ensure parent dir entry is durable for create.
        sync_dir(&paths.root)?;

        let mut store = Self {
            paths,
            store_id,
            limits: SafetyLimits::default(),
            index: PrimaryIndex::new(),
            active: None,
            segment_seq: 0,
            seal_threshold: DEFAULT_SEAL_THRESHOLD,
            event_counter: 0,
            chunk_threshold: DEFAULT_CHUNK_THRESHOLD,
            chunk_size: DEFAULT_CHUNK_SIZE,
            collection_catalog: CollectionCatalog::new(),
        };
        store.start_active_segment()?;
        store.persist_active(DurabilityMode::Durable)?;
        store.persist_index_cache()?;
        store.refresh_collection_catalog()?;
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
            active: None,
            segment_seq: 0,
            seal_threshold: DEFAULT_SEAL_THRESHOLD,
            event_counter: 0,
            chunk_threshold: DEFAULT_CHUNK_THRESHOLD,
            chunk_size: DEFAULT_CHUNK_SIZE,
            collection_catalog: CollectionCatalog::new(),
        };
        store.load_or_rebuild_index()?;
        store.load_or_rebuild_catalog()?;
        store.resume_or_start_active()?;
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

    /// Iterate live subjects and payload bodies (derived primary index).
    ///
    /// Used by higher layers (collection SDK) for scans. Catalog-free salvage
    /// remains available via [`Self::salvage`] / [`Self::rebuild_index`].
    pub fn live_entries(&self) -> impl Iterator<Item = (&[u8], &[u8])> + '_ {
        self.index
            .live_entries()
            .map(|(k, v)| (k.as_slice(), v.body.as_slice()))
    }

    /// Put opaque bytes under `subject` (OVERVIEW put event).
    pub fn put(
        &mut self,
        subject: &str,
        value: &[u8],
        mode: DurabilityMode,
    ) -> Result<WriteReceipt, StoreError> {
        self.write_event(subject, EventKind::Put, value, mode)
    }

    /// Get current live value for `subject`, if any.
    pub fn get(&self, subject: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let key = subject.as_bytes();
        Ok(self.index.get_live(key).map(|b| b.to_vec()))
    }

    /// Record a logical delete for `subject`.
    pub fn delete(
        &mut self,
        subject: &str,
        mode: DurabilityMode,
    ) -> Result<WriteReceipt, StoreError> {
        self.write_event(subject, EventKind::Delete, &[], mode)
    }

    /// Rebuild the primary index by scanning all segment files (no catalog trust).
    ///
    /// Also refreshes the optional on-disk index cache (Stage 3c).
    pub fn rebuild_index(&mut self) -> Result<(), StoreError> {
        self.rebuild_index_from_segments()?;
        // Best-effort cache refresh; failure to write cache must not fail rebuild.
        let _ = self.persist_index_cache();
        Ok(())
    }

    /// Load optional index cache when fingerprint matches; otherwise rebuild.
    fn load_or_rebuild_index(&mut self) -> Result<(), StoreError> {
        let paths = all_segment_paths(&self.paths)?;
        let fp = segment_fingerprint(&paths)?;
        let cache_path = primary_cache_path(&self.paths.indexes_dir());
        if let Some(index) = try_load_primary_index(&cache_path, self.store_id, fp)? {
            self.index = index;
            let sealed = list_dingo_files(&self.paths.segments_dir())?;
            self.segment_seq = max_segment_seq_from_paths(&paths).max(sealed.len() as u64);
            return Ok(());
        }
        self.rebuild_index()
    }

    fn rebuild_index_from_segments(&mut self) -> Result<(), StoreError> {
        self.index = index_from_segments(&self.paths, self.limits)?;
        let sealed = list_dingo_files(&self.paths.segments_dir())?;
        let paths = all_segment_paths(&self.paths)?;
        self.segment_seq = max_segment_seq_from_paths(&paths).max(sealed.len() as u64);
        Ok(())
    }

    /// Write the optional primary index cache under `indexes/` (Stage 3c).
    ///
    /// The cache is built from a segment scan (authoritative bytes only), so
    /// in-process memory-mode publishes are never persisted. Safe to delete:
    /// open/rebuild rescans segments.
    pub fn persist_index_cache(&self) -> Result<(), StoreError> {
        let paths = all_segment_paths(&self.paths)?;
        let fp = segment_fingerprint(&paths)?;
        let disk_index = index_from_segments(&self.paths, self.limits)?;
        write_primary_index(
            &primary_cache_path(&self.paths.indexes_dir()),
            self.store_id,
            fp,
            &disk_index,
        )
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

        for path in all_segment_paths(&self.paths)? {
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

        let temp_index = index_from_segments(&self.paths, self.limits)?;
        Ok(SalvageReport {
            files_scanned,
            verified_frames,
            item_events,
            holes,
            live_subjects: temp_index.live_entries().count(),
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
        for path in all_segment_paths(&self.paths)? {
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
        sync_dir(&self.paths.segments_dir())?;

        // Truncate/remove active.
        drop(writer.file);
        let active_path = self.paths.active_segment();
        if active_path.exists() {
            fs::remove_file(&active_path)?;
        }
        sync_dir(&self.paths.active_dir())?;

        self.segment_seq = self.segment_seq.saturating_add(1);
        self.start_active_segment()?;
        self.persist_active(DurabilityMode::Durable)?;
        let _ = self.persist_index_cache();
        Ok(())
    }

    /// Paths used for derived state (safe to delete for salvage tests).
    pub fn derived_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.paths.catalogs_dir(),
            self.paths.indexes_dir(),
            self.paths.snapshots_dir(),
        ]
    }

    // --- internals ---

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
        let event_id = self.next_event_id();
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

        // Persist according to durability (memory skips disk for the new frame).
        match mode {
            DurabilityMode::Memory => {
                // In-memory only: do not extend durable_len; file not updated.
            }
            DurabilityMode::Buffered | DurabilityMode::Durable => {
                Self::write_segment_tail(writer, mode)?;
            }
        }

        // Publish visibility in the index only after encode succeeded.
        self.index.apply_event(
            subject_bytes.to_vec(),
            kind,
            body.to_vec(),
            item_id,
            event_id,
            segment_id,
            0, // writer_sequence already inside frame; not required for index
        );

        // Refresh derived cache after durable/buffered acks (not memory-only).
        if mode != DurabilityMode::Memory {
            let _ = self.persist_index_cache();
        }

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
        let bytes = writer.segment.as_bytes();
        let start = writer.durable_len as usize;
        if start > bytes.len() {
            return Err(StoreError::CorruptMeta("durable_len past segment"));
        }
        if start < bytes.len() {
            writer.file.seek(SeekFrom::Start(writer.durable_len))?;
            writer.file.write_all(&bytes[start..])?;
            writer.durable_len = bytes.len() as u64;
        }
        if mode == DurabilityMode::Durable {
            writer.file.sync_all()?;
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
        let mut id = [0u8; 16];
        id[..8].copy_from_slice(&self.segment_seq.to_le_bytes());
        // Mix store id for uniqueness across stores.
        for i in 0..8 {
            id[8 + i] = self.store_id[i] ^ id[i];
        }
        id
    }

    fn next_event_id(&mut self) -> [u8; 16] {
        self.event_counter = self.event_counter.saturating_add(1);
        let mut id = random_id();
        id[..8].copy_from_slice(&self.event_counter.to_le_bytes());
        id
    }
}

/// One verified item event discovered on disk.
struct DiskEvent {
    file: PathBuf,
    offset: u64,
    writer_sequence: u64,
    subject: Vec<u8>,
    kind: EventKind,
    body: Vec<u8>,
    item_id: [u8; 16],
    event_id: [u8; 16],
    segment_id: [u8; 16],
}

/// Compare recovery order for item events (segment mint order, then sequence).
fn cmp_disk_events(a: &DiskEvent, b: &DiskEvent) -> Ordering {
    segment_seq_key(&a.segment_id)
        .cmp(&segment_seq_key(&b.segment_id))
        .then(a.writer_sequence.cmp(&b.writer_sequence))
        .then(a.offset.cmp(&b.offset))
        .then(a.file.cmp(&b.file))
        .then(a.event_id.cmp(&b.event_id))
}

/// First 8 LE bytes of segment_id are the mint counter (see `next_segment_id`).
fn segment_seq_key(segment_id: &[u8; 16]) -> u64 {
    u64::from_le_bytes(segment_id[..8].try_into().unwrap_or([0; 8]))
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
    fs::write(&path, frame)?;
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

fn all_segment_paths(paths: &StorePaths) -> Result<Vec<PathBuf>, StoreError> {
    let mut out = list_dingo_files(&paths.segments_dir())?;
    let active = paths.active_segment();
    if active.is_file() {
        out.push(active);
    }
    // Sealed first (sorted), then active last — list_dingo already sorted sealed.
    Ok(out)
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
) -> Result<Vec<DiskEvent>, StoreError> {
    let mut events = Vec::new();
    for path in all_segment_paths(paths)? {
        let bytes = fs::read(&path)?;
        let report = scan_forward(&bytes, limits);
        for (offset, frame) in report.verified_frames() {
            if frame.header.known_kind() != Some(FrameKind::ItemEvent) {
                continue;
            }
            let Some(env) = decode_item_envelope(&frame.envelope) else {
                continue;
            };
            events.push(DiskEvent {
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
    Ok(events)
}

/// Rebuild a primary index solely from segment bytes (ignores in-memory state).
///
/// Recovery order is content-based so renames / reordering of segment files
/// (OVERVIEW §16.10) do not scramble put/delete application: segment mint
/// order (LE u64 in `segment_id`) → `writer_sequence` → offset. Duplicate
/// segment copies are ignored via `event_id` dedup (first occurrence wins).
fn index_from_segments(
    paths: &StorePaths,
    limits: SafetyLimits,
) -> Result<PrimaryIndex, StoreError> {
    let mut events = collect_item_events(paths, limits)?;
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
        return Ok((Vec::new(), random_id()));
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
    let sid = segment_id.unwrap_or_else(random_id);
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
        // Stage 3a: only re-append item events into the new active buffer.
        if frame.header.known_kind() != Some(FrameKind::ItemEvent) {
            continue;
        }
        seg.append(
            FrameKind::ItemEvent,
            &frame.envelope,
            &frame.body,
            frame.header.event_id,
        )?;
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

fn subject_item_id(subject: &[u8]) -> [u8; 16] {
    let hash = blake3::hash(subject);
    let mut id = [0u8; 16];
    id.copy_from_slice(&hash.as_bytes()[..16]);
    id
}

fn random_id() -> [u8; 16] {
    // Prefer OS randomness when available; fall back to time-based mix.
    let mut id = [0u8; 16];
    if fill_os_random(&mut id) {
        return id;
    }
    let t = now_ns().to_le_bytes();
    id[..8].copy_from_slice(&t);
    let h = blake3::hash(&id);
    id.copy_from_slice(&h.as_bytes()[..16]);
    id
}

fn fill_os_random(buf: &mut [u8; 16]) -> bool {
    #[cfg(unix)]
    {
        use std::io::Read;
        if let Ok(mut f) = File::open("/dev/urandom") {
            return f.read_exact(buf).is_ok();
        }
    }
    let _ = buf;
    false
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
