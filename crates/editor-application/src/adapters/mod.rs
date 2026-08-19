//! Application adapters — concrete implementations of ports.

pub mod in_memory;
#[cfg(target_arch = "wasm32")]
pub mod opfs;

pub use in_memory::InMemoryProjectStore;
#[cfg(target_arch = "wasm32")]
pub use opfs::OpfsProjectStore;
