//! Sorted Eytzinger layout for tiny segments.
//!
//! Keys are stored in heap order (root at index 0, children at 2i+1 / 2i+2)
//! so a point probe walks a cache-friendly path without binary-search pivots.

/// Eytzinger-ordered keys and parallel offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EytzingerIndex {
    /// Keys in Eytzinger order.
    keys: Vec<Vec<u8>>,
    /// Frame offsets aligned with `keys`.
    offsets: Vec<u64>,
    /// Original sorted-order indices for range scans (permutation of 0..n).
    /// `sorted_pos[i]` = Eytzinger slot of the i-th sorted key.
    sorted_to_eyt: Vec<u32>,
}

impl EytzingerIndex {
    /// Build from unique keys sorted ascending.
    pub fn build(sorted: &[(Vec<u8>, u64)]) -> Self {
        let n = sorted.len();
        if n == 0 {
            return Self {
                keys: Vec::new(),
                offsets: Vec::new(),
                sorted_to_eyt: Vec::new(),
            };
        }
        let mut keys = vec![Vec::new(); n];
        let mut offsets = vec![0u64; n];
        let mut sorted_to_eyt = vec![0u32; n];
        // Classic Eytzinger: in-order visit of heap indices 0..n places sorted
        // keys while preserving the BST property (left < node < right).
        fn inorder(
            sorted: &[(Vec<u8>, u64)],
            keys: &mut [Vec<u8>],
            offsets: &mut [u64],
            sorted_to_eyt: &mut [u32],
            eyt: usize,
            next: &mut usize,
            n: usize,
        ) {
            if eyt >= n {
                return;
            }
            inorder(sorted, keys, offsets, sorted_to_eyt, 2 * eyt + 1, next, n);
            keys[eyt] = sorted[*next].0.clone();
            offsets[eyt] = sorted[*next].1;
            sorted_to_eyt[*next] = eyt as u32;
            *next += 1;
            inorder(sorted, keys, offsets, sorted_to_eyt, 2 * eyt + 2, next, n);
        }
        let mut next = 0usize;
        inorder(
            sorted,
            &mut keys,
            &mut offsets,
            &mut sorted_to_eyt,
            0,
            &mut next,
            n,
        );
        Self {
            keys,
            offsets,
            sorted_to_eyt,
        }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn get(&self, key: &[u8]) -> Option<u64> {
        if self.keys.is_empty() {
            return None;
        }
        let mut i = 0usize;
        while i < self.keys.len() {
            match key.cmp(self.keys[i].as_slice()) {
                std::cmp::Ordering::Equal => return Some(self.offsets[i]),
                std::cmp::Ordering::Less => i = 2 * i + 1,
                std::cmp::Ordering::Greater => i = 2 * i + 2,
            }
        }
        None
    }

    pub fn scan_after(&self, after: Option<&[u8]>, limit: usize) -> Vec<(Vec<u8>, u64)> {
        if limit == 0 || self.keys.is_empty() {
            return Vec::new();
        }
        let n = self.keys.len();
        // sorted_to_eyt[s] = eytzinger slot of the s-th sorted key.
        let start = match after {
            None => 0,
            Some(a) => {
                // First sorted rank with key > a.
                let mut lo = 0usize;
                let mut hi = n;
                while lo < hi {
                    let mid = lo + (hi - lo) / 2;
                    let eyt = self.sorted_to_eyt[mid] as usize;
                    if self.keys[eyt].as_slice() <= a {
                        lo = mid + 1;
                    } else {
                        hi = mid;
                    }
                }
                lo
            }
        };
        let mut out = Vec::with_capacity(limit.min(n.saturating_sub(start)));
        for s in start..n {
            if out.len() >= limit {
                break;
            }
            let eyt = self.sorted_to_eyt[s] as usize;
            out.push((self.keys[eyt].clone(), self.offsets[eyt]));
        }
        out
    }
}

pub(crate) fn encode(index: &EytzingerIndex, out: &mut Vec<u8>) {
    out.extend_from_slice(&(index.keys.len() as u64).to_le_bytes());
    for (k, &off) in index.keys.iter().zip(index.offsets.iter()) {
        let kl = u16::try_from(k.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&kl.to_le_bytes());
        out.extend_from_slice(&k[..kl as usize]);
        out.extend_from_slice(&off.to_le_bytes());
    }
    // sorted_to_eyt
    for &v in &index.sorted_to_eyt {
        out.extend_from_slice(&v.to_le_bytes());
    }
}

pub(crate) fn decode(bytes: &[u8]) -> Option<EytzingerIndex> {
    if bytes.len() < 8 {
        return None;
    }
    let n = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as usize;
    let mut off = 8;
    let mut keys = Vec::with_capacity(n);
    let mut offsets = Vec::with_capacity(n);
    for _ in 0..n {
        if off + 2 > bytes.len() {
            return None;
        }
        let kl = u16::from_le_bytes(bytes[off..off + 2].try_into().ok()?) as usize;
        off += 2;
        if off + kl + 8 > bytes.len() {
            return None;
        }
        keys.push(bytes[off..off + kl].to_vec());
        off += kl;
        offsets.push(u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?));
        off += 8;
    }
    let mut sorted_to_eyt = Vec::with_capacity(n);
    for _ in 0..n {
        if off + 4 > bytes.len() {
            return None;
        }
        sorted_to_eyt.push(u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?));
        off += 4;
    }
    Some(EytzingerIndex {
        keys,
        offsets,
        sorted_to_eyt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eytzinger_all_hits() {
        let sorted: Vec<_> = (0..31u64)
            .map(|i| (format!("{i:03}").into_bytes(), i * 7))
            .collect();
        let idx = EytzingerIndex::build(&sorted);
        for (k, o) in &sorted {
            assert_eq!(idx.get(k), Some(*o));
        }
        assert!(idx.get(b"999").is_none());
    }
}
