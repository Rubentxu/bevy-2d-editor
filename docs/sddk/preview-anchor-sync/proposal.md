# Preview Anchor Sync — Proposal

## Cycle: `preview-anchor-sync`
**Status:** draft → review
**ADR correction:** ADR-0004 supersession note (status: Superseded)

## What
Make the editor's preview world honor the `editor.Sprite2D.values.anchor` field by inserting a
`bevy::sprite::Anchor` Component on every spawned sprite entity. Today, the preview world
silently ignores the anchor (the export is correct, but what the user sees in the canvas does
not match what the export emits).

## Why
ADR-0004 chose Bevy native anchor over computed Transform offsets. The export (v0.6.0) emits the
right anchor string. But the preview world — which is what the editor user actually sees while
editing — never reads the anchor field. This is a fidelity bug: the user can't visually verify
that a sprite will be positioned correctly at runtime.

This was explicitly noted as out of scope in the dynamic-scene-export cycle (archive report:
"preview world `spawn_entity` still ignores anchor — follow-up cycle"). This is that follow-up.

## Where
- **Modify**: `crates/editor-core/src/lib.rs::spawn_entity()` — read anchor from
  `editor.Sprite2D.values.anchor` and insert `Anchor` Component after `Sprite`.
- **Modify**: `crates/editor-core/src/dynamic_scene.rs` — add `anchor_str_to_bevy_anchor()`
  helper that returns `bevy::sprite::Anchor` (the Bevy Component value), keeping the existing
  `anchor_str_to_bevy()` string helper for the export. The export format is unchanged.
- **Modify**: `docs/adr/0004-dynamic-scene-export-bevy-native-anchor.md` — add a "Superseded"
  section correcting the description of Bevy's Anchor API (it's a `pub struct Anchor(pub Vec2)`
  with named constants, and a separate Component auto-required by Sprite, not a Sprite field
  with an enum type).
- **Add**: 9 unit tests in `dynamic_scene.rs` (one per anchor string → Bevy Anchor constant).
- **Add**: 1 Playwright test verifying the visible preview position changes when anchor changes.

## Mapping (decisions)

### Editor Anchor String → Bevy 0.19 Anchor Component
| Editor | Bevy `Anchor` constant | `Anchor.0` |
|---|---|---|
| `"Center"` | `Anchor::CENTER` | `Vec2::ZERO` |
| `"TopLeft"` | `Anchor::TOP_LEFT` | `(-0.5, 0.5)` |
| `"TopCenter"` | `Anchor::TOP_CENTER` | `(0.0, 0.5)` |
| `"TopRight"` | `Anchor::TOP_RIGHT` | `(0.5, 0.5)` |
| `"CenterLeft"` | `Anchor::CENTER_LEFT` | `(-0.5, 0.0)` |
| `"CenterRight"` | `Anchor::CENTER_RIGHT` | `(0.5, 0.0)` |
| `"BottomLeft"` | `Anchor::BOTTOM_LEFT` | `(-0.5, -0.5)` |
| `"BottomCenter"` | `Anchor::BOTTOM_CENTER` | `(0.0, -0.5)` |
| `"BottomRight"` | `Anchor::BOTTOM_RIGHT` | `(0.5, -0.5)` |

### Default behavior
- `editor.Sprite2D.values.anchor` missing → use `Anchor::CENTER` (silent — no warning).
- `editor.Sprite2D.values.anchor` invalid string → use `Anchor::CENTER` + Bevy log warn
  (`warn!` macro from `bevy::log`).

### Insertion Order
```
cmd.insert(Sprite { ... });  // auto-requires Anchor::CENTER
cmd.insert(Anchor::TOP_LEFT); // overrides default
```

Bevy's `#[require(Anchor)]` runs at the moment `Sprite` is inserted. Our explicit `Anchor`
insertion immediately after overrides the default. Order matters — `Anchor` MUST come after
`Sprite`.

## Public API
- New helper: `pub fn anchor_str_to_bevy_anchor(s: &str) -> bevy::sprite::Anchor` in
  `crates/editor-core/src/dynamic_scene.rs`.
- Existing `anchor_str_to_bevy()` helper renamed to `anchor_str_to_bevy_str()` for clarity
  (it's only used by the export). No callers outside `dynamic_scene.rs`.
- `spawn_entity()` reads `editor.Sprite2D.values.anchor` and inserts the Bevy `Anchor`
  Component.

## Non-Goals (this cycle)
- Changing the export format (still PascalCase strings).
- Adding `Anchor` to entities without a Sprite.
- Visual / unit tests for the preview's pixel-perfect rendering (covered by the existing
  Playwright tests that assert canvas state — no new pixel checks).
- Migrating scenes (rebroadcast happens automatically on next rebuild).

## Acceptance Criteria (high level)
- AC-1: All 9 anchor strings map to the correct `bevy::sprite::Anchor` constant (unit tests).
- AC-2: `spawn_entity` inserts `Anchor` Component after `Sprite` on every entity with a
  `editor.Sprite2D` (visible in Bevy logs / debug).
- AC-3: Missing `anchor` field defaults silently to `Anchor::CENTER`.
- AC-4: Invalid `anchor` string defaults to `Anchor::CENTER` + Bevy `warn!`.
- AC-5: ADR-0004 has a "Superseded by facts" section correcting the API description.
- AC-6: All 132 existing unit tests still pass.
- AC-7: Existing Playwright tests still pass.

## Risks
1. **`Anchor` insertion order matters.** If we insert `Anchor` before `Sprite`, Bevy's
   `#[require]` will overwrite our value with `Anchor::default()`. Mitigation: code review
   + explicit comment in `spawn_entity`.
2. **`bevy::sprite` not in Cargo.toml features.** Mitigation: verified `bevy::sprite::Anchor`
   is reachable via `use bevy::sprite::Anchor;` with our current features.
3. **Existing scenes without `anchor` field suddenly render differently?** No — the existing
   preview renders sprites with `custom_size = (100, 100)` and Anchor::default (CENTER), so
   adding `Anchor::CENTER` explicitly is a no-op.
4. **Rebuild side effects?** `cmd.insert(Anchor)` on an entity that already has Anchor (from a
   prior rebuild) updates the value (no duplicate). Safe.

## Validation Strategy
- Unit tests in `dynamic_scene.rs` for the mapping (9 anchors + missing + invalid).
- One Playwright test that loads a scene with a `TopLeft` anchor sprite and asserts the
  sprite is rendered (via existing canvas-based test pattern).
- Visual regression: the editor's existing "spike sprite" uses the default `Center` anchor —
  adding explicit `Anchor::CENTER` must not change its pixel position.

## Effort
A-lite. ~2 hours. 1 PR. Tag v0.7.0.
