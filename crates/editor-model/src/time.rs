//! Pure time abstraction. The production clock is provided by `editor_core::time::JsSysClock`;
//! tests use `editor_model::time::FakeClock`.

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

/// Returns the current Unix time in milliseconds (v0.91 PR2: moved from
/// editor-core for use by the new scene_asset_catalog module).
pub fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
