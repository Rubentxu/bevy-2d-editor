# DynamicScene Export — Proposal

## Cycle: `dynamic-scene-export`
**Status:** draft → review
**ADR:** to be drafted in Fase 4 (ADR-0004)

## What
Add a `dynamic_scene_export` module to `editor-core` that consumes a `SceneDocument` and emits a
serializable JSON artifact describing what a real Bevy 0.19 application should spawn. Expose the
export to React via a new WASM function `export_dynamic_scene()` returning the JSON string plus a
list of warnings (missing assets, skipped unknown schemas). This is the runtime adapter that
satisfies Hito 0 success criterion #3 — "Bevy integration: a real Bevy app loads the exported
scene and renders it correctly".

## Why
Hito 0 §9.5 specifies the mapping between SceneDocument fields and Bevy runtime types. Without the
export, the editor only has a preview world — there's no way for an external Bevy app to consume
the scene. The preview world (`spawn_entity`) already proves the mapping works in-process; the
export is the serialized artifact that proves the mapping is stable across processes and Bevy
versions.

## Where
- New module: `crates/editor-core/src/dynamic_scene.rs` (algorithm + types + warnings)
- New WASM binding: `crates/editor-core/src/lib.rs::export_dynamic_scene()` returning `JsValue`
  with `{ "json": String, "warnings": Vec<String> }`
- New unit tests: `crates/editor-core/src/dynamic_scene.rs` inline (≥10 tests covering each
  mapping, hierarchy, warnings, error rules)
- New Playwright test: `frontend/tests/export.spec.ts` (≥3 tests: empty doc, doc with all 3
  components, missing asset warning)
- No changes to `document.rs`, `schema.rs`, `template.rs`, `operation_log.rs`, `command.rs`

## Mapping (decisions)

### Editor → Bevy
| SceneDocument | Bevy 0.19 |
|---|---|
| `editor.Name.values.name` | `Name::new(name)` |
| `editor.Transform2D.values.translation` (Vec2) | `Transform.translation = (x, y, 0)` |
| `editor.Transform2D.values.rotation` (f32, radians) | `Transform.rotation = Quat::from_rotation_z(rotation)` |
| `editor.Transform2D.values.scale` (Vec2) | `Transform.scale = (x, y, 1)` |
| `editor.Sprite2D.values.asset` (non-empty) | `Sprite { image: AssetServer.load(path), .. }` (placeholder for runtime; export emits asset path string) |
| `editor.Sprite2D.values.color` | `Sprite.color = Color::srgba(r,g,b,a)` |
| `editor.Sprite2D.values.anchor` | `Sprite.anchor = Bevy::Anchor::Center/BottomLeft/...` (native mapping) |
| Entity `parent` | `ChildOf(parent_entity)` — Bevy mints new entity IDs |
| `editor.Visible` / `editor.Locked` | **NOT exported** |
| Unknown `type_id` | **Skipped**, warning recorded |
| Missing `asset` on Sprite2D | `Sprite` omitted, warning recorded |

### Output Format (JSON shape)
```json
{
  "version": "0.1.0",
  "source_scene_id": "scene_001",
  "entities": [
    {
      "stable_id": "ent_01",
      "name": "Player",
      "parent_stable_id": null,
      "components": {
        "bevy.Name": { "name": "Player" },
        "bevy.Transform": {
          "translation": [100.0, 200.0, 0.0],
          "rotation": [0.0, 0.0, 0.0, 1.0],
          "scale": [1.0, 1.0, 1.0]
        },
        "bevy.Sprite": {
          "asset": "assets/player.png",
          "color": [1.0, 1.0, 1.0, 1.0],
          "anchor": "Center"
        }
      }
    }
  ],
  "warnings": []
}
```

- JSON shape uses **Bevy type names with `bevy.` prefix** to disambiguate from editor schema
  names. A real Bevy loader maps these keys to bevy components.
- `parent_stable_id` is a string reference, NOT a Bevy Entity ID (Bevy mints new IDs at load
  time). The loader walks the array and resolves stable_id → Bevy Entity, then inserts ChildOf.
- `rotation` is the Bevy quaternion as `[x, y, z, w]`.
- `scale` and `translation` are arrays of 3 (z forced to 0 for translation, 1 for scale).
- `anchor` is Bevy's `bevy_sprite::Anchor` enum serialized as PascalCase.

### Anchor Mapping (Decision)
**Use Bevy's native `Sprite::anchor` (added in Bevy 0.14).** Do NOT compute a Transform offset.
The §9.5 spec said "Computed Transform offset" but that was written before we knew Bevy has
native anchor support. Native anchor is simpler, more correct, and what Bevy games actually use.
ADR-0004 documents this deviation from §9.5 spec language (functional behavior matches §9.5:
"anchor determines sprite position" — just via a different mechanism).

### Warnings (not errors)
The export NEVER fails. It always returns JSON + a list of warnings. Warnings surface in the UI
console log (via `console.warn` from WASM) so developers see them.

| Condition | Warning message |
|---|---|
| Sprite2D with empty asset path | `"Sprite2D on entity {id} has empty asset path; Sprite omitted"` |
| Unknown `type_id` | `"Skipping unknown component '{type_id}' on entity {id}"` |
| Invalid Vec2 (missing x or y) | `"Transform2D translation invalid on entity {id}; using (0, 0)"` |
| Invalid Color | `"Sprite2D color invalid on entity {id}; using white"` |
| Invalid Anchor | `"Sprite2D anchor invalid on entity {id}; using Center"` |
| Child references unknown parent | `"Entity {id} parent {parent_id} not found in scene; exporting as root"` |

## Public API
```rust
// In editor-core/src/dynamic_scene.rs

pub fn export_dynamic_scene(doc: &SceneDocument) -> DynamicSceneExport;

pub struct DynamicSceneExport {
    pub json: String,
    pub warnings: Vec<ExportWarning>,
}

pub struct ExportWarning {
    pub entity_stable_id: Option<String>,
    pub component_type_id: Option<String>,
    pub message: String,
}

// WASM binding in lib.rs:
#[wasm_bindgen]
pub fn export_dynamic_scene(doc_json: &str) -> Result<JsValue, JsValue>;
// Returns { json: String, warnings: [{ entity_stable_id, component_type_id, message }, ...] }
```

## Non-Goals (Hito 0)
- Live link / hot-reload.
- Incremental diff export.
- Real Bevy DynamicScene format (using bevy_scene crate) — we use a stable JSON shape instead.
- Multi-scene bundling.
- Asset bundling (the export references asset paths; the loader must have them on disk).
- A separate external Bevy binary that loads the JSON — that's Hito 1. For Hito 0, the
  preview world IS the Bevy runtime and the export proves the data is Bevy-shaped.

## Acceptance Criteria (high level)
- AC-1: `export_dynamic_scene` returns a JSON string that round-trips through serde_json.
- AC-2: A SceneDocument with one Name+Transform2D entity produces an export with one
  `bevy.Name` + one `bevy.Transform` component and zero warnings.
- AC-3: A SceneDocument with a Sprite2D entity containing all 4 fields (asset, color, anchor)
  produces an export with a complete `bevy.Sprite` component.
- AC-4: A SceneDocument with a parent + child produces an export where `parent_stable_id` is
  set on the child.
- AC-5: A SceneDocument with `editor.Visible` produces an export with NO `bevy.Visible` (or any
  equivalent) and no warning (editorial components are silent, not warnings).
- AC-6: A Sprite2D with empty asset produces an export with no `bevy.Sprite` AND a warning.
- AC-7: An unknown `type_id` (e.g., `game.PlayerHealth`) produces an export without that
  component AND a warning.
- AC-8: Child with missing parent produces an export where the child has `parent_stable_id:
  null` AND a warning.
- AC-9: Empty document produces `{"entities": [], "warnings": []}`.
- AC-10: Export bytes are deterministic (same input → same output bytes).

## Risks
1. **Format stability** — the export becomes a contract with downstream Bevy consumers. Mitigation:
   include `version: "0.1.0"` field; ADR-0004 documents the schema.
2. **Bevy version coupling** — if Bevy changes its Transform serialization, the export changes.
   Mitigation: export the full Transform (translation Vec3 + rotation Quat + scale Vec3) not the
   "matrix4" representation; document Bevy version requirement.
3. **Quaternion convention** — Bevy uses `[x, y, z, w]` for quaternions. Mitigation: tests
   assert the exact JSON shape.
4. **Asset paths at runtime** — the export says `assets/player.png` but the Bevy app must have
   the asset available. Out of scope for Hito 0 (asset bundling is Hito 1).

## Validation Strategy
- Unit tests in `dynamic_scene.rs` (≥10 tests, each Given/When/Then from the Spec).
- WASM smoke test: `frontend/tests/export.spec.ts` invokes
  `window.export_dynamic_scene(json_str)` and asserts on the response.
- Determinism test: export the same document twice, compare bytes.
- The existing `engine.spec.ts` and `smoke.spec.ts` must continue to pass (regression).

## Effort
A-lite path. 1 cycle, ~4–6 hours of work. 1 PR. Tag v0.6.0.
