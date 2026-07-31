//! Evaluate pure SDA programs over examination units (SDA_PROFILE §13).

use crate::error::ExamineError;
use crate::limits::ExaminePage;
use crate::unit::ExaminationUnit;
use sda_core::{run, Value};

/// Run `program` with each unit as `input` (SDA `Prod`).
///
/// Returns the JSON encoding of each SDA result (value or `Fail`). Storage
/// damage is already data on the unit; language errors remain `Fail`.
pub fn map_units(
    units: &[ExaminationUnit],
    program: &str,
) -> Result<Vec<serde_json::Value>, ExamineError> {
    let mut out = Vec::with_capacity(units.len());
    for unit in units {
        let input = unit.to_json();
        let result = run(program, input)?;
        out.push(result);
    }
    Ok(out)
}

/// Keep units for which `program` evaluates to SDA `true`.
///
/// The program receives the examination unit as `input`. Non-boolean results
/// are an error (caller must write a predicate).
pub fn filter_units(
    units: &[ExaminationUnit],
    program: &str,
) -> Result<Vec<ExaminationUnit>, ExamineError> {
    let mut out = Vec::new();
    for unit in units {
        let input = unit.to_json();
        let result = run(program, input)?;
        match result {
            serde_json::Value::Bool(true) => out.push(unit.clone()),
            serde_json::Value::Bool(false) => {}
            other => {
                return Err(ExamineError::FilterNotBool(other.to_string()));
            }
        }
    }
    Ok(out)
}

/// Evaluate `program` once with the full page as `input`.
pub fn eval_page(page: &ExaminePage, program: &str) -> Result<serde_json::Value, ExamineError> {
    let input = sda_core::to_json(page.to_sda_value());
    Ok(run(program, input)?)
}

/// Evaluate `program` with a single unit as `input`, returning the SDA value.
pub fn eval_unit(unit: &ExaminationUnit, program: &str) -> Result<serde_json::Value, ExamineError> {
    Ok(run(program, unit.to_json())?)
}

/// Whether an SDA JSON result is `true`.
pub fn is_sda_true(v: &serde_json::Value) -> bool {
    matches!(v, serde_json::Value::Bool(true))
}

/// Whether an SDA JSON result is a language `Fail`.
pub fn is_sda_fail(v: &serde_json::Value) -> bool {
    v.get("$type").and_then(|t| t.as_str()) == Some("fail")
}

/// Convenience: filter to units whose `status` field equals `want` via SDA.
pub fn filter_status(
    units: &[ExaminationUnit],
    want: &str,
) -> Result<Vec<ExaminationUnit>, ExamineError> {
    // Escape is unnecessary for profile tags (alphanumeric + hyphens).
    let program = format!(r#"input<status> = "{want}""#);
    filter_units(units, &program)
}

/// Convenience: filter to hole units.
pub fn filter_holes(units: &[ExaminationUnit]) -> Result<Vec<ExaminationUnit>, ExamineError> {
    filter_units(units, r#"input<unit_kind> = "hole""#)
}

/// Convenience: filter to verified-complete units (islands).
pub fn filter_verified_complete(
    units: &[ExaminationUnit],
) -> Result<Vec<ExaminationUnit>, ExamineError> {
    filter_status(units, "verified-complete")
}

/// Project a unit's SDA value (for hosts that already hold `Value`).
pub fn unit_as_value(unit: &ExaminationUnit) -> Value {
    unit.to_sda_value()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::{IntegrityEvidence, PayloadInfo, PhysicalLocation};

    fn sample_unit(status: &str, kind: &str) -> ExaminationUnit {
        ExaminationUnit {
            unit_kind: kind.into(),
            status: status.into(),
            store_id: None,
            segment_id: Some("aa".into()),
            item_id: None,
            event_id: Some("bb".into()),
            event_kind: None,
            physical: PhysicalLocation {
                source: "s.residiuum".into(),
                offset: Some(0),
                encoded_length: Some(1),
                wire_major: Some(1),
                wire_minor: Some(0),
            },
            integrity: IntegrityEvidence::verified_no_auth(),
            envelope: vec![],
            payload: PayloadInfo::not_applicable(),
            holes: vec![],
            provenance: vec![],
            uncertainty: vec![],
        }
    }

    #[test]
    fn filter_by_status_sda() {
        let units = vec![
            sample_unit("verified-complete", "event"),
            sample_unit("corrupt", "hole"),
        ];
        let kept = filter_verified_complete(&units).unwrap();
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].status, "verified-complete");
    }

    #[test]
    fn map_extracts_status() {
        let units = vec![sample_unit("verified-complete", "event")];
        let out = map_units(&units, "input<status>").unwrap();
        assert_eq!(out[0], serde_json::json!("verified-complete"));
    }
}
