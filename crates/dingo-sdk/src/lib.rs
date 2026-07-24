//! DingoDB embedded collection SDK (Stages 4 + 6).
//!
//! Ordinary application surface: open a store directory, name a collection,
//! put/get/delete JSON or bytes, filter JSON documents, manage secondary
//! indexes, and inspect per-key history — without learning SDA.
//!
//! Normative: DX_SPEC §§1–10; DELIVERY_PLAN Stages 4 and 6.

#![deny(missing_docs)]

mod collection;
mod dingo;
mod error;
mod filter;
mod history;
mod indexes;
mod receipt;
mod subject;
mod value;

pub use collection::Collection;
pub use dingo::Dingo;
pub use error::{Error, ErrorCode};
pub use filter::{
    FieldBuilder, Filter, Pred, QueryBudget, QueryBuilder, QueryOptions, SortOrder,
};
pub use history::{KeyHistory, Version};
pub use indexes::{IndexInfo, Indexes};
pub use receipt::{DeleteReceipt, PutOptions, WriteReceipt};
pub use subject::{
    collection_prefix, decode_subject, encode_subject, validate_collection_name, validate_key,
    MAX_COLLECTION_NAME_LEN, MAX_KEY_LEN,
};

/// Re-export durability modes used on receipts and put options.
pub use dingo_store::DurabilityMode;
/// Re-export index lifecycle states (DX_SPEC §8.2).
pub use dingo_store::IndexState;
/// Re-export chunked payload completeness (FORMAT_SPEC §8).
pub use dingo_store::PayloadResult;

/// Build a `serde_json::Value` from a JSON literal (re-export for examples/tests).
pub use serde_json::json;

/// JSON value type used by [`Collection::get`] and filters.
pub type JsonValue = serde_json::Value;
