//! DingoDB embedded collection SDK (Stage 4).
//!
//! Ordinary application surface: open a store directory, name a collection,
//! put/get/delete JSON or bytes, and filter JSON documents without learning
//! SDA. Frames, segments, and salvage remain under [`dingo_store`] for
//! operators and lower stages.
//!
//! Normative: DX_SPEC §§1–7 (journeys 1–3, 6 partial); DELIVERY_PLAN Stage 4a–4d.

#![deny(missing_docs)]

mod collection;
mod dingo;
mod error;
mod filter;
mod receipt;
mod subject;
mod value;

pub use collection::Collection;
pub use dingo::Dingo;
pub use error::{Error, ErrorCode};
pub use filter::{FieldBuilder, Filter, Pred, QueryBuilder, QueryOptions, SortOrder};
pub use receipt::{DeleteReceipt, PutOptions, WriteReceipt};
pub use subject::{
    collection_prefix, decode_subject, encode_subject, validate_collection_name, validate_key,
    MAX_COLLECTION_NAME_LEN, MAX_KEY_LEN,
};

/// Re-export durability modes used on receipts and put options.
pub use dingo_store::DurabilityMode;

/// Build a `serde_json::Value` from a JSON literal (re-export for examples/tests).
pub use serde_json::json;

/// JSON value type used by [`Collection::get`] and filters.
pub type JsonValue = serde_json::Value;
