// SPDX-FileCopyrightText: 2026 Alexander R. Croft
// SPDX-License-Identifier: MIT

use mongodb::bson::{Bson, Document};
use uuid::Uuid;

use crate::error::{K2DbError, ServiceError};

const CROCKFORD32: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn generate_k2_id() -> String {
    let uuid = Uuid::now_v7();
    let bytes = uuid.as_bytes();

    let mut value = 0u128;
    for byte in bytes {
        value = (value << 8) | (*byte as u128);
    }
    value <<= 2;

    let mut encoded = String::with_capacity(26);
    for idx in (0..26).rev() {
        let shift = idx * 5;
        let alphabet_index = ((value >> shift) & 0x1f) as usize;
        encoded.push(CROCKFORD32[alphabet_index] as char);
    }

    format!(
        "{}-{}-{}-{}-{}",
        &encoded[0..8],
        &encoded[8..12],
        &encoded[12..16],
        &encoded[16..20],
        &encoded[20..26]
    )
}

pub fn is_k2_id(id: &str) -> bool {
    let trimmed = id.trim().to_ascii_uppercase();
    let bytes = trimmed.as_bytes();
    if bytes.len() != 30 {
      return false;
    }

    for (index, byte) in bytes.iter().enumerate() {
        let is_hyphen = matches!(index, 8 | 13 | 18 | 23);
        if is_hyphen {
            if *byte != b'-' {
                return false;
            }
            continue;
        }

        let allowed = matches!(
            *byte,
            b'0'..=b'9' | b'A'..=b'H' | b'J'..=b'K' | b'M'..=b'N' | b'P'..=b'T' | b'V'..=b'Z'
        );
        if !allowed {
            return false;
        }
    }

    true
}

pub fn normalize_id(id: &str) -> String {
    id.trim().to_owned()
}

pub fn strip_reserved_fields(document: &Document) -> Document {
    document
        .iter()
        .filter(|(key, _)| !key.starts_with('_'))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

pub fn without_mongo_id(document: Document) -> Document {
    let mut out = document;
    out.remove("_id");
    out
}

pub fn require_owner(owner: &str) -> Result<String, K2DbError> {
    let trimmed = owner.trim();
    if trimmed.is_empty() {
        return Err(K2DbError::new(
            ServiceError::BadRequest,
            "Invalid method usage, parameters not defined",
            Some("sys_mdb_crv1".to_owned()),
        ));
    }
    if trimmed == "*" {
        return Err(K2DbError::new(
            ServiceError::BadRequest,
            "Owner cannot be '*'",
            Some("sys_mdb_owner_star".to_owned()),
        ));
    }
    Ok(trimmed.to_owned())
}

pub fn secure_projection_allowed(field: &str, secure_prefixes: &[String]) -> bool {
    !path_has_secure_segment(field, secure_prefixes)
}

pub fn path_has_secure_segment(path: &str, secure_prefixes: &[String]) -> bool {
    path.trim()
        .split('.')
        .filter(|segment| !segment.is_empty())
        .any(|segment| secure_prefixes.iter().any(|prefix| segment.starts_with(prefix)))
}

pub fn strip_secure_fields_deep(value: Bson, secure_prefixes: &[String]) -> Bson {
    match value {
        Bson::Document(document) => {
            let mapped = document
                .into_iter()
                .filter(|(key, _)| !secure_prefixes.iter().any(|prefix| key.starts_with(prefix)))
                .map(|(key, value)| (key, strip_secure_fields_deep(value, secure_prefixes)))
                .collect();
            Bson::Document(mapped)
        }
        Bson::Array(values) => {
            Bson::Array(values.into_iter().map(|value| strip_secure_fields_deep(value, secure_prefixes)).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use mongodb::bson::doc;

    use super::{generate_k2_id, is_k2_id, normalize_id, path_has_secure_segment, strip_reserved_fields};

    #[test]
    fn generated_id_has_expected_shape() {
        let id = generate_k2_id();
        assert!(is_k2_id(&id));
    }

    #[test]
    fn normalize_id_trims_only() {
        assert_eq!(normalize_id("  AbCd  "), "AbCd");
    }

    #[test]
    fn strip_reserved_is_shallow() {
        let input = doc! {
            "name": "x",
            "_uuid": "bad",
            "nested": { "_internal": true, "ok": true }
        };
        let actual = strip_reserved_fields(&input);
        assert_eq!(actual, doc! { "name": "x", "nested": { "_internal": true, "ok": true } });
    }

    #[test]
    fn secure_path_detects_nested_segment() {
        assert!(path_has_secure_segment("a.#secret.value", &["#".to_owned()]));
        assert!(!path_has_secure_segment("a.secret.value", &["#".to_owned()]));
    }
}