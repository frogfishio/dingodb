//! Stream examination units from store salvage or raw bytes.

use crate::error::ExamineError;
use crate::limits::{ExamineLimits, ExaminePage, PageCoverage};
use crate::project::{project_bytes, ProjectOptions};
use crate::unit::{cmp_units, ExaminationUnit};
use residuum_format::SafetyLimits;
use residuum_store::Store;

/// Examine every authoritative segment of an open store (catalog-free).
///
/// Projection uses the same forward salvage scanner as Stage 2–3. Units are
/// sorted by SDA_PROFILE §12 before paging. Resource limits produce incomplete
/// pages, never a fake empty complete result.
pub fn examine_store(store: &Store, limits: ExamineLimits) -> Result<ExaminePage, ExamineError> {
    let sources = store.examination_sources()?;
    let safety = store.safety_limits();
    let opts = ProjectOptions {
        materialize_payloads: limits.materialize_payloads,
        store_id: Some(store.store_id()),
    };
    examine_sources(&sources, safety, &limits, &opts)
}

/// Examine a single named byte source (tests and offline salvage tools).
pub fn examine_bytes(
    source: &str,
    bytes: &[u8],
    safety: SafetyLimits,
    limits: ExamineLimits,
) -> Result<ExaminePage, ExamineError> {
    let opts = ProjectOptions {
        materialize_payloads: limits.materialize_payloads,
        store_id: None,
    };
    examine_sources(
        &[(source.to_string(), bytes.to_vec())],
        safety,
        &limits,
        &opts,
    )
}

/// Project and order units from pre-loaded sources.
pub fn examine_sources(
    sources: &[(String, Vec<u8>)],
    safety: SafetyLimits,
    limits: &ExamineLimits,
    opts: &ProjectOptions,
) -> Result<ExaminePage, ExamineError> {
    let mut units: Vec<ExaminationUnit> = Vec::new();
    let mut bytes_read: u64 = 0;
    let mut truncated = false;
    let mut uncertainty: Vec<String> = Vec::new();

    // Deterministic source order: by source name (sealed hex names sort; active last).
    let mut ordered: Vec<&(String, Vec<u8>)> = sources.iter().collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));

    for (source, bytes) in ordered {
        if let Some(max_b) = limits.max_bytes_read {
            if bytes_read >= max_b {
                truncated = true;
                break;
            }
            // If this file would exceed the budget, still scan a prefix? Stage 5:
            // skip remaining whole sources and mark incomplete (honest).
            if bytes_read.saturating_add(bytes.len() as u64) > max_b && bytes_read > 0 {
                truncated = true;
                break;
            }
        }
        bytes_read = bytes_read.saturating_add(bytes.len() as u64);
        let mut projected = project_bytes(source, bytes, safety, opts);
        units.append(&mut projected);
    }

    units.sort_by(cmp_units);

    let mut continuation = None;
    if let Some(max_u) = limits.max_units {
        if units.len() > max_u {
            truncated = true;
            // Continuation token: start index for a subsequent page (opaque bytes).
            let next = (max_u as u64).to_le_bytes();
            continuation = Some(next.to_vec());
            units.truncate(max_u);
        }
    }

    if truncated {
        push_unique(&mut uncertainty, "resource-limited");
    }

    // Empty incomplete page still reports truncation (never "complete empty").
    let complete = !truncated;

    let query_id = query_id_for(sources, &units);

    Ok(ExaminePage {
        query_id,
        page_number: 0,
        complete,
        units,
        coverage: PageCoverage::salvage_default(),
        continuation,
        uncertainty,
    })
}

fn query_id_for(sources: &[(String, Vec<u8>)], units: &[ExaminationUnit]) -> String {
    // Stable diagnostic id from source names + unit count (not a security hash).
    let mut acc = String::new();
    for (name, bytes) in sources {
        acc.push_str(name);
        acc.push(':');
        acc.push_str(&bytes.len().to_string());
        acc.push(';');
    }
    acc.push_str(&format!("n={}", units.len()));
    // First 16 hex chars of a simple FNV-1a over the description.
    let h = fnv1a64(acc.as_bytes());
    format!("{h:016x}")
}

fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for b in data {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn push_unique(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|t| t == tag) {
        tags.push(tag.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use residuum_store::DurabilityMode;
    use tempfile::tempdir;

    #[test]
    fn examine_clean_store_has_verified_events() {
        let dir = tempdir().unwrap();
        let mut store = Store::create(dir.path()).unwrap();
        store.put("alpha", b"A", DurabilityMode::Durable).unwrap();
        let page = examine_store(&store, ExamineLimits::default()).unwrap();
        assert!(page.complete);
        assert!(page
            .units
            .iter()
            .any(|u| { u.unit_kind == "event" && u.status == "verified-complete" }));
    }

    #[test]
    fn max_units_is_incomplete_not_empty_success() {
        let dir = tempdir().unwrap();
        let mut store = Store::create(dir.path()).unwrap();
        for i in 0..5 {
            store
                .put(&format!("k{i}"), b"v", DurabilityMode::Durable)
                .unwrap();
        }
        let page = examine_store(&store, ExamineLimits::default().max_units(1)).unwrap();
        assert!(!page.complete);
        assert!(page.uncertainty.iter().any(|t| t == "resource-limited"));
        assert_eq!(page.units.len(), 1);
        assert!(page.continuation.is_some());
    }
}
