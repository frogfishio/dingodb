//! Crate-private physical store alias (`HEAP_SPEC` §30.5 / HP-003).

/// Physical store implementation. Not part of the qualified heap public surface;
/// use [`crate::heap`] façades from capability-gated callers.
pub(crate) type PhysicalStore = crate::store::Store;

#[cfg(test)]
mod tests {
    #[test]
    fn physical_alias_exists() {
        let _ = std::any::type_name::<super::PhysicalStore>();
    }
}
