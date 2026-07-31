//! PGM++ style piecewise-linear model and RadixSpline for ordered numeric keys.
//!
//! Keys are ranked by big-endian integer interpretation (4- or 8-byte fixed
//! width, or the first 8 bytes of longer keys). The model predicts the sorted
//! position; a bounded last-mile search corrects prediction error.

use super::select::be_u64;

/// One linear segment: position ≈ slope * key + intercept, within ±error.
#[derive(Debug, Clone, PartialEq)]
struct ModelSeg {
    /// First key covered (inclusive), as u64 rank.
    key_lo: u64,
    /// Slope in Q32.32 fixed point (delta_pos << 32 / delta_key), or 0 if single point.
    slope_q32: i64,
    /// Intercept in Q32.32 (predicted position of key_lo).
    intercept_q32: i64,
    /// First sorted index covered.
    pos_lo: u32,
    /// One past last sorted index covered.
    pos_hi: u32,
}

/// Piecewise-linear learned index with stored keys for exact verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgmIndex {
    keys: Vec<Vec<u8>>,
    offsets: Vec<u64>,
    segments: Vec<ModelSeg>,
    epsilon: u32,
}

// Manual Eq because ModelSeg uses i64 fields that are PartialEq.
impl Eq for ModelSeg {}

impl PgmIndex {
    /// Build a PGM with the given error bound (positions).
    pub fn build(sorted: &[(Vec<u8>, u64)], epsilon: u32) -> Self {
        let n = sorted.len();
        let keys: Vec<Vec<u8>> = sorted.iter().map(|(k, _)| k.clone()).collect();
        let offsets: Vec<u64> = sorted.iter().map(|(_, o)| *o).collect();
        let ranks: Vec<u64> = keys.iter().map(|k| be_u64(k)).collect();
        let epsilon = epsilon.max(1);
        let segments = if n == 0 {
            Vec::new()
        } else {
            build_pgm_segments(&ranks, epsilon)
        };
        Self {
            keys,
            offsets,
            segments,
            epsilon,
        }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn get(&self, key: &[u8]) -> Option<u64> {
        let n = self.keys.len();
        if n == 0 {
            return None;
        }
        let rank = be_u64(key);
        let pred = self.predict(rank);
        let eps = self.epsilon as usize;
        let lo = pred.saturating_sub(eps);
        let hi = (pred + eps + 1).min(n);
        // Last-mile: linear when window tiny, binary otherwise.
        if hi - lo <= 8 {
            for i in lo..hi {
                if self.keys[i].as_slice() == key {
                    return Some(self.offsets[i]);
                }
            }
            // Expand slightly if model under-estimated (rare boundary).
            let lo2 = lo.saturating_sub(eps);
            let hi2 = (hi + eps).min(n);
            for i in lo2..hi2 {
                if self.keys[i].as_slice() == key {
                    return Some(self.offsets[i]);
                }
            }
            return None;
        }
        let mut a = lo;
        let mut b = hi;
        while a < b {
            let mid = a + (b - a) / 2;
            match self.keys[mid].as_slice().cmp(key) {
                std::cmp::Ordering::Equal => return Some(self.offsets[mid]),
                std::cmp::Ordering::Less => a = mid + 1,
                std::cmp::Ordering::Greater => b = mid,
            }
        }
        // Fallback full binary search if prediction missed (correctness first).
        let mut a = 0usize;
        let mut b = n;
        while a < b {
            let mid = a + (b - a) / 2;
            match self.keys[mid].as_slice().cmp(key) {
                std::cmp::Ordering::Equal => return Some(self.offsets[mid]),
                std::cmp::Ordering::Less => a = mid + 1,
                std::cmp::Ordering::Greater => b = mid,
            }
        }
        None
    }

    pub fn scan_after(&self, after: Option<&[u8]>, limit: usize) -> Vec<(Vec<u8>, u64)> {
        scan_sorted(&self.keys, &self.offsets, after, limit)
    }

    fn predict(&self, rank: u64) -> usize {
        if self.segments.is_empty() {
            return 0;
        }
        // Binary search segment by key_lo.
        let mut lo = 0usize;
        let mut hi = self.segments.len();
        while lo + 1 < hi {
            let mid = lo + (hi - lo) / 2;
            if self.segments[mid].key_lo <= rank {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        let seg = &self.segments[lo];
        let dx = rank.saturating_sub(seg.key_lo) as i128;
        let pred_q = seg.intercept_q32 as i128 + (seg.slope_q32 as i128 * dx);
        let pred = (pred_q >> 32).clamp(0, (self.keys.len().saturating_sub(1)) as i128) as usize;
        // Clamp to segment position range with a little slack.
        let p_lo = seg.pos_lo as usize;
        let p_hi = (seg.pos_hi as usize).min(self.keys.len());
        pred.clamp(p_lo, p_hi.saturating_sub(1).max(p_lo))
    }
}

/// Radix table + linear spline knots for dense ordered numerics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadixSplineIndex {
    keys: Vec<Vec<u8>>,
    offsets: Vec<u64>,
    /// Knot positions in sorted order (including 0 and n-1).
    knot_pos: Vec<u32>,
    knot_rank: Vec<u64>,
    /// radix_table[r] = first knot index whose key's top bits ≥ r (or last).
    radix_table: Vec<u32>,
    radix_bits: u8,
    min_rank: u64,
    max_rank: u64,
}

impl RadixSplineIndex {
    pub fn build(sorted: &[(Vec<u8>, u64)], radix_bits: u8) -> Self {
        let n = sorted.len();
        let keys: Vec<Vec<u8>> = sorted.iter().map(|(k, _)| k.clone()).collect();
        let offsets: Vec<u64> = sorted.iter().map(|(_, o)| *o).collect();
        let ranks: Vec<u64> = keys.iter().map(|k| be_u64(k)).collect();
        let radix_bits = radix_bits.clamp(4, 16);
        if n == 0 {
            return Self {
                keys,
                offsets,
                knot_pos: Vec::new(),
                knot_rank: Vec::new(),
                radix_table: Vec::new(),
                radix_bits,
                min_rank: 0,
                max_rank: 0,
            };
        }
        let min_rank = ranks[0];
        let max_rank = ranks[n - 1];
        // Place knots every ~sqrt(n) positions (at least every 16, at most every 256).
        let step = ((n as f64).sqrt() as usize).clamp(16, 256);
        let mut knot_pos = Vec::new();
        let mut knot_rank = Vec::new();
        let mut p = 0usize;
        while p < n {
            knot_pos.push(p as u32);
            knot_rank.push(ranks[p]);
            if p + 1 == n {
                break;
            }
            p = (p + step).min(n - 1);
            if p == knot_pos.last().copied().unwrap_or(0) as usize && p + 1 < n {
                p += 1;
            }
        }
        if *knot_pos.last().unwrap() != (n - 1) as u32 {
            knot_pos.push((n - 1) as u32);
            knot_rank.push(ranks[n - 1]);
        }

        let table_size = 1usize << radix_bits;
        let mut radix_table = vec![0u32; table_size];
        let span = max_rank.saturating_sub(min_rank).max(1);
        // Fill radix table: for each bucket, first knot with rank mapping ≥ bucket.
        let mut k = 0usize;
        for (b, slot) in radix_table.iter_mut().enumerate() {
            while k + 1 < knot_rank.len()
                && radix_bucket(knot_rank[k], min_rank, span, radix_bits) < b
            {
                k += 1;
            }
            *slot = k as u32;
        }

        Self {
            keys,
            offsets,
            knot_pos,
            knot_rank,
            radix_table,
            radix_bits,
            min_rank,
            max_rank,
        }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn get(&self, key: &[u8]) -> Option<u64> {
        let n = self.keys.len();
        if n == 0 {
            return None;
        }
        let rank = be_u64(key);
        let (lo, hi) = self.predict_range(rank);
        let mut a = lo;
        let mut b = hi.min(n);
        while a < b {
            let mid = a + (b - a) / 2;
            match self.keys[mid].as_slice().cmp(key) {
                std::cmp::Ordering::Equal => return Some(self.offsets[mid]),
                std::cmp::Ordering::Less => a = mid + 1,
                std::cmp::Ordering::Greater => b = mid,
            }
        }
        // Full fallback for correctness.
        let mut a = 0usize;
        let mut b = n;
        while a < b {
            let mid = a + (b - a) / 2;
            match self.keys[mid].as_slice().cmp(key) {
                std::cmp::Ordering::Equal => return Some(self.offsets[mid]),
                std::cmp::Ordering::Less => a = mid + 1,
                std::cmp::Ordering::Greater => b = mid,
            }
        }
        None
    }

    pub fn scan_after(&self, after: Option<&[u8]>, limit: usize) -> Vec<(Vec<u8>, u64)> {
        scan_sorted(&self.keys, &self.offsets, after, limit)
    }

    fn predict_range(&self, rank: u64) -> (usize, usize) {
        if self.knot_pos.len() < 2 {
            return (0, self.keys.len());
        }
        let span = self.max_rank.saturating_sub(self.min_rank).max(1);
        let bucket = radix_bucket(rank, self.min_rank, span, self.radix_bits);
        let mut ki = self.radix_table[bucket.min(self.radix_table.len() - 1)] as usize;
        // Walk knots until rank is bracketed.
        while ki + 1 < self.knot_rank.len() && self.knot_rank[ki + 1] < rank {
            ki += 1;
        }
        if ki + 1 >= self.knot_rank.len() {
            let p = self.knot_pos[ki] as usize;
            return (p.saturating_sub(1), (p + 2).min(self.keys.len()));
        }
        let r0 = self.knot_rank[ki];
        let r1 = self.knot_rank[ki + 1];
        let p0 = self.knot_pos[ki] as usize;
        let p1 = self.knot_pos[ki + 1] as usize;
        if r1 == r0 {
            return (p0, (p1 + 1).min(self.keys.len()));
        }
        let t = (rank.saturating_sub(r0)) as f64 / (r1 - r0) as f64;
        let pred = p0 as f64 + t * (p1 - p0) as f64;
        let pred = pred.round() as isize;
        let pad = ((p1 - p0) / 4).max(2) as isize;
        let lo = (pred - pad).max(0) as usize;
        let hi = ((pred + pad + 1) as usize).min(self.keys.len());
        (lo, hi)
    }
}

fn radix_bucket(rank: u64, min_rank: u64, span: u64, radix_bits: u8) -> usize {
    let table_size = 1u64 << radix_bits;
    let rel = rank.saturating_sub(min_rank);
    let b = (rel as u128 * table_size as u128 / (span as u128 + 1)) as u64;
    b.min(table_size - 1) as usize
}

fn build_pgm_segments(ranks: &[u64], epsilon: u32) -> Vec<ModelSeg> {
    let n = ranks.len();
    let mut segs = Vec::new();
    let mut start = 0usize;
    while start < n {
        let mut end = start + 1;
        // Grow segment while max linear-fit error ≤ epsilon.
        while end < n {
            let candidate_end = end + 1;
            if max_fit_error(ranks, start, candidate_end) <= epsilon as u64 {
                end = candidate_end;
            } else {
                break;
            }
        }
        segs.push(make_seg(ranks, start, end));
        start = end;
    }
    segs
}

fn max_fit_error(ranks: &[u64], lo: usize, hi: usize) -> u64 {
    // hi exclusive. Fit line through (ranks[lo], lo) and (ranks[hi-1], hi-1).
    if hi - lo <= 2 {
        return 0;
    }
    let x0 = ranks[lo] as i128;
    let x1 = ranks[hi - 1] as i128;
    let y0 = lo as i128;
    let y1 = (hi - 1) as i128;
    let mut max_err = 0u64;
    if x1 == x0 {
        for (i, _) in ranks.iter().enumerate().take(hi).skip(lo) {
            let err = (i as i128 - y0).unsigned_abs() as u64;
            max_err = max_err.max(err);
        }
        return max_err;
    }
    for (i, &rank) in ranks.iter().enumerate().take(hi).skip(lo) {
        let x = rank as i128;
        let pred = y0 + (y1 - y0) * (x - x0) / (x1 - x0);
        let err = (i as i128 - pred).unsigned_abs() as u64;
        max_err = max_err.max(err);
    }
    max_err
}

fn make_seg(ranks: &[u64], lo: usize, hi: usize) -> ModelSeg {
    let key_lo = ranks[lo];
    let key_hi = ranks[hi - 1];
    let pos_lo = lo as u32;
    let pos_hi = hi as u32;
    let (slope_q32, intercept_q32) = if key_hi == key_lo || hi - lo == 1 {
        (0i64, (lo as i64) << 32)
    } else {
        let dx = (key_hi - key_lo) as i128;
        let dy = (hi - 1 - lo) as i128;
        let slope = (dy << 32) / dx;
        let intercept = (lo as i128) << 32;
        (slope as i64, intercept as i64)
    };
    ModelSeg {
        key_lo,
        slope_q32,
        intercept_q32,
        pos_lo,
        pos_hi,
    }
}

fn scan_sorted(
    keys: &[Vec<u8>],
    offsets: &[u64],
    after: Option<&[u8]>,
    limit: usize,
) -> Vec<(Vec<u8>, u64)> {
    if limit == 0 || keys.is_empty() {
        return Vec::new();
    }
    let start = match after {
        None => 0,
        Some(a) => {
            let mut lo = 0usize;
            let mut hi = keys.len();
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if keys[mid].as_slice() <= a {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            lo
        }
    };
    keys[start..]
        .iter()
        .zip(offsets[start..].iter())
        .take(limit)
        .map(|(k, o)| (k.clone(), *o))
        .collect()
}

pub(crate) fn encode_pgm(index: &PgmIndex, out: &mut Vec<u8>) {
    out.extend_from_slice(&index.epsilon.to_le_bytes());
    encode_kv(&index.keys, &index.offsets, out);
    out.extend_from_slice(&(index.segments.len() as u32).to_le_bytes());
    for s in &index.segments {
        out.extend_from_slice(&s.key_lo.to_le_bytes());
        out.extend_from_slice(&s.slope_q32.to_le_bytes());
        out.extend_from_slice(&s.intercept_q32.to_le_bytes());
        out.extend_from_slice(&s.pos_lo.to_le_bytes());
        out.extend_from_slice(&s.pos_hi.to_le_bytes());
    }
}

pub(crate) fn decode_pgm(bytes: &[u8]) -> Option<PgmIndex> {
    if bytes.len() < 4 {
        return None;
    }
    let epsilon = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let (keys, offsets, mut off) = decode_kv(bytes, 4)?;
    if off + 4 > bytes.len() {
        return None;
    }
    let nseg = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?) as usize;
    off += 4;
    let mut segments = Vec::with_capacity(nseg);
    for _ in 0..nseg {
        if off + 8 + 8 + 8 + 4 + 4 > bytes.len() {
            return None;
        }
        let key_lo = u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?);
        off += 8;
        let slope_q32 = i64::from_le_bytes(bytes[off..off + 8].try_into().ok()?);
        off += 8;
        let intercept_q32 = i64::from_le_bytes(bytes[off..off + 8].try_into().ok()?);
        off += 8;
        let pos_lo = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?);
        off += 4;
        let pos_hi = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?);
        off += 4;
        segments.push(ModelSeg {
            key_lo,
            slope_q32,
            intercept_q32,
            pos_lo,
            pos_hi,
        });
    }
    Some(PgmIndex {
        keys,
        offsets,
        segments,
        epsilon,
    })
}

pub(crate) fn encode_rs(index: &RadixSplineIndex, out: &mut Vec<u8>) {
    out.push(index.radix_bits);
    out.extend_from_slice(&index.min_rank.to_le_bytes());
    out.extend_from_slice(&index.max_rank.to_le_bytes());
    encode_kv(&index.keys, &index.offsets, out);
    out.extend_from_slice(&(index.knot_pos.len() as u32).to_le_bytes());
    for (&p, &r) in index.knot_pos.iter().zip(index.knot_rank.iter()) {
        out.extend_from_slice(&p.to_le_bytes());
        out.extend_from_slice(&r.to_le_bytes());
    }
    out.extend_from_slice(&(index.radix_table.len() as u32).to_le_bytes());
    for &v in &index.radix_table {
        out.extend_from_slice(&v.to_le_bytes());
    }
}

pub(crate) fn decode_rs(bytes: &[u8]) -> Option<RadixSplineIndex> {
    if bytes.len() < 1 + 8 + 8 {
        return None;
    }
    let radix_bits = bytes[0];
    let min_rank = u64::from_le_bytes(bytes[1..9].try_into().ok()?);
    let max_rank = u64::from_le_bytes(bytes[9..17].try_into().ok()?);
    let (keys, offsets, mut off) = decode_kv(bytes, 17)?;
    if off + 4 > bytes.len() {
        return None;
    }
    let nk = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?) as usize;
    off += 4;
    let mut knot_pos = Vec::with_capacity(nk);
    let mut knot_rank = Vec::with_capacity(nk);
    for _ in 0..nk {
        if off + 4 + 8 > bytes.len() {
            return None;
        }
        knot_pos.push(u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?));
        off += 4;
        knot_rank.push(u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?));
        off += 8;
    }
    if off + 4 > bytes.len() {
        return None;
    }
    let nt = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?) as usize;
    off += 4;
    let mut radix_table = Vec::with_capacity(nt);
    for _ in 0..nt {
        if off + 4 > bytes.len() {
            return None;
        }
        radix_table.push(u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?));
        off += 4;
    }
    Some(RadixSplineIndex {
        keys,
        offsets,
        knot_pos,
        knot_rank,
        radix_table,
        radix_bits,
        min_rank,
        max_rank,
    })
}

fn encode_kv(keys: &[Vec<u8>], offsets: &[u64], out: &mut Vec<u8>) {
    out.extend_from_slice(&(keys.len() as u64).to_le_bytes());
    for (k, &o) in keys.iter().zip(offsets.iter()) {
        let kl = u16::try_from(k.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&kl.to_le_bytes());
        out.extend_from_slice(&k[..kl as usize]);
        out.extend_from_slice(&o.to_le_bytes());
    }
}

fn decode_kv(bytes: &[u8], mut off: usize) -> Option<(Vec<Vec<u8>>, Vec<u64>, usize)> {
    if off + 8 > bytes.len() {
        return None;
    }
    let n = u64::from_le_bytes(bytes[off..off + 8].try_into().ok()?) as usize;
    off += 8;
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
    Some((keys, offsets, off))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pgm_sparse_hits() {
        let sorted: Vec<_> = (0..300u64)
            .map(|i| {
                let v = i * i + 7;
                (v.to_be_bytes().to_vec(), i * 11)
            })
            .collect();
        let idx = PgmIndex::build(&sorted, 16);
        for (k, o) in &sorted {
            assert_eq!(idx.get(k), Some(*o), "miss {:?}", k);
        }
        assert!(idx.get(&999999u64.to_be_bytes()).is_none());
    }

    #[test]
    fn radix_spline_dense_hits() {
        let sorted: Vec<_> = (0..500u64)
            .map(|i| (i.to_be_bytes().to_vec(), i * 3))
            .collect();
        let idx = RadixSplineIndex::build(&sorted, 8);
        for (k, o) in &sorted {
            assert_eq!(idx.get(k), Some(*o));
        }
    }
}
