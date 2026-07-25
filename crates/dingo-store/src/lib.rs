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

mod catalog;
mod chunk_payload;
mod compact;
mod durability;
mod envelope;
mod error;
mod history;
mod index;
mod index_cache;
mod layout;
mod secondary;
mod segment_catalog;
mod store;
mod tier;

pub use catalog::{
    collection_name_from_subject, collections_catalog_path, CollectionCatalog,
    COLLECTIONS_CATALOG_FILE,
};
pub use chunk_payload::{
    decode_chunk_manifest, encode_chunk_manifest, is_chunk_manifest, reassemble_with_manifest,
    ChunkManifest, ChunkSlot, PayloadResult, CHUNK_MANIFEST_MAGIC, DEFAULT_CHUNK_SIZE,
    DEFAULT_CHUNK_THRESHOLD,
};
/// Extent map types used by [`PayloadResult::Partial`] (FORMAT_SPEC §8).
pub use dingo_format::{ByteRange, LogicalExtent};
pub use compact::{CheckpointMeta, CompactReport};
pub use durability::DurabilityMode;
pub use envelope::{
    decode_item_envelope, encode_item_envelope, EventKind, ItemEnvelope, MAX_SUBJECT_LEN,
};
pub use error::StoreError;
pub use history::{HistoryEvent, SubjectHistory};
pub use index::{IndexEntry, LiveValue};
pub use index_cache::PRIMARY_CACHE_FILE;
pub use layout::{hex16, list_dingo_files, segment_id_from_filename, unhex16, StorePaths};
pub use secondary::{
    delete_secondary_index, list_secondary_index_paths, secondary_index_path,
    try_load_secondary_index, write_secondary_index, IndexState, SecondaryIndex,
    SecondaryIndexMeta,
};
pub use segment_catalog::{
    segment_catalog_path, SegmentCatalog, SegmentSummary, SEGMENT_CATALOG_FILE,
};
pub use store::{SalvageCopyReport, SalvageReport, Store, WriteReceipt};
pub use tier::{
    classify_segment_bytes, tier_placement_path, FormatClassification, MigrationEvidence,
    SegmentPlacement, TierAwareGet, TierClass, TierCoverage, TierMoveMode, TierPlacement,
    TIER_PLACEMENT_FILE,
};
