# Design: OPFS Persistence

> Change: `opfs-persistence` · Phase: sddk-design · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

---

## §1. Module Layout

```
crates/editor-core/src/
├── lib.rs                (modified) — wasm_bindgen extern + high-level functions
├── persistence.rs        (new) — Rust helper: paths, project metadata, save/load logic

frontend/src/
├── opfs-bridge.ts        (new) — OPFS wrapper module
├── engine-bridge.ts      (modified) — expose OPFS functions on window
```

## §2. Architecture

### §2.1 JS Bridge (opfs-bridge.ts)

```typescript
const OPFS_ROOT = "bevy-2d-editor";  // namespace inside OPFS root

async function getRoot(): Promise<FileSystemDirectoryHandle> {
  const root = await navigator.storage.getDirectory();
  return root.getDirectoryHandle(OPFS_ROOT, { create: true });
}

async function getSubdir(...path: string[]): Promise<FileSystemDirectoryHandle> {
  let dir = await getRoot();
  for (const segment of path) {
    dir = await dir.getDirectoryHandle(segment, { create: true });
  }
  return dir;
}

export async function opfsSaveFile(path: string, contents: string): Promise<{ok: boolean, error?: string}> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter(s => s.length > 0);
    const filename = segments.pop()!;
    const dir = await getSubdir(...segments);
    const fileHandle = await dir.getFileHandle(filename, { create: true });
    const writable = await fileHandle.createWritable();
    await writable.write(contents);
    await writable.close();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

export async function opfsLoadFile(path: string): Promise<{ok: boolean, value?: string, error?: string}> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter(s => s.length > 0);
    const filename = segments.pop()!;
    const dir = await getSubdir(...segments);
    const fileHandle = await dir.getFileHandle(filename, { create: false });
    const file = await fileHandle.getFile();
    const text = await file.text();
    return { ok: true, value: text };
  } catch (e) {
    if (e instanceof DOMException && e.name === "NotFoundError") {
      return { ok: false, error: "File not found" };
    }
    return { ok: false, error: String(e) };
  }
}

export async function opfsListFiles(path: string): Promise<{ok: boolean, value?: string[], error?: string}> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter(s => s.length > 0);
    const dir = await getSubdir(...segments);
    const files: string[] = [];
    for await (const [name, handle] of dir.entries()) {
      if (handle.kind === "file") files.push(name);
    }
    return { ok: true, value: files };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

export async function opfsExists(path: string): Promise<boolean> {
  try {
    const segments = path.split("/").filter(s => s.length > 0);
    const filename = segments.pop()!;
    const dir = await getSubdir(...segments);
    await dir.getFileHandle(filename, { create: false });
    return true;
  } catch {
    return false;
  }
}
```

### §2.2 wasm_bindgen extern declarations (Rust)

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = opfs_save_file, catch)]
    pub async fn opfs_save_file(path: &str, contents: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_load_file, catch)]
    pub async fn opfs_load_file(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_list_files, catch)]
    pub async fn opfs_list_files(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = window, js_name = opfs_exists)]
    pub async fn opfs_exists(path: &str) -> bool;
}
```

### §2.3 High-level Rust API (lib.rs)

```rust
const PROJECT_FILE: &str = "project.json";
const SCENES_DIR: &str = "scenes";

/// Save the current SceneDocument to OPFS at scenes/<name>.scene.json.
/// Creates project.json if it doesn't exist.
#[wasm_bindgen]
pub async fn save_scene(name: &str) -> Result<String, JsValue> {
    // 1. Get current SCENE_DOC, serialize to JSON
    let doc_json = SCENE_DOC.with(|s| {
        let doc_ref = s.borrow();
        let doc = doc_ref.as_ref().ok_or_else(|| JsValue::from_str("No scene loaded"))?;
        serde_json::to_string(doc).map_err(|e| JsValue::from_str(&e.to_string()))
    })?;

    // 2. Write to scenes/<name>.scene.json
    let scene_path = format!("{}/{}.scene.json", SCENES_DIR, name);
    let save_result = opfs_save_file(&scene_path, &doc_json).await?;
    let save_obj: serde_json::Value = serde_json::from_value(save_result)
        .map_err(|e| JsValue::from_str(&format!("Bad bridge response: {}", e)))?;
    if save_obj.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(JsValue::from_str(&format!("Save failed: {}", save_obj.get("error").unwrap_or(&serde_json::Value::Null))));
    }

    // 3. Update project.json with scene entry
    update_project_metadata(name).await?;

    Ok(scene_path)
}

/// Load a SceneDocument from OPFS into SCENE_DOC.
#[wasm_bindgen]
pub async fn load_scene(name: &str) -> Result<(), JsValue> {
    let scene_path = format!("{}/{}.scene.json", SCENES_DIR, name);
    let load_result = opfs_load_file(&scene_path).await?;
    let load_obj: serde_json::Value = serde_json::from_value(load_result)
        .map_err(|e| JsValue::from_str(&format!("Bad bridge response: {}", e)))?;
    if load_obj.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(JsValue::from_str(&format!("Load failed: {}", load_obj.get("error").unwrap_or(&serde_json::Value::Null))));
    }
    let json_str = load_obj.get("value").and_then(|v| v.as_str())
        .ok_or_else(|| JsValue::from_str("Missing value in bridge response"))?;

    let doc: SceneDocument = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    SCENE_DOC.with(|s| *s.borrow_mut() = Some(doc));
    mark_dirty();
    Ok(())
}

/// List all scene names from project.json.
#[wasm_bindgen]
pub async fn list_scenes() -> Result<JsValue, JsValue> {
    if !opfs_exists(PROJECT_FILE).await {
        return Ok(serde_wasm_bindgen::to_value(&Vec::<String>::new())?);
    }
    let result = opfs_load_file(PROJECT_FILE).await?;
    let obj: serde_json::Value = serde_json::from_value(result)
        .map_err(|e| JsValue::from_str(&format!("Bad bridge response: {}", e)))?;
    if obj.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(JsValue::from_str(&format!("Read failed: {}", obj.get("error").unwrap_or(&serde_json::Value::Null))));
    }
    let json_str = obj.get("value").and_then(|v| v.as_str())
        .ok_or_else(|| JsValue::from_str("Missing value"))?;
    let project: ProjectMetadata = serde_json::from_str(json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    serde_wasm_bindgen::to_value(&project.scenes).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Check if project.json exists in OPFS.
#[wasm_bindgen]
pub async fn project_exists() -> bool {
    opfs_exists(PROJECT_FILE).await
}

async fn update_project_metadata(scene_name: &str) -> Result<(), JsValue> {
    let project = if opfs_exists(PROJECT_FILE).await {
        let result = opfs_load_file(PROJECT_FILE).await?;
        let obj: serde_json::Value = serde_json::from_value(result)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        if obj.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            let json_str = obj.get("value").and_then(|v| v.as_str()).unwrap_or("{}");
            serde_json::from_str::<ProjectMetadata>(json_str).unwrap_or_default()
        } else {
            ProjectMetadata::default()
        }
    } else {
        ProjectMetadata::default()
    };
    
    let mut project = project;
    if !project.scenes.contains(&scene_name.to_string()) {
        project.scenes.push(scene_name.to_string());
    }
    
    let json = serde_json::to_string(&project)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let result = opfs_save_file(PROJECT_FILE, &json).await?;
    let obj: serde_json::Value = serde_json::from_value(result)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    if obj.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(JsValue::from_str(&format!("Save project.json failed: {}", obj.get("error").unwrap_or(&serde_json::Value::Null))));
    }
    Ok(())
}
```

## §3. Project Metadata Type

```rust
// In persistence.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub version: String,
    pub name: String,
    pub scenes: Vec<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            version: "0.1".to_string(),
            name: "Untitled Project".to_string(),
            scenes: Vec::new(),
        }
    }
}
```

## §4. Path Convention

| Path | Content |
|---|---|
| `project.json` (OPFS root) | Project metadata |
| `scenes/<name>.scene.json` | SceneDocument |
| `schemas/...` (future) | Component schemas |
| `assets/...` (future) | Binary assets |
| `entities/...` (future) | Entity templates |
| `.editor/...` (future) | Editor state |

All paths relative to OPFS root. OPFS root is named `bevy-2d-editor` to namespace it.

## §5. Error Handling

- JS bridge returns `{ok: bool, value?, error?}` JSON
- Rust parses JSON and converts to `Result<T, JsValue>`
- Errors propagate up via `JsValue::from_str(message)`
- OPFS unavailable → typed error from JS bridge
- File not found → `NotFoundError` caught, returned as `Err`
- Quota exceeded → propagated as `Err`

## §6. Backward Compatibility

- All existing wasm_bindgen functions unchanged (`create_buses`, `load_scene_json`, `dispatch_command`, `undo`, `redo`, `get_log_state`)
- OPFS is **additive** — only used when explicitly called
- Existing 16 Playwright tests + 79 Rust tests pass unchanged
- OPFS namespace `bevy-2d-editor` isolates from other apps using same origin

## §7. Test Strategy

### Rust unit tests (`persistence.rs`)
- Mock `opfs_*` externs (use a local function pointer substitution pattern OR feature-gate for test builds)
- Test path construction: `format!("{}/{}.scene.json", "scenes", name)` → `"scenes/foo.scene.json"`
- Test ProjectMetadata default
- Test ProjectMetadata serialization roundtrip

Note: actual OPFS testing requires browser. Rust unit tests cover path resolution and metadata logic.

### Playwright E2E (`engine.spec.ts`)
- New test: `save and load scene roundtrip with 50+ entities`
  1. `load_scene_json(testScene)` with 50 entities
  2. `save_scene("e2e_test")` 
  3. Reload page
  4. `load_scene("e2e_test")`
  5. Verify scene has 50 entities
- New test: `list_scenes returns saved scenes`
  1. Save 3 scenes with different names
  2. `list_scenes()` returns array with all 3 names

### Existing tests
- All 16 existing Playwright tests pass unchanged (OPFS not called)
- All 79 existing Rust tests pass unchanged

## §8. Performance Notes

- OPFS async API: ~5-50ms per file operation (small JSON files)
- 50-entity scene: ~30KB JSON, <10ms save/load
- No caching needed for MVP (each save/load is direct)
- Future: in-memory cache + dirty tracking

## §9. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| OPFS unavailable in browser | Feature detection in JS bridge, typed error |
| Quota exceeded | Catch in JS bridge, surface as `Err` |
| `serde_wasm_bindgen::to_value` complexity | Alternative: serialize to JSON string, return JsValue::from_str |
| Async wasm_bindgen not supported | Use `#[wasm_bindgen]` async syntax (requires wasm-bindgen-futures) |
| OPFS state not cleared between tests | Playwright launches fresh browser context per test |
| Race conditions | Single-threaded WASM, no contention in MVP |

## §10. Open Questions

1. **`serde_wasm_bindgen` dependency** — Required for `Vec<String>` return. Adds a dep. Alternative: return JSON string and parse in JS.
2. **`wasm-bindgen-futures`** — Required for `async` wasm_bindgen. Already a dep of wasm-bindgen itself? Need to verify.
3. **OPFS permission** — Some browsers prompt user. We accept that for MVP.
4. **Auto-save** — Out of scope. Future change.

## §11. wasm-bindgen-futures

The `async fn` syntax in `#[wasm_bindgen]` requires `wasm-bindgen-futures`. Check if it's already a transitive dep via `wasm-bindgen 0.2`. If not, add to Cargo.toml.

Looking at wasm-bindgen 0.2 docs: `wasm-bindgen-futures` is a separate crate. Need to add it.