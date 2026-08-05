//! Query VM instruction set (RQL-VM0 vocabulary) + profile stamps.
//!
//! Dispatch loop lives in [`super::vm_exec`] (**RQL-VM1**).
//! Profile: **`residiuum-query-vm-v1`**
//! Normative: [QUERY_VM_V1.md](../../../../../doc/todo/rql/QUERY_VM_V1.md)
//!
//! This module freezes the opcode vocabulary. Execution is [`super::vm_exec`].
//! Core opcode bodies still call `execute_plan` until **RQL-VM2**.
//! Decision 0 remains OPEN; RQL-C1 must not be accepted.

/// Query VM profile id (instruction set freeze).
pub const VM_PROFILE: &str = "residiuum-query-vm-v1";

/// Instruction-set version (bump only with QUERY_VM_V1 amendment).
pub const VM_VERSION: u8 = 1;

/// Opcode byte values for the Query VM (RQL-VM0).
///
/// Reserved ranges:
/// - `0x00` — invalid / padding
/// - `0x01..=0x0F` — bind / frame
/// - `0x10..=0x1F` — host open (scan / index)
/// - `0x20..=0x2F` — filter
/// - `0x30..=0x3F` — project
/// - `0x40..=0x4F` — order
/// - `0x50..=0x5F` — page / coverage / cursor
/// - `0x60..=0x6F` — enrich / within / brace project
/// - `0xF0..=0xFE` — reserved for future control
/// - `0xFF` — halt
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpCode {
    /// Bind the base collection by immutable [`residiuum_heap::CollectionId`].
    BindCollection = 0x01,
    /// Open a deterministic key stream on the bound collection (host `list_keys`).
    Scan = 0x10,
    /// Probe equality-index candidate keys (host `lookup_index_keys`); may fall back to Scan.
    IndexEq = 0x11,
    /// Filter current working set with a kernel/SDA predicate (const-pool ref).
    Filter = 0x20,
    /// Core path-project (flat path list).
    ProjectPaths = 0x30,
    /// Order / sort-tuple compare (order-term list).
    Order = 0x40,
    /// Page clamp + coverage policy + optional cursor mint/decode.
    Page = 0x50,
    /// Root enrich attach (foreign collection id + match + cardinality + output).
    Enrich = 0x60,
    /// Within: map over bag at path; nested ops follow until matching WithinEnd.
    Within = 0x61,
    /// End nested within body.
    WithinEnd = 0x62,
    /// Post-attach row filter (same kernel as Filter; attach pipeline position).
    FilterAttach = 0x63,
    /// Brace `project { … }` (nested product / rename / bag map).
    ProjectBrace = 0x64,
    /// End of program — yield current page.
    Halt = 0xFF,
}

impl OpCode {
    /// All defined opcodes (stable order for tests / docs).
    pub const ALL: &'static [OpCode] = &[
        OpCode::BindCollection,
        OpCode::Scan,
        OpCode::IndexEq,
        OpCode::Filter,
        OpCode::ProjectPaths,
        OpCode::Order,
        OpCode::Page,
        OpCode::Enrich,
        OpCode::Within,
        OpCode::WithinEnd,
        OpCode::FilterAttach,
        OpCode::ProjectBrace,
        OpCode::Halt,
    ];

    /// Discriminant byte.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Decode a known opcode byte; unknown bytes are rejected.
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::BindCollection),
            0x10 => Some(Self::Scan),
            0x11 => Some(Self::IndexEq),
            0x20 => Some(Self::Filter),
            0x30 => Some(Self::ProjectPaths),
            0x40 => Some(Self::Order),
            0x50 => Some(Self::Page),
            0x60 => Some(Self::Enrich),
            0x61 => Some(Self::Within),
            0x62 => Some(Self::WithinEnd),
            0x63 => Some(Self::FilterAttach),
            0x64 => Some(Self::ProjectBrace),
            0xFF => Some(Self::Halt),
            _ => None,
        }
    }

    /// Short name for diagnostics.
    pub const fn name(self) -> &'static str {
        match self {
            Self::BindCollection => "BindCollection",
            Self::Scan => "Scan",
            Self::IndexEq => "IndexEq",
            Self::Filter => "Filter",
            Self::ProjectPaths => "ProjectPaths",
            Self::Order => "Order",
            Self::Page => "Page",
            Self::Enrich => "Enrich",
            Self::Within => "Within",
            Self::WithinEnd => "WithinEnd",
            Self::FilterAttach => "FilterAttach",
            Self::ProjectBrace => "ProjectBrace",
            Self::Halt => "Halt",
        }
    }
}

/// One instruction: opcode + optional immediate (const-pool / section offsets).
///
/// Immediate layout is normative in QUERY_VM_V1.md; VM0 freezes the opcode
/// vocabulary only. Encoding of immediates lands with RQL-VM1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    /// Opcode.
    pub op: OpCode,
    /// Immediate payload bytes (empty for Halt / WithinEnd).
    pub imm: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn vm_profile_constant() {
        assert_eq!(VM_PROFILE, "residiuum-query-vm-v1");
        assert_eq!(VM_VERSION, 1);
    }

    #[test]
    fn opcodes_are_unique_and_roundtrip() {
        let mut seen = BTreeSet::new();
        for op in OpCode::ALL {
            assert!(seen.insert(op.as_u8()), "duplicate {:?}", op);
            assert_eq!(OpCode::from_u8(op.as_u8()), Some(*op));
        }
        assert!(OpCode::from_u8(0x00).is_none());
        assert!(OpCode::from_u8(0x70).is_none());
    }

    #[test]
    fn core_and_full_coverage_named() {
        // Principal required coverage: scan, filter, project, order, page, enrich, within.
        let names: BTreeSet<_> = OpCode::ALL.iter().map(|o| o.name()).collect();
        for need in [
            "Scan",
            "Filter",
            "ProjectPaths",
            "Order",
            "Page",
            "Enrich",
            "Within",
        ] {
            assert!(names.contains(need), "missing {need}");
        }
    }
}
