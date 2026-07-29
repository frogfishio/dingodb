//! SDK surface claim honesty (CPR-001 / HP-010).
//!
//! The default package build still enables [`crate`]'s `legacy-flat-sdk` feature
//! so Stages 3–9 keep working. That surface is **not** Gate H6 qualified.
//! Builds with `--no-default-features` expose only heap-bound entry points.

use dingo_heap::{claim_language, may_advertise_qualified};

/// Cargo feature name that enables the flat collection / token-remote surface.
pub const LEGACY_FLAT_SDK_FEATURE: &str = "legacy-flat-sdk";

/// Whether this build includes the legacy flat `Dingo::open` / `collection(name)` path.
///
/// When `true` (package default), CPR-001 is still open for product claim honesty:
/// the default entry is not heap-bound. When `false` (`--no-default-features`),
/// only deployment / `HeapCap` / `connect_heap` entry points are available.
pub const fn legacy_flat_sdk_enabled() -> bool {
    cfg!(feature = "legacy-flat-sdk")
}

/// Whether this build's embedded default is the heap-only profile (no flat collection API).
pub const fn heap_only_embedded_profile() -> bool {
    !legacy_flat_sdk_enabled()
}

/// Machine-readable label for the flat collection surface.
///
/// Never implies `dingo-heap-v1` qualification.
pub const FLAT_COLLECTION_SURFACE_LABEL: &str =
    "legacy-flat-sdk; not dingo-heap-v1 qualified; CPR-001";

/// Human claim language for the flat collection surface.
pub fn flat_collection_claim_language() -> &'static str {
    "DingoDB provides a legacy flat collection API for compatibility; strong \
     access-isolation qualification applies only to heap-bound APIs under \
     dingo-heap-v1 after HP-010 gates accept."
}

/// Combined product claim check for operators (always Level 1 until matrix flips).
pub fn product_may_advertise_qualified_heap() -> bool {
    may_advertise_qualified()
}

/// Canonical Level-1 language (delegates to `dingo-heap`).
pub fn product_claim_language() -> &'static str {
    claim_language()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_honesty_never_qualified_while_matrix_false() {
        assert!(!product_may_advertise_qualified_heap());
        assert!(!product_claim_language().is_empty());
        assert!(!flat_collection_claim_language().is_empty());
        // Default package build enables legacy-flat-sdk (CPR-001 residual).
        // Heap-only CI uses --no-default-features (see check_heap_architecture.sh).
        if legacy_flat_sdk_enabled() {
            assert!(!heap_only_embedded_profile());
            assert!(FLAT_COLLECTION_SURFACE_LABEL.contains("CPR-001"));
        } else {
            assert!(heap_only_embedded_profile());
        }
    }
}
