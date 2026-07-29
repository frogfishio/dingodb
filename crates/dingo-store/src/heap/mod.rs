//! Capability-gated store façades (`HEAP_SPEC` §30.4–§30.5 / HP-003–HP-004).

mod catalog;
mod heap_store;
mod host;
mod maintenance_store;
mod one_heap;
mod recovery_store;
mod replica_store;

pub use catalog::{
    create_object, delete_rebuildable_catalogs, load_staged_genesis, publish_staged_genesis,
    rebuild_and_persist_all_catalogs, rebuild_heap_entry_from_chain, rebuild_object_entry_from_chain,
    rename_heap, rename_object, retire_heap, retire_object, stage_heap_genesis,
    staging_is_non_discoverable, try_load_collections_catalog, try_load_heap_catalog,
    try_load_streams_catalog, AdminReceipt, HeapCatalogEntry, HeapMetaLayout, ObjectCatalogEntry,
    ObjectKind, StagedGenesis, COLLECTIONS_CATALOG_FILE, HEAP_CATALOG_FILE, STREAMS_CATALOG_FILE,
};
pub use heap_store::HeapStore;
pub use host::StoreHost;
pub use maintenance_store::MaintenanceStore;
pub use one_heap::{heap_binding_envelope, require_admit};
pub use recovery_store::RecoveryStore;
pub use replica_store::ReplicaCapStore as ReplicaStore;
