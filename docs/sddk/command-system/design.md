# Design: Command System

> Change: `command-system` · Phase: sddk-design · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

---

## §1. Module Layout

```
crates/editor-core/src/
├── document.rs          (existing — SceneDocument, Entity, etc.)
├── schema.rs            (existing — ComponentSchemaRegistry)
├── command.rs           (new) — Command enum, CommandMetadata, CommandError, CommandResult
├── processor.rs         (new) — apply(), validate(), inverse() logic per command
└── lib.rs               (modified) — dispatch_command wasm_bindgen, SceneDocumentState, rebuild_preview_world
```

## §2. Type Design

### §2.1 Command enum (internally tagged)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum Command {
    CreateEntity {
        id: StableId,
        name: String,
        components: Vec<ComponentInstance>,
    },
    DeleteEntity {
        id: StableId,
    },
    AddComponent {
        entity_id: StableId,
        type_id: String,
        values: serde_json::Value,
    },
    RemoveComponent {
        entity_id: StableId,
        type_id: String,
    },
    SetComponentField {
        entity_id: StableId,
        type_id: String,
        field_path: String,  // dotted, e.g., "translation.x"
        value: serde_json::Value,
    },
    ReparentEntity {
        entity_id: StableId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_parent: Option<StableId>,  // captured pre-state for inverse
        new_parent: Option<StableId>,
    },
    InstantiateEntityTemplate {
        template_id: String,
        target_parent: Option<StableId>,
    },
    RenameEntity {
        entity_id: StableId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        old_name: Option<String>,  // captured pre-state for inverse
        new_name: String,
    },
    Batch {
        label: String,
        commands: Vec<Command>,
    },
}
```

**JSON shape example:**
```json
{
  "type": "CreateEntity",
  "id": "ent_01J...",
  "name": "Player",
  "components": []
}
```

### §2.2 CommandMetadata

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub authorship: String,   // "user", "agent:<id>", "system"
    pub timestamp: u64,        // Unix millis
    pub rationale: Option<String>,
}
```

Wrapped in `CommandEnvelope` for actual dispatch:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub command: Command,
    pub metadata: CommandMetadata,
}
```

### §2.3 CommandResult

```rust
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub inverse: Command,
    pub snapshot: SceneDocument,  // post-apply document
}
```

The inverse is computed during apply; the snapshot is a clone of the mutated document.

### §2.4 CommandError

```rust
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Entity not found: {0}")]
    EntityNotFound(StableId),
    #[error("Duplicate entity id: {0}")]
    DuplicateId(StableId),
    #[error("Unknown schema: {0}")]
    UnknownSchema(String),
    #[error("Field not found: {0}")]
    FieldNotFound(String),
    #[error("Reparent would create cycle through {0}")]
    WouldCreateCycle(StableId),
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    #[error("Batch failed at command {index}: {source}")]
    BatchFailed { index: usize, source: Box<CommandError> },
}
```

## §3. Processor Design

### §3.1 Stateless module structure

```rust
// processor.rs
pub fn validate(doc: &SceneDocument, cmd: &Command) -> Result<(), CommandError>;
pub fn apply(doc: &mut SceneDocument, cmd: &Command) -> Result<Command, CommandError>;
```

`apply` returns the inverse `Command` ready for re-application. Validation is split out so failed commands don't touch the document.

### §3.2 Per-command logic (sketch)

```rust
pub fn apply(doc: &mut SceneDocument, cmd: &Command) -> Result<Command, CommandError> {
    match cmd {
        Command::CreateEntity { id, name, components } => {
            if doc.entities.iter().any(|e| &e.id == id) {
                return Err(CommandError::DuplicateId(id.clone()));
            }
            doc.entities.push(Entity {
                id: id.clone(),
                name: name.clone(),
                parent: None,
                components: components.clone(),
            });
            Ok(Command::DeleteEntity { id: id.clone() })
        }
        Command::DeleteEntity { id } => {
            let pos = doc.entities.iter().position(|e| &e.id == id)
                .ok_or_else(|| CommandError::EntityNotFound(id.clone()))?;
            let removed = doc.entities.remove(pos);
            // Reparent children to root
            for entity in doc.entities.iter_mut() {
                if entity.parent.as_ref() == Some(id) {
                    entity.parent = None;
                }
            }
            Ok(Command::CreateEntity {
                id: removed.id,
                name: removed.name,
                components: removed.components,
            })
        }
        Command::AddComponent { entity_id, type_id, values } => {
            validate_schema_exists(type_id)?;
            let entity = find_entity_mut(doc, entity_id)?;
            entity.components.push(ComponentInstance {
                type_id: type_id.clone(),
                values: values.clone(),
            });
            Ok(Command::RemoveComponent {
                entity_id: entity_id.clone(),
                type_id: type_id.clone(),
            })
        }
        Command::RemoveComponent { entity_id, type_id } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let pos = entity.components.iter().position(|c| &c.type_id == type_id);
            let removed = match pos {
                Some(p) => entity.components.remove(p),
                None => return Ok(Command::RemoveComponent {
                    entity_id: entity_id.clone(),
                    type_id: type_id.clone(),
                }), // no-op, inverse is self
            };
            Ok(Command::AddComponent {
                entity_id: entity_id.clone(),
                type_id: removed.type_id,
                values: removed.values,
            })
        }
        Command::SetComponentField { entity_id, type_id, field_path, value } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let component = entity.components.iter_mut()
                .find(|c| &c.type_id == type_id)
                .ok_or_else(|| CommandError::UnknownSchema(type_id.clone()))?;
            let old_value = set_field_path(&mut component.values, field_path, value.clone())?;
            Ok(Command::SetComponentField {
                entity_id: entity_id.clone(),
                type_id: type_id.clone(),
                field_path: field_path.clone(),
                value: old_value,
            })
        }
        Command::ReparentEntity { entity_id, old_parent: _, new_parent } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let actual_old = entity.parent.clone();
            // Cycle check
            if let Some(new_p) = new_parent {
                if would_create_cycle(doc, entity_id, new_p)? {
                    return Err(CommandError::WouldCreateCycle(entity_id.clone()));
                }
            }
            entity.parent = new_parent.clone();
            Ok(Command::ReparentEntity {
                entity_id: entity_id.clone(),
                old_parent: actual_old,  // pre-state captured
                new_parent: actual_old,  // inverse points back
            })
        }
        Command::InstantiateEntityTemplate { template_id, target_parent: _ } => {
            // STUB: full tree instantiation deferred
            Err(CommandError::TemplateNotFound(template_id.clone()))
        }
        Command::RenameEntity { entity_id, old_name: _, new_name } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let actual_old = entity.name.clone();
            entity.name = new_name.clone();
            Ok(Command::RenameEntity {
                entity_id: entity_id.clone(),
                old_name: Some(actual_old),  // pre-state captured
                new_name: actual_old,        // inverse restores
            })
        }
        Command::Batch { commands, .. } => {
            let mut inverses = Vec::new();
            for (i, c) in commands.iter().enumerate() {
                match apply(doc, c) {
                    Ok(inv) => inverses.push(inv),
                    Err(e) => {
                        // Rollback all previously applied
                        for inv in inverses.into_iter().rev() {
                            let _ = apply(doc, &inv);
                        }
                        return Err(CommandError::BatchFailed { index: i, source: Box::new(e) });
                    }
                }
            }
            inverses.reverse();
            Ok(Command::Batch {
                label: format!("inverse_of_batch"),
                commands: inverses,
            })
        }
    }
}
```

### §3.3 Field path helper

```rust
fn set_field_path(
    value: &mut serde_json::Value,
    path: &str,
    new: serde_json::Value,
) -> Result<serde_json::Value, CommandError> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return Err(CommandError::FieldNotFound(path.to_string()));
    }
    // Navigate to parent
    let mut current = value;
    for part in &parts[..parts.len() - 1] {
        current = current.get_mut(part)
            .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?;
    }
    let leaf = parts.last().unwrap();
    let old = current.get(leaf)
        .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?
        .clone();
    current[leaf] = new;
    Ok(old)
}
```

### §3.4 Cycle detection

```rust
fn would_create_cycle(
    doc: &SceneDocument,
    entity_id: &StableId,
    proposed_parent: &StableId,
) -> Result<bool, CommandError> {
    if entity_id == proposed_parent {
        return Ok(true);
    }
    let mut current = Some(proposed_parent.clone());
    while let Some(id) = current {
        if &id == entity_id {
            return Ok(true);
        }
        let entity = doc.entities.iter().find(|e| &e.id == &id);
        match entity {
            Some(e) => current = e.parent.clone(),
            None => return Ok(false),
        }
    }
    Ok(false)
}
```

## §4. Bevy Integration

### §4.1 SceneDocumentState resource

```rust
#[derive(Resource, Clone)]
pub struct SceneDocumentState {
    pub document: SceneDocument,
    pub dirty: bool,
}
```

This resource lives INSIDE the Bevy World. The actual SceneDocument lives in the `thread_local!` `SCENE_DOC` (kept for backward compat with existing spike). They stay in sync:
- Initial load: `SCENE_DOC` → `SceneDocumentState` via `setup()`
- After command: mutate `SCENE_DOC`, set `dirty = true`
- `rebuild_preview_world` reads `dirty`, respawns entities

### §4.2 rebuild_preview_world system

```rust
fn rebuild_preview_world(
    mut commands: Commands,
    mut state: ResMut<SceneDocumentState>,
    entities: Query<Entity, With<SceneEntity>>,
) {
    if !state.dirty {
        return;
    }
    // Despawn existing scene entities (keep Camera2d)
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
    // Spawn new entities from document
    for entity in state.document.entities.iter() {
        spawn_entity(&mut commands, entity);
    }
    state.dirty = false;
}
```

A marker component `SceneEntity` distinguishes scene entities from camera/gizmos (which persist across rebuilds).

### §4.3 Command dispatch flow

```rust
#[wasm_bindgen]
pub fn dispatch_command(json: &str) -> Result<String, JsValue> {
    let envelope: CommandEnvelope = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;
    
    let result = SCENE_DOC.with(|s| {
        let mut doc_ref = s.borrow_mut();
        let doc = doc_ref.as_mut().ok_or_else(|| JsValue::from_str("No scene loaded"))?;
        let result = processor::apply(doc, &envelope.command)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok::<_, JsValue>(result)
    })?;
    
    // Mark Bevy resource dirty (sets it during the next system tick)
    // ... via a static flag that rebuild_preview_world reads
    
    let result_json = serde_json::to_string(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(result_json)
}
```

Since the Bevy app runs on its own thread (or single-threaded in WASM), the dirty flag must be visible to `rebuild_preview_world`. A simple approach: a `thread_local!` flag that `dispatch_command` sets and `rebuild_preview_world` reads.

## §5. WASM Surface

Single entry point `dispatch_command(json: &str) -> Result<String, JsValue>`. Inputs and outputs are JSON strings. Internally tagged enum makes the JSON self-describing.

This is preferred over 8 typed functions because:
- Smaller wasm_bindgen surface
- Easier to add new commands
- Type-safe roundtrip
- Matches the future AI agent tool API pattern (single function dispatching typed commands)

## §6. Backward Compatibility

- `LinearBus` + `CMD_MOVE_SPRITE` continue to work unchanged
- Default scene fallback preserved
- Existing Playwright tests untouched (they exercise LinearBus, not commands)
- New command path is additive; no behavior change unless `dispatch_command` is called

## §7. Testing Strategy

**Unit tests in `command.rs` and `processor.rs`:**
- All 25 §2 scenarios
- All 3 §3 scenarios (batch apply, atomicity, inverse)
- ~30 total unit tests

**Integration test in `lib.rs`:**
- Dispatch a `SetComponentField` command, verify `SceneDocumentState.dirty` is set, verify `rebuild_preview_world` respawns correctly

**Playwright E2E in `engine.spec.ts`:**
- New test: dispatch `CreateEntity` from `page.evaluate()`, reload page, verify entity count changes

## §8. Performance Notes

- `apply()` is O(n) in entity count for most commands (lookup via linear scan)
- For Hito 0 (50-200 entities), acceptable
- Future optimization: build a HashMap<StableId, EntityIndex> for O(1) lookup
- `rebuild_preview_world` despawns and respawns all scene entities per command — acceptable for Hito 0, matches decision 23 ("selective rebuild on commit")

## §9. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| Bevy rebuild system runs before dispatch flag is set | Use `thread_local!` flag; rebuild reads it every frame |
| Cycle detection misses deep hierarchies | Walk full parent chain, not just one level |
| Field path parser misinterprets keys with dots | Hito 0 has no dotted keys; document limitation |
| `Batch` inverse order wrong | Reverse the inverses vector explicitly |
| `InstantiateEntityTemplate` stub confuses future devs | Document as deferred in module doc comment + spec §4 |

## §10. Open Questions

1. **CommandMetadata on every command vs. envelope:** Current design uses envelope (single metadata block). Alternative: metadata field inside Command enum. Envelope is cleaner for batching (all sub-commands share metadata).
2. **Field path with array indices:** `colors[0].r` style paths? Hito 0 has no array fields. Defer.
3. **Cycle detection in self-referential ReparentEntity:** A's new parent is A itself. Caught by explicit `entity_id == proposed_parent` check.
4. **Behavior when inverse field is `None` for `ReparentEntity`:** Captured `old_parent: None` means "was root". Inverse's `new_parent: None` restores root. Roundtrip works.