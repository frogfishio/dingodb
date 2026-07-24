//! Normative ExaminationUnit product shape (SDA_PROFILE §3).

use sda_core::{ExactNum, Value};

/// Physical location of a recovered region (SDA_PROFILE §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalLocation {
    /// Stable scan-report name for the medium object.
    pub source: String,
    /// Inclusive start offset, when known.
    pub offset: Option<u64>,
    /// Encoded length in bytes, when known.
    pub encoded_length: Option<u64>,
    /// Wire major version when known from a verified header.
    pub wire_major: Option<u8>,
    /// Wire minor version when known from a verified header.
    pub wire_minor: Option<u8>,
}

/// Integrity evidence fields (SDA_PROFILE §5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityEvidence {
    /// Framing check outcome.
    pub framing: String,
    /// Structural check outcome.
    pub structural: String,
    /// Content / body hash outcome.
    pub content: String,
    /// Signature or AE evidence.
    pub authentication: String,
}

impl IntegrityEvidence {
    /// All structural checks verified; no authentication present.
    pub fn verified_no_auth() -> Self {
        Self {
            framing: "verified".into(),
            structural: "verified".into(),
            content: "verified".into(),
            authentication: "not-present".into(),
        }
    }

    /// Framing/structural/content failed (hole / corrupt candidate).
    pub fn failed() -> Self {
        Self {
            framing: "failed".into(),
            structural: "failed".into(),
            content: "not-checked".into(),
            authentication: "not-present".into(),
        }
    }
}

/// One extent of a partial payload (SDA_PROFILE §7.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extent {
    /// Logical start offset.
    pub logical_start: u64,
    /// Logical length of this extent.
    pub logical_length: u64,
    /// Extent status tag.
    pub status: String,
    /// Optional chunk identity (hex).
    pub chunk_id: Option<String>,
    /// Materialized bytes when present.
    pub value: Option<Vec<u8>>,
}

/// Payload descriptor (SDA_PROFILE §7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadInfo {
    /// Availability tag (`complete`, `partial`, `unavailable`, …).
    pub availability: String,
    /// Representation tag (`bytes`, `sda`, `unknown`, …).
    pub representation: String,
    /// Optional media type.
    pub media_type: Option<String>,
    /// Logical payload length when known.
    pub logical_length: Option<u64>,
    /// Materialized value when the host chose to include it.
    pub value: Option<Vec<u8>>,
    /// Ordered non-overlapping extents (chunk maps).
    pub extents: Vec<Extent>,
}

impl PayloadInfo {
    /// Complete opaque bytes payload.
    pub fn complete_bytes(body: &[u8], media_type: Option<&str>) -> Self {
        Self {
            availability: "complete".into(),
            representation: "bytes".into(),
            media_type: media_type.map(str::to_string),
            logical_length: Some(body.len() as u64),
            value: Some(body.to_vec()),
            extents: Vec::new(),
        }
    }

    /// Payload intentionally not materialized (resource limit).
    pub fn resource_limited(logical_length: Option<u64>) -> Self {
        Self {
            availability: "unavailable".into(),
            representation: "bytes".into(),
            media_type: None,
            logical_length,
            value: None,
            extents: Vec::new(),
        }
    }

    /// No payload applies (e.g. pure hole unit).
    pub fn not_applicable() -> Self {
        Self {
            availability: "not-applicable".into(),
            representation: "unknown".into(),
            media_type: None,
            logical_length: None,
            value: None,
            extents: Vec::new(),
        }
    }
}

/// Provenance step (SDA_PROFILE §9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceEntry {
    /// Action tag (`recovered`, `ingested`, …).
    pub action: String,
    /// Optional source identity.
    pub source_id: Option<String>,
    /// Tool name.
    pub tool: String,
    /// Tool version string.
    pub tool_version: String,
}

/// One envelope map entry after profile key projection (SDA_PROFILE §6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeEntry {
    /// Profile key (canonical name, `wire:N`, or `ext:…`).
    pub key: String,
    /// String form of the value for Stage 5 (opaque draft envelopes use strings/bytes).
    pub value: EnvelopeValue,
}

/// Envelope values projected into SDA for Stage 5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvelopeValue {
    /// UTF-8 text.
    Str(String),
    /// Opaque bytes.
    Bytes(Vec<u8>),
    /// Integer (e.g. event kind ordinals).
    Num(i64),
    /// Nested string set (hole `affects`).
    StrSet(Vec<String>),
}

/// Normative examination unit (SDA_PROFILE §3).
///
/// Storage damage and partial recovery are represented as data fields, never
/// collapsed into a single SDA `Fail` unless the SDA program itself fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExaminationUnit {
    /// `item`, `event`, `hole`, `structural-frame`, …
    pub unit_kind: String,
    /// Core or namespaced status tag.
    pub status: String,
    /// Store id hex, when recovered.
    pub store_id: Option<String>,
    /// Segment id hex, when recovered.
    pub segment_id: Option<String>,
    /// Item id hex, when recovered.
    pub item_id: Option<String>,
    /// Event id hex, when recovered.
    pub event_id: Option<String>,
    /// Event kind name (`put`, `delete`), when recovered.
    pub event_kind: Option<String>,
    /// Physical location.
    pub physical: PhysicalLocation,
    /// Integrity evidence.
    pub integrity: IntegrityEvidence,
    /// Projected envelope map entries.
    pub envelope: Vec<EnvelopeEntry>,
    /// Payload availability and optional bytes.
    pub payload: PayloadInfo,
    /// Nested hole references (Stage 5 usually empty; hole is its own unit).
    pub holes: Vec<ExaminationUnit>,
    /// Provenance sequence.
    pub provenance: Vec<ProvenanceEntry>,
    /// Uncertainty tags (set semantics; stored ordered unique).
    pub uncertainty: Vec<String>,
}

impl ExaminationUnit {
    /// Convert to the normative SDA `Prod` value (host → SDA boundary).
    pub fn to_sda_value(&self) -> Value {
        Value::Prod(vec![
            ("unit_kind".into(), Value::Str(self.unit_kind.clone())),
            ("status".into(), Value::Str(self.status.clone())),
            ("store_id".into(), opt_str(&self.store_id)),
            ("segment_id".into(), opt_str(&self.segment_id)),
            ("item_id".into(), opt_str(&self.item_id)),
            ("event_id".into(), opt_str(&self.event_id)),
            ("event_kind".into(), opt_str(&self.event_kind)),
            ("physical".into(), physical_to_value(&self.physical)),
            ("integrity".into(), integrity_to_value(&self.integrity)),
            ("envelope".into(), envelope_to_value(&self.envelope)),
            ("payload".into(), payload_to_value(&self.payload)),
            (
                "holes".into(),
                Value::Seq(self.holes.iter().map(ExaminationUnit::to_sda_value).collect()),
            ),
            (
                "provenance".into(),
                Value::Seq(
                    self.provenance
                        .iter()
                        .map(provenance_to_value)
                        .collect(),
                ),
            ),
            (
                "uncertainty".into(),
                Value::Set(self.uncertainty.iter().map(|s| Value::Str(s.clone())).collect()),
            ),
        ])
    }

    /// JSON view of the SDA product (via `sda_core::to_json`).
    pub fn to_json(&self) -> serde_json::Value {
        sda_core::to_json(self.to_sda_value())
    }
}

fn opt_str(v: &Option<String>) -> Value {
    match v {
        Some(s) => Value::Some_(Box::new(Value::Str(s.clone()))),
        None => Value::None_,
    }
}

fn opt_num_u64(v: Option<u64>) -> Value {
    match v {
        Some(n) => Value::Some_(Box::new(num_u64(n))),
        None => Value::None_,
    }
}

fn num_u64(n: u64) -> Value {
    Value::Num(
        ExactNum::parse_literal(&n.to_string()).expect("u64 is a valid ExactNum literal"),
    )
}

fn num_i64(n: i64) -> Value {
    Value::Num(
        ExactNum::parse_literal(&n.to_string()).expect("i64 is a valid ExactNum literal"),
    )
}

fn physical_to_value(p: &PhysicalLocation) -> Value {
    Value::Prod(vec![
        ("source".into(), Value::Str(p.source.clone())),
        ("offset".into(), opt_num_u64(p.offset)),
        ("encoded_length".into(), opt_num_u64(p.encoded_length)),
        (
            "wire_major".into(),
            match p.wire_major {
                Some(m) => Value::Some_(Box::new(num_u64(u64::from(m)))),
                None => Value::None_,
            },
        ),
        (
            "wire_minor".into(),
            match p.wire_minor {
                Some(m) => Value::Some_(Box::new(num_u64(u64::from(m)))),
                None => Value::None_,
            },
        ),
    ])
}

fn integrity_to_value(i: &IntegrityEvidence) -> Value {
    Value::Prod(vec![
        ("framing".into(), Value::Str(i.framing.clone())),
        ("structural".into(), Value::Str(i.structural.clone())),
        ("content".into(), Value::Str(i.content.clone())),
        ("authentication".into(), Value::Str(i.authentication.clone())),
    ])
}

fn envelope_to_value(entries: &[EnvelopeEntry]) -> Value {
    Value::Map(
        entries
            .iter()
            .map(|e| {
                let v = match &e.value {
                    EnvelopeValue::Str(s) => Value::Str(s.clone()),
                    EnvelopeValue::Bytes(b) => Value::Bytes(b.clone()),
                    EnvelopeValue::Num(n) => num_i64(*n),
                    EnvelopeValue::StrSet(items) => {
                        Value::Set(items.iter().map(|s| Value::Str(s.clone())).collect())
                    }
                };
                (e.key.clone(), v)
            })
            .collect(),
    )
}

fn payload_to_value(p: &PayloadInfo) -> Value {
    let value = match &p.value {
        Some(bytes) => Value::Some_(Box::new(Value::Bytes(bytes.clone()))),
        None => Value::None_,
    };
    let extents = Value::Seq(
        p.extents
            .iter()
            .map(|e| {
                Value::Prod(vec![
                    ("logical_start".into(), num_u64(e.logical_start)),
                    ("logical_length".into(), num_u64(e.logical_length)),
                    ("status".into(), Value::Str(e.status.clone())),
                    ("chunk_id".into(), opt_str(&e.chunk_id)),
                    (
                        "value".into(),
                        match &e.value {
                            Some(b) => Value::Some_(Box::new(Value::Bytes(b.clone()))),
                            None => Value::None_,
                        },
                    ),
                ])
            })
            .collect(),
    );
    Value::Prod(vec![
        ("availability".into(), Value::Str(p.availability.clone())),
        ("representation".into(), Value::Str(p.representation.clone())),
        ("media_type".into(), opt_str(&p.media_type)),
        ("logical_length".into(), opt_num_u64(p.logical_length)),
        ("value".into(), value),
        ("extents".into(), extents),
    ])
}

fn provenance_to_value(p: &ProvenanceEntry) -> Value {
    Value::Prod(vec![
        ("action".into(), Value::Str(p.action.clone())),
        ("source_id".into(), opt_str(&p.source_id)),
        ("tool".into(), Value::Str(p.tool.clone())),
        ("tool_version".into(), Value::Str(p.tool_version.clone())),
        ("evidence".into(), Value::Map(vec![])),
    ])
}

/// Compare two units under SDA_PROFILE §12 default order.
pub fn cmp_units(a: &ExaminationUnit, b: &ExaminationUnit) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    opt_str_ord(&a.segment_id, &b.segment_id)
        .then_with(|| a.physical.source.cmp(&b.physical.source))
        .then_with(|| opt_u64_ord(a.physical.offset, b.physical.offset))
        .then_with(|| opt_str_ord(&a.event_id, &b.event_id))
        .then_with(|| {
            // Final tie-breaker: deterministic JSON of the SDA product.
            let ja = a.to_json().to_string();
            let jb = b.to_json().to_string();
            ja.cmp(&jb)
        })
        .then(Ordering::Equal)
}

fn opt_str_ord(a: &Option<String>, b: &Option<String>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

fn opt_u64_ord(a: Option<u64>, b: Option<u64>) -> std::cmp::Ordering {
    match (a, b) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_sda_value_has_fixed_fields() {
        let u = ExaminationUnit {
            unit_kind: "item".into(),
            status: "verified-complete".into(),
            store_id: Some("aa".into()),
            segment_id: Some("bb".into()),
            item_id: Some("cc".into()),
            event_id: Some("dd".into()),
            event_kind: Some("put".into()),
            physical: PhysicalLocation {
                source: "segments/x.dingo".into(),
                offset: Some(0),
                encoded_length: Some(100),
                wire_major: Some(1),
                wire_minor: Some(0),
            },
            integrity: IntegrityEvidence::verified_no_auth(),
            envelope: vec![],
            payload: PayloadInfo::complete_bytes(b"hi", Some("application/octet-stream")),
            holes: vec![],
            provenance: vec![],
            uncertainty: vec![],
        };
        let v = u.to_sda_value();
        match v {
            Value::Prod(fields) => {
                let keys: Vec<_> = fields.iter().map(|(k, _)| k.as_str()).collect();
                assert_eq!(
                    keys,
                    [
                        "unit_kind",
                        "status",
                        "store_id",
                        "segment_id",
                        "item_id",
                        "event_id",
                        "event_kind",
                        "physical",
                        "integrity",
                        "envelope",
                        "payload",
                        "holes",
                        "provenance",
                        "uncertainty",
                    ]
                );
            }
            other => panic!("expected Prod, got {other:?}"),
        }
    }
}
