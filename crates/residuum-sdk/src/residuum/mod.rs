//! Database handle entry points (`DX_SPEC` §4 / `HEAP_SPEC` §7.1 / CPR-001).
//!
//! - **Heap-bound (always):** [`Dingo::open_deployment`], [`Dingo::connect_heap`]
//! - **Legacy flat (feature `legacy-flat-sdk`, default on):** [`Dingo::open`],
//!   [`Dingo::collection`], token [`Dingo::connect`], cluster open

#[cfg(feature = "legacy-flat-sdk")]
mod flat;

#[cfg(feature = "legacy-flat-sdk")]
pub use flat::Dingo;
#[cfg(feature = "legacy-flat-sdk")]
pub(crate) use flat::Backend;

#[cfg(not(feature = "legacy-flat-sdk"))]
mod heap_only {
    use crate::error::Error;
    use crate::heap::DingoDeployment;
    use std::path::Path;

    /// Namespace for heap-bound entry points when `legacy-flat-sdk` is disabled.
    pub struct Dingo;

    impl Dingo {
        /// Open a store directory as a **deployment host** (heap-bound).
        pub fn open_deployment(path: impl AsRef<Path>) -> Result<DingoDeployment, Error> {
            DingoDeployment::open(path)
        }

        /// Create a new store directory as a deployment host.
        pub fn create_deployment(path: impl AsRef<Path>) -> Result<DingoDeployment, Error> {
            DingoDeployment::create(path)
        }

        /// Connect a **qualified** remote heap via HeapKey.
        pub fn connect_heap(
            url: impl AsRef<str>,
            options: crate::RemoteHeapOptions,
        ) -> Result<crate::RemoteHeap, Error> {
            crate::remote_heap::connect_heap(url, options)
        }
    }
}

#[cfg(not(feature = "legacy-flat-sdk"))]
pub use heap_only::Dingo;
