# Preview Anchor Sync — Spec (Behavior Contract)

## Cycle: `preview-anchor-sync`
**Status:** draft → review
**Source:** ADR-0004 (corrected); dynamic-scene-export cycle follow-up.

This spec defines the observable behavior of the editor's preview world
(`spawn_entity()`) when handling `editor.Sprite2D.values.anchor`. Each scenario is
Given/When/Then and must be covered by at least one unit test or Playwright test.

---

## Scenario 1: Anchor "Center" maps to Anchor::CENTER

**Given** an entity with Sprite2D `{ asset: "x.png", anchor: "Center" }`
**When** `spawn_entity` is called
**Then** the spawned Bevy entity has both `Sprite` and `bevy::sprite::Anchor` Components
**And** the Anchor Component is `Anchor(Vec2::ZERO)` (= `Anchor::CENTER`)

---

## Scenario 2: All 9 anchors map correctly

**Given** 9 entities each with Sprite2D having a different anchor string from {Center, TopLeft,
TopRight, BottomLeft, BottomRight, TopCenter, BottomCenter, CenterLeft, CenterRight}
**When** `spawn_entity` is called for each
**Then** each spawned entity has the correct `Anchor` Component value matching the editor
string

| Editor | Bevy Anchor.0 |
|---|---|
| `"Center"` | `Vec2::ZERO` |
| `"TopLeft"` | `Vec2::new(-0.5, 0.5)` |
| `"TopCenter"` | `Vec2::new(0.0, 0.5)` |
| `"TopRight"` | `Vec2::new(0.5, 0.5)` |
| `"CenterLeft"` | `Vec2::new(-0.5, 0.0)` |
| `"CenterRight"` | `Vec2::new(0.5, 0.0)` |
| `"BottomLeft"` | `Vec2::new(-0.5, -0.5)` |
| `"BottomCenter"` | `Vec2::new(0.0, -0.5)` |
| `"BottomRight"` | `Vec2::new(0.5, -0.5)` |

---

## Scenario 3: Missing anchor field defaults silently to Center

**Given** an entity with Sprite2D `{ asset: "x.png" }` (no `anchor` field)
**When** `spawn_entity` is called
**Then** the spawned entity has `Anchor(Vec2::ZERO)` (= `Anchor::CENTER`)
**And** no warning is emitted (silent default — matches Bevy's auto-required default)

---

## Scenario 4: Invalid anchor string defaults to Center + warn

**Given** an entity with Sprite2D `{ asset: "x.png", anchor: "NotAnAnchor" }`
**When** `spawn_entity` is called
**Then** the spawned entity has `Anchor(Vec2::ZERO)` (= `Anchor::CENTER`)
**And** a Bevy `warn!` is logged with a message mentioning the invalid anchor

---

## Scenario 5: Anchor inserted after Sprite (overrides #[require] default)

**Given** an entity with Sprite2D `{ anchor: "TopLeft" }`
**When** `spawn_entity` is called
**Then** the final entity has `Anchor(Vec2::new(-0.5, 0.5))` (NOT `Anchor::default()`)
**And** the insertion order in code is `Sprite` first, then `Anchor` second

---

## Scenario 6: Helper `anchor_str_to_bevy_anchor` returns correct Bevy Anchor

**Given** the helper function `dynamic_scene::anchor_str_to_bevy_anchor("TopLeft")`
**When** called
**Then** returns `bevy::sprite::Anchor(Vec2::new(-0.5, 0.5))`

**And** `anchor_str_to_bevy_anchor("Invalid")` returns `bevy::sprite::Anchor::default()` (=
`Anchor::CENTER`)

---

## Scenario 7: Sprite2D without asset still gets Anchor Component

**Given** an entity with Sprite2D `{ asset: "", anchor: "TopRight" }`
**When** `spawn_entity` is called
**Then** the spawned entity has an `Anchor` Component (Anchor inserted regardless of asset
presence — `Anchor` is conceptually independent of the texture)

---

## Scenario 8: Rebuild is idempotent — changing anchor re-renders correctly

**Given** the preview world has an entity with anchor "Center"
**When** the SceneDocument is mutated so the entity's anchor becomes "BottomRight"
**Then** after the next preview rebuild, the entity's `Anchor` Component is
`Anchor(Vec2::new(0.5, -0.5))` (update via `cmd.insert(Anchor)`)

---

## Scenario 9: Non-sprite entities never get an Anchor Component

**Given** an entity with only `editor.Name` and `editor.Transform2D` (no `editor.Sprite2D`)
**When** `spawn_entity` is called
**Then** the spawned entity has no `Sprite` component
**And** the spawned entity has no `Anchor` Component (Anchor is only inserted on sprite
entities)

---

## Scenario 10: Visual regression — default spike sprite position unchanged

**Given** the default scene shipped with the editor ("spike-sprite-01" with default Center
anchor)
**When** the preview world rebuilds
**Then** the sprite renders at the same pixel position as before this cycle (no visual change
for existing scenes with default anchor)

---

## Mapping Reference Table

| Editor anchor string | Bevy `Anchor` constant | `Anchor.0` Vec2 |
|---|---|---|
| `"Center"` | `Anchor::CENTER` | `Vec2::ZERO` |
| `"TopLeft"` | `Anchor::TOP_LEFT` | `Vec2::new(-0.5, 0.5)` |
| `"TopCenter"` | `Anchor::TOP_CENTER` | `Vec2::new(0.0, 0.5)` |
| `"TopRight"` | `Anchor::TOP_RIGHT` | `Vec2::new(0.5, 0.5)` |
| `"CenterLeft"` | `Anchor::CENTER_LEFT` | `Vec2::new(-0.5, 0.0)` |
| `"CenterRight"` | `Anchor::CENTER_RIGHT` | `Vec2::new(0.5, 0.0)` |
| `"BottomLeft"` | `Anchor::BOTTOM_LEFT` | `Vec2::new(-0.5, -0.5)` |
| `"BottomCenter"` | `Anchor::BOTTOM_CENTER` | `Vec2::new(0.0, -0.5)` |
| `"BottomRight"` | `Anchor::BOTTOM_RIGHT` | `Vec2::new(0.5, -0.5)` |

---

## Acceptance Criteria (rolled up)

- AC-1: All 10 scenarios pass as unit tests or Playwright tests.
- AC-2: All 132 existing unit tests continue to pass.
- AC-3: ADR-0004 has a "Superseded by facts" correction note.
- AC-4: `bevy::sprite::Anchor` is reachable via `use bevy::sprite::Anchor;` in
  `crates/editor-core/src/lib.rs`.
- AC-5: The export format (PascalCase strings) is unchanged.
