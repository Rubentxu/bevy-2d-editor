# Preview Anchor Sync — Explore Report

## Cycle: `preview-anchor-sync`
**Branch:** `feat/preview-anchor-sync`
**Source:** ADR-0004 ("DynamicScene Export — Bevy Native Anchor")
**Drives by:** dynamic-scene-export/archive-report.md follow-up candidate #1

## Context Quality
**C2** — small, focused change. The export already produces the right `bevy.Sprite.anchor`
string (PascalCase). What's missing is making the editor's **preview world** (the Bevy canvas
where the editor renders scenes live) honor that anchor — today it ignores it entirely.

## Knowledge Coverage
- `crates/editor-core/src/lib.rs::spawn_entity()` (lines 395–471): the function that rebuilds
  the preview Bevy world from the SceneDocument. Currently:
  - Reads `editor.Name` → Bevy `Name`
  - Reads `editor.Transform2D` → Bevy `Transform` (translation/rotation/scale)
  - Reads `editor.Sprite2D` → Bevy `Sprite` (color, custom_size=100, NO anchor)
  - Skips editorial components
  - Does NOT read `anchor` field
- Bevy 0.19 `bevy_sprite::Anchor` API (verified by reading `~/.cargo/registry/.../bevy_sprite-0.19.0/src/sprite.rs`):
  - `pub struct Anchor(pub Vec2)` — NOT an enum.
  - Named constants: `CENTER` (`Vec2::ZERO`), `TOP_LEFT` (`-0.5, 0.5`), `TOP_CENTER`
    (`0.0, 0.5`), `TOP_RIGHT` (`0.5, 0.5`), `CENTER_LEFT` (`-0.5, 0.0`), `CENTER_RIGHT`
    (`0.5, 0.0`), `BOTTOM_LEFT` (`-0.5, -0.5`), `BOTTOM_CENTER` (`0.0, -0.5`), `BOTTOM_RIGHT`
    (`0.5, -0.5`).
  - `Anchor` is a SEPARATE Component, not a Sprite field. Sprite has
    `#[require(Transform, Visibility, VisibilityClass, Anchor)]` — when a Sprite is inserted
    into an entity, Bevy auto-inserts `Anchor::default()` (= `Anchor::CENTER`) if missing.
- `crates/editor-core/src/dynamic_scene.rs::anchor_str_to_bevy()`: the existing mapping
  function (PascalCase string → Bevy anchor string). Returns the SAME string we want to map
  to a `bevy::sprite::Anchor` constant.
- Hito 0 §9.5 (line 357): "`editor.Sprite2D.values.anchor` | Computed `Transform` offset" —
  ADR-0004 supersedes with native Bevy mapping. This cycle makes the preview honor that.

## Taxonomy
| Axis | Classification |
|---|---|
| Domain | Bug fix / preview world fidelity |
| Risk | Low — affects only the editor's preview rendering; no public API change |
| Reversibility | Trivial — pure code, no schema or format change |
| Scope | 1 function modified + 1 ADR corrected + 1 unit test |

## Critical Correction Required: ADR-0004

**ADR-0004 says:** "Bevy 0.19 `Sprite::anchor: bevy_sprite::Anchor` enum serialized as PascalCase".

**Reality:** In Bevy 0.19, `Anchor` is:
1. NOT an enum — it's a struct wrapping a `Vec2`.
2. NOT a field of `Sprite` — it's a separate `Component`.
3. Auto-required by `Sprite` via `#[require(...)]`.

The export (PascalCase string) is still correct and the right interface. What's wrong is the
ADR's description of Bevy's internal mechanism. We need to update ADR-0004 to be accurate
(supersede, don't rewrite — preserve history).

## Domain Language (resolved)
- **Preview World** = the Bevy canvas inside the editor that renders the SceneDocument live.
- **Sprite Entity** = a Bevy entity with a `Sprite` component (auto-requires `Anchor`,
  `Transform`, `Visibility`, `VisibilityClass`).
- **Anchor Component** = a `bevy::sprite::Anchor` Component on a sprite entity that controls
  the pivot point used for rendering (and for transform-based positioning math).

## Resolved vs Unresolved Decisions

### Resolved
- Use Bevy 0.19's `Anchor` Component (not a computed Transform offset).
- Insert `Anchor` as a separate Component on the sprite entity (after Sprite, so it overrides
  the auto-required default).
- Map editor anchor strings to `bevy::sprite::Anchor` named constants via a small helper
  function (similar pattern to `dynamic_scene::anchor_str_to_bevy`).

### Unresolved (need design decisions in Fase 4)
1. **Where to put the mapping function?** `spawn_entity` itself, or a shared helper in
   `dynamic_scene.rs` (which already has `anchor_str_to_bevy`)? _Tradeoff_: shared helper
   deduplicates the logic; in-place keeps the preview-world code self-contained.
2. **What about the export?** Should `anchor_str_to_bevy` also return the `Vec2` so the export
   could include the literal normalized offset? _Decision_: keep the export as PascalCase
   strings (human-readable, debuggable). Add a sibling helper `anchor_str_to_bevy_anchor`
   that returns the Bevy `Anchor` Component value.
3. **Default behavior when anchor field is missing in SceneDocument?** _Decision_: silently
   use `Anchor::CENTER` (matches Bevy's default + matches the export's "missing anchor →
   Center" warning behavior).

## Invariants
- Preview world must rebuild cleanly when SceneDocument changes (no orphan Anchor components).
- `Anchor` insertion must happen AFTER `Sprite` insertion (otherwise Bevy's `#[require]`
  won't know to skip the default and use ours).
- Custom `Sprite.custom_size = (100, 100)` remains as today — anchor only changes the pivot,
  not the size.

## Recommended Effort
**verify** — small, focused, low-risk. Path A-min (no full propose→design cycle needed). But
since the user prefers the standard 8-phase flow, we do A-lite with all 8 phases.

## Risk / Open Questions
- Q1: Is `Anchor` insertion idempotent if called twice (e.g., on rebuild)? Bevy should handle
  this fine — `insert(Anchor)` on an entity that already has `Anchor` updates the value.
- Q2: Does the Anchor default-to-CENTER behavior of `#[require(Anchor)]` mean we MUST always
  insert `Anchor` after `Sprite`, or can we let Bevy auto-insert? Bevy's auto-insert uses
  `Anchor::default()` (= `CENTER`) which is fine for our Center case but wrong for TopLeft etc.
  So yes, we MUST always insert `Anchor` explicitly when the editor anchor is not Center.

## Out of Scope
- Changing the export format (still PascalCase strings).
- Adding `Anchor` Component to entities without a `Sprite` (no-op — Anchor is only meaningful
  on sprite entities).
- Migrating existing test scenes (rebuild on next scene load applies the new mapping).
- Animating anchor (not a feature).
