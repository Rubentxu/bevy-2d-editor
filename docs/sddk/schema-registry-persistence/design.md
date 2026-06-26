# Design: Schema Registry Persistence

> Change: `schema-registry-persistence` · Phase: sddk-design · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

---

## §1. Module Layout

```
crates/editor-core/src/
├── schema.rs           (modified) — Add register/unregister + mutable state
├── persistence.rs      (modified) — Add schemas_dir, schema_path, extend ProjectMetadata
├── processor.rs        (modified) — Use combined_registry() for validation
└── lib.rs              (modified) — wasm_bindgen functions, USER_SCHEMAS thread_local

frontend/src/
├── engine-bridge.ts    (modified) — Expose schema persistence on window
└── tests/engine.spec.ts (modified) — Add 2 E2E tests
```

## §2. Architecture: Two-Layer Registry

```
┌──────────────────────────────────────────────────────────┐
│              combined_registry()                         │
│                                                          │
│   ┌─────────────────────┐   ┌─────────────────────────┐  │
│   │  BUILT-IN REGISTRY  │ + │  USER SCHEMAS REGISTRY  │  │
│   │  (OnceLock, immutable)│   │  (RefCell, mutable)    │  │
│   │                     │   │                         │  │
│   │  - editor.Name      │   │  - game.PlayerHealth   │  │
│   │  - editor.Transform2D   │   │  - game.EnemyAI       │  │
│   │  - editor.Sprite2D  │   │                         │  │
│   │  - editor.Visible   │   │                         │  │
│   │  - editor.Locked    │   │                         │  │
│   └─────────────────────┘   └─────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

## §3. Type Design

### §3.1 Extended `ProjectMetadata`

```rust
// In persistence.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub version: String,
    pub name: String,
    pub scenes: Vec<String>,
    #[serde(default)]
    pub schemas: Vec<String>,  // NEW
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            version: "0.1".to_string(),
            name: "Untitled Project".to_string(),
            scenes: Vec::new(),
            schemas: Vec::new(),
        }
    }
}
```

### §3.2 Schema path helpers

```rust
// In persistence.rs
pub const SCHEMAS_DIR: &str = "schemas";

pub fn schema_path(type_id: &str) -> String {
    format!("{}/{}.schema.json", SCHEMAS_DIR, type_id)
}
```

### §3.3 Mutable user schema registry

```rust
// In schema.rs (additions)
thread_local! {
    static USER_SCHEMAS: RefCell<ComponentSchemaRegistry> = 
        const { RefCell::new(ComponentSchemaRegistry::new()) };
}

pub fn is_builtin_type(type_id: &str) -> bool {
    type_id.starts_with("editor.")
}

pub fn register_schema(schema: ComponentSchema) -> Result<(), SchemaError> {
    if is_builtin_type(&schema.type_id) {
        return Err(SchemaError::CannotRegisterBuiltin);
    }
    USER_SCHEMAS.with(|r| {
        let mut reg = r.borrow_mut();
        reg.insert(schema);
    });
    Ok(())
}

pub fn unregister_schema(type_id: &str) -> Result<(), SchemaError> {
    if is_builtin_type(type_id) {
        return Err(SchemaError::CannotUnregisterBuiltin);
    }
    USER_SCHEMAS.with(|r| {
        r.borrow_mut().remove(type_id);  // ignore if not present
    });
    Ok(())
}

pub fn combined_registry() -> ComponentSchemaRegistry {
    let mut combined = ComponentSchemaRegistry::new();
    // Copy built-ins
    for schema in global_registry().iter() {
        combined.insert(schema.clone());
    }
    // Add user schemas (override built-ins if same type_id)
    USER_SCHEMAS.with(|r| {
        for schema in r.borrow().iter() {
            combined.insert(schema.clone());
        }
    });
    combined
}

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Cannot register built-in schema: {0}")]
    CannotRegisterBuiltin(String),
    #[error("Cannot unregister built-in schema: {0}")]
    CannotUnregisterBuiltin(String),
    #[error("Schema not found: {0}")]
    NotFound(String),
}
```

`ComponentSchemaRegistry` needs a `remove(type_id)` method. Let me add it:

```rust
impl ComponentSchemaRegistry {
    pub fn remove(&mut self, type_id: &str) -> Option<ComponentSchema> {
        self.schemas.remove(type_id)
    }
}
```

## §4. WASM Surface

### §4.1 New wasm_bindgen functions (lib.rs)

```rust
/// Save a schema to OPFS at `schemas/<type_id>.schema.json`.
#[wasm_bindgen]
pub async fn save_schema(type_id: &str) -> Result<String, JsValue> {
    let schema_json = get_schema_json(type_id)?;  // looks in combined registry
    let path = persistence::schema_path(type_id);
    js_save_file(&path, &schema_json).await.map_err(JsValue::from_str)?;
    update_project_schemas(type_id, true).await.map_err(JsValue::from_str)?;
    Ok(path)
}

/// Load a schema from OPFS and register it.
#[wasm_bindgen]
pub async fn load_schema(type_id: &str) -> Result<(), JsValue> {
    let path = persistence::schema_path(type_id);
    let json_str = js_load_file(&path).await.map_err(JsValue::from_str)?;
    let schema: ComponentSchema = serde_json::from_str(&json_str)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    schema::register_schema(schema).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(())
}

/// Delete a schema (built-ins protected).
#[wasm_bindgen]
pub async fn delete_schema(type_id: &str) -> Result<(), JsValue> {
    if schema::is_builtin_type(type_id) {
        return Err(JsValue::from_str("Cannot delete built-in schema"));
    }
    let path = persistence::schema_path(type_id);
    // Delete OPFS file via new bridge function opfs_delete_file
    opfs_delete_file_raw(&path);
    js_await(opfs_delete_file_raw(&path)).await.map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    schema::unregister_schema(type_id).map_err(|e| JsValue::from_str(&e.to_string()))?;
    update_project_schemas(type_id, false).await.map_err(JsValue::from_str)?;
    Ok(())
}

/// List all schemas (built-in + user).
#[wasm_bindgen]
pub async fn list_schemas() -> Result<JsValue, JsValue> {
    let combined = schema::combined_registry();
    let type_ids: Vec<String> = combined.iter().map(|s| s.type_id.clone()).collect();
    serde_wasm_bindgen::to_value(&type_ids).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Register a schema from JSON (in-memory, no OPFS save).
#[wasm_bindgen]
pub fn register_schema_from_json(schema_json: &str) -> Result<(), JsValue> {
    let schema: ComponentSchema = serde_json::from_str(schema_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    schema::register_schema(schema).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(())
}

/// Unregister a schema (built-ins protected).
#[wasm_bindgen]
pub fn unregister_schema(type_id: &str) -> Result<(), JsValue> {
    schema::unregister_schema(type_id).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Check if a type_id is a built-in.
#[wasm_bindgen]
pub fn is_builtin_type(type_id: &str) -> bool {
    schema::is_builtin_type(type_id)
}

/// Combined registry size (for UI).
#[wasm_bindgen]
pub fn combined_registry_size() -> usize {
    schema::combined_registry().iter().count()
}

/// Load complete project: project.json + scenes + schemas.
#[wasm_bindgen]
pub async fn load_project() -> Result<(), JsValue> {
    // 1. Read project.json
    if !js_exists(PROJECT_FILE).await {
        return Err(JsValue::from_str("project.json not found"));
    }
    let project_json = js_load_file(PROJECT_FILE).await.map_err(JsValue::from_str)?;
    let project: ProjectMetadata = serde_json::from_str(&project_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    // 2. Register all schemas first (so AddComponent works)
    for schema_id in &project.schemas {
        load_schema(schema_id).await.map_err(|e| 
            JsValue::from_str(&format!("Failed to load schema {}: {}", schema_id, e.as_string().unwrap_or_default()))
        )?;
    }
    
    // 3. Load first scene (or none)
    if let Some(first_scene) = project.scenes.first() {
        load_scene(first_scene).await?;
    }
    
    Ok(())
}
```

### §4.2 New OPFS bridge function (opfs-bridge.ts)

```typescript
export async function opfsDeleteFile(path: string): Promise<OpfsResult> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter((s) => s.length > 0);
    const filename = segments.pop();
    if (!filename) return { ok: false, error: "Invalid path" };
    const dir = await getSubdir(segments, false);
    if (!dir) return { ok: false, error: "File not found" };
    await dir.removeEntry(filename);
    return { ok: true };
  } catch (e) {
    if (e instanceof DOMException && e.name === "NotFoundError") {
      return { ok: false, error: "File not found" };
    }
    return { ok: false, error: String(e) };
  }
}
```

### §4.3 Helpers in lib.rs

```rust
fn get_schema_json(type_id: &str) -> Result<String, JsValue> {
    let combined = schema::combined_registry();
    let schema = combined.get(type_id)
        .ok_or_else(|| JsValue::from_str(&format!("Schema not found: {}", type_id)))?;
    serde_json::to_string(schema).map_err(|e| JsValue::from_str(&e.to_string()))
}

async fn update_project_schemas(type_id: &str, add: bool) -> Result<(), String> {
    let mut project = if js_exists(PROJECT_FILE).await {
        match js_load_file(PROJECT_FILE).await {
            Ok(json_str) => serde_json::from_str::<ProjectMetadata>(&json_str).unwrap_or_default(),
            Err(_) => ProjectMetadata::default(),
        }
    } else {
        ProjectMetadata::default()
    };
    
    if add {
        if !project.schemas.contains(&type_id.to_string()) {
            project.schemas.push(type_id.to_string());
        }
    } else {
        project.schemas.retain(|s| s != type_id);
    }
    
    let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    js_save_file(PROJECT_FILE, &json).await
}
```

## §5. Processor Integration

```rust
// In processor.rs validate()
pub fn validate(doc: &SceneDocument, cmd: &Command) -> Result<(), CommandError> {
    match cmd {
        Command::AddComponent { entity_id, type_id, .. } => {
            find_entity(doc, entity_id)?;
            if schema::combined_registry().get(type_id).is_none() {
                return Err(CommandError::UnknownSchema(type_id.clone()));
            }
        }
        // ... other commands
    }
    Ok(())
}
```

Replace `global_registry()` with `combined_registry()` in all 8 places where validation happens.

## §6. Backward Compatibility

- `global_registry()` still returns built-ins (5 schemas) — unchanged
- `combined_registry()` is the new combined view
- `processor::validate` now uses combined; existing 5 built-in schemas still validate
- ProjectMetadata gets `schemas: Vec<String>` with `#[serde(default)]` — old `project.json` files without the field still parse

## §7. Test Strategy

### Rust unit tests
- `test_is_builtin_type`: editor.* → true, game.* → false
- `test_register_schema_rejects_builtin`: register editor.NewName fails
- `test_register_schema_adds_user`: game.PlayerHealth registered
- `test_register_schema_replaces_existing`: register twice with different fields
- `test_unregister_schema_removes_user`: register → unregister → gone
- `test_unregister_schema_rejects_builtin`: unregister editor.Transform2D fails
- `test_unregister_schema_nonexistent_is_noop`: unregister missing returns Ok
- `test_combined_registry_includes_builtins`: 5 built-ins present
- `test_combined_registry_includes_user`: register user → combined has 6
- `test_schema_path_format`: `schemas/game.X.schema.json`
- `test_project_metadata_default_has_schemas`: default has empty schemas Vec
- `test_project_metadata_roundtrip_with_schemas`: serialize + deserialize preserves schemas

### Playwright E2E tests
- `register and validate custom schema end-to-end`
  1. Register `game.PlayerHealth` via `register_schema_from_json`
  2. Use `AddComponent` with type_id `game.PlayerHealth`
  3. Verify success
- `save and load schema roundtrip with auto-restore`
  1. Register + save `game.EnemyAI`
  2. Reload page
  3. Call `load_project()`
  4. Verify schema is available
  5. Use `AddComponent` with `game.EnemyAI` — succeeds

## §8. Performance Notes

- `combined_registry()` creates a new registry each call (HashMap copy). For MVP this is fine (5-50 schemas). Profile and optimize if needed.
- `USER_SCHEMAS` is a `RefCell` — single-threaded WASM, no contention
- Schema files are small JSON (~1-5KB each)

## §9. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| `combined_registry()` cost on every validate | Cache combined registry in `USER_SCHEMAS` rebuild; or invalidate cache on register/unregister |
| Schema file collision with built-ins | Built-in protection in register_schema |
| `opfsDeleteFile` race with `unregister_schema` | Both atomic operations; failure leaves consistent state |
| Project reload fails mid-way | `load_project` returns Err early; SCENE_DOC unchanged |

## §10. Open Questions

1. **`combined_registry()` caching** — Rebuild on each call vs cache with invalidation? MVP: rebuild (simpler). Optimize if profile shows hot.
2. **`opfsDeleteFile` bridge function** — New addition to opfs-bridge.ts.
3. **Schema validation on load** — When loading a schema, should we validate its fields against the schema's own field definitions? Out of scope for MVP (just deserialize).
4. **Schema file overwrite on save** — Existing file overwritten without warning. Document.
5. **Empty schema list in old project.json** — Handled by `#[serde(default)]` → empty Vec.