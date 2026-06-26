# Design: SceneDocument + Component Schema Registry

> Change: `scene-document` · Phase: sddk-design · Path: A-lite
> Spec: [`spec.md`](./spec.md) · Proposal: [`proposal.md`](./proposal.md) · Explore: [`explore-report.md`](./explore-report.md)

## Technical Approach

Introduce editor-owned Rust value types (`document.rs`) and a schema registry (`schema.rs`) in `crates/editor-core/src/`, then migrate the hardcoded `setup()` spike to spawn entities from a `SceneDocument` injected via a new `load_scene_json` `wasm_bindgen` channel (separate from LinearBus). Bevy types appear only behind a single `spawn_entity` mapping boundary. This satisfies spec §2 (document model + lossless roundtrip) and §3 (registry) while preserving all 7 invariants from the explore report (ADR-0001, ADR-0002, §6.2/6.3/6.6/6.9/6.1).

## Architecture Decisions

### Decision: Forward-compat `values: serde_json::Value`

**Choice**: `ComponentInstance.values` is `serde_json::Value` (an open object), not a typed `HashMap`.
**Alternatives**: typed `HashMap<String, FieldValue>` enum; generic `Vec<u8>` blob.
**Rationale**: Hito 0 §6.9 forbids deleting unknown fields. A typed map drops fields outside the enum; `Value` preserves them losslessly and survives schema migrations. This is the forward-compatibility decision flagged in spec §2.8 / proposal §4.5.

### Decision: Opaque `StableId(String)` newtype

**Choice**: `#[serde(transparent)] struct StableId(String)` rather than a raw `String` alias.
**Alternatives**: `pub type StableId = String;`; integer id.
**Rationale**: §6.2 — ids must never be confused with mutable `name`. The newtype prevents `entity.id = entity.name` type errors at compile time while serializing as a plain JSON string.

### Decision: Global registry via `OnceLock`

**Choice**: `static REGISTRY: OnceLock<ComponentSchemaRegistry>`, read-only after init, exposed as `pub fn registry()`.
**Alternatives**: `thread_local!`; Bevy `Resource`; constructor injection.
**Rationale**: §3.8 + ADR-0002 require one instance per session, outside the Bevy World. WASM is single-threaded; `OnceLock` gives a safe `&'static` borrow without interior mutability. A Bevy `Resource` would put document metadata inside the World, violating ADR-0002.

### Decision: `load_scene_json` separate channel, LinearBus untouched

**Choice**: New `#[wasm_bindgen] pub fn load_scene_json(&str)` stores into a `thread_local SCENE_DOC`; `setup()` reads it.
**Alternatives**: pipe JSON through the 64 KiB command bus; embed the scene only.
**Rationale**: The bus is sized for high-frequency raw-byte commands (move-sprite), not document blobs. Mixing protocols risks truncation. A dedicated string channel keeps the bus contract intact (explore §4.3).

### Decision: Single `spawn_entity` mapping boundary

**Choice**: All JSON→Bevy translation lives in one `spawn_entity` function; editor types never leak into Bevy world.
**Alternatives**: per-component Bevy systems reading editor structs.
**Rationale**: Bevy 0.19 API drift (explore §4.1) is isolated to one function. Maps `editor.Transform2D`→`Transform`, `editor.Sprite2D`→`Sprite`, `editor.Name`→`Name`; skips `editor.Visible`/`editor.Locked` (editorial-only, §9.5).

### Decision: Default-scene fallback for backward compat

**Choice**: If `load_scene_json` was not called, `setup()` spawns a built-in default scene (the old green sprite encoded as a `SceneDocument` constant).
**Alternatives**: hard error; keep two code paths.
**Rationale**: Existing Playwright tests (explore §1.4) call neither function and must keep passing. One code path (always `spawn_entity`) is simpler to test than two.

## Data Flow

```
JS:  editor.load_scene_json(jsonString)
          │  serde_json::from_str
          ▼
Rust: SCENE_DOC (thread_local Option<SceneDocument>)
          │  setup() reads on Startup
          ▼
   for each Entity ──spawn_entity()──► Bevy World (Sprite/Transform/Camera2d)
          │                                  │
          └── ComponentSchemaRegistry ◄──────┘  (read for defaults/validation,
              (OnceLock, outside World)            never mutated by Bevy)
```

`process_commands` / `emit_events` stay byte-oriented and query `<Sprite>` via `single_mut()`. For Hito 0's single-sprite scene this remains valid; multi-entity queries are a later change (spec §4).

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/editor-core/src/document.rs` | Create | `SceneDocument`, `Entity`, `ComponentInstance`, `StableId`, `Vec2`/`Color`/`Anchor`, `SceneError`, `validate()` |
| `crates/editor-core/src/schema.rs` | Create | `ComponentSchema`, `FieldDef`, `FieldType`, `Constraint`, `ComponentSchemaRegistry`, 5 seed schemas, `registry()` |
| `crates/editor-core/src/lib.rs` | Modify | `load_scene_json`, `SCENE_DOC`, `spawn_entity`, migrate `setup()` to document-driven spawn with default fallback; `mod document; mod schema;` |
| `crates/editor-core/Cargo.toml` | Modify | Add `serde`, `serde_json`, `thiserror` |

## Interfaces / Contracts

```rust
// document.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneDocument { pub version: String, pub scene_id: String, pub name: String, pub entities: Vec<Entity> }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: StableId, pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")] pub parent: Option<StableId>,
    pub components: Vec<ComponentInstance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentInstance {
    pub type_id: String,
    #[serde(default)] pub values: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 { pub x: f32, pub y: f32 }                 // { "x":.., "y":.. }
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Anchor { Center, TopLeft, TopRight, BottomLeft, BottomRight, TopCenter, BottomCenter, CenterLeft, CenterRight }

#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error("Invalid JSON: {0}")] Json(#[from] serde_json::Error),
    #[error("Unknown component type: {0}")] UnknownType(String),
    #[error("Parent not found: {0}")] ParentNotFound(StableId),
}
pub fn validate(doc: &SceneDocument, registry: &ComponentSchemaRegistry) -> Vec<ValidationIssue>;
```

```rust
// schema.rs
pub struct ComponentSchema { pub type_id: String, pub display_name: String, pub fields: Vec<FieldDef>, pub exports_to_bevy: bool }
pub struct FieldDef { pub name: String, pub field_type: FieldType, pub default: serde_json::Value, pub constraints: Vec<Constraint> }
pub enum FieldType { String, F32, Bool, Vec2, Color, Anchor, AssetReference }
pub enum Constraint { Min(f32), Max(f32), NonEmpty }
pub struct ComponentSchemaRegistry { /* HashMap<String, ComponentSchema> */ }
impl ComponentSchemaRegistry {
    pub fn with_builtin_seeds() -> Self;          // 5: Name, Transform2D, Sprite2D, Visible, Locked
    pub fn get(&self, type_id: &str) -> Option<&ComponentSchema>;
    pub fn all(&self) -> impl Iterator<Item = &ComponentSchema>;
    pub fn register(&mut self, schema: ComponentSchema);
}
pub fn registry() -> &'static ComponentSchemaRegistry;  // OnceLock singleton
```

Bevy mapping table (§9.5), applied in the single `spawn_entity` boundary:

| Editor | Bevy 0.19 |
|--------|-----------|
| `editor.Name.name` | `Name(..)` |
| `Transform2D.translation` | `Transform::from_xyz(x, y, 0.0)` |
| `Transform2D.rotation` | `Quat::from_rotation_z(r)` |
| `Transform2D.scale` | `Vec3::new(x, y, 1.0)` |
| `Sprite2D.color` | `Color::srgba(r,g,b,a)` |
| `Sprite2D.asset` | `Sprite` + `custom_size` (path load deferred) |
| `Visible` / `Locked` | not spawned (editorial) |

Hierarchy: two-pass — spawn all entities, then attach `ChildOf(parent)` relations. Default scene has no hierarchy this change; exercised in a later change.

## Testing Strategy

| Layer | What | Approach |
|-------|------|----------|
| Unit (`document.rs`) | spec §2 scenarios (9 reqs) | `#[cfg(test)]`: serialize/deserialize/roundtrip, shapes (`Vec2`→`{x,y}`, `Color`→`{r,g,b,a}`, `Anchor`→`"Center"`), id immutability, unknown-field preservation, version survival |
| Unit (`schema.rs`) | spec §3 scenarios (8 reqs) | 5 seeds present; `get` hit/miss; Transform2D fields; Name default `""`; Sprite2D AssetReference; Visible/Locked `exports_to_bevy==false`; `registry()` returns same instance |
| Unit (`lib.rs`) | `load_scene_json` roundtrip | smoke test in `cfg(test)` |
| E2E (`engine.spec.ts`) | scene renders from document | new test: call `load_scene_json` before `start_engine`, assert canvas has non-empty pixels via `getImageData`; existing 5 tests stay green via default-scene fallback |

## Migration / Rollout

No data migration (no persisted scenes yet). Rollout = single PR. `setup()` checks `SCENE_DOC`: `Some` → spawn from document; `None` → spawn default scene constant. Existing tests untouched.

## Open Questions

- [ ] `Sprite2D.asset` path → `Handle<Image>` loading in WASM needs an asset server path; for Hito 0 the spike sprite has no asset, so this is deferred — spawn uses `custom_size` only. Confirm during apply.
- [ ] Bevy 0.19 `ChildOf` relation API exact signature (vs `add_child`) — verified during apply when hierarchy is first exercised.

## ADR Candidates

- **Forward-compat `values: serde_json::Value`** — hard to reverse (pervasive across document/inspector/export), surprising vs typed map, real trade-off (type safety vs §6.9 unknown-field preservation) → ADR-003 draft in orchestrator step.
