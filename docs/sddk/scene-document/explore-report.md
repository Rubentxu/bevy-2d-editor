# Explore Report: scene-document

> Change: `scene-document` · Phase: sddk-explore · Path: A-lite · Context quality: C2
> Model: GLM-5.1 (explore executor)

---

## 1. Current Spike Architecture

The working spike lives in a single file: `crates/editor-core/src/lib.rs` (198 lines).

### 1.1 LinearBus (zero-cost shared memory)

A fixed 64 KiB byte buffer (`Box<[u8]>`) used bidirectionally:

| Bus | Direction | Layout | Capacity |
|-----|-----------|--------|----------|
| `COMMAND_BUS` | JS → Rust | `[u16 type][u16 len][payload...]` repeated, write-offset at bytes 0..4 | 64 KiB |
| `EVENT_BUS` | Rust → JS | Same slot framing | 64 KiB |

Both are `thread_local` `RefCell<Option<LinearBus>>`. The JS side holds a `DataView` into WASM linear memory via `get_command_bus_ptr()` / `get_event_bus_ptr()`. No serialization — raw f32/u16 little-endian bytes.

### 1.2 Bevy App

- `DefaultPlugins` with `Window { canvas: "#id" }`, `fit_canvas_to_parent: true`.
- `Startup::setup` spawns `Camera2d` + one hardcoded `Sprite` (green, 100x100 at origin).
- `Update::process_commands` drains the command bus; `CMD_MOVE_SPRITE` (id 1) reads two f32s and mutates the single sprite's `Transform`.
- `Last::emit_events` writes sprite position (`EVT_SPRITE_POSITION` = 1) and FPS every 0.5 s (`EVT_FPS` = 2), then calls JS `onFrameEnd()` callback.

### 1.3 Frontend Bridge

`engine-bridge.ts` — loads WASM via dynamic import, acquires bus DataViews, polls events on `onFrameEnd`, exposes `sendMoveSprite(x, y)` that writes directly into command-bus memory. `App.tsx` is a minimal sidebar with X/Y inputs + a "Move Sprite" button.

### 1.4 Tests

`engine.spec.ts` — 5 Playwright E2E tests: WASM load, bridge logs, move-sprite roundtrip, FPS counter, rapid-command stress. All operate on the hardcoded single-sprite UI.

---

## 2. Gap Analysis — What's Missing for SceneDocument

| Need | Current state | Gap |
|------|---------------|-----|
| `SceneDocument` data model | Does not exist | No Rust types for document, entities, component instances |
| `ComponentSchemaRegistry` | Does not exist | No schema types, no registry, no 5 built-in components |
| JSON serialization | No serde in deps | `Cargo.toml` has only `bevy`, `wasm-bindgen`, `console_error_panic_hook`, `web-sys` |
| Stable ID generation | Does not exist | No ULID or equivalent; spike has no entity IDs |
| Spike → SceneDocument migration | Hardcoded `setup()` | Bevy startup must read SceneDocument, spawn entities per component instances |
| Roundtrip test | No JSON tests at all | Need serialize → deserialize → assert-equal test |
| Scene data injection channel | Only `CMD_MOVE_SPRITE` | Need a way to pass scene JSON from JS → WASM (or embed default scene in WASM) |

### 2.1 The 5 Built-in Components (Hito 0 §7)

None exist as data yet. Required:

| `type_id` | Fields | Exports to Bevy |
|-----------|--------|-----------------|
| `editor.Name` | `name: string` | `Name` |
| `editor.Transform2D` | `translation: Vec2, rotation: f32, scale: Vec2` | `Transform` |
| `editor.Sprite2D` | `asset: AssetReference, color: Color, anchor: Anchor` | sprite bundle |
| `editor.Visible` | `visible: bool` | No |
| `editor.Locked` | `locked: bool` | No |

---

## 3. Binding Constraints (from CONTEXT.md + Hito 0 spec + ADRs)

These are **invariants** that cannot be violated by the design:

1. **JSON is the source of truth** (ADR-0001). `DynamicScene`/RON are export targets only. The editor owns the document.
2. **Stable IDs are opaque + immutable** (§6.2). Entity `id` never mutates; `name` is mutable. References use `id`.
3. **Schemas are global** (§6.3). Component instances carry only `type_id` + `values`; field definitions live in the registry.
4. **Hierarchy is canonical** (§6.6). Each entity has optional `parent`; transforms are local-to-parent.
5. **Single Bevy canvas** (ADR-0002). React never touches canvas; SceneDocument + registry live outside the World as Rust structs.
6. **Forward compatibility** (§6.9). Unknown fields are preserved on load, never auto-deleted.
7. **Document versioning** (§6.1). `"version": "0.1"` — old scenes must remain loadable.

### 3.1 Scope Boundary for THIS Change

Per the launch plan, this change is narrowly scoped to **data model + spike migration**. Explicitly OUT: command system, undo/redo (Operation Log), OPFS persistence, DynamicScene Export, hierarchy/inspector UI panels.

---

## 4. Codebase Risks

### 4.1 Bevy 0.19 API Compatibility (Medium)

The spike uses Bevy 0.19 API that works (`Camera2d`, `Sprite`, `Transform`, `Color::srgb`). The risk is in mapping editor-owned JSON field values to Bevy runtime types. For this change the migration is read-only (SceneDocument → spawn), so the export-direction mapping (Bevy → JSON) is out of scope. We only need JSON types → Bevy spawn for the spike sprite.

**Mitigation:** Use editor-owned types (`Vec2`, `Color`) in the JSON model; map to Bevy types only in the `setup()` migration. Keep the mapping in one place.

### 4.2 serde in WASM (Low)

`serde` + `serde_json` compile cleanly to `wasm32-unknown-unknown` — this is standard. The spike already builds with `wasm-pack`. Adding `serde = { version = "1", features = ["derive"] }` and `serde_json = "1"` to `Cargo.toml` is low-risk.

**Note:** `serde_json` has a small WASM footprint increase (~50-100 KiB). Acceptable for Hito 0.

### 4.3 Scene Data Injection Channel (Medium)

The LinearBus is designed for high-frequency typed commands (raw bytes), not for passing a JSON document string. Two options:

- **(a)** Embed a default scene JSON directly in the WASM binary (simplest for this change — validates roundtrip without changing the bus protocol).
- **(b)** Add a `#[wasm_bindgen]` function `load_scene_json(&str)` that deserializes into `SceneDocument` and triggers a rebuild.

**Recommendation for THIS change:** Option (a) for the roundtrip unit test (pure Rust, no WASM), plus option (b) for the spike migration so JS can pass a real scene. This keeps the LinearBus untouched for its high-frequency role.

### 4.4 Color / Anchor Types (Low-Medium)

`editor.Sprite2D` needs `color` and `anchor`. Bevy 0.19's `Color` is an enum (sRGB, Linear, etc.). The JSON model should use a simple representation (e.g., `{ "r": 0.3, "g": 0.8, "b": 0.5, "a": 1.0 }`) and map to `Color::srgba` in the spawn code. `Anchor` is editor-owned (not a Bevy type) — define it as a small enum.

### 4.5 WASM Binary Size (Low)

Adding serde + serde_json increases the WASM binary. The release profile already uses `opt-level = "s"`, `lto = true`, `strip = true`. Acceptable for Hito 0.

---

## 5. Effort Estimate

| Work item | Size | Notes |
|-----------|------|-------|
| Data model types (SceneDocument, Entity, ComponentInstance, ComponentSchema) | S | Straightforward serde structs |
| Seed 5 component schemas | S | Static data, declarative |
| serde JSON roundtrip (unit test in Rust) | S | `#[test]` serialize/deserialize/assert_eq |
| Stable ID type (opaque string, ULID-like) | XS | String wrapper for now; real ULID later |
| Spike migration: `setup()` reads SceneDocument → spawns entities | M | Core integration point |
| `load_scene_json(&str)` wasm_bindgen function | XS | One function |
| Playwright scene roundtrip test | S | Extend existing test suite |

**Total:** Small-to-medium. No new crates needed; all work in `editor-core`.

---

## 6. Recommendations for Proposal

1. **Capabilities (NEW):** `scene-document-model` (data types + JSON roundtrip) and `component-schema-registry` (5 built-in schemas). No openspec/ dir exists, so all capabilities are new.
2. **Approach:** Option (a) full schema-driven registry (per Hito 0 §6.3) — NOT lightweight typed entities with JSON passthrough.
3. **Keep LinearBus untouched** — scene data injection via a separate `wasm_bindgen` string function.
4. **Editor-owned types** for Color/Anchor/Vec2 in JSON model; map to Bevy only in spawn code.
5. **Roundtrip test** as the primary success gate for this change.
