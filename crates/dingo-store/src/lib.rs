//! DingoDB single-node authoritative store (Stages 3 + 6).
//!
//! Append-only segments on the filesystem, subject-keyed put/get/delete, and
//! catalog-independent recovery via [`dingo_format`] salvage scanning.
//!
//! Stage 6 adds rebuildable catalogs, secondary index files, subject history,
//! chunked payloads with partial maps, live-state compaction, and checkpoints.
//!
//! Normative: OVERVIEW §§5–7, §13; FORMAT_SPEC frames/segments/chunks.

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
mod store;

pub use catalog::{
    collection_name_from_subject, collections_catalog_path, CollectionCatalog,
    COLLECTIONS_CATALOG_FILE,
};
pub use chunk_payload::{
    decode_chunk_manifest, encode_chunk_manifest, is_chunk_manifest, reassemble_with_manifest,
    ChunkManifest, ChunkSlot, PayloadResult, CHUNK_MANIFEST_MAGIC, DEFAULT_CHUNK_SIZE,
    DEFAULT_CHUNK_THRESHOLD,
};
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
pub use store::{SalvageCopyReport, SalvageReport, Store, WriteReceipt};
