//! Closed constraint decoder and evaluator (`HEAP_SPEC` §32.2).

use crate::error::{HeapError, HeapUnavailableCause};
use crate::ids::{CollectionId, StreamId};
use crate::rights::Rights;

/// Source-network constraint value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceNetwork {
    /// 4 = IPv4, 6 = IPv6.
    pub family: u8,
    /// Prefix length.
    pub prefix: u8,
    /// Network address bytes.
    pub address: Vec<u8>,
}

/// One critical constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// Collection allowlist.
    CollectionAllowlist(Vec<CollectionId>),
    /// Stream allowlist.
    StreamAllowlist(Vec<StreamId>),
    /// Operation allowlist.
    OperationAllowlist(Vec<u16>),
    /// Max request bytes.
    MaxRequestBytes(u64),
    /// Max result bytes.
    MaxResultBytes(u64),
    /// Max query work units.
    MaxQueryWork(u64),
    /// Max duration ms.
    MaxDurationMs(u64),
    /// Source network.
    SourceNetwork(SourceNetwork),
}

impl Constraint {
    /// Kind tag.
    pub fn kind(&self) -> u64 {
        match self {
            Self::CollectionAllowlist(_) => 1,
            Self::StreamAllowlist(_) => 2,
            Self::OperationAllowlist(_) => 3,
            Self::MaxRequestBytes(_) => 4,
            Self::MaxResultBytes(_) => 5,
            Self::MaxQueryWork(_) => 6,
            Self::MaxDurationMs(_) => 7,
            Self::SourceNetwork(_) => 8,
        }
    }
}

/// Effective constraint set after intersection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Constraints {
    inner: Vec<Constraint>,
}

impl Constraints {
    /// Empty (no narrowing).
    pub fn empty() -> Self {
        Self { inner: Vec::new() }
    }

    /// Decode canonical constraint array from CBOR-like tuples `(kind, critical, value)`.
    pub fn decode_canonical(entries: &[(u64, bool, Constraint)]) -> Result<Self, HeapError> {
        let mut last_kind = 0u64;
        let mut out = Vec::with_capacity(entries.len());
        for (i, (kind, critical, c)) in entries.iter().enumerate() {
            if !*critical {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::MalformedOrBadSignature,
                ));
            }
            if c.kind() != *kind {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::MalformedOrBadSignature,
                ));
            }
            if i > 0 && *kind <= last_kind {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::MalformedOrBadSignature,
                ));
            }
            last_kind = *kind;
            out.push(c.clone());
        }
        Ok(Self { inner: out })
    }

    /// From already-validated list sorted by kind.
    pub fn from_sorted(inner: Vec<Constraint>) -> Result<Self, HeapError> {
        let mut last = 0u64;
        for (i, c) in inner.iter().enumerate() {
            let k = c.kind();
            if i > 0 && k <= last {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::MalformedOrBadSignature,
                ));
            }
            last = k;
        }
        Ok(Self { inner })
    }

    /// Intersect two constraint sets (allowlists ∩, maxima min).
    pub fn intersect(&self, other: &Self) -> Result<Self, HeapError> {
        let mut map = std::collections::BTreeMap::<u64, Constraint>::new();
        for c in self.inner.iter().chain(other.inner.iter()) {
            match map.get(&c.kind()) {
                None => {
                    map.insert(c.kind(), c.clone());
                }
                Some(existing) => {
                    let merged = merge_pair(existing, c)?;
                    map.insert(c.kind(), merged);
                }
            }
        }
        Ok(Self {
            inner: map.into_values().collect(),
        })
    }

    /// Borrow constraints.
    pub fn as_slice(&self) -> &[Constraint] {
        &self.inner
    }

    /// Whether an operation ID is permitted.
    pub fn allows_operation(&self, op: u16) -> bool {
        for c in &self.inner {
            if let Constraint::OperationAllowlist(ids) = c {
                if !ids.contains(&op) {
                    return false;
                }
            }
        }
        true
    }

    /// Whether a collection is permitted.
    pub fn allows_collection(&self, id: CollectionId) -> bool {
        for c in &self.inner {
            if let Constraint::CollectionAllowlist(ids) = c {
                if !ids.contains(&id) {
                    return false;
                }
            }
        }
        true
    }

    /// Whether a stream is permitted.
    pub fn allows_stream(&self, id: StreamId) -> bool {
        for c in &self.inner {
            if let Constraint::StreamAllowlist(ids) = c {
                if !ids.contains(&id) {
                    return false;
                }
            }
        }
        true
    }

    /// Maximum query work units when constrained; `None` means unbounded by certificate.
    pub fn max_query_work(&self) -> Option<u64> {
        for c in &self.inner {
            if let Constraint::MaxQueryWork(max) = c {
                return Some(*max);
            }
        }
        None
    }

    /// Maximum result bytes when constrained.
    pub fn max_result_bytes(&self) -> Option<u64> {
        for c in &self.inner {
            if let Constraint::MaxResultBytes(max) = c {
                return Some(*max);
            }
        }
        None
    }
}

fn merge_pair(a: &Constraint, b: &Constraint) -> Result<Constraint, HeapError> {
    use Constraint::*;
    match (a, b) {
        (CollectionAllowlist(x), CollectionAllowlist(y)) => {
            let set: Vec<_> = x.iter().filter(|id| y.contains(id)).copied().collect();
            if set.is_empty() {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::ConstraintDenied,
                ));
            }
            Ok(CollectionAllowlist(set))
        }
        (StreamAllowlist(x), StreamAllowlist(y)) => {
            let set: Vec<_> = x.iter().filter(|id| y.contains(id)).copied().collect();
            if set.is_empty() {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::ConstraintDenied,
                ));
            }
            Ok(StreamAllowlist(set))
        }
        (OperationAllowlist(x), OperationAllowlist(y)) => {
            let set: Vec<_> = x.iter().filter(|id| y.contains(id)).copied().collect();
            if set.is_empty() {
                return Err(HeapError::unavailable(
                    HeapUnavailableCause::ConstraintDenied,
                ));
            }
            Ok(OperationAllowlist(set))
        }
        (MaxRequestBytes(x), MaxRequestBytes(y)) => Ok(MaxRequestBytes((*x).min(*y))),
        (MaxResultBytes(x), MaxResultBytes(y)) => Ok(MaxResultBytes((*x).min(*y))),
        (MaxQueryWork(x), MaxQueryWork(y)) => Ok(MaxQueryWork((*x).min(*y))),
        (MaxDurationMs(x), MaxDurationMs(y)) => Ok(MaxDurationMs((*x).min(*y))),
        (SourceNetwork(x), SourceNetwork(y)) if x == y => Ok(SourceNetwork(x.clone())),
        (SourceNetwork(_), SourceNetwork(_)) => Err(HeapError::unavailable(
            HeapUnavailableCause::ConstraintDenied,
        )),
        _ => Err(HeapError::unavailable(
            HeapUnavailableCause::MalformedOrBadSignature,
        )),
    }
}

/// Combine certificate rights with required operation rights.
pub fn require_rights(effective: Rights, required: Rights) -> Result<(), HeapError> {
    if effective.contains(required) {
        Ok(())
    } else {
        Err(HeapError::unavailable(
            HeapUnavailableCause::InsufficientRights,
        ))
    }
}
