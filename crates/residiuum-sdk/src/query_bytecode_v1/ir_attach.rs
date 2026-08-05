//! Full-language attach IR phase label (RQL-IR4).
//!
//! Profile: **`residiuum-query-ir-attach-v1`**
//! Normative: [QUERY_IR_ATTACH_V1.md](../../../../../doc/todo/rql/QUERY_IR_ATTACH_V1.md)
//!
//! Product Full execute dispatches enrich/within/filter as flat Query VM opcodes
//! ([`super::vm_exec::run_vm`]). The old Rust-loop attach orchestrator
//! (`run_attach_pipeline`) is **deleted** (one executor). Decision 0 remains OPEN;
//! RQL-C1 must not be accepted.

/// IR profile id for full-language attach (honesty stamp).
pub const ATTACH_IR_PROFILE: &str = "residiuum-query-ir-attach-v1";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_ir_profile_constant() {
        assert_eq!(ATTACH_IR_PROFILE, "residiuum-query-ir-attach-v1");
    }
}
