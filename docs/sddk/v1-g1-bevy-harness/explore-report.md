# v1.0-stabilization G1 Bevy Harness — Explore Report

> **Cycle:** `p-28fce7028ac3c497/v1-g1-bevy-harness`
> **Path:** A-min (explore → spec → tasks → apply → verify → debt-verify → release → archive)
> **Phase:** explore
> **Date:** 2026-09-08
> **Author:** discovery pass against `HEAD = a4e020a` (v0.108.9).

## 1. Scope and methodology

This report scopes **one bounded step** of the larger G1 "canonical playable
sample game" deliverable. The evidence-map (`docs/v1.0-stabilization-evidence-map.md`)
declares G1 as the highest-impact gap blocking v1.0 declaration and
identifies P1 ("Canonical playable sample game") as the recommended first
concrete work. P1 itself contains four sub-deliverables:

1. ✅ `examples/platformer-minimal/` — already exists with 13 files
   (project.json, 2 schemas, 1 scene, 4 scene-assets, 1 logic-graph, 4 BSN
   exports, README). Landed in commit before this cycle.
2. ✅ `frontend/tests/e2e-game-creation.spec.ts` — Playwright spec
   verifying that the editor can load the sample and export BSN for each
   asset. Verified to pass in 34.2 s on `playwright.full.config.ts`
   (2/2 tests green).
3. ❌ **Bevy runtime harness** that consumes the sample's assets and
   spawns Bevy entities. Not in repo. **This cycle's deliverable.**
4. ❌ UI-creation Playwright test (create sample via editor UI, not from
   filesystem). Out of scope for this cycle; tracked separately.

## 2. Choices

The sample ships assets in two formats:
- **Source of truth:** `scene-assets/*.actor.json` and
  `scene-assets/environment/ground.fragment.json`. These are the
  `SceneAssetDocument`s the editor loads and edits.
- **Derived export:** `export/*.bsn` — produced by
  `crates/editor-bevy/src/bsn_codegen.rs::emit_bsn_source_from_document`.
  Each `.bsn` file is **Rust source code** that wraps a `bsn!{}` block in
  a `pub fn spawn_<asset>(commands: Commands)` function (not a
  standalone `.bsn` artefact consumable by an external loader).

The harness must choose how to consume the sample. Three options
considered:

### Option A — Harness compiles `.bsn` files as Rust modules (rejected)

Each `.bsn` file would need to be `include!`-ed by the harness's
`lib.rs` and recompiled. Requires the `.bsn` files to be valid Rust
(no missing imports, no missing component types like `Playercontroller`
that the sample assumes are registered elsewhere). High risk of
compile-cycle coupling between the harness and the editor's BSN
emitter. Rejected.

### Option B — Harness parses `.bsn` files (rejected)

The `.bsn` files contain Rust wrapper code; parsing requires a
Rust-subset parser to recover the `bsn!{}` block, then a BSN parser to
recover entities/components. Two parsers for one feature is too much
surface for a P1 deliverable. Rejected.

### Option C — Harness reads `SceneAssetDocument` JSON directly (chosen)

The harness is a Bevy 2D app that:
1. Reads `examples/platformer-minimal/scene-assets/characters/player.actor.json`
   (and 3 sibling files) using `editor_model::SceneAssetDocument::from_str`.
2. Reads `examples/platformer-minimal/scenes/main.scene.json` using
   `editor_model::SceneDocument` types.
3. Spawns Bevy entities with built-in Bevy components (Name, Transform,
   Sprite) plus registered user-schema components (PlayerController,
   EnemyPatrol).
4. Asserts the resulting Bevy World contains 4 placed instances with
   the expected component values.

**Why Option C:**
- The harness exercises the same `SceneAssetDocument` types the editor
  uses, proving the JSON shape is consumable outside the editor.
- No `.bsn`-to-Rust parsing; the editor's BSN export pipeline is
  verified by the existing `e2e-game-creation.spec.ts`.
- The Bevy harness is a **consuming witness**, not a parser of the
  editor's export pipeline. This is the right separation of concerns
  for v1.0.

## 3. Files to create

| Path                                                   | Purpose                                                |
|--------------------------------------------------------|--------------------------------------------------------|
| `crates/examples-bevy-harness/Cargo.toml`             | New workspace member crate. Bevy 0.19, editor-model.   |
| `crates/examples-bevy-harness/src/main.rs`            | Bevy App entry point. Reads sample JSON, spawns.       |
| `crates/examples-bevy-harness/src/components.rs`      | Bevy components for `game.PlayerController`, etc.      |
| `crates/examples-bevy-harness/src/loader.rs`          | JSON → Bevy entity translation.                        |
| `crates/examples-bevy-harness/tests/load_sample.rs`   | Integration test: 4 assets + 4 placed instances.       |
| `Cargo.toml` (workspace)                               | Add `crates/examples-bevy-harness` to `members`.       |
| `docs/sddk/v1-g1-bevy-harness/explore-report.md`       | This file.                                             |

## 4. Files NOT touched in this cycle (out of scope)

- `examples/platformer-minimal/**` — already correct; this cycle
  consumes it, doesn't modify it.
- `frontend/tests/e2e-game-creation.spec.ts` — already passes; no
  changes needed.
- `crates/editor-bevy/**` — the Bevy harness is a separate crate that
  consumes editor-model, not extends editor-bevy.

## 5. Open questions

None for this cycle. The next cycle (G1 step 2, Bevy harness extends
to `play_mode` runtime state) will need to address how the harness
exercises `enter_play_mode()` semantics.

## 6. Acceptance criteria (preview)

The cycle's verify phase will require:

- `cargo check -p examples-bevy-harness` exits 0.
- `cargo test -p examples-bevy-harness` runs ≥1 integration test that:
  - Loads `examples/platformer-minimal/project.json` and asserts the
    project declares the 4 sample assets.
  - Loads each `*.actor.json` and asserts `SceneAssetDocument` parses
    with the expected entity count + component count.
  - Loads `scenes/main.scene.json` and asserts 4 placed instances
    reference the expected asset paths.
  - Spawns the 4 instances into a Bevy `World` and asserts the world
    contains 4 entities with the expected `Name` and `Transform` values.
- Existing `playwright.full.config.ts` suite still passes
  (e2e-game-creation included).

## 7. Risks

| Risk                                                                                  | Mitigation                                                                |
|---------------------------------------------------------------------------------------|---------------------------------------------------------------------------|
| Bevy 0.19 dependency may be heavy (slow compile) for a small harness                  | Use Bevy 0.19 with `default-features = false, features = ["2d"]` (matches editor-bevy). |
| `editor-model` types may not expose all needed fields for spawning                    | Use `serde_json::Value` directly via `ComponentInstance::values`.          |
| Custom schemas (`game.PlayerController`) require Bevy component registration           | Define minimal Bevy components in `components.rs` with the right fields.  |
| Long cargo test run on CI (already at C-3 risk)                                       | Mark this cycle's test with `#[ignore]` for cargo test default; document explicit run via `cargo test -p examples-bevy-harness -- --ignored --nocapture`. |

## 8. References

- `docs/v1.0-stabilization-evidence-map.md` §6.1, §7-P1 (G1 + P1 spec).
- `docs/roadmaps/v1.0-stabilization.md` §G1 (Canonical game authoring).
- `examples/platformer-minimal/README.md` (sample design).
- `frontend/tests/e2e-game-creation.spec.ts` (existing Playwright evidence).
- `crates/editor-model/src/scene_asset.rs` (types the harness consumes).
- `crates/editor-model/src/scene_instance.rs` (SceneInstance type).
- `crates/editor-model/src/component.rs` (ComponentInstance type).
