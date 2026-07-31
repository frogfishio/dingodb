//! Counter-based deterministic key/payload generation (no RNG heap).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Payload compressibility profile (SPEC §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadProfile {
    /// High-entropy bytes (incompressible).
    Incompressible,
    /// Low-entropy / highly compressible pattern.
    Compressible,
    /// Mixed: first half incompressible, second half compressible.
    Mixed,
}

impl PayloadProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incompressible => "incompressible",
            Self::Compressible => "compressible",
            Self::Mixed => "mixed",
        }
    }
}

/// Deterministic key bytes for `key_index` under `seed`.
///
/// Format: `k/{seed_hex8}/{key_index}` — stable ASCII, never random.
pub fn generate_key(seed: u64, key_index: u64) -> Vec<u8> {
    format!("k/{seed:016x}/{key_index}").into_bytes()
}

/// Fill `buf` with deterministic payload bytes for `(seed, seq, generation, profile)`.
///
/// Does **not** allocate beyond the caller's buffer. Safe for large values
/// streamed in chunks (caller may call repeatedly with different offsets via
/// [`fill_payload_range`]).
pub fn fill_payload(seed: u64, seq: u64, generation: u32, profile: PayloadProfile, buf: &mut [u8]) {
    fill_payload_range(seed, seq, generation, profile, 0, buf);
}

/// Fill `buf` as the slice of the logical payload starting at `offset`.
pub fn fill_payload_range(
    seed: u64,
    seq: u64,
    generation: u32,
    profile: PayloadProfile,
    offset: u64,
    buf: &mut [u8],
) {
    if buf.is_empty() {
        return;
    }
    match profile {
        PayloadProfile::Incompressible => fill_incompressible(seed, seq, generation, offset, buf),
        PayloadProfile::Compressible => fill_compressible(seed, seq, generation, offset, buf),
        PayloadProfile::Mixed => {
            // Treat logical payload as infinite mixed stream; compressibility
            // alternates every 4 KiB block of absolute offset.
            let mut done = 0usize;
            while done < buf.len() {
                let abs = offset + done as u64;
                let block = abs / 4096;
                let in_block = (abs % 4096) as usize;
                let room = (4096 - in_block).min(buf.len() - done);
                let slice = &mut buf[done..done + room];
                if block % 2 == 0 {
                    fill_incompressible(seed, seq, generation, abs, slice);
                } else {
                    fill_compressible(seed, seq, generation, abs, slice);
                }
                done += room;
            }
        }
    }
}

fn fill_incompressible(seed: u64, seq: u64, generation: u32, offset: u64, buf: &mut [u8]) {
    // Stream of SHA-256 blocks keyed by (seed, seq, gen, block_index).
    let mut pos = 0usize;
    while pos < buf.len() {
        let abs = offset + pos as u64;
        let block_idx = abs / 32;
        let within = (abs % 32) as usize;
        let block = hash_block(seed, seq, generation, block_idx);
        let take = (32 - within).min(buf.len() - pos);
        buf[pos..pos + take].copy_from_slice(&block[within..within + take]);
        pos += take;
    }
}

fn fill_compressible(seed: u64, seq: u64, generation: u32, offset: u64, buf: &mut [u8]) {
    // Repeating 16-byte pattern derived from seed/seq — highly compressible.
    let mut h = Sha256::new();
    h.update(b"pqh2-comp\0");
    h.update(seed.to_le_bytes());
    h.update(seq.to_le_bytes());
    h.update(generation.to_le_bytes());
    let pattern = h.finalize();
    let pat = &pattern[..16];
    for (i, b) in buf.iter_mut().enumerate() {
        let abs = offset as usize + i;
        *b = pat[abs % 16];
    }
}

fn hash_block(seed: u64, seq: u64, generation: u32, block_idx: u64) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"pqh2-inc\0");
    h.update(seed.to_le_bytes());
    h.update(seq.to_le_bytes());
    h.update(generation.to_le_bytes());
    h.update(block_idx.to_le_bytes());
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// SHA-256 of the full logical payload (streamed in 64 KiB windows — O(1) RAM).
pub fn payload_digest(
    seed: u64,
    seq: u64,
    generation: u32,
    profile: PayloadProfile,
    payload_len: u64,
) -> [u8; 32] {
    let mut h = Sha256::new();
    const WIN: usize = 64 * 1024;
    let mut buf = vec![0u8; WIN.min(payload_len as usize).max(1)];
    if payload_len == 0 {
        return h.finalize().into();
    }
    let mut offset = 0u64;
    while offset < payload_len {
        let n = ((payload_len - offset) as usize).min(WIN);
        fill_payload_range(seed, seq, generation, profile, offset, &mut buf[..n]);
        h.update(&buf[..n]);
        offset += n as u64;
    }
    let out = h.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_deterministic() {
        assert_eq!(generate_key(1, 0), generate_key(1, 0));
        assert_ne!(generate_key(1, 0), generate_key(1, 1));
        assert_ne!(generate_key(1, 0), generate_key(2, 0));
    }

    #[test]
    fn payload_deterministic_and_range_consistent() {
        let mut a = vec![0u8; 1000];
        let mut b = vec![0u8; 1000];
        fill_payload(42, 7, 0, PayloadProfile::Incompressible, &mut a);
        fill_payload(42, 7, 0, PayloadProfile::Incompressible, &mut b);
        assert_eq!(a, b);

        let mut left = vec![0u8; 400];
        let mut right = vec![0u8; 600];
        fill_payload_range(42, 7, 0, PayloadProfile::Incompressible, 0, &mut left);
        fill_payload_range(42, 7, 0, PayloadProfile::Incompressible, 400, &mut right);
        assert_eq!(&a[..400], &left[..]);
        assert_eq!(&a[400..], &right[..]);
    }

    #[test]
    fn compressible_is_repetitive() {
        let mut buf = vec![0u8; 256];
        fill_payload(1, 0, 0, PayloadProfile::Compressible, &mut buf);
        assert_eq!(&buf[..16], &buf[16..32]);
    }

    #[test]
    fn large_payload_digest_constant_ram_window() {
        // 1 MiB streamed — digest is stable.
        let d1 = payload_digest(9, 1, 0, PayloadProfile::Incompressible, 1024 * 1024);
        let d2 = payload_digest(9, 1, 0, PayloadProfile::Incompressible, 1024 * 1024);
        assert_eq!(d1, d2);
        let d3 = payload_digest(9, 2, 0, PayloadProfile::Incompressible, 1024 * 1024);
        assert_ne!(d1, d3);
    }
}
