//! Shared logical work package for cross-engine adapters.
//!
//! Mongo / CBL / Residiuum all load the same [`LogicalDataset`] content hash
//! so comparative cells start from equivalent fixtures (programme hard law).

use crate::generator::LogicalDataset;
use serde::{Deserialize, Serialize};

/// Handle to shared logical work (datasets identical across engines).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedLogicalWork {
    pub content_hash: String,
    pub doc_count: u64,
    pub seed: u64,
    pub dataset: LogicalDataset,
}

impl SharedLogicalWork {
    pub fn from_dataset(dataset: LogicalDataset) -> Self {
        let doc_count = dataset
            .collections
            .get("docs")
            .map(|d| d.len() as u64)
            .unwrap_or(0);
        let seed = dataset.spec.seed;
        let content_hash = dataset.content_hash.clone();
        Self {
            content_hash,
            doc_count,
            seed,
            dataset,
        }
    }

    pub fn matches_hash(&self, other: &str) -> bool {
        self.content_hash == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::DatasetSpec;
    use crate::generator::generate_dataset;

    #[test]
    fn shared_work_hash_stable() {
        let ds = generate_dataset(&DatasetSpec::smoke_default(9));
        let w1 = SharedLogicalWork::from_dataset(ds.clone());
        let w2 = SharedLogicalWork::from_dataset(ds);
        assert_eq!(w1.content_hash, w2.content_hash);
        assert_eq!(w1.doc_count, 64);
    }
}
