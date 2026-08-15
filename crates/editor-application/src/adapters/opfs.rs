//! OPFS-backed project store. WASM-only stub for v0.87 (full wiring in v0.88).

#[cfg(target_arch = "wasm32")]
#[allow(missing_docs)]
pub mod inner {
    //! WASM32 inner module — full OPFS wiring deferred to v0.88.
    use crate::ports::project_store::{ProjectStore, StoreEntry, StoreError};

    /// OPFS-backed project store.
    #[derive(Debug, Default)]
    pub struct OpfsProjectStore;

    impl OpfsProjectStore {
        /// Create a new OPFS-backed project store.
        pub fn new() -> Self {
            Self
        }
    }

    impl ProjectStore for OpfsProjectStore {
        fn list(&self, _: &str) -> Result<Vec<StoreEntry>, StoreError> {
            unimplemented!("OpfsProjectStore::list — full wiring in v0.88 per ADR-0048")
        }

        fn read(&self, _: &str) -> Result<Vec<u8>, StoreError> {
            unimplemented!("OpfsProjectStore::read — full wiring in v0.88 per ADR-0048")
        }

        fn write(&self, _: &str, _: &[u8], _: bool) -> Result<(), StoreError> {
            unimplemented!("OpfsProjectStore::write — full wiring in v0.88 per ADR-0048")
        }

        fn delete(&self, _: &str) -> Result<(), StoreError> {
            unimplemented!("OpfsProjectStore::delete — full wiring in v0.88 per ADR-0048")
        }

        fn exists(&self, _: &str) -> Result<bool, StoreError> {
            unimplemented!("OpfsProjectStore::exists — full wiring in v0.88 per ADR-0048")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(missing_docs)]
pub mod inner {
    //! Non-WASM stub implementation — panics on any method call.
    //!
    //! This module exists only to keep the type available on non-wasm32 targets.
    //! Actual OPFS operations require wasm32.

    /// Non-WASM stub that always panics if instantiated.
    #[derive(Debug, Default)]
    pub struct OpfsProjectStore;
}

pub use inner::OpfsProjectStore;
