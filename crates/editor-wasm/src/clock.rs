//! Clock using `js_sys::Date.now()` — production WASM clock.

use editor_model::time::{Clock, Timestamp};

/// Clock using `js_sys::Date.now()` — production WASM clock.
#[derive(Debug, Default)]
pub struct SysClock;

impl SysClock {
    /// Create a new `SysClock`.
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SysClock {
    fn now(&self) -> Timestamp {
        Timestamp(js_sys::Date::now() as u64)
    }
}
