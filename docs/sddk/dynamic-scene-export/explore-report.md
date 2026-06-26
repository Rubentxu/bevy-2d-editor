# DynamicScene Export — Explore Report

## Cycle: `dynamic-scene-export`
**Branch:** `feat/dynamic-scene-export`
**Source spec:** `docs/hito-0-spec.md` §9.5 (lines 347–365)
**Decision log ref:** #26 (`architecture/dynamic-scene-export-mapping`)

## Context Quality
**C1** — Hito 0 §9.5 is concrete (mapping table + error rules). The domain language is established
(SceneDocument, Component Schema Registry, exports_to_bevy). The editor-core already has 112 tests
and the preview-world `spawn_entity()` does roughly half the mapping work today. What's missing:
the actual EXPORT — a serializable runtime-friendly representation that an external Bevy app can
consume.

## Knowledge Coverage
- `docs/hito-0-spec.md` §9.5: full mapping + error rules (missing asset = warn, unknown schema =
  skip+warn, unknown field = preserved in document).
- `crates/editor-core/src/document.rs`: `SceneDocument`, `Entity`, `ComponentInstance`,
  `StableId`, `Vec2`, `Color`, `Anchor` already defined as `Serialize/Deserialize`.
- `crates/editor-core/src/schema.rs`: `ComponentSchema.exports_to_bevy` field exists; editorial
  components (Visible, Locked) already marked `false`.
- `crates/editor-core/src/lib.rs` `spawn_entity()` (lines 390–466): the editor preview already does
  the runtime mapping for 3 components. Anchor is currently IGNORED (no offset applied). Sprite2D
  asset path is ignored (sprite always rendered with `custom_size = (100, 100)` white). This is
  the preview stub — we need the EXPORT to do the same mapping but emit a serialized artifact.
- Bevy 0.19: `ChildOf(parent)` is the modern hierarchy component (replaces legacy `Parent`).
  `Transform::from_translation/with_rotation/with_scale` API in use.
- Cargo.toml: `bevy = "0.19", default-features = false, features = ["2d"]`. `bevy_scene` is NOT
  in deps — we deliberately avoid pulling Bevy's full scene serialization into the editor core.

## Taxonomy
| Axis | Classification |
|---|---|
| Domain | Architecture / runtime export |
| Risk | Medium — wrong mapping breaks Hito 0 §3 success criterion #3 ("real Bevy app loads the exported scene and renders it correctly") |
| Reversibility | Low once external Bevy apps consume the format |
| Scope | New public API on `editor-core`, new module, no schema changes |

## Domain Language (resolved)
- **DynamicScene Export** = the adapter that materializes editor-owned SceneDocument data into a
  Bevy-compatible runtime scene representation. _Avoid_: Bevy DynamicScene (the bevy_scene crate's
  type — different concept), runtime scene, scene loader.
- **Export Artifact** = the serialized JSON describing what a real Bevy app should spawn.
- **Export Step** = one transformation in the mapping pipeline (e.g., `editor.Transform2D →
  bevy Transform`).
- **Exportable Component** = a Component Instance whose schema has `exports_to_bevy = true` (the
  built-in 3: Name, Transform2D, Sprite2D).
- **Editorial Component** = a Component Instance whose schema has `exports_to_bevy = false`
  (Visible, Locked, and any user schema marked similarly).
- **Asset Reference** = the logical Project path (e.g., `assets/characters/player.png`) used by
  the editor; export converts it to a Bevy `AssetPath` for `AssetServer.load(...)`.

## Resolved vs Unresolved Decisions

### Resolved (from §9.5)
- Mapping table is fixed and complete (lines 349–358 of `hito-0-spec.md`).
- Editorial components (Visible, Locked) are NOT exported.
- Parent → `ChildOf(parent_entity)` for hierarchy.
- Error rules: missing asset = warn but don't fail; unknown schema = skip+warn; unknown field =
  preserved in document (not exported).

### Unresolved (need design decisions in Fase 4)
1. **Output format**: real Bevy `DynamicScene` (bevy_scene crate) vs. a stable JSON document that
   a Bevy loader consumes. _Tradeoff_: `bevy_scene` is canonical but couples editor to Bevy's
   internal serialization; a stable JSON is debuggable, versioned, and lets external Bevy apps
   write a tiny loader.
2. **Asset loading**: when asset path is empty, do we omit `Sprite` entirely, or emit a
   placeholder? _Tradeoff_: omit = no visual (test-friendly); placeholder = always visible
   (better UX).
3. **Anchor offset**: how do we compute the Transform translation offset from anchor without a
   known sprite size? _Tradeoff_: assume default size (100×100) vs. emit a separate
   `AnchorOffset` component vs. require sprite size in Sprite2D schema.
4. **WASM exposure**: is the export callable from React (e.g., `window.export_dynamic_scene()`)
   for the Hito 0 success criterion #3 demo?
5. **What is "a real Bevy app"**: the WASM Bevy renderer (preview) vs. a separate Rust binary
   that loads the JSON? For Hito 0, the preview IS the Bevy app — the export proves the data is
   Bevy-shaped. The actual external Bevy loader is Hito 1.

## Invariants
- The SceneDocument remains the source of truth — the export is a one-way read.
- Editorial components are NEVER exported, even if their values change.
- Stable IDs persist across exports (the export can include them but they are NOT Bevy Entity
  IDs — Bevy mints its own).
- The export is deterministic: same SceneDocument → same export bytes.

## Recommended Effort
**deepen** — this is a new public API on `editor-core` that becomes a contract with downstream
Bevy consumers. Path A-lite (propose → spec → design → tasks → apply → verify) with one
coherence gate at apply→verify.

## Risk / Open Questions to Resolve in Design
- Q1: JSON-as-bridge vs. true Bevy DynamicScene.
- Q2: What does "asset path" mean in the exported artifact — same `assets/...` string the editor
  uses, or a converted `bevy_asset::AssetPath`?
- Q3: Where do the warnings surface — return a `Vec<ExportWarning>` alongside the bytes, or
  push to a global?

## Known Knowns
- 5 built-in editor schemas; 3 exportable (Name, Transform2D, Sprite2D).
- `Anchor` enum has 9 variants (PascalCase JSON).
- `Vec2` is `{x, y}`, `Color` is `{r, g, b, a}` floats.
- Bevy 0.19 `Transform` API: `Transform::from_translation(translation).with_rotation(...).with_scale(...)`.
- `ChildOf(parent)` is the modern hierarchy component.
- Bevy sprite anchor: `Sprite::anchor` is an `Anchor` enum (`Anchor::Center`, `Anchor::BottomLeft`,
  etc.) that controls how the sprite is positioned relative to the entity Transform.

## Critical Insight
Bevy 0.19 has a native `Sprite::anchor` field (added in 0.14). This means we DON'T need to
compute a Transform offset from our `Anchor` enum — we map our `Anchor` directly to Bevy's
`Anchor`. The §9.5 spec table line "`editor.Sprite2D.values.anchor` | Computed `Transform`
offset" was written before we knew Bevy's native anchor support. **The mapping is simpler than
the spec implies.** We'll keep the spec language but update the mapping to use Bevy's native
`Sprite::anchor` — this needs an ADR note in Fase 4.

## What Goes Out of Scope (Hito 0)
- Hot-reload / live link between editor and Bevy runtime.
- Incremental export (full document each time).
- Asset bundling (assets must exist at the path the Bevy app expects).
- Component versioning (user adds field to schema → old exports break silently for that field).
- Multi-scene export.
