# Preview Anchor Sync — Design

## Cycle: `preview-anchor-sync`
**Status:** draft → review
**ADR correction:** ADR-0004 (Superseded by facts)

---

## Architecture

```
                  ┌──────────────────────────────┐
                  │  SceneDocument               │
                  │  (editor-core::document)     │
                  └──────────┬───────────────────┘
                             │
                             ▼
              ┌──────────────────────────────────────┐
              │  spawn_entity(commands, &Entity)    │  (lib.rs)
              │  ├─ parse editor.Name → Name        │
              │  ├─ parse editor.Transform2D →      │
              │  │   Transform                       │
              │  ├─ parse editor.Sprite2D →         │
              │  │   Sprite                          │
              │  │   + Anchor ← NEW (this cycle)    │
              │  └─ skip editorial components       │
              └──────────────────────────────────────┘
                             │
                             ▼
        ┌────────────────────────────────────────────────┐
        │  dynamic_scene::anchor_str_to_bevy_anchor(s)  │  (NEW helper)
        │  → returns bevy::sprite::Anchor Component     │
        └────────────────────────────────────────────────┘
```

## Module Changes

### 1. `crates/editor-core/src/dynamic_scene.rs`

Add a new helper that returns the Bevy Component value (not just a string):

```rust
use bevy::prelude::Vec2;
use bevy::sprite::Anchor;

/// Map our `Anchor` enum string to Bevy 0.19's `Anchor` Component value.
/// Returns `Anchor::default()` (= `Anchor::CENTER`) for unknown strings.
pub fn anchor_str_to_bevy_anchor(s: &str) -> Anchor {
    match s {
        "Center" => Anchor::CENTER,
        "TopLeft" => Anchor::TOP_LEFT,
        "TopCenter" => Anchor::TOP_CENTER,
        "TopRight" => Anchor::TOP_RIGHT,
        "CenterLeft" => Anchor::CENTER_LEFT,
        "CenterRight" => Anchor::CENTER_RIGHT,
        "BottomLeft" => Anchor::BOTTOM_LEFT,
        "BottomCenter" => Anchor::BOTTOM_CENTER,
        "BottomRight" => Anchor::BOTTOM_RIGHT,
        _ => Anchor::default(),
    }
}
```

The existing `anchor_str_to_bevy()` helper (returns `&'static str`) is kept for the export
unchanged. The export continues to emit PascalCase strings — that's the stable interface. The
new helper is for the preview world.

### 2. `crates/editor-core/src/lib.rs::spawn_entity`

Modified flow:

```rust
fn spawn_entity(commands: &mut Commands, entity: &Entity) {
    use bevy::prelude::Name as BevyName;
    use bevy::sprite::Anchor;

    let mut name: Option<BevyName> = None;
    let mut transform: Option<Transform> = None;
    let mut sprite: Option<Sprite> = None;
    let mut anchor_str: Option<String> = None;
    let mut invalid_anchor: Option<String> = None;

    for component in &entity.components {
        match component.type_id.as_str() {
            "editor.Name" => { /* unchanged */ }
            "editor.Transform2D" => { /* unchanged */ }
            "editor.Sprite2D" => {
                // ... existing color + custom_size logic ...

                sprite = Some(Sprite {
                    color,
                    custom_size: Some(Vec2::splat(100.0)),
                    ..default()
                });

                // Read anchor (NEW). Track separately so we can insert AFTER Sprite.
                match component.values.get("anchor").and_then(|v| v.as_str()) {
                    Some(s) => anchor_str = Some(s.to_string()),
                    None => {
                        // Missing field — silently default to Center.
                        anchor_str = Some("Center".to_string());
                    }
                }
            }
            _ => {}
        }
    }

    let mut cmd = commands.spawn_empty();
    cmd.insert(SceneEntity);

    if let Some(n) = name { cmd.insert(n); }
    if let Some(t) = transform { cmd.insert(t); }
    if let Some(s) = sprite {
        cmd.insert(s);  // This auto-requires Anchor::default() (CENTER).
        // Now insert our Anchor — overrides the auto-required default.
        if let Some(s) = anchor_str {
            let bevy_anchor = dynamic_scene::anchor_str_to_bevy_anchor(&s);
            // If the editor string didn't match any known anchor, log a warning.
            if !is_known_anchor(&s) {
                warn!("Sprite2D anchor '{}' is not recognized; using Center", s);
            }
            cmd.insert(bevy_anchor);
        }
    }
}

fn is_known_anchor(s: &str) -> bool {
    matches!(s,
        "Center" | "TopLeft" | "TopCenter" | "TopRight"
        | "CenterLeft" | "CenterRight"
        | "BottomLeft" | "BottomCenter" | "BottomRight"
    )
}
```

Key design choices:
- **Anchor inserted AFTER Sprite** — Bevy's `#[require(Anchor)]` triggers when Sprite is
  inserted; our explicit insertion overrides it.
- **Missing anchor field → silent default** — matches Bevy's behavior + matches the export's
  silent default for missing field.
- **Invalid anchor string → Center + warn!** — Bevy's `warn!` macro for editor log visibility.

## Cargo Dependencies

No changes. `bevy::sprite::Anchor` is reachable via the existing
`bevy = { version = "0.19", default-features = false, features = ["2d"] }` (verified during
exploration).

## Test Strategy

### Unit tests (in `dynamic_scene.rs::tests`, ≥11 tests)
- `test_anchor_str_to_bevy_anchor_center`
- `test_anchor_str_to_bevy_anchor_top_left`
- `test_anchor_str_to_bevy_anchor_top_center`
- `test_anchor_str_to_bevy_anchor_top_right`
- `test_anchor_str_to_bevy_anchor_center_left`
- `test_anchor_str_to_bevy_anchor_center_right`
- `test_anchor_str_to_bevy_anchor_bottom_left`
- `test_anchor_str_to_bevy_anchor_bottom_center`
- `test_anchor_str_to_bevy_anchor_bottom_right`
- `test_anchor_str_to_bevy_anchor_invalid_defaults_to_center`
- `test_anchor_str_to_bevy_anchor_empty_string_defaults_to_center`

Note: scenarios involving `spawn_entity()` directly are hard to unit-test because `spawn_entity`
requires a Bevy `Commands` queue (an ECS context). We rely on Playwright tests for those (the
preview world rebuilds on scene change, and we observe the rendered sprite position via canvas).

### Playwright test (1 test in `frontend/tests/anchor-sync.spec.ts`)
- Load a scene with a sprite at position (0, 0) with `anchor: "TopLeft"`.
- Wait for the canvas to render.
- Assert the entity has the right `Anchor` Component (via `get_scene_snapshot` + verify the
  anchor string round-trips).
- (Visual pixel position verification is fragile in headless Chrome without a screenshot
  diff infrastructure — out of scope.)

### Regression
- All 132 existing unit tests must still pass.
- All 29 existing Playwright tests must still pass.

## ADR-0004 Correction

Append a "Superseded by facts" section to the existing ADR-0004 that:

1. Acknowledges the original ADR described Bevy's Anchor as an enum + Sprite field.
2. Corrects the description: Bevy 0.19 has `pub struct Anchor(pub Vec2)` with 9 named
   constants, and `Anchor` is a separate Component auto-required by `Sprite` via
   `#[require(...)]`.
3. Reaffirms the decision is still correct (use Bevy native anchor) — only the API details
   were wrong in the original ADR.
4. Cross-references this cycle's docs.

Do NOT rewrite the original ADR — preserve history.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Anchor inserted before Sprite → Bevy's `#[require]` overwrites | Low | Code review + explicit comment in code |
| Existing scenes suddenly render differently | Very low | Anchor::CENTER is Bevy's default; our explicit Center matches it |
| `bevy::sprite::Anchor` not actually available | Low (verified) | Exploration confirmed `use bevy::sprite::Anchor;` compiles |
| Rebuild side effects | None | `cmd.insert(Anchor)` is idempotent |

## Out of Scope

- Pixel-perfect visual tests (would need a screenshot diff infrastructure).
- Animation/tweening of anchor (not a feature).
- Export format change (PascalCase strings stay).
- Anchor on non-sprite entities (no semantic meaning).
