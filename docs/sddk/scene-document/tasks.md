# Tasks: SceneDocument + Component Schema Registry

> Change: `scene-document` · Phase: tasks · Path: A-lite · Mode: auto
> Spec: [`spec.md`](./spec.md) · Design: [`design.md`](./design.md) · Strategy: single-pr

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~450 (≈400 Rust + ≈50 TypeScript) |
| 400-line budget risk | Medium |
| Chained PRs recommended | No |
| Suggested split | single PR (per `delivery_strategy: single-pr`) |
| Delivery strategy | single-pr |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: stacked-to-main
400-line budget risk: Medium

## Phase 1: Foundation / Dependencies

- [ ] 1.1 Add `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `thiserror = "1"` to `crates/editor-core/Cargo.toml` deps. Acceptance: `cargo check --target wasm32-unknown-unknown` succeeds with no new code.
  Commit: `build(deps): add serde, serde_json, thiserror to editor-core`

## Phase 2: SceneDocument Types

- [ ] 2.1 Create `crates/editor-core/src/document.rs` with `StableId` (transparent newtype), `Vec2`, `Color`, `Anchor` (PascalCase), `SceneDocument`, `Entity`, `ComponentInstance`; `ComponentInstance.values: serde_json::Value`. Acceptance: module compiles; no behavior yet.
  Commit: `feat(document): add SceneDocument, Entity, ComponentInstance types`

- [ ] 2.2 Add `#[cfg(test)] mod tests` in `crates/editor-core/src/document.rs` covering all 10 spec §2 scenarios (serialize populated/empty, deserialize well-formed, roundtrip multi-entity + hierarchy, rename preserves id, ids opaque & value-comparable, Vec2/Color/Anchor shapes, unknown field preserved, version preserved, namespaced `type_id`). Acceptance: `cargo test --lib document` passes all tests.
  Commit: `test(document): add SceneDocument serde roundtrip tests`

## Phase 3: Component Schema Registry

- [ ] 3.1 Create `crates/editor-core/src/schema.rs` with `FieldType`, `Constraint`, `FieldDef`, `ComponentSchema`, `ComponentSchemaRegistry`; `with_builtin_seeds()` returns the 5 seed schemas (Name, Transform2D, Sprite2D, Visible, Locked); Visible/Locked set `exports_to_bevy: false`. Acceptance: module compiles.
  Commit: `feat(schema): add ComponentSchemaRegistry with 5 built-in seeds`

- [ ] 3.2 Add `#[cfg(test)] mod tests` in `crates/editor-core/src/schema.rs` covering all 8 spec §3 scenarios (5 seeds present, `get` hit, `get` miss no panic, Transform2D fields, Name default `""`, Sprite2D asset is logical path, Visible/Locked editorial-only flag, `registry()` OnceLock singleton). Acceptance: `cargo test --lib schema` passes.
  Commit: `test(schema): add registry lookup and field tests`

## Phase 4: Integration / Wiring

- [ ] 4.1 Wire modules into `crates/editor-core/src/lib.rs`: add `mod document; mod schema;`, re-export `SceneDocument` and `StableId`. Acceptance: `cargo check --target wasm32-unknown-unknown` passes; WASM spike still builds & renders.
  Commit: `refactor(lib): expose SceneDocument module`

- [ ] 4.2 Add `thread_local! SCENE_DOC` and `#[wasm_bindgen] pub fn load_scene_json(json: &str)` in `crates/editor-core/src/lib.rs` (errors → `JsValue`). Acceptance: compiles; existing spike unchanged (SCENE_DOC == None → fallback path).
  Commit: `feat(lib): add load_scene_json wasm_bindgen channel`

- [ ] 4.3 Migrate `setup()` in `crates/editor-core/src/lib.rs` to read `SCENE_DOC`: if Some, iterate entities through a single `spawn_entity()` mapping boundary (Name→Bevy Name, Transform2D→Transform, Sprite2D→Sprite+custom_size; Visible/Locked skipped); if None, spawn a hardcoded default `SceneDocument` JSON constant that reproduces the current green sprite. Acceptance: spike renders sprite via default fallback; WASM builds.
  Commit: `feat(spike): spawn entities from SceneDocument with default fallback`

## Phase 5: End-to-End Validation

- [ ] 5.1 Add a new test in `frontend/tests/engine.spec.ts` that calls `window.load_scene_json(json)` with a custom 2-entity scene before `start_engine`; asserts "Bevy running" still appears and canvas has non-empty content via `getImageData`. Acceptance: new test passes; existing 5 tests still pass via default fallback.
  Commit: `test(e2e): add load_scene_json Playwright test`

- [ ] 5.2 Run `cargo test`, `just wasm`, `just test`. Acceptance: all Rust unit tests pass, WASM builds clean, all 6 Playwright tests pass (5 existing + 1 new).
  Commit: `chore(tests): verify full scene-document test suite green`