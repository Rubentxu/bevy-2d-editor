//! Bevy 0.19 anchor helper for the editor's preview world.
//!
//! Maps editor anchor strings (PascalCase: "Center", "TopLeft", ...) to Bevy 0.19's
//! `bevy::sprite::Anchor` Component value.
//!
//! This module is Bevy-dependent and NOT included in `scene-doc-verify` (the
//! bevy-independent test harness that bypasses `libudev-sys` on Fedora).

use bevy::math::Vec2;
use bevy::sprite::Anchor;

/// Map our `Anchor` enum string to Bevy 0.19's `bevy::sprite::Anchor` Component value.
///
/// In Bevy 0.19, `Anchor` is `pub struct Anchor(pub Vec2)` with 9 named constants.
/// It is a separate Component auto-required by `Sprite` via `#[require(...)]`.
/// Returns `Anchor::default()` (= `Anchor::CENTER`) for unknown or empty strings.
pub fn anchor_str_to_bevy_anchor(s: &str) -> Anchor {
    let (x, y) = crate::dynamic_scene::anchor_str_to_normalized_offset(s);
    Anchor(Vec2::new(x, y))
}

/// Returns true if the string is one of the 9 known anchor names.
///
/// Re-exported here for convenience; the canonical implementation is in
/// `crate::dynamic_scene::is_known_anchor_str`.
pub fn is_known_anchor_str(s: &str) -> bool {
    crate::dynamic_scene::is_known_anchor_str(s)
}
