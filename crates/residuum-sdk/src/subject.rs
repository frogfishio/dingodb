//! Map (collection, key) ↔ store subject bytes.

use crate::error::Error;
use residuum_store::MAX_SUBJECT_LEN;

/// Draft subject layout version (length-prefixed collection + key).
const SUBJECT_VERSION: u8 = 0x01;

/// Maximum UTF-8 collection name length (bytes).
pub const MAX_COLLECTION_NAME_LEN: usize = 256;

/// Maximum UTF-8 application key length (bytes).
pub const MAX_KEY_LEN: usize = 2048;

/// Validate a collection name for the string SDK surface.
pub fn validate_collection_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(Error::InvalidCollectionName("empty"));
    }
    if name.len() > MAX_COLLECTION_NAME_LEN {
        return Err(Error::InvalidCollectionName("too long"));
    }
    if name.as_bytes().contains(&0) {
        return Err(Error::InvalidCollectionName("contains NUL"));
    }
    Ok(())
}

/// Validate an application key for the string SDK surface.
pub fn validate_key(key: &str) -> Result<(), Error> {
    if key.is_empty() {
        return Err(Error::InvalidKey("empty"));
    }
    if key.len() > MAX_KEY_LEN {
        return Err(Error::InvalidKey("too long"));
    }
    if key.as_bytes().contains(&0) {
        return Err(Error::InvalidKey("contains NUL"));
    }
    Ok(())
}

/// Encode `(collection, key)` into a store subject.
pub fn encode_subject(collection: &str, key: &str) -> Result<Vec<u8>, Error> {
    validate_collection_name(collection)?;
    validate_key(key)?;
    let coll = collection.as_bytes();
    let key_b = key.as_bytes();
    let coll_len =
        u16::try_from(coll.len()).map_err(|_| Error::InvalidCollectionName("too long"))?;
    // version(1) + coll_len(2) + coll + key
    let total = 1 + 2 + coll.len() + key_b.len();
    if total > MAX_SUBJECT_LEN {
        return Err(Error::InvalidKey("subject would exceed store limit"));
    }
    let mut out = Vec::with_capacity(total);
    out.push(SUBJECT_VERSION);
    out.extend_from_slice(&coll_len.to_le_bytes());
    out.extend_from_slice(coll);
    out.extend_from_slice(key_b);
    Ok(out)
}

/// Subject prefix that selects every key in `collection` (for scans).
pub fn collection_prefix(collection: &str) -> Result<Vec<u8>, Error> {
    validate_collection_name(collection)?;
    let coll = collection.as_bytes();
    let coll_len =
        u16::try_from(coll.len()).map_err(|_| Error::InvalidCollectionName("too long"))?;
    let mut out = Vec::with_capacity(1 + 2 + coll.len());
    out.push(SUBJECT_VERSION);
    out.extend_from_slice(&coll_len.to_le_bytes());
    out.extend_from_slice(coll);
    Ok(out)
}

/// Decode subject bytes produced by [`encode_subject`].
///
/// Returns `None` if the layout is not a Stage 4 collection subject.
pub fn decode_subject(subject: &[u8]) -> Option<(&str, &str)> {
    if subject.len() < 3 || subject[0] != SUBJECT_VERSION {
        return None;
    }
    let coll_len = u16::from_le_bytes([subject[1], subject[2]]) as usize;
    let coll_start: usize = 3;
    let coll_end = coll_start.checked_add(coll_len)?;
    if coll_end > subject.len() {
        return None;
    }
    let coll = std::str::from_utf8(&subject[coll_start..coll_end]).ok()?;
    let key = std::str::from_utf8(&subject[coll_end..]).ok()?;
    if coll.is_empty() || key.is_empty() {
        return None;
    }
    Some((coll, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_subject() {
        let s = encode_subject("users", "user-42").unwrap();
        let (c, k) = decode_subject(&s).unwrap();
        assert_eq!(c, "users");
        assert_eq!(k, "user-42");
        assert!(s.starts_with(&collection_prefix("users").unwrap()));
    }

    #[test]
    fn rejects_empty_and_nul() {
        assert!(validate_collection_name("").is_err());
        assert!(validate_key("").is_err());
        assert!(validate_collection_name("a\0b").is_err());
        assert!(validate_key("a\0b").is_err());
    }
}
