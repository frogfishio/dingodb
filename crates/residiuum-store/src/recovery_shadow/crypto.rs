//! Optional Shadow payload envelope (CSE-3 security variants).
//!
//! Dual-run default wire stores cleartext values (Materialized still authoritative).
//! When sealed with [`envelope_seal`], the on-disk put payload must not contain
//! the plaintext substring. Key rotation: reseal under a new key and rewrite
//! Shadow; recovery opens with the active key.
//!
//! Construction uses BLAKE3 keyed keystream (no extra crypto deps). Not a
//! production AEAD claim — CSE proves “no plaintext / rotation / erase” hooks.

use blake3::Hasher;

/// Envelope magic prefix for sealed put payloads.
pub const ENVELOPE_MAGIC: &[u8; 8] = b"RSHENV01";

/// Seal plaintext under `key` → envelope bytes (no plaintext substring).
pub fn envelope_seal(key: &[u8; 32], key_id: u64, plaintext: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 8 + 4 + plaintext.len() + 32);
    out.extend_from_slice(ENVELOPE_MAGIC);
    out.extend_from_slice(&key_id.to_le_bytes());
    out.extend_from_slice(&(plaintext.len() as u32).to_le_bytes());
    let mut cipher = vec![0u8; plaintext.len()];
    keystream_xor(key, key_id, plaintext, &mut cipher);
    out.extend_from_slice(&cipher);
    let mut h = Hasher::new_keyed(key);
    h.update(ENVELOPE_MAGIC);
    h.update(&key_id.to_le_bytes());
    h.update(&(plaintext.len() as u32).to_le_bytes());
    h.update(&cipher);
    out.extend_from_slice(h.finalize().as_bytes());
    out
}

/// Open envelope under `key`. Fail-closed on tag / length / magic mismatch.
pub fn envelope_open(key: &[u8; 32], sealed: &[u8]) -> Result<(u64, Vec<u8>), &'static str> {
    if sealed.len() < 8 + 8 + 4 + 32 {
        return Err("envelope truncated");
    }
    if &sealed[0..8] != ENVELOPE_MAGIC.as_slice() {
        return Err("bad envelope magic");
    }
    let key_id = u64::from_le_bytes(sealed[8..16].try_into().unwrap());
    let len = u32::from_le_bytes(sealed[16..20].try_into().unwrap()) as usize;
    let body_end = 20 + len;
    if sealed.len() != body_end + 32 {
        return Err("envelope length mismatch");
    }
    let cipher = &sealed[20..body_end];
    let tag = &sealed[body_end..];
    let mut h = Hasher::new_keyed(key);
    h.update(ENVELOPE_MAGIC);
    h.update(&key_id.to_le_bytes());
    h.update(&(len as u32).to_le_bytes());
    h.update(cipher);
    if h.finalize().as_bytes() != tag {
        return Err("envelope tag mismatch");
    }
    let mut plain = vec![0u8; len];
    keystream_xor(key, key_id, cipher, &mut plain);
    Ok((key_id, plain))
}

/// True if `haystack` contains `needle` as a contiguous byte slice.
pub fn contains_plaintext(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn keystream_xor(key: &[u8; 32], key_id: u64, input: &[u8], out: &mut [u8]) {
    assert_eq!(input.len(), out.len());
    let mut counter = 0u64;
    let mut offset = 0usize;
    while offset < input.len() {
        let mut h = Hasher::new_keyed(key);
        h.update(b"rsh-ks");
        h.update(&key_id.to_le_bytes());
        h.update(&counter.to_le_bytes());
        let block = *h.finalize().as_bytes();
        let n = (input.len() - offset).min(32);
        for i in 0..n {
            out[offset + i] = input[offset + i] ^ block[i];
        }
        offset += n;
        counter = counter.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_has_no_plaintext() {
        let key = *blake3::hash(b"test-key").as_bytes();
        let plain = b"super-secret-payload-value";
        let sealed = envelope_seal(&key, 1, plain);
        assert!(!contains_plaintext(&sealed, plain));
        let (kid, opened) = envelope_open(&key, &sealed).unwrap();
        assert_eq!(kid, 1);
        assert_eq!(opened, plain);
    }

    #[test]
    fn wrong_key_fails_closed() {
        let key = *blake3::hash(b"k1").as_bytes();
        let other = *blake3::hash(b"k2").as_bytes();
        let sealed = envelope_seal(&key, 7, b"hello");
        assert!(envelope_open(&other, &sealed).is_err());
    }
}
