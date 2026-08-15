//! Application adapters — concrete implementations of ports.

pub mod in_memory;
pub mod opfs;

pub use in_memory::InMemoryProjectStore;
pub use opfs::OpfsProjectStore;
