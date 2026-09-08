//! Bevy components registered by the harness for custom editor schemas.
//!
//! The editor's component schema registry lets users author schemas
//! like `game.PlayerController` with typed fields. When the harness
//! consumes the editor's JSON, it needs Bevy components to attach
//! the deserialised values to entities. These components match the
//! schemas declared in `examples/platformer-minimal/schemas/`.
//!
//! The harness does NOT implement gameplay logic for these components
//! — it only proves the JSON round-trips. A future Bevy-native game
//! would define its own components; this module is a witness, not a
//! gameplay implementation.

use bevy::prelude::Component;

/// Bevy mirror of `game.PlayerController` (see
/// `examples/platformer-minimal/schemas/game.PlayerController.schema.json`).
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct PlayerController {
    pub speed: f32,
    pub jump_force: f32,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            speed: 200.0,
            jump_force: 400.0,
        }
    }
}

/// Bevy mirror of `game.EnemyPatrol` (see
/// `examples/platformer-minimal/schemas/game.EnemyPatrol.schema.json`).
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct EnemyPatrol {
    pub speed: f32,
    pub patrol_range: f32,
}

impl Default for EnemyPatrol {
    fn default() -> Self {
        Self {
            speed: 60.0,
            patrol_range: 150.0,
        }
    }
}

/// Per-entity patrol direction. Holds `+1.0` (right) or `-1.0` (left).
///
/// The harness attaches this to entities named `"Enemy"` via a
/// loader-level heuristic (mirrors the `Pickup` marker pattern from
/// G1-step3). The patrol system flips the value at the
/// `±patrol_range` boundary. Future cycles may declare a
/// `game.EnemyPatrol` schema field for the initial direction.
#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct EnemyDirection(pub f32);

impl Default for EnemyDirection {
    fn default() -> Self {
        Self(1.0) // start moving right
    }
}

/// Bevy mirror of `editor.Visible` (`{ "visible": bool }`).
///
/// The harness uses a tuple-struct rather than a single bool field
/// so the `Component` derive does not collide with primitive types.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub struct Visible(pub bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

/// Captures the `editor.Sprite2D.asset` path so the harness can prove
/// the field round-trips. The harness does not load image data
/// (Bevy's `Sprite.image: Handle<Image>` would require an asset
/// server + image loader, which is out of scope for this witness).
#[derive(Debug, Component, Clone, PartialEq, Eq, Default)]
pub struct EditorSpriteAsset(pub String);

/// Marker component for entities representing a collectible pickup.
///
/// The sample uses a heuristic (`editor.Name == "Pickup"`) to attach
/// this marker; future versions may declare a `game.Pickup` schema
/// with fields (e.g. score value, respawn timer).
///
/// See `crates/examples-bevy-harness/src/collision.rs` for the
/// runtime system that despawns pickups on player overlap.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pickup;
