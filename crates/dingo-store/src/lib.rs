//! DingoDB single-node authoritative store (Stages 3 + 6 + 9).
//!
//! Append-only segments on the filesystem, subject-keyed put/get/delete, and
//! catalog-independent recovery via [`dingo_format`] salvage scanning.
//!
//! Stage 6 adds rebuildable catalogs, secondary index files, subject history,
//! chunked payloads with partial maps, live-state compaction, and checkpoints.
//!
//! Hydra adds adaptive **per-segment** read indexes at seal time (Eytzinger,
//! PGM/RadixSpline, compressed radix, MPHF+fingerprint) with multithreaded
//! rebuild — derived only under `indexes/seg/`.
//!
//! Chimera adds workload-compiled **value placement** (inline / point micro-pages
//! / scan extents / large-value log), adaptive I/O path selection, a background
//! compiler planner, and **seal/compaction layout sidecars** under
//! `indexes/chimera/` (seal/compact derived placement). Hot `Store::get` uses a
//! **locator-first PrimaryIndex** (DEF-095): map lookup then resident body or
//! bounded frame pread — not full-dataset body residency. Use
//! `Store::get_via_chimera` to probe layouts.
//!
//! Stage 9 adds storage tiers (hot/warm/cold/archive), segment move/copy with
//! stable identities, hierarchical segment catalogs, offline-tier coverage
//! honesty, and multi-generation format classification (byte preservation).
//!
//! DEF-052 adds phased format migration (preflight/plan/apply/verify/rollback)
//! with a declared wire and protocol compatibility matrix.
//!
//! Normative: OVERVIEW §§5–7, §9, §13; FORMAT_SPEC frames/segments/chunks.

#![deny(missing_docs)]

mod atomic_file;
mod backup;
mod catalog;
mod chimera;
mod chunk_payload;
mod compact;
mod crash_matrix;
mod cursor;
mod durability;
mod envelope;
mod erasure;
mod error;
mod failpoint;
mod heap;
mod history;
mod hydra;
mod ids;
mod index;
mod index_cache;
mod kernel;
mod layout;
mod lifecycle;
mod media;
mod migrate;
mod recovery;
mod scrub;
mod seal_pipeline;
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
pub use chimera::{
    build_layout, chimera_dir, chimera_layout_path, classify_value, decode_record,
    delete_chimera_layout, initial_locator_kind, pack_point_containers, place_value, plan_compile,
    plan_recluster_range, read_slot, resolve, select_io_path, try_load_chimera_layout,
    write_chimera_layout, ChimeraKindCounts, ChimeraLayout, ClassifyOptions, CompilerOp,
    CompilerOptions, CompilerPlan, ContainerBuilder, ContainerSlot, IoHints, IoPath,
    IoSelectOptions, LifetimeClass, LocatorKind, PlacementHints, PointContainer, RecordStats,
    ResolveContext, ResolvedValue, TemperatureClass, ValueClass, ValueLocator, ValueLog,
    ValueLogRecord, CODEC_RAW, CONTAINER_MAGIC, CONTAINER_VERSION, DEFAULT_CONTAINER_TARGET,
    DEFAULT_MEDIUM_MAX, DEFAULT_TINY_MAX, VALUE_LOG_HEADER_LEN, VALUE_LOG_MAGIC,
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
pub use crash_matrix::{
    all_cells, ci_subset_cells, load_embedded as load_crash_matrix,
    validate as validate_crash_matrix, CrashMatrix, ExpectedReopen, MatrixFailpoint,
    MatrixOperation, CRASH_MATRIX_JSON,
};
pub use cursor::{
    scan_generation, LiveScanPage, LiveScanPageOptions, CURSOR_PROFILE, DEFAULT_PAGE_SIZE,
    MAX_PAGE_SIZE, MAX_TOKEN_BYTES,
};
/// Extent map types used by [`PayloadResult::Partial`] (FORMAT_SPEC §8).
pub use dingo_format::{ByteRange, LogicalExtent};
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
/// Capability-gated heap façades (HP-003). Prefer these over the legacy raw store
/// for qualified heap isolation; the unscoped store remains available behind the
/// default `legacy-raw-store` feature.
pub use heap::{
    create_object, delete_rebuildable_catalogs, heap_binding_envelope, load_staged_genesis,
    publish_staged_genesis, rebuild_and_persist_all_catalogs, rebuild_heap_entry_from_chain,
    rebuild_object_entry_from_chain, rename_heap, rename_object, require_admit, retire_heap,
    retire_object, stage_heap_genesis, staging_is_non_discoverable, try_load_collections_catalog,
    try_load_heap_catalog, try_load_streams_catalog, AdminReceipt, HeapCatalogEntry, HeapMetaLayout,
    HeapStore, MaintenanceStore, ObjectCatalogEntry, ObjectKind, RecoveryStore, ReplicaStore,
    StagedGenesis, StoreHost, COLLECTIONS_CATALOG_FILE as HEAP_COLLECTIONS_CATALOG_FILE,
    HEAP_CATALOG_FILE, STREAMS_CATALOG_FILE as HEAP_STREAMS_CATALOG_FILE,
};
pub use history::{HistoryEvent, SubjectHistory};
pub use hydra::{
    build as build_hydra_index, build_many as build_hydra_indexes, classify_keys,
    delete_hydra_index, hydra_dir, hydra_index_path, records_from_segment_bytes, select_index_kind,
    try_load_hydra_index, write_hydra_index, HydraBuildOptions, HydraIndex, IndexKind, KeyShape,
    SegmentRecord, DEFAULT_TINY_THRESHOLD,
};
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
pub use migrate::{
    load_migration_job, migrate_apply, migrate_dir, migrate_job_path, migrate_plan,
    migrate_preflight, migrate_rollback, migrate_store, migrate_verify, snapshot_protocol_compat,
    snapshot_wire_matrix, MigrateFileAction, MigrateFilePlan, MigrateOptions, MigratePhase,
    MigratePreflight, MigrateReport, MigrationJob, ProtocolCompatSnapshot, WireMatrixRow,
    MIGRATE_DIR, MIGRATE_JOB_FILE, MIGRATE_PROFILE, PROTOCOL_MAJOR_DECLARED,
    PROTOCOL_MINOR_DECLARED, PROTOCOL_PROFILE_DECLARED, RPC_WIRE_LABEL_DECLARED,
};
pub use recovery::{
    salvage_manifest_path, try_load_recovery_manifest, FrameEvidence, HoleEvidence, LimitsSnapshot,
    RecoveryManifest, SalvageMode, SourceFileEvidence, SALVAGE_MANIFEST_FILE,
};
pub use scrub::{
    list_scrub_findings, load_or_init_scrub_state, load_scrub_findings, pause_scrub,
    plan_scrub_targets, resume_scrub, scrub_dir, scrub_findings_path, scrub_once, scrub_state_path,
    scrub_status, status_from_state, verify_scrub_target, write_scrub_findings, write_scrub_state,
    ScrubFinding, ScrubFindingKind, ScrubFindingsDoc, ScrubOptions, ScrubReport, ScrubState,
    ScrubStatus, ScrubTarget, ScrubTargetKind, ScrubTargetResult, DEFAULT_SCRUB_MAX_BYTES,
    DEFAULT_SCRUB_MAX_FILES, SCRUB_DIR, SCRUB_FINDINGS_FILE, SCRUB_PROFILE, SCRUB_QUARANTINE_DIR,
    SCRUB_STATE_FILE,
};
pub use seal_pipeline::{
    list_pending_paths, recover_all_pending, SealPipeline, DEFAULT_MAX_PENDING_SEALS,
};
pub use secondary::{
    delete_secondary_index, list_secondary_index_paths, secondary_index_path,
    try_load_secondary_index, write_secondary_index, IndexState, SecondaryIndex,
    SecondaryIndexMeta, INDEX_LIFECYCLE_PROFILE,
};
pub use segment_catalog::{
    segment_catalog_path, SegmentCatalog, SegmentSummary, SEGMENT_CATALOG_FILE,
};
pub use store::{
    subject_writer_shard, IncompleteReason, IndexBuildPage, LiveIncomplete, LiveLogicalScan,
    SalvageCopyReport, SalvageReport, WriteReceipt, MAX_WRITER_SHARDS,
};
/// Legacy unscoped store API. Prefer [`StoreHost`] / [`HeapStore`] on the
/// qualified heap path (`--no-default-features` hides this export).
#[cfg(feature = "legacy-raw-store")]
pub use store::Store;
pub use tier::{
    classify_segment_bytes, tier_placement_path, FormatClassification, MigrationEvidence,
    SegmentPlacement, TierAwareGet, TierClass, TierCoverage, TierMoveMode, TierPlacement,
    TIER_PLACEMENT_FILE,
};
pub use write_dedup::{
    content_identity, write_dedup_path, DedupRecord, WriteDedupTable, WRITE_DEDUP_FILE,
};
pub use writer_lock::{WriterLock, WRITER_LOCK_FILE};
