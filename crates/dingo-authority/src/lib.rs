//! DingoDB local-only heap authority (`HEAP_SPEC` HP-005 / §8.9 / §35).
//!
//! This crate is **AGPL** and must never be linked by the qualified data-service
//! binary. Signing and master-key material live only here.

#![deny(missing_docs)]

mod ceremony;
mod error;
mod head;
mod issue;
mod provider;
mod reload;
mod slot;
mod store;

pub use ceremony::{commit_genesis, GenesisRequest, GenesisResult};
pub use error::{AuthorityError, AuthorityStoreError};
pub use head::{AccessPolicy, AuthorityHead, RecoveryProfile};
pub use issue::{issue_heap_key, IssueRequest, IssuedHeapKey};
pub use provider::{EphemeralMasterKeyProvider, MasterKeyProvider};
pub use reload::{
    apply_reload_request, notify_reload, peek_reload_request, ReloadNotify, ReloadRequest,
};
pub use store::{AuthorityPaths, MasterAuthorityStore};
