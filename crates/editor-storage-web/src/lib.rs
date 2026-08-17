//! OPFS storage adapter for browser WASM environments.
//!
//! ## Architecture
//!
//! - [`opfs_core::OpfsCore`] — pure in-memory mirror + pending-op queue. No JS deps,
//!   compiled on all targets.
//! - [`raw_store_bridge::RawStoreBridge`] — async I/O abstraction trait. On wasm32
//!   backed by the `window.opfs_*` JS bridge via `js_sys::Promise`; on native backed
//!   by [`raw_store_bridge::MemoryBridge`] for tests.
//! - [`memory_bridge::InMemoryProjectStore`] — in-memory implementation for tests and
//!   development.
//! - `wasm_bridge` module (`#[cfg(target_arch = "wasm32")]`) — wasm32-only bridge ops
//!   and `SysClock` implementations.
//! - [`opfs_core::OpfsProjectStore`] — public struct used at runtime. Holds
//!   `Arc<Mutex<OpfsCore>>` + `Arc<dyn RawStoreBridge>` + `Arc<dyn Clock>`.

pub mod memory_bridge;
pub mod opfs_core;
pub mod raw_store_bridge;

#[cfg(target_arch = "wasm32")]
pub mod wasm_bridge;

pub use memory_bridge::InMemoryProjectStore;
pub use opfs_core::OpfsProjectStore;
