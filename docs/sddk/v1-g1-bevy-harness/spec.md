# v1.0-stabilization G1 Bevy Harness — Specification

> **Cycle:** `p-28fce7028ac3c497/v1-g1-bevy-harness`
> **Path:** A-min
> **Phase:** Specify
> **Date:** 2026-09-08
> **Based on:** `explore-report.md`

## Goal

Close one bounded step of v1.0-stabilization G1 (canonical playable
sample game) by creating a Bevy 2D runtime harness that consumes
`examples/platformer-minimal/` as a witness. The harness:

1. Reads the sample's `SceneAssetDocument` JSON files.
2. Spawns Bevy entities with the editor's authoring components
   translated to Bevy's component types.
3. Asserts that 4 placed scene instances appear in the Bevy World
   with the expected component values.

This proves the editor's authoring JSON is consumable by Bevy without
the editor's runtime. The existing Playwright spec
(`frontend/tests/e2e-game-creation.spec.ts`) already proves the editor
can load the sample; this cycle proves a **separate Bevy app** can too.

## Functional requirements

### FR-1: New workspace crate `examples-bevy-harness`

A new workspace member `crates/examples-bevy-harness/` MUST be added
with the following structure:

```
crates/examples-bevy-harness/
├── Cargo.toml            # workspace-style deps: bevy 0.19 (2d), editor-model, serde_json
├── src/
│   ├── lib.rs            # public API: load_sample_project(), spawn_main_scene()
│   ├── components.rs     # Bevy components: PlayerController, EnemyPatrol, Visible
│   └── loader.rs         # SceneAssetDocument → Bevy entity translation
└── tests/
    └── load_sample.rs    # Integration test (#[ignore] by default for CI budget)
```

The crate MUST be added to `Cargo.toml` workspace `members`.

### FR-2: Bevy components for custom schemas

The sample defines two custom schemas:
`game.PlayerController` (fields: `speed: f32`, `jump_force: f32`) and
`game.EnemyPatrol` (fields: `speed: f32`, `patrol_range: f32`). The
harness MUST register Bevy components with matching fields:

```rust
// crates/examples-bevy-harness/src/components.rs
#[derive(Component)]
pub struct PlayerController {
    pub speed: f32,
    pub jump_force: f32,
}

#[derive(Component)]
pub struct EnemyPatrol {
    pub speed: f32,
    pub patrol_range: f32,
}

#[derive(Component)]
pub struct Visible(pub bool);
```

These are minimal Bevy component shells that the harness uses to
*prove* the schema JSON round-trips. The harness does NOT need to
implement gameplay logic for these components — only prove they can
be deserialised from `ComponentInstance::values`.

### FR-3: Loader translates `SceneAssetDocument` → Bevy entities

`crates/examples-bevy-harness/src/loader.rs::spawn_scene_asset`
MUST:

1. Take `&SceneAssetDocument` (from `editor_model`) and `&mut World`.
2. For each entity in `doc.entities`:
   - Spawn a Bevy `Entity` in the world.
   - Translate `editor.Name` → Bevy `Name` component.
   - Translate `editor.Transform2D` → Bevy `Transform` component
     (translation `Vec3{x, y, 0}`, rotation around Z, scale as Vec3).
   - Translate `editor.Sprite2D` → Bevy `Sprite` component
     (color from `Color::srgba(r, g, b, a)`; asset path captured in
     custom component `EditorSpriteAsset(String)` since the harness
     has no image loader).
   - Translate `editor.Visible` → harness `Visible(bool)` component.
   - Translate `game.PlayerController` → harness `PlayerController`.
   - Translate `game.EnemyPatrol` → harness `EnemyPatrol`.
3. Return the count of spawned entities.

### FR-4: Spawn main scene instances

`crates/examples-bevy-harness/src/lib.rs::spawn_main_scene` MUST:

1. Read `examples/platformer-minimal/scenes/main.scene.json` and parse
   it using `editor_model::scene_instance::SceneInstance` per entry in
   the `instances` map.
2. For each placed instance, look up the referenced `SceneAssetDocument`
   (provided as a `HashMap<String, SceneAssetDocument>` keyed by
   `logical_path`).
3. Spawn the asset's root entity using `spawn_scene_asset`.
4. Apply the instance's `instance_components` (e.g. per-instance
   `editor.Transform2D` override) on top of the spawned entity.
5. Return the count of spawned entities + the count of instance
   overrides applied.

### FR-5: Integration test `load_sample`

`crates/examples-bevy-harness/tests/load_sample.rs` MUST contain a
test (marked `#[ignore]` for the default `cargo test` run to respect
the C-3 cargo-test budget) that performs:

```rust
#[test]
#[ignore] // Run explicitly: cargo test -p examples-bevy-harness -- --ignored
fn sample_round_trips_into_bevy_world() {
    // 1. Load project.json, assert 4 assets declared.
    // 2. Load each scene-assets/*.actor.json into SceneAssetDocument,
    //    assert each has exactly 1 entity.
    // 3. Load main.scene.json, assert 4 SceneInstance entries.
    // 4. Spawn a Bevy World; call spawn_main_scene; assert world has 4
    //    entities with the expected Name + Transform values.
}
```

The test MUST pass on a fresh `cargo test -p examples-bevy-harness -- --ignored` run.

### FR-6: Public re-export for downstream callers

`crates/examples-bevy-harness/src/lib.rs` MUST re-export the harness
public API:

```rust
pub use components::{PlayerController, EnemyPatrol, Visible, EditorSpriteAsset};
pub use loader::{spawn_scene_asset, spawn_main_scene};
```

This lets a future Bevy-native game (or the next G1 cycle) depend on
the harness without duplicating the loader code.

## Non-goals (explicitly out of scope for this cycle)

- **Bevy `play_mode` runtime state** — the harness does not exercise
  `enter_play_mode()` semantics. Tracked for the next G1 cycle.
- **UI-creation Playwright test** — does not author the sample from
  the editor's UI. Tracked separately.
- **BSN exporter integration** — the harness does NOT consume the
  `export/*.bsn` files; that path is verified by
  `frontend/tests/e2e-game-creation.spec.ts` instead. The
  architecture decision (Option C in the explore report) makes the
  Bevy harness a **JSON witness**, not a BSN parser.
- **Gameplay assertions** — the harness proves entity structure, not
  gameplay (no jumping, no enemy patrol, no pickup collision).
  Gameplay is documented but not exercised.

## Acceptance criteria

| Criterion                                                                  | Verification |
|----------------------------------------------------------------------------|--------------|
| `crates/examples-bevy-harness/` exists and is in workspace members        | `cargo metadata --format-version 1 \| jq '.workspace_members'` lists the new crate. |
| `cargo check -p examples-bevy-harness` exits 0                            | Cargo output ends with `Finished dev profile`. |
| `cargo build -p examples-bevy-harness` exits 0                            | Bevy compiles against the editor's `editor-model` types. |
| `cargo test -p examples-bevy-harness -- --ignored` runs ≥1 test that passes | Cargo test output: `test result: ok. N passed; 0 failed`. |
| Existing `cargo check --workspace --tests` (Block I) still green           | Output ends with `Finished dev profile`. |
| Existing `playwright.full.config.ts` still passes (incl. e2e-game-creation) | 296 tests in 40 files all pass. |

## References

- `docs/sddk/v1-g1-bevy-harness/explore-report.md`
- `examples/platformer-minimal/README.md`
- `crates/editor-model/src/scene_asset.rs`
- `crates/editor-model/src/scene_instance.rs`
- `crates/editor-model/src/component.rs`
- ADR-0005 (Scene Asset as the BSN-aligned reusable scene model).
