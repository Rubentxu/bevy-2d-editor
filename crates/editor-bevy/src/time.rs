//! Wall-clock time helpers that work on both wasm32 and native.
//!
//! On `wasm32-unknown-unknown` with rustc >= 1.96.0,
//! `std::time::SystemTime::now()` traps because the new stdlib's
//! `Instant::now()` calls `unreachable!()` at
//! `library/std/src/sys/time/unsupported.rs:35:9`. We route through
//! `js_sys::Date::now()` instead, which calls `Date.now()` in the
//! browser — always available, never traps.
//!
//! Pattern mirrors the pre-existing wasm/native split on
//! `scene_asset_catalog::random_hex_8` (see
//! `crates/editor-core/src/scene_asset_catalog.rs:365`).
//!
//! Refs: `sddk/active/systemtime-wasm-panic/spec/time-helpers/spec.md`.

use editor_model::time::{Clock, Timestamp};

/// Production wall-clock using `js_sys::Date` on wasm32 and `std::time::SystemTime` on native.
#[derive(Debug, Default)]
pub struct JsSysClock;

impl JsSysClock {
    pub fn new() -> Self {
        Self
    }
}

impl Clock for JsSysClock {
    #[cfg(target_arch = "wasm32")]
    fn now(&self) -> Timestamp {
        Timestamp(js_sys::Date::now() as u64)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn now(&self) -> Timestamp {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| Timestamp(d.as_millis() as u64))
            .unwrap_or(Timestamp(0))
    }
}

/// Milliseconds since the UNIX epoch.
#[cfg(target_arch = "wasm32")]
pub fn now_millis() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Nanoseconds since the UNIX epoch.
///
/// On wasm32 the value is `Date::now() * 1e6` and has <= 1 ms precision.
/// All current callers use this for opaque unique-string formatting
/// (e.g. `scene-{nanos}`, `scratch-{nanos}`, `inst_{:x}`), so the
/// precision loss is semantically inert.
#[cfg(target_arch = "wasm32")]
pub fn now_nanos() -> u64 {
    (js_sys::Date::now() * 1e6) as u64
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_millis_is_monotonic_within_a_call() {
        let a = now_millis();
        let b = now_millis();
        assert!(b >= a);
    }

    #[test]
    fn now_nanos_is_monotonic_within_a_call() {
        let a = now_nanos();
        let b = now_nanos();
        assert!(b >= a);
    }

    #[test]
    fn now_millis_is_in_a_reasonable_range() {
        let now = now_millis();
        // 2026-01-01 -> ~1_767_000_000_000 ms; 2030-01-01 -> ~1_893_000_000_000.
        assert!(now > 1_700_000_000_000 && now < 1_900_000_000_000);
    }
}
