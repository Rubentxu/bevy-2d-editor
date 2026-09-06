//! Pure time abstraction shared by the editor model.
//!
//! This crate is **pure** — it has zero WASM / Bevy dependencies (enforced by
//! `tools/archcheck` rule B8, ADR-0030). Callers that need a production clock
//! must inject a [`Clock`] trait object from a crate that is allowed to use
//! the WASM bindings. The canonical production impl lives in the
//! `editor-bevy` crate (`time::JsSysClock`); tests use [`FakeClock`].
//!
//! ```ignore
//! // In editor-bevy (wasm-allowed crate):
//! use editor_bevy as eb;
//! use editor_model::time::Clock;
//! let clock: Box<dyn Clock> = Box::new(eb::time::JsSysClock::new());
//! clock.now(); // date now() on wasm32, SystemTime::now() on native
//! ```

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

/// Milliseconds since the unix epoch (1970-01-01T00:00:00Z).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Unwrap the inner u64 value.
    pub fn into_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for Timestamp {
    fn from(v: u64) -> Self {
        Timestamp(v)
    }
}

impl core::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(Timestamp)
    }
}

/// abstraction for reading the current wall-clock time in milliseconds since epoch.
pub trait Clock: Debug + Send + Sync {
    /// Returns the current timestamp in milliseconds since the Unix epoch.
    fn now(&self) -> Timestamp;
}

/// A clock for use in tests only. Not available in WASM production builds.
#[derive(Debug, Default)]
pub struct FakeClock {
    current_ms: AtomicU64,
}

impl FakeClock {
    /// Construct a new FakeClock with time starting at 0.
    pub fn new() -> Self {
        Self {
            current_ms: AtomicU64::new(0),
        }
    }

    /// Set the clock to a fixed timestamp (milliseconds since epoch).
    pub fn set(&self, t: impl Into<Timestamp>) {
        self.current_ms.store(t.into().0, Ordering::SeqCst);
    }

    /// Advance the clock by `delta_ms` milliseconds.
    pub fn advance(&self, delta_ms: u64) {
        self.current_ms.fetch_add(delta_ms, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        Timestamp(self.current_ms.load(Ordering::SeqCst))
    }
}

// Note: `now_millis` / `now_nanos` free functions were removed in recovery-1
// to satisfy archcheck rule B8 (ADR-0030). Production callers must inject a
// [`Clock`] impl — typically the `JsSysClock` from the `editor-bevy` crate.
// Tests use [`FakeClock`]. The pre-removal wasm branch called the WASM date
// helper directly inside this pure crate; that violated the editor-model
// purity contract and is now correctly delegated to a wasm-allowed crate.
