//! CSQ-4 coverage-aware key/document scan model (DEF-100 / CSQ-ABS-002).

use crate::{ModelStore, ValueObservation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One key row in a scan page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanKeyRow {
    /// Subject bytes.
    pub subject: Vec<u8>,
    /// Point observation for the subject.
    pub observation: ValueObservation,
    /// True when key-bearing authority is present even if body is damaged.
    pub key_survives: bool,
}

/// Coverage claim for a scan (CSQ-ABS-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanCompleteness {
    /// Key-bearing authority coverage is complete.
    Complete,
    /// Incomplete; must not claim a full key set.
    Incomplete,
}

/// Scan page result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanPage {
    /// Rows in subject order.
    pub rows: Vec<ScanKeyRow>,
    /// Completeness claim.
    pub completeness: ScanCompleteness,
    /// Body completeness is separate from key coverage (CSQ-OBS-005).
    pub body_incomplete_subjects: Vec<Vec<u8>>,
}

impl ModelStore {
    /// Coverage-aware key scan.
    ///
    /// Body damage cannot hide an independently surviving key: the subject still
    /// appears with `Unavailable` observation and `key_survives = true`.
    pub fn scan_keys(&self) -> ScanPage {
        let mut subjects: BTreeMap<Vec<u8>, ()> = BTreeMap::new();
        for e in &self.events {
            subjects.insert(e.subject.clone(), ());
        }
        for sk in self
            .known_damage
            .keys()
            .chain(self.unavailable_coverage.iter())
        {
            if let Ok(bytes) = crate::hex::decode(sk) {
                subjects.insert(bytes, ());
            }
        }

        let mut body_incomplete = Vec::new();
        let mut rows = Vec::new();
        let mut any_incomplete = false;

        for subject in subjects.keys() {
            let sk = Self::subject_key(subject);
            let obs = self.get(subject);
            let key_survives = true; // presence in authority map
            if matches!(
                obs,
                ValueObservation::Unavailable { .. } | ValueObservation::Conflict { .. }
            ) {
                body_incomplete.push(subject.clone());
            }
            if self.unavailable_coverage.contains(&sk) || self.known_damage.contains_key(&sk) {
                any_incomplete = true;
            }
            // Tombstoned / never present: still list if we have event history.
            rows.push(ScanKeyRow {
                subject: subject.clone(),
                observation: obs,
                key_survives,
            });
        }

        // Completeness only when no coverage holes (CSQ-ABS-002).
        let completeness = if any_incomplete {
            ScanCompleteness::Incomplete
        } else {
            ScanCompleteness::Complete
        };

        ScanPage {
            rows,
            completeness,
            body_incomplete_subjects: body_incomplete,
        }
    }
}
