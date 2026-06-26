# Design: Entity Template Persistence + Instantiation

> Change: `entity-template-persistence` · Phase: sddk-design · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

---

## §1. Module Layout

```
crates/editor-core/src/
├── template.rs        (new) — EntityTemplate, TemplateEntity, validator, instantiator
├── persistence.rs     (modified) — entities_dir, template_path, ProjectMetadata.templates
├── processor.rs       (modified) — Full InstantiateEntityTemplate impl
└── lib.rs             (modified) — wasm_bindgen surface + cache + load_project
```

## §2. Type Design

### §2.1 EntityTemplate

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTemplate {
    pub template_id: String,
    pub display_name: String,
    pub version: String,             // "0.1"
    pub entities: Vec<TemplateEntity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateEntity {
    pub local_id: String,             // template-local ID (not a StableId!)
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_local_id: Option<String>, // references another entity's local_id
    pub components: Vec<ComponentInstance>,
}
```

### §2.2 StableId minter

```rust
use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

thread_local! {
    static ID_COUNTER: Cell<u64> = const { Cell::new(0) };
}

pub fn mint_stable_id() -> StableId {
    let counter = ID_COUNTER.with(|c| {
        let n = c.get() + 1;
        c.set(n);
        n
    });
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    StableId::new(format!("ent_{:x}_{:x}", timestamp_ms, counter))
}
```

### §2.3 Template validator

```rust
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template must have exactly one root, found {0}")]
    MultipleRoots(usize),
    #[error("Template has no entities")]
    EmptyTemplate,
    #[error("Template contains a cycle through {0}")]
    Cycle(String),
    #[error("Parent local_id '{0}' not found in template")]
    DanglingParent(String),
    #[error("Unknown component schema: {0}")]
    UnknownSchema(String),
    #[error("Template not loaded: {0}")]
    NotLoaded(String),
    #[error("JSON parse error: {0}")]
    Parse(String),
}

pub fn validate(template: &EntityTemplate) -> Result<(), TemplateError> {
    if template.entities.is_empty() {
        return Err(TemplateError::EmptyTemplate);
    }
    let roots: Vec<_> = template.entities.iter()
        .filter(|e| e.parent_local_id.is_none())
        .collect();
    if roots.len() != 1 {
        return Err(TemplateError::MultipleRoots(roots.len()));
    }
    // Build local_id → index map
    let map: HashMap<&str, usize> = template.entities.iter()
        .enumerate()
        .map(|(i, e)| (e.local_id.as_str(), i))
        .collect();
    // Dangling parent check
    for entity in &template.entities {
        if let Some(parent) = &entity.parent_local_id {
            if !map.contains_key(parent.as_str()) {
                return Err(TemplateError::DanglingParent(parent.clone()));
            }
        }
    }
    // Cycle detection: walk parent chain from each entity, ensure no back-edge
    for entity in &template.entities {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut current_local = entity.local_id.as_str();
        loop {
            visited.insert(current_local);
            let parent = match map.get(current_local)
                .and_then(|i| template.entities.get(*i))
                .and_then(|e| e.parent_local_id.as_deref())
            {
                Some(p) => p,
                None => break,
            };
            if visited.contains(parent) {
                return Err(TemplateError::Cycle(parent.to_string()));
            }
            current_local = parent;
        }
    }
    // Component schema validation
    let registry = schema::combined_registry();
    for entity in &template.entities {
        for component in &entity.components {
            if registry.get(&component.type_id).is_none() {
                return Err(TemplateError::UnknownSchema(component.type_id.clone()));
            }
        }
    }
    Ok(())
}
```

### §2.4 Template instantiator

```rust
pub fn instantiate(
    template: &EntityTemplate,
    target_parent: Option<&StableId>,
    doc: &mut SceneDocument,
) -> Result<Vec<StableId>, TemplateError> {
    // Mint fresh IDs for each template entity
    let local_to_minted: HashMap<&str, StableId> = template.entities.iter()
        .map(|e| (e.local_id.as_str(), mint_stable_id()))
        .collect();
    
    // Build all entities (parent refs initially None)
    let mut minted_entities: Vec<Entity> = template.entities.iter().map(|te| {
        let id = local_to_minted.get(te.local_id.as_str()).unwrap().clone();
        Entity {
            id,
            name: te.name.clone(),
            parent: None, // set later
            components: te.components.clone(),
        }
    }).collect();
    
    // Set parent references
    for (i, te) in template.entities.iter().enumerate() {
        if let Some(parent_local) = &te.parent_local_id {
            let parent_id = local_to_minted.get(parent_local.as_str()).unwrap().clone();
            minted_entities[i].parent = Some(parent_id);
        }
    }
    
    // Apply target_parent to root entity
    if let Some(target) = target_parent {
        // Root is the first entity with parent == None (only one)
        for entity in minted_entities.iter_mut() {
            if entity.parent.is_none() {
                entity.parent = Some(target.clone());
                break;
            }
        }
    }
    
    // Add to scene
    let minted_ids: Vec<StableId> = minted_entities.iter().map(|e| e.id.clone()).collect();
    doc.entities.extend(minted_entities);
    Ok(minted_ids)
}
```

## §3. In-Memory Template Cache

```rust
// In template.rs
thread_local! {
    static TEMPLATE_CACHE: RefCell<HashMap<String, EntityTemplate>> = RefCell::new(HashMap::new());
}

pub fn cache_template(template: EntityTemplate) {
    TEMPLATE_CACHE.with(|c| {
        c.borrow_mut().insert(template.template_id.clone(), template);
    });
}

pub fn get_cached_template(template_id: &str) -> Option<EntityTemplate> {
    TEMPLATE_CACHE.with(|c| c.borrow().get(template_id).cloned())
}

pub fn remove_cached_template(template_id: &str) -> Option<EntityTemplate> {
    TEMPLATE_CACHE.with(|c| c.borrow_mut().remove(template_id))
}

pub fn clear_template_cache() {
    TEMPLATE_CACHE.with(|c| c.borrow_mut().clear());
}
```

## §4. WASM Surface

```rust
// In lib.rs

/// Save an EntityTemplate to OPFS at entities/<template_id>.template.json.
#[wasm_bindgen]
pub fn save_template(template_id: &str, template_json: &str) -> Result<(), JsValue> {
    let template: EntityTemplate = serde_json::from_str(template_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    template::validate(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let path = persistence::template_path(template_id);
    let json = serde_json::to_string(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    // Synchronous OPFS save via thread_local! shared cache? No — needs async
    // We'll need a sync wrapper using promise.then + atomic wait
    // OR: cache the template, return immediately, async save in background
    // For MVP: save synchronously via spawn_local
    save_template_async(template_id, template, path, json);
    Ok(())
}

fn save_template_async(template_id: String, template: EntityTemplate, path: String, json: String) {
    wasm_bindgen_futures::spawn_local(async move {
        // js_save_file is async; spawn_local runs it on the same task queue
        if let Err(e) = js_save_file(&path, &json).await {
            web_sys::console::error_1(&format!("Template save failed: {}", e).into());
            return;
        }
        if let Err(e) = update_project_templates(&template_id, true).await {
            web_sys::console::error_1(&format!("Project update failed: {}", e).into());
        }
        template::cache_template(template);
    });
}

/// Load an EntityTemplate from OPFS and cache in memory.
#[wasm_bindgen]
pub async fn load_template(template_id: &str) -> Result<(), JsValue> {
    let path = persistence::template_path(template_id);
    let json = js_load_file(&path).await.map_err(|e| JsValue::from_str(&e))?;
    let template: EntityTemplate = serde_json::from_str(&json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    template::validate(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    template::cache_template(template);
    Ok(())
}

/// List all template IDs in OPFS.
#[wasm_bindgen]
pub async fn list_templates() -> Result<JsValue, JsValue> {
    let files = js_list_files(persistence::ENTITIES_DIR).await
        .map_err(|e| JsValue::from_str(&e))?;
    // Filter .template.json suffix
    let ids: Vec<String> = files.into_iter()
        .filter(|f| f.ends_with(".template.json"))
        .map(|f| f.trim_end_matches(".template.json").to_string())
        .collect();
    serde_wasm_bindgen::to_value(&ids).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Delete an EntityTemplate from OPFS and clear from cache.
#[wasm_bindgen]
pub async fn delete_template(template_id: &str) -> Result<(), JsValue> {
    let path = persistence::template_path(template_id);
    let promise = opfs_delete_file_raw(&path);
    js_await(promise).await.map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
    template::remove_cached_template(template_id);
    update_project_templates(template_id, false)
        .await
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(())
}

/// Check if a template is loaded in memory cache.
#[wasm_bindgen]
pub fn is_template_loaded(template_id: &str) -> bool {
    template::get_cached_template(template_id).is_some()
}
```

## §5. Processor Integration

Replace the stub in `processor.rs`:

```rust
Command::InstantiateEntityTemplate { template_id, target_parent } => {
    let template = template::get_cached_template(template_id)
        .ok_or_else(|| CommandError::TemplateNotFound(template_id.clone()))?;
    let minted_ids = template::instantiate(&template, target_parent.as_ref(), doc)?;
    // Inverse: delete all minted entities
    Ok(Command::DeleteEntity {
        id: minted_ids[0].clone(), // inverse is delete the root
    })
}
```

Wait — the inverse should delete all entities from the instantiation, not just the root. Since we use the inverse command to undo, we need a way to delete multiple. We can use Batch:

```rust
Command::InstantiateEntityTemplate { template_id, target_parent } => {
    let template = template::get_cached_template(template_id)
        .ok_or_else(|| CommandError::TemplateNotFound(template_id.clone()))?;
    let minted_ids = template::instantiate(&template, target_parent.as_ref(), doc)?;
    // Inverse: batch of DeleteEntity for each minted entity
    let inverse_commands: Vec<Command> = minted_ids.iter()
        .map(|id| Command::DeleteEntity { id: id.clone() })
        .collect();
    Ok(Command::Batch {
        label: format!("undo_instantiate_{}", template_id),
        commands: inverse_commands,
    })
}
```

## §6. load_project Extension

```rust
#[wasm_bindgen]
pub async fn load_project() -> Result<(), JsValue> {
    if !js_exists(PROJECT_FILE).await {
        return Err(JsValue::from_str("project.json not found"));
    }
    let project_json = js_load_file(PROJECT_FILE).await.map_err(JsValue::from_str)?;
    let project: ProjectMetadata = serde_json::from_str(&project_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    // 1. Register all schemas first
    for schema_id in &project.schemas {
        load_schema(schema_id).await?;
    }
    
    // 2. Load all templates into cache
    for template_id in &project.templates {
        load_template(template_id).await?;
    }
    
    // 3. Load first scene (or none)
    if let Some(first_scene) = project.scenes.first() {
        load_scene(first_scene).await?;
    }
    
    Ok(())
}
```

## §7. Persistence Helpers

```rust
// In persistence.rs
pub const ENTITIES_DIR: &str = "entities";

pub fn template_path(template_id: &str) -> String {
    format!("{}/{}.template.json", ENTITIES_DIR, template_id)
}
```

Extend `ProjectMetadata` with `templates: Vec<String>` field.

```rust
async fn update_project_templates(template_id: &str, add: bool) -> Result<(), String> {
    // Same pattern as update_project_schemas
    let mut project = if js_exists(PROJECT_FILE).await {
        match js_load_file(PROJECT_FILE).await {
            Ok(json_str) => serde_json::from_str::<ProjectMetadata>(&json_str).unwrap_or_default(),
            Err(_) => ProjectMetadata::default(),
        }
    } else {
        ProjectMetadata::default()
    };
    if add {
        if !project.templates.contains(&template_id.to_string()) {
            project.templates.push(template_id.to_string());
        }
    } else {
        project.templates.retain(|t| t != template_id);
    }
    let json = serde_json::to_string(&project).map_err(|e| e.to_string())?;
    js_save_file(PROJECT_FILE, &json).await
}
```

## §8. Async Strategy

`save_template` is sync (immediate OK) but actual OPFS write is async via `spawn_local`. Other operations (load, delete, list) are async.

**Trade-off:** Sync return with async save means save_template returns "OK" before file is on disk. For MVP this is acceptable. The cache is populated immediately so `instantiate_template` works. Document this.

Alternative: make `save_template` async too (await the file write). More correct but slower API. Let me make it async:

```rust
#[wasm_bindgen]
pub async fn save_template(template_id: &str, template_json: &str) -> Result<(), JsValue> {
    let template: EntityTemplate = serde_json::from_str(template_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    template::validate(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let path = persistence::template_path(template_id);
    let json = serde_json::to_string(&template).map_err(|e| JsValue::from_str(&e.to_string()))?;
    js_save_file(&path, &json).await.map_err(|e| JsValue::from_str(&e))?;
    update_project_templates(template_id, true).await
        .map_err(|e| JsValue::from_str(&e))?;
    template::cache_template(template);
    Ok(())
}
```

Cache + OPFS write both happen. Slower but correct.

## §9. Test Strategy

### Rust unit tests (`template.rs`)
- `test_entity_template_single_root_serialization`
- `test_entity_template_tree_serialization`
- `test_validate_empty_template_fails`
- `test_validate_multiple_roots_fails`
- `test_validate_dangling_parent_fails`
- `test_validate_cycle_detected`
- `test_validate_unknown_schema_fails`
- `test_validate_valid_template_succeeds`
- `test_instantiate_single_root`
- `test_instantiate_tree`
- `test_instantiate_with_target_parent`
- `test_instantiate_twice_different_ids`
- `test_mint_stable_id_unique`
- `test_template_cache_basic`

### Playwright E2E tests
- `save template and instantiate end-to-end`
  1. Register needed schemas (or use built-ins)
  2. Save template with 3 entities (tree)
  3. Load empty scene
  4. Dispatch InstantiateEntityTemplate
  5. Verify scene has 3 entities with tree hierarchy
- `template lifecycle with load_project restore`
  1. Save template
  2. Reload page
  3. Call `load_project()`
  4. Verify template is in cache (is_template_loaded returns true)
  5. Instantiate via command — succeeds

## §10. Performance Notes

- ID minting: thread_local counter + timestamp, O(1)
- Tree walk for instantiation: O(n) where n = template entities
- Validation: O(n) for cycle detection (n walk per entity worst case O(n²))
- Cache: HashMap, O(1) lookup
- For 100-entity templates, performance is fine

## §11. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| ID collision across threads | thread_local counter + timestamp suffix |
| Race between save and instantiate | Cache is in-memory; instantiate checks cache |
| Template file exists but not loaded | `load_template` returns error or auto-loads (choose: error, manual load) |
| Cycle in tree | Validate during load, reject |
| Component references unknown schema | Validate during load via combined_registry |
| Missing target_parent | target_parent is Option, None means scene root |
| Tree instantiation race | Single-threaded WASM, no contention |
| Large templates (100+ entities) | Linear walk, acceptable for MVP |
| Inverse command for instantiated entities | Use Batch of DeleteEntity (one per minted) |

## §12. Open Questions

1. **Sync vs async `save_template`** — Going with async for correctness.
2. **`is_template_loaded` vs auto-load on instantiate** — Going with explicit load + clear error if not loaded. Simpler model.
3. **Cache persistence** — Cache is in-memory only, lost on reload. Use `load_project()` to restore. Documented.
4. **Inverse command size** — Batch with N deletes for N entities. Acceptable.