//! DingoDB single-node authoritative store (Stages 3 + 6 + 9).
//!
//! Append-only segments on the filesystem, subject-keyed put/get/delete, and
//! catalog-independent recovery via [`dingo_format`] salvage scanning.
//!
//! Stage 6 adds rebuildable catalogs, secondary index files, subject history,
//! chunked payloads with partial maps, live-state compaction, and checkpoints.
//!
//! Stage 9 adds storage tiers (hot/warm/cold/archive), segment move/copy with
//! stable identities, hierarchical segment catalogs, offline-tier coverage
//! honesty, and multi-generation format classification (byte preservation).
//!
//! Normative: OVERVIEW §§5–7, §9, §13; FORMAT_SPEC frames/segments/chunks.

#![deny(missing_docs)]

mod atomic_file;
mod backup;
mod catalog;
mod chunk_payload;
mod compact;
mod crash_matrix;
mod cursor;
mod durability;
mod envelope;
mod erasure;
mod error;
mod failpoint;
mod history;
mod ids;
mod index;
mod index_cache;
mod layout;
mod lifecycle;
mod media;
mod recovery;
mod secondary;
mod segment_catalog;
mod store;
mod tier;
mod write_dedup;
mod writer_lock;

pub use atomic_file::{
    previous_path, read_with_previous, recover_previous_or_corrupt, sync_dir as sync_parent_dir,
    write_atomic, write_atomic_keep_previous, write_atomic_with, AtomicWriteOptions, PREV_SUFFIX,
};
pub use backup::{
    backup_manifest_path, backup_store_path, load_and_verify_manifest, restore_full_backup,
    verify_package_files, write_full_backup, BackupConsistency, BackupFileEntry, BackupManifest,
    BackupReport, RestoreOptions, RestoreReport, BACKUP_MANIFEST_FILE, BACKUP_PROFILE,
    BACKUP_STORE_DIR,
};
pub use catalog::{
    collection_name_from_subject, collections_catalog_path, try_load_collection_catalog,
    CollectionCatalog, COLLECTIONS_CATALOG_FILE,
};
pub use chunk_payload::{
    decode_chunk_manifest, encode_chunk_manifest, is_chunk_manifest, reassemble_with_manifest,
    ChunkManifest, ChunkSlot, PayloadResult, CHUNK_MANIFEST_MAGIC, DEFAULT_CHUNK_SIZE,
    DEFAULT_CHUNK_THRESHOLD,
};
pub use compact::{
    compaction_job_path, compaction_jobs_dir, list_compact_jobs, try_load_compact_job,
    CheckpointMeta, CompactJob, CompactOptions, CompactPhase, CompactReport, COMPACTION_JOB_DIR,
    COMPACTION_JOB_SUFFIX,
};
pub use cursor::{
    scan_generation, LiveScanPage, LiveScanPageOptions, CURSOR_PROFILE, DEFAULT_PAGE_SIZE,
    MAX_PAGE_SIZE, MAX_TOKEN_BYTES,
};
/// Extent map types used by [`PayloadResult::Partial`] (FORMAT_SPEC §8).
pub use dingo_format::{ByteRange, LogicalExtent};
pub use crash_matrix::{
    all_cells, ci_subset_cells, load_embedded as load_crash_matrix, validate as validate_crash_matrix,
    CrashMatrix, ExpectedReopen, MatrixFailpoint, MatrixOperation, CRASH_MATRIX_JSON,
};
pub use durability::DurabilityMode;
pub use envelope::{
    decode_item_envelope, encode_item_envelope, EventKind, ItemEnvelope, MAX_SUBJECT_LEN,
};
pub use erasure::{
    decode_shards, encode_shards, is_shard_key, shard_layout_note, ErasureManifest, ErasureProfile,
    DEFAULT_DATA_SHARDS, DEFAULT_PARITY_SHARDS,
};
pub use error::StoreError;
pub use failpoint::{
    any_armed as failpoints_armed, arm as arm_failpoint, arm_n as arm_failpoint_n,
    arm_once as arm_failpoint_once, clear as clear_failpoints,
    consume_short_write as consume_failpoint_short_write, disarm as disarm_failpoint,
    hit as hit_failpoint, short_write_len as failpoint_short_write_len, Action as FailpointAction,
};
pub use history::{HistoryEvent, SubjectHistory};
pub use ids::{
    fill_random, hex16 as id_hex16, mint_sortable_segment_id, random_id, segment_seq_from_id,
    subject_item_id, ID_LEN, ID_PROFILE,
};
pub use index::{IndexEntry, LiveValue};
pub use index_cache::{IndexFrontier, PRIMARY_CACHE_FILE};
pub use layout::{hex16, list_dingo_files, segment_id_from_filename, unhex16, StorePaths};
pub use lifecycle::{policy_path, LifecyclePolicy, LifecycleRule, LIFECYCLE_POLICY_FILE};
pub use media::{
    media_root_directory, media_root_directory_with, open_media, open_media_with,
    CloudMirrorConfig, FilesystemMedia, LocalObjectMedia, MediaBackend, MediaLocator,
    MirroredCloudMedia, ObjectMediaUri, ObjectScheme, UnsupportedCloudMedia,
};
pub use secondary::{
    delete_secondary_index, list_secondary_index_paths, secondary_index_path,
    try_load_secondary_index, write_secondary_index, IndexState, SecondaryIndex,
    SecondaryIndexMeta, INDEX_LIFECYCLE_PROFILE,
};
pub use segment_catalog::{
    segment_catalog_path, SegmentCatalog, SegmentSummary, SEGMENT_CATALOG_FILE,
};
pub use recovery::{
    salvage_manifest_path, try_load_recovery_manifest, FrameEvidence, HoleEvidence,
    LimitsSnapshot, RecoveryManifest, SalvageMode, SourceFileEvidence, SALVAGE_MANIFEST_FILE,
};
pub use store::{
    IncompleteReason, IndexBuildPage, LiveIncomplete, LiveLogicalScan, SalvageCopyReport,
    SalvageReport, Store, WriteReceipt,
};
pub use tier::{
    classify_segment_bytes, tier_placement_path, FormatClassification, MigrationEvidence,
    SegmentPlacement, TierAwareGet, TierClass, TierCoverage, TierMoveMode, TierPlacement,
    TIER_PLACEMENT_FILE,
};
pub use write_dedup::{
    content_identity, write_dedup_path, DedupRecord, WriteDedupTable, WRITE_DEDUP_FILE,
};
pub use writer_lock::{WriterLock, WRITER_LOCK_FILE};
