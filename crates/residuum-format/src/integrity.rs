//! CRC32C and BLAKE3 integrity helpers (FORMAT_SPEC §3, §4.1, §4.6).

/// BLAKE3-256 digest length.
pub const BODY_HASH_LEN: usize = 32;

/// CRC32C over the 64-byte prefix (CRC field zeroed) concatenated with the envelope.
///
/// Covers FORMAT_SPEC §4.1: bytes 56..60 of the prefix are treated as zero while hashing.
pub fn prefix_crc32c(prefix64: &[u8; 64], envelope: &[u8]) -> u32 {
    let mut buf = [0u8; 64];
    buf.copy_from_slice(prefix64);
    // Zero the prefix_crc32c field at offset 56..60 for the covered region.
    buf[56..60].fill(0);
    let mut crc = crc32c::crc32c(&buf);
    crc = crc32c::crc32c_append(crc, envelope);
    crc
}

/// CRC32C over the 56-byte suffix with the CRC field (bytes 48..52) treated as zero.
pub fn suffix_crc32c(suffix56: &[u8; 56]) -> u32 {
    let mut buf = *suffix56;
    buf[48..52].fill(0);
    crc32c::crc32c(&buf)
}

/// BLAKE3-256 over the stored body bytes (empty body is the empty digest).
pub fn body_hash(body: &[u8]) -> [u8; BODY_HASH_LEN] {
    *blake3::hash(body).as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_body_hash_is_stable() {
        let h = body_hash(b"");
        // blake3("") is a fixed known value; spot-check first and last bytes.
        assert_eq!(
            h,
            [
                0xaf, 0x13, 0x49, 0xb9, 0xf5, 0xf9, 0xa1, 0xa6, 0xa0, 0x40, 0x4d, 0xea, 0x36, 0xdc,
                0xc9, 0x49, 0x9b, 0xcb, 0x25, 0xc9, 0xad, 0xc1, 0x12, 0xb7, 0xcc, 0x9a, 0x93, 0xca,
                0xe4, 0x1f, 0x32, 0x62,
            ]
        );
    }

    #[test]
    fn prefix_crc_changes_with_envelope() {
        let prefix = [0u8; 64];
        let a = prefix_crc32c(&prefix, b"");
        let b = prefix_crc32c(&prefix, b"x");
        assert_ne!(a, b);
    }

    #[test]
    fn suffix_crc_ignores_crc_field_bytes() {
        let mut s = [0u8; 56];
        let base = suffix_crc32c(&s);
        s[48..52].copy_from_slice(&base.to_le_bytes());
        assert_eq!(suffix_crc32c(&s), base);
        s[0] = 1;
        assert_ne!(suffix_crc32c(&s), base);
    }
}
