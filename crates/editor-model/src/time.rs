//! Pure time abstraction. The production clock is provided by `editor_core::time::JsSysClock`;
//! tests use `editor_model::time::FakeClock`.

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

/// Milliseconds since the unix epoch (1970-01-01T00:00:00Z).
pub type Timestamp = u64;

pub trait Clock: Debug + Send + Sync {
    fn now(&self) -> Timestamp;
}

/// A clock for use in tests only. Not available in WASM production builds.
#[derive(Debug, Default)]
pub struct FakeClock {
    current_ms: AtomicU64,
}

impl FakeClock {
    pub fn new() -> Self {
        Self {
            current_ms: AtomicU64::new(0),
        }
    }

    pub fn set(&self, t: Timestamp) {
        self.current_ms.store(t, Ordering::SeqCst);
    }

    pub fn advance(&self, delta_ms: u64) {
        self.current_ms.fetch_add(delta_ms, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        self.current_ms.load(Ordering::SeqCst)
    }
}
