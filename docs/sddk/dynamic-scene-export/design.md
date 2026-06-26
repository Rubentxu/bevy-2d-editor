# DynamicScene Export — Design

## Cycle: `dynamic-scene-export`
**Status:** draft → review
**ADR:** ADR-0004 (to be drafted alongside this design)

---

## Architecture

```
┌─────────────────┐
│ SceneDocument   │ (editor-core/src/document.rs — existing)
└────────┬────────┘
         │ &SceneDocument
         ▼
┌─────────────────────────────────────────────┐
│ dynamic_scene::export_dynamic_scene(doc)    │ (NEW module)
│   ├── builds Map<StableId, EntityExport>    │
│   ├── for each entity: resolve parent,      │
│   │   walk components, map to bevy.*        │
│   ├── collect warnings                      │
│   └── serialize to canonical JSON           │
└────────┬────────────────────────────────────┘
         │ DynamicSceneExport { json, warnings }
         ▼
┌─────────────────────────────────────────────┐
│ lib.rs::export_dynamic_scene(doc_json)      │ (NEW WASM binding)
│   ├── parse doc_json → SceneDocument        │
│   ├── call dynamic_scene::export_dynamic_.. │
│   └── return JsValue { json, warnings }     │
└─────────────────────────────────────────────┘
         │
         ▼
   window.exportDynamicScene(jsonString) → { json, warnings }
```

## Module: `crates/editor-core/src/dynamic_scene.rs`

### Public types

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use crate::document::{SceneDocument, Entity, ComponentInstance, StableId, Anchor};

/// Top-level export artifact — the JSON-serializable Bevy-compatible scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicSceneExport {
    /// Format version. Bump when breaking the JSON schema.
    pub version: String,
    /// The source SceneDocument's scene_id, for traceability.
    pub source_scene_id: String,
    /// All entities, keyed by their StableId. BTreeMap ensures deterministic ordering.
    pub entities: Vec<EntityExport>,
    /// Non-fatal issues encountered during the mapping.
    pub warnings: Vec<ExportWarning>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityExport {
    pub stable_id: String,
    pub name: String,
    pub parent_stable_id: Option<String>,
    /// Component key is the Bevy type name with `bevy.` prefix.
    /// E.g., "bevy.Name", "bevy.Transform", "bevy.Sprite".
    pub components: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportWarning {
    pub entity_stable_id: Option<String>,
    pub component_type_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Failed to parse SceneDocument JSON: {0}")]
    ParseError(String),
    #[error("Failed to serialize export: {0}")]
    SerializeError(String),
}

/// Export version constant. Bumped on breaking schema changes.
pub const EXPORT_VERSION: &str = "0.1.0";

/// Export a SceneDocument to a Bevy-compatible JSON artifact.
///
/// Never fails on business-rule violations — only on parse/serialize errors.
/// All issues (missing assets, unknown schemas, invalid values) are recorded as warnings.
pub fn export_dynamic_scene(doc: &SceneDocument) -> Result<DynamicSceneExport, ExportError> {
    // 1. Build stable_id → Bevy anchor context (validation pass).
    let valid_ids: std::collections::HashSet<&str> =
        doc.entities.iter().map(|e| e.id.as_str()).collect();

    let mut warnings: Vec<ExportWarning> = Vec::new();
    let mut entity_exports: Vec<EntityExport> = Vec::with_capacity(doc.entities.len());

    // 2. For each entity, build its export.
    for entity in &doc.entities {
        let parent_stable_id = resolve_parent(entity, &valid_ids, &mut warnings);
        let components = map_components(entity, &mut warnings);
        entity_exports.push(EntityExport {
            stable_id: entity.id.to_string(),
            name: entity.name.clone(),
            parent_stable_id,
            components,
        });
    }

    Ok(DynamicSceneExport {
        version: EXPORT_VERSION.to_string(),
        source_scene_id: doc.scene_id.clone(),
        entities: entity_exports,
        warnings,
    })
}

/// Helper: resolve the parent_stable_id, promoting orphans to root with warning.
fn resolve_parent<'a>(
    entity: &'a Entity,
    valid_ids: &std::collections::HashSet<&str>,
    warnings: &mut Vec<ExportWarning>,
) -> Option<String> {
    match &entity.parent {
        None => None,
        Some(p) => {
            if valid_ids.contains(p.as_str()) {
                Some(p.to_string())
            } else {
                warnings.push(ExportWarning {
                    entity_stable_id: Some(entity.id.to_string()),
                    component_type_id: None,
                    message: format!(
                        "Entity {} parent {} not found in scene; exporting as root",
                        entity.id, p
                    ),
                });
                None
            }
        }
    }
}

/// Helper: map components for one entity. Returns a deterministic-order BTreeMap.
fn map_components(
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> BTreeMap<String, serde_json::Value> {
    let mut out: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for component in &entity.components {
        match component.type_id.as_str() {
            "editor.Name" => {
                out.insert("bevy.Name".to_string(), map_name(component, entity, warnings));
            }
            "editor.Transform2D" => {
                out.insert(
                    "bevy.Transform".to_string(),
                    map_transform(component, entity, warnings),
                );
            }
            "editor.Sprite2D" => {
                if let Some(sprite) = map_sprite(component, entity, warnings) {
                    out.insert("bevy.Sprite".to_string(), sprite);
                }
                // map_sprite handles the "empty asset" warning itself; nothing else to do.
            }
            "editor.Visible" | "editor.Locked" => {
                // Editorial-only: silently skipped per §9.5.
            }
            unknown => {
                warnings.push(ExportWarning {
                    entity_stable_id: Some(entity.id.to_string()),
                    component_type_id: Some(unknown.to_string()),
                    message: format!(
                        "Skipping unknown component '{}' on entity {}",
                        unknown, entity.id
                    ),
                });
            }
        }
    }

    out
}

fn map_name(
    component: &ComponentInstance,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> serde_json::Value {
    let name = component
        .values
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Name".to_string()),
                message: format!(
                    "Name missing on entity {}; using empty string",
                    entity.id
                ),
            });
            ""
        });
    serde_json::json!({ "name": name })
}

fn map_transform(
    component: &ComponentInstance,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> serde_json::Value {
    // translation
    let translation = component
        .values
        .get("translation")
        .and_then(|v| v.get("x").and_then(|x| x.as_f64()).zip(v.get("y").and_then(|y| y.as_f64())))
        .map(|(x, y)| [x as f32, y as f32, 0.0_f32])
        .unwrap_or_else(|| {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Transform2D".to_string()),
                message: format!(
                    "Transform2D translation invalid on entity {}; using (0, 0, 0)",
                    entity.id
                ),
            });
            [0.0_f32, 0.0_f32, 0.0_f32]
        });

    // rotation (radians around z)
    let rotation_rad = component
        .values
        .get("rotation")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let half = rotation_rad * 0.5;
    let (s, c) = half.sin_cos();
    // Bevy quaternion [x, y, z, w]
    let rotation = [0.0_f32, 0.0_f32, s, c];

    // scale
    let scale = component
        .values
        .get("scale")
        .and_then(|v| v.get("x").and_then(|x| x.as_f64()).zip(v.get("y").and_then(|y| y.as_f64())))
        .map(|(x, y)| [x as f32, y as f32, 1.0_f32])
        .unwrap_or([1.0_f32, 1.0_f32, 1.0_f32]);

    serde_json::json!({
        "translation": translation,
        "rotation": rotation,
        "scale": scale,
    })
}

fn map_sprite(
    component: &ComponentInstance,
    entity: &Entity,
    warnings: &mut Vec<ExportWarning>,
) -> Option<serde_json::Value> {
    let asset = component
        .values
        .get("asset")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if asset.is_empty() {
        warnings.push(ExportWarning {
            entity_stable_id: Some(entity.id.to_string()),
            component_type_id: Some("editor.Sprite2D".to_string()),
            message: format!(
                "Sprite2D on entity {} has empty asset path; Sprite omitted",
                entity.id
            ),
        });
        return None;
    }

    // color (defaults to white if missing/invalid)
    let color = component
        .values
        .get("color")
        .and_then(|v| {
            let r = v.get("r").and_then(|x| x.as_f64())? as f32;
            let g = v.get("g").and_then(|x| x.as_f64())? as f32;
            let b = v.get("b").and_then(|x| x.as_f64())? as f32;
            let a = v.get("a").and_then(|x| x.as_f64())? as f32;
            Some([r, g, b, a])
        })
        .unwrap_or_else(|| {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Sprite2D".to_string()),
                message: format!(
                    "Sprite2D color invalid or missing on entity {}; using white",
                    entity.id
                ),
            });
            [1.0_f32, 1.0_f32, 1.0_f32, 1.0_f32]
        });

    // anchor (defaults to Center if invalid)
    let anchor = component
        .values
        .get("anchor")
        .and_then(|v| v.as_str())
        .and_then(anchor_str_to_bevy)
        .unwrap_or_else(|| {
            warnings.push(ExportWarning {
                entity_stable_id: Some(entity.id.to_string()),
                component_type_id: Some("editor.Sprite2D".to_string()),
                message: format!(
                    "Sprite2D anchor invalid on entity {}; using Center",
                    entity.id
                ),
            });
            "Center"
        });

    Some(serde_json::json!({
        "asset": asset,
        "color": color,
        "anchor": anchor,
    }))
}

/// Map our `Anchor` enum to Bevy's `bevy_sprite::Anchor` PascalCase string.
/// Returns `None` if the string doesn't match any known anchor.
fn anchor_str_to_bevy(s: &str) -> Option<&'static str> {
    Some(match s {
        "Center" => "Center",
        "TopLeft" => "TopLeft",
        "TopRight" => "TopRight",
        "BottomLeft" => "BottomLeft",
        "BottomRight" => "BottomRight",
        "TopCenter" => "TopCenter",
        "BottomCenter" => "BottomCenter",
        "CenterLeft" => "CenterLeft",
        "CenterRight" => "CenterRight",
        _ => return None,
    })
}
```

### Determinism
- `BTreeMap<String, serde_json::Value>` for the components map → alphabetical key order.
- Entities in `Vec<EntityExport>` follow the input `doc.entities` order.
- All values serialized via `serde_json::to_string` with default options (no pretty-printing).
- Tests assert byte-for-byte equality.

## WASM Binding: `crates/editor-core/src/lib.rs`

```rust
mod dynamic_scene;
pub use dynamic_scene::{export_dynamic_scene, DynamicSceneExport, ExportWarning};

#[wasm_bindgen]
pub fn export_dynamic_scene_wasm(doc_json: &str) -> Result<JsValue, JsValue> {
    let doc: SceneDocument = serde_json::from_str(doc_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    let export = dynamic_scene::export_dynamic_scene(&doc)
        .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))?;
    serde_wasm_bindgen::to_value(&export)
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
}
```

> **Naming note:** The WASM binding is named `export_dynamic_scene_wasm` to avoid clashing
> with the re-exported `export_dynamic_scene` function from the module. Frontend code calls
> `window.export_dynamic_scene_wasm(jsonString)`.

### Type marshalling (WASM ↔ JS)

`serde_wasm_bindgen::to_value` on `DynamicSceneExport` works because the struct contains only:
- `String` → JS string
- `Vec<EntityExport>` → JS array
- `BTreeMap<String, serde_json::Value>` → JS object (keys are strings, values are `serde_json::Value` which marshals correctly to JS primitives/arrays/objects)
- `Vec<ExportWarning>` → JS array of plain objects

No `serde_json::Value` nested inside the component values triggers the "empty values" bug from
`get_scene_snapshot` because the WASM return value is being fully serialized, not partially
re-marshalled. (This is the same pattern that worked for `dispatch_command`'s payloads.)

## Frontend Bridge: `frontend/src/engine-bridge.ts`

Add a typed helper:

```typescript
export interface ExportWarning {
  entity_stable_id: string | null;
  component_type_id: string | null;
  message: string;
}

export interface DynamicSceneExportResult {
  version: string;
  source_scene_id: string;
  entities: Array<{
    stable_id: string;
    name: string;
    parent_stable_id: string | null;
    components: Record<string, unknown>;
  }>;
  warnings: ExportWarning[];
}

export async function exportDynamicScene(sceneJson: string): Promise<DynamicSceneExportResult> {
  const fn = (window as any).export_dynamic_scene_wasm;
  if (typeof fn !== 'function') {
    throw new Error('export_dynamic_scene_wasm not available');
  }
  return await fn(sceneJson);
}
```

## Anchor Decision (ADR-0004)

### Context
Hito 0 §9.5 (line 357) says: `editor.Sprite2D.values.anchor` → "Computed `Transform` offset".

### Decision
**Use Bevy 0.19's native `Sprite::anchor` field (added in Bevy 0.14).** Map our 9 anchor strings
directly to Bevy's 9 anchor strings (same PascalCase names). Do NOT compute a Transform offset.

### Why
1. **Simpler.** No need to know sprite size to compute offset. Sprite size would need to be
   part of the schema.
2. **Correct.** Bevy's native anchor is the canonical way Bevy games position sprites.
3. **Spec language was aspirational.** §9.5 was written before we knew Bevy 0.14+ has native
   anchor support. The functional behavior matches §9.5's intent: "anchor determines sprite
   position".

### Consequences
- The export `bevy.Sprite.anchor` is a string, not a Transform offset.
- Any external Bevy loader should use `Sprite::anchor = Anchor::from_str(...)` when spawning.
- The preview world's `spawn_entity` should also use Bevy native anchor (current code ignores
  anchor entirely — that's a TODO for a follow-up cycle, not this one).

## Error Rules (from §9.5)
- Missing asset → warning, no `bevy.Sprite` emitted.
- Unknown schema → warning, component omitted.
- Unknown field → preserved in SceneDocument (ADR-0003), NOT exported.
- Empty document → empty `entities`, no warnings.

## Test Strategy

### Unit tests (≥13, in `dynamic_scene.rs` `#[cfg(test)] mod tests`)
Each Given/When/Then scenario from spec.md → at least one `#[test]`:
- test_export_empty_document
- test_export_name_component
- test_export_transform_translation_z_zero
- test_export_transform_rotation_quaternion
- test_export_transform_scale_z_one
- test_export_sprite_all_fields
- test_export_sprite_all_9_anchors (parameterized)
- test_export_sprite_empty_asset_warning
- test_export_sprite_missing_color_default
- test_export_sprite_invalid_anchor_default
- test_export_editorial_components_silent
- test_export_unknown_component_warning
- test_export_parent_child_hierarchy
- test_export_orphan_promoted_to_root
- test_export_invalid_vec2_default
- test_export_default_transform
- test_export_deterministic
- test_export_50_entities

### Playwright test (`frontend/tests/export.spec.ts`, 3 tests)
- empty document exports empty
- all 3 components export with correct mapping
- missing asset surfaces warning in console

### Regression
- `cargo test --workspace` — all existing 112 unit tests still pass.
- `frontend/tests/engine.spec.ts` + `frontend/tests/smoke.spec.ts` — all 26 still pass.

## Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| `serde_wasm_bindgen::to_value` fails on `BTreeMap<String, serde_json::Value>` | Medium | Fall back to `JsValue::from_str(&serde_json::to_string(&export)?)` + JS `JSON.parse()` like `get_scene_snapshot` does. |
| Bevy 0.19 changes Transform serialization | Low | Pin Bevy version in Cargo.toml. Document in ADR-0004. |
| Quaternion convention mismatch | Low | Test asserts exact `[0,0,sin(half),cos(half)]` shape. |
| Anchor enum drift (we add anchor, Bevy doesn't) | Very Low | Tests cover all 9 anchors explicitly. |

## Out of Scope
- Updating the preview world's `spawn_entity` to use Bevy native anchor (current code ignores
  anchor — works for visual preview, doesn't match runtime export. TODO follow-up cycle.)
- A separate external Bevy binary that loads the JSON (Hito 1).
- Asset bundling (Hito 1).
- Component versioning (Hito 1+).
- Live link / hot reload (never in scope for editor core).
