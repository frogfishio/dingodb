//! Hashed evidence bundle for independent verification.

use super::disclosure::DisclosureSummary;
use super::reports::CampaignReports;
use super::run::CampaignResult;
use super::CampaignError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const BUNDLE_SCHEMA: &str = "residiuum-performance-evidence-bundle-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub schema: String,
    pub campaign_id: String,
    pub profile: String,
    pub platform: String,
    pub allows_product_baseline: bool,
    pub result: CampaignResult,
    pub reports: CampaignReports,
    pub disclosure: DisclosureSummary,
    /// SHA-256 over ordered `relative_path\\0sha256\\n` of file_hashes.
    pub content_hash: String,
    pub file_hashes: Vec<FileHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileHash {
    pub relative_path: String,
    pub sha256_hex: String,
}

/// Write campaign evidence under `campaign_dir/` and return the bundle.
pub fn write_evidence_bundle(
    campaign_dir: &Path,
    result: &CampaignResult,
    reports: &CampaignReports,
    disclosure: &DisclosureSummary,
) -> Result<EvidenceBundle, CampaignError> {
    fs::create_dir_all(campaign_dir)?;
    fs::create_dir_all(campaign_dir.join("runs"))?;

    let plan_path = campaign_dir.join("plan.json");
    write_json(&plan_path, &result.plan)?;

    let result_path = campaign_dir.join("campaign_result.json");
    write_json(&result_path, result)?;

    let reports_path = campaign_dir.join("reports.json");
    write_json(&reports_path, reports)?;

    let disclosure_path = campaign_dir.join("disclosure.json");
    write_json(&disclosure_path, disclosure)?;

    let disclosure_md = super::disclosure::render_disclosure_markdown(disclosure, reports);
    fs::write(campaign_dir.join("DISCLOSURE.md"), disclosure_md)?;

    // Per-run summaries (bounded; no payloads)
    for rep in &result.repetitions {
        let run_dir = campaign_dir.join("runs").join(&rep.run_id);
        fs::create_dir_all(&run_dir)?;
        write_json(&run_dir.join("result.json"), &rep.report)?;
    }

    // Hash on-disk bytes (stable independent check — no re-serialize drift).
    let mut file_hashes = Vec::new();
    for rel in [
        "plan.json",
        "campaign_result.json",
        "reports.json",
        "disclosure.json",
        "DISCLOSURE.md",
    ] {
        let p = campaign_dir.join(rel);
        file_hashes.push(FileHash {
            relative_path: rel.into(),
            sha256_hex: hash_file(&p)?,
        });
    }
    let content_hash = hash_file_list(&file_hashes);

    let bundle = EvidenceBundle {
        schema: BUNDLE_SCHEMA.into(),
        campaign_id: result.plan.campaign_id.clone(),
        profile: result.plan.profile.clone(),
        platform: result.plan.platform.as_str().into(),
        allows_product_baseline: result.plan.platform.allows_product_baseline(),
        result: result.clone(),
        reports: reports.clone(),
        disclosure: disclosure.clone(),
        content_hash,
        file_hashes,
    };

    write_json(&campaign_dir.join("bundle.json"), &bundle)?;
    // hashes manifest
    write_json(
        &campaign_dir.join("hashes.json"),
        &serde_json::json!({
            "schema": "residiuum-performance-bundle-hashes-v1",
            "content_hash": bundle.content_hash,
            "files": bundle.file_hashes,
        }),
    )?;

    Ok(bundle)
}

/// Re-hash on-disk files and compare to bundle.file_hashes (independent check).
pub fn verify_bundle_hashes(campaign_dir: &Path) -> Result<(), CampaignError> {
    let raw = fs::read_to_string(campaign_dir.join("bundle.json"))
        .map_err(|e| CampaignError::Bundle(e.to_string()))?;
    let bundle: EvidenceBundle =
        serde_json::from_str(&raw).map_err(|e| CampaignError::Bundle(e.to_string()))?;

    if bundle.schema != BUNDLE_SCHEMA {
        return Err(CampaignError::Bundle(format!(
            "unexpected schema {}",
            bundle.schema
        )));
    }

    let mut recomputed_files = Vec::new();
    for fh in &bundle.file_hashes {
        let p = campaign_dir.join(&fh.relative_path);
        let got = hash_file(&p)?;
        if got != fh.sha256_hex {
            return Err(CampaignError::Bundle(format!(
                "hash mismatch for {}: expected {} got {}",
                fh.relative_path, fh.sha256_hex, got
            )));
        }
        recomputed_files.push(FileHash {
            relative_path: fh.relative_path.clone(),
            sha256_hex: got,
        });
    }

    let recomputed = hash_file_list(&recomputed_files);
    if recomputed != bundle.content_hash {
        return Err(CampaignError::Bundle(format!(
            "content_hash mismatch: expected {} got {}",
            bundle.content_hash, recomputed
        )));
    }
    Ok(())
}

fn hash_file_list(files: &[FileHash]) -> String {
    let mut hasher = Sha256::new();
    for fh in files {
        hasher.update(fh.relative_path.as_bytes());
        hasher.update(b"\0");
        hasher.update(fh.sha256_hex.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

fn hash_file(path: &Path) -> Result<String, CampaignError> {
    let bytes = fs::read(path).map_err(|e| CampaignError::Io(e.to_string()))?;
    Ok(hex::encode(Sha256::digest(&bytes)))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), CampaignError> {
    let s = serde_json::to_string_pretty(value).map_err(|e| CampaignError::Bundle(e.to_string()))?;
    fs::write(path, s)?;
    let _ = PathBuf::from(path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::disclosure::build_disclosure;
    use crate::campaign::plan::campaign_plan_synthetic;
    use crate::campaign::reports::build_campaign_reports;
    use crate::campaign::run::{run_campaign, CampaignConfig};

    #[test]
    fn bundle_roundtrip_verifies() {
        let dir = tempfile::tempdir().unwrap();
        let plan = campaign_plan_synthetic(7, 2);
        let result = run_campaign(&CampaignConfig { plan }).unwrap();
        let reports = build_campaign_reports(&result);
        let disclosure = build_disclosure(&result, &reports);
        let bundle = write_evidence_bundle(dir.path(), &result, &reports, &disclosure).unwrap();
        assert_eq!(bundle.schema, BUNDLE_SCHEMA);
        assert!(!bundle.content_hash.is_empty());
        verify_bundle_hashes(dir.path()).unwrap();
        // tamper
        fs::write(dir.path().join("plan.json"), b"{}").unwrap();
        assert!(verify_bundle_hashes(dir.path()).is_err());
    }
}