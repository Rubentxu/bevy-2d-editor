//! Command processor for the Bevy 2D Editor.
//!
//! Provides `validate(doc, cmd)` and `apply(doc, cmd) -> Result<Command, CommandError>`.
//! Each command captures pre-state explicitly so inverse generation is mechanical.
//! Validation runs before mutation; failed commands leave the document unchanged.
//!
//! Batches apply atomically: on any failure inside a batch, previously applied
//! commands are rolled back in reverse order.

use crate::command::{Command, CommandError};
use crate::document::{ComponentInstance, Entity, SceneDocument, StableId};
use crate::schema::global_registry;

/// Find an entity by id and return a mutable reference.
fn find_entity_mut<'a>(
    doc: &'a mut SceneDocument,
    id: &StableId,
) -> Result<&'a mut Entity, CommandError> {
    doc.entities
        .iter_mut()
        .find(|e| &e.id == id)
        .ok_or_else(|| CommandError::EntityNotFound(id.clone()))
}

/// Find an entity by id and return an immutable reference.
fn find_entity<'a>(doc: &'a SceneDocument, id: &StableId) -> Result<&'a Entity, CommandError> {
    doc.entities
        .iter()
        .find(|e| &e.id == id)
        .ok_or_else(|| CommandError::EntityNotFound(id.clone()))
}

/// Set a field at a dotted path within a JSON object. Returns the old value.
///
/// Path navigation: split on '.', navigate to parent, set leaf.
/// For `"translation.x"` on `{"translation": {"x": 0, "y": 0}}`,
/// the result is `{"translation": {"x": <new>, "y": 0}}`.
fn set_field_path(
    value: &mut serde_json::Value,
    path: &str,
    new: serde_json::Value,
) -> Result<serde_json::Value, CommandError> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
        return Err(CommandError::FieldNotFound(path.to_string()));
    }
    let mut current = value;
    for part in &parts[..parts.len() - 1] {
        current = current
            .as_object_mut()
            .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?
            .get_mut(*part)
            .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?;
    }
    let leaf = parts.last().unwrap();
    let obj = current
        .as_object_mut()
        .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?;
    let old = obj
        .get(*leaf)
        .ok_or_else(|| CommandError::FieldNotFound(path.to_string()))?
        .clone();
    obj.insert(leaf.to_string(), new);
    Ok(old)
}

/// Check if reparenting `entity_id` under `proposed_parent` would create a cycle.
fn would_create_cycle(
    doc: &SceneDocument,
    entity_id: &StableId,
    proposed_parent: &StableId,
) -> bool {
    if entity_id == proposed_parent {
        return true;
    }
    let mut current = Some(proposed_parent.clone());
    while let Some(id) = current {
        if &id == entity_id {
            return true;
        }
        match find_entity(doc, &id) {
            Ok(e) => current = e.parent.clone(),
            Err(_) => return false,
        }
    }
    false
}

/// Validate a command against the document and schema registry, without mutating.
/// Returns `Ok(())` if the command can be applied, or `Err(CommandError)` otherwise.
pub fn validate(doc: &SceneDocument, cmd: &Command) -> Result<(), CommandError> {
    match cmd {
        Command::CreateEntity { id, .. } => {
            if doc.entities.iter().any(|e| &e.id == id) {
                return Err(CommandError::DuplicateId(id.clone()));
            }
        }
        Command::DeleteEntity { id } => {
            find_entity(doc, id)?;
        }
        Command::AddComponent {
            entity_id, type_id, ..
        } => {
            find_entity(doc, entity_id)?;
            if crate::schema::combined_registry().get(type_id).is_none() {
                return Err(CommandError::UnknownSchema(type_id.clone()));
            }
        }
        Command::RemoveComponent {
            entity_id,
            type_id,
        } => {
            let entity = find_entity(doc, entity_id)?;
            if !entity.components.iter().any(|c| &c.type_id == type_id) {
                // No-op: valid but produces self-inverse
            }
        }
        Command::SetComponentField {
            entity_id,
            type_id,
            field_path,
            ..
        } => {
            let entity = find_entity(doc, entity_id)?;
            let component = entity
                .components
                .iter()
                .find(|c| &c.type_id == type_id)
                .ok_or_else(|| CommandError::UnknownSchema(type_id.clone()))?;
            // Verify field path exists
            let parts: Vec<&str> = field_path.split('.').collect();
            let mut current = &component.values;
            for part in &parts {
                current = current
                    .as_object()
                    .and_then(|o| o.get(*part))
                    .ok_or_else(|| CommandError::FieldNotFound(field_path.clone()))?;
            }
        }
        Command::ReparentEntity {
            entity_id,
            new_parent,
            ..
        } => {
            find_entity(doc, entity_id)?;
            if let Some(new_p) = new_parent {
                find_entity(doc, new_p)?;
                if would_create_cycle(doc, entity_id, new_p) {
                    return Err(CommandError::WouldCreateCycle(entity_id.clone()));
                }
            }
        }
        Command::InstantiateEntityTemplate { template_id, .. } => {
            // Validate that template is loaded in cache
            if crate::template::get_cached_template(template_id).is_none() {
                return Err(CommandError::TemplateNotFound(template_id.clone()));
            }
        }
        Command::RenameEntity { entity_id, .. } => {
            find_entity(doc, entity_id)?;
        }
        Command::Batch { commands, .. } => {
            // Validate each command in order. Snapshot doc state in memory for
            // accurate validation (later commands see earlier ones).
            let mut temp_doc = doc.clone();
            for (i, c) in commands.iter().enumerate() {
                validate(&temp_doc, c).map_err(|e| CommandError::BatchFailed {
                    index: i,
                    source: Box::new(e),
                })?;
                // Apply to temp doc so subsequent validates see the effect
                apply(&mut temp_doc, c).map_err(|e| CommandError::BatchFailed {
                    index: i,
                    source: Box::new(e),
                })?;
            }
        }
    }
    Ok(())
}

/// Apply a command to the document, returning the inverse command.
///
/// Validation runs first; if it fails, the document is unchanged.
pub fn apply(doc: &mut SceneDocument, cmd: &Command) -> Result<Command, CommandError> {
    // Validate before mutating
    validate(doc, cmd)?;

    match cmd {
        Command::CreateEntity {
            id,
            name,
            components,
        } => {
            doc.entities.push(Entity {
                id: id.clone(),
                name: name.clone(),
                parent: None,
                components: components.clone(),
            });
            Ok(Command::DeleteEntity { id: id.clone() })
        }
        Command::DeleteEntity { id } => {
            let pos = doc
                .entities
                .iter()
                .position(|e| &e.id == id)
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
        Command::AddComponent {
            entity_id,
            type_id,
            values,
        } => {
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
        Command::RemoveComponent {
            entity_id,
            type_id,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let pos = entity.components.iter().position(|c| &c.type_id == type_id);
            match pos {
                Some(p) => {
                    let removed = entity.components.remove(p);
                    Ok(Command::AddComponent {
                        entity_id: entity_id.clone(),
                        type_id: removed.type_id,
                        values: removed.values,
                    })
                }
                None => {
                    // No-op: inverse is self
                    Ok(Command::RemoveComponent {
                        entity_id: entity_id.clone(),
                        type_id: type_id.clone(),
                    })
                }
            }
        }
        Command::SetComponentField {
            entity_id,
            type_id,
            field_path,
            value,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let component = entity
                .components
                .iter_mut()
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
        Command::ReparentEntity {
            entity_id,
            old_parent: _,
            new_parent,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let actual_old = entity.parent.clone();
            entity.parent = new_parent.clone();
            // Inverse: new_parent should point back to actual_old
            Ok(Command::ReparentEntity {
                entity_id: entity_id.clone(),
                old_parent: actual_old.clone(),
                new_parent: actual_old,
            })
        }
        Command::InstantiateEntityTemplate { template_id, target_parent } => {
            // Look up template from in-memory cache
            let template = crate::template::get_cached_template(template_id)
                .ok_or_else(|| CommandError::TemplateNotFound(template_id.clone()))?;
            // Instantiate: mints fresh StableIds, adds entities to scene
            let minted_ids = crate::template::instantiate(&template, target_parent.as_ref(), doc)
                .map_err(|e| CommandError::TemplateNotFound(format!("Template error: {}", e)))?;
            // Inverse: Batch of DeleteEntity for each minted entity
            let inverse_commands: Vec<Command> = minted_ids
                .iter()
                .map(|id| Command::DeleteEntity { id: id.clone() })
                .collect();
            Ok(Command::Batch {
                label: format!("undo_instantiate_{}", template_id),
                commands: inverse_commands,
            })
        }
        Command::RenameEntity {
            entity_id,
            old_name: _,
            new_name,
        } => {
            let entity = find_entity_mut(doc, entity_id)?;
            let actual_old = entity.name.clone();
            entity.name = new_name.clone();
            Ok(Command::RenameEntity {
                entity_id: entity_id.clone(),
                old_name: Some(actual_old.clone()),
                new_name: actual_old,
            })
        }
        Command::Batch { commands, .. } => {
            let mut inverses: Vec<Command> = Vec::new();
            for (i, c) in commands.iter().enumerate() {
                match apply(doc, c) {
                    Ok(inv) => inverses.push(inv),
                    Err(e) => {
                        // Rollback: apply inverses in reverse
                        for inv in inverses.iter().rev() {
                            let _ = apply(doc, inv);
                        }
                        return Err(CommandError::BatchFailed {
                            index: i,
                            source: Box::new(e),
                        });
                    }
                }
            }
            // Inverse batch: reverse the order of inverses
            inverses.reverse();
            Ok(Command::Batch {
                label: "inverse".to_string(),
                commands: inverses,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ComponentInstance;
    use serde_json::json;

    fn empty_doc() -> SceneDocument {
        SceneDocument {
            version: "0.1".to_string(),
            scene_id: "test".to_string(),
            name: "Test".to_string(),
            entities: vec![],
        }
    }

    fn entity_with_components(id: &str, name: &str, components: Vec<ComponentInstance>) -> Entity {
        Entity {
            id: StableId::new(id),
            name: name.to_string(),
            parent: None,
            components,
        }
    }

    fn transform2d(x: f32, y: f32) -> ComponentInstance {
        ComponentInstance {
            type_id: "editor.Transform2D".to_string(),
            values: json!({
                "translation": {"x": x, "y": y},
                "rotation": 0.0,
                "scale": {"x": 1.0, "y": 1.0}
            }),
        }
    }

    // ===== CreateEntity =====

    #[test]
    fn test_create_entity_adds_fresh_entity() {
        let mut doc = empty_doc();
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_new"),
            name: "Foo".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].id.as_str(), "ent_new");

        // Inverse should be DeleteEntity
        match inverse {
            Command::DeleteEntity { id } => assert_eq!(id.as_str(), "ent_new"),
            _ => panic!("Wrong inverse variant"),
        }
    }

    #[test]
    fn test_create_entity_rejects_duplicate_id() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_dup", "Foo", vec![]));
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_dup"),
            name: "Bar".to_string(),
            components: vec![],
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::DuplicateId(_))));
        assert_eq!(doc.entities.len(), 1); // unchanged
    }

    // ===== DeleteEntity =====

    #[test]
    fn test_delete_entity_removes_leaf() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::DeleteEntity {
            id: StableId::new("ent_01"),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 0);
        // Inverse is CreateEntity
        assert!(matches!(inverse, Command::CreateEntity { .. }));
    }

    #[test]
    fn test_delete_entity_reparents_children_to_root() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("parent", "Parent", vec![]));
        doc.entities.push(Entity {
            id: StableId::new("child"),
            name: "Child".to_string(),
            parent: Some(StableId::new("parent")),
            components: vec![],
        });
        let cmd = Command::DeleteEntity {
            id: StableId::new("parent"),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 1);
        assert_eq!(doc.entities[0].parent, None); // child reparented to root
    }

    #[test]
    fn test_delete_entity_missing_fails() {
        let mut doc = empty_doc();
        let cmd = Command::DeleteEntity {
            id: StableId::new("ent_missing"),
        };
        assert!(matches!(apply(&mut doc, &cmd), Err(CommandError::EntityNotFound(_))));
    }

    // ===== AddComponent =====

    #[test]
    fn test_add_component_with_valid_schema() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            values: json!({"translation": {"x": 1.0, "y": 2.0}, "rotation": 0.0, "scale": {"x": 1.0, "y": 1.0}}),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].components.len(), 1);
        assert_eq!(doc.entities[0].components[0].type_id, "editor.Transform2D");
    }

    #[test]
    fn test_add_component_unknown_schema_rejected() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Bogus".to_string(),
            values: json!({}),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::UnknownSchema(_))));
        assert!(doc.entities[0].components.is_empty());
    }

    #[test]
    fn test_add_component_preserves_unknown_fields() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            values: json!({"translation": {"x": 0.0, "y": 0.0}, "rotation": 0.0, "scale": {"x": 1.0, "y": 1.0}, "future_field": 42}),
        };
        apply(&mut doc, &cmd).unwrap();
        let val = &doc.entities[0].components[0].values;
        assert_eq!(val.get("future_field").unwrap(), &json!(42));
    }

    // ===== RemoveComponent =====

    #[test]
    fn test_remove_component_removes_existing() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![transform2d(0.0, 0.0)],
        ));
        let cmd = Command::RemoveComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
        };
        apply(&mut doc, &cmd).unwrap();
        assert!(doc.entities[0].components.is_empty());
    }

    #[test]
    fn test_remove_component_absent_is_noop() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01", "Foo", vec![]));
        let cmd = Command::RemoveComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Sprite2D".to_string(),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        // No-op: inverse is self
        assert!(matches!(inverse, Command::RemoveComponent { .. }));
        assert!(doc.entities[0].components.is_empty());
    }

    // ===== SetComponentField =====

    #[test]
    fn test_set_component_field_simple_path() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![transform2d(0.0, 0.0)]));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: json!(100.0),
        };
        apply(&mut doc, &cmd).unwrap();
        let val = &doc.entities[0].components[0].values;
        assert_eq!(val["translation"]["x"], json!(100.0));
        assert_eq!(val["translation"]["y"], json!(0.0));
    }

    #[test]
    fn test_set_component_field_nested_path() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components(
            "ent_01",
            "Foo",
            vec![ComponentInstance {
                type_id: "editor.Sprite2D".to_string(),
                values: json!({"color": {"r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0}}),
            }],
        ));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Sprite2D".to_string(),
            field_path: "color.r".to_string(),
            value: json!(0.5),
        };
        apply(&mut doc, &cmd).unwrap();
        let val = &doc.entities[0].components[0].values;
        assert_eq!(val["color"]["r"], json!(0.5));
        assert_eq!(val["color"]["g"], json!(1.0));
    }

    #[test]
    fn test_set_component_field_missing_fails() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![transform2d(0.0, 0.0)]));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "nonexistent.field".to_string(),
            value: json!(42),
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::FieldNotFound(_))));
    }

    // ===== ReparentEntity =====

    #[test]
    fn test_reparent_entity_valid() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("A", "A", vec![]));
        doc.entities.push(entity_with_components("B", "B", vec![]));
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("B")),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].parent, Some(StableId::new("B")));

        // Inverse points back: new_parent = what old_parent was (None, was root)
        match inverse {
            Command::ReparentEntity { old_parent, new_parent, .. } => {
                assert_eq!(old_parent, None);
                assert_eq!(new_parent, None);
            }
            _ => panic!("Wrong inverse"),
        }
    }

    #[test]
    fn test_reparent_entity_cycle_rejected() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("A", "A", vec![]));
        doc.entities.push(entity_with_components("B", "B", vec![]));
        doc.entities.push(Entity {
            id: StableId::new("C"),
            name: "C".to_string(),
            parent: Some(StableId::new("B")),
            components: vec![],
        });
        // B is child of A's chain? No, B is root. C is child of B.
        // Setting A's parent to C would create cycle: A → C → B (root, stops)
        // Actually that's not a cycle. Let's create a real cycle: A under C, then C under A.
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("C")),
        };
        // A -> C is fine (C is not A's descendant). Let's test self-parenting.
        let cmd_self = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("A")),
        };
        let result = apply(&mut doc, &cmd_self);
        assert!(matches!(result, Err(CommandError::WouldCreateCycle(_))));

        // Real cycle: A -> C -> B and then B -> A
        let _ = apply(&mut doc, &cmd); // A's parent = C
        let cmd_cycle = Command::ReparentEntity {
            entity_id: StableId::new("B"),
            old_parent: None,
            new_parent: Some(StableId::new("A")),
        };
        let result = apply(&mut doc, &cmd_cycle);
        assert!(matches!(result, Err(CommandError::WouldCreateCycle(_))));
    }

    // ===== RenameEntity =====

    #[test]
    fn test_rename_entity_updates_name_preserves_id() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01J", "Player", vec![]));
        let cmd = Command::RenameEntity {
            entity_id: StableId::new("ent_01J"),
            old_name: None,
            new_name: "PlayerSpawn".to_string(),
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].name, "PlayerSpawn");
        assert_eq!(doc.entities[0].id.as_str(), "ent_01J");
    }

    // ===== InstantiateEntityTemplate (stub) =====

    #[test]
    fn test_instantiate_template_stub_rejects() {
        let mut doc = empty_doc();
        let cmd = Command::InstantiateEntityTemplate {
            template_id: "tmpl_missing".to_string(),
            target_parent: None,
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::TemplateNotFound(_))));
    }

    // ===== Batch =====

    #[test]
    fn test_batch_applies_all_commands() {
        let mut doc = empty_doc();
        let cmd = Command::Batch {
            label: "test-batch".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("ent_01"),
                    name: "Foo".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("ent_02"),
                    name: "Bar".to_string(),
                    components: vec![],
                },
            ],
        };
        apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), 2);
    }

    #[test]
    fn test_batch_atomic_rollback_on_failure() {
        let mut doc = empty_doc();
        let cmd = Command::Batch {
            label: "test-batch".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("ent_01"),
                    name: "Foo".to_string(),
                    components: vec![],
                },
                Command::AddComponent {
                    entity_id: StableId::new("ent_01"),
                    type_id: "editor.Bogus".to_string(),
                    values: json!({}),
                },
            ],
        };
        let result = apply(&mut doc, &cmd);
        assert!(matches!(result, Err(CommandError::BatchFailed { .. })));
        // Document should be rolled back: 0 entities
        assert_eq!(doc.entities.len(), 0);
    }

    #[test]
    fn test_batch_inverse_reverses_order() {
        let mut doc = empty_doc();
        let cmd = Command::Batch {
            label: "test".to_string(),
            commands: vec![
                Command::CreateEntity {
                    id: StableId::new("A"),
                    name: "A".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("B"),
                    name: "B".to_string(),
                    components: vec![],
                },
                Command::CreateEntity {
                    id: StableId::new("C"),
                    name: "C".to_string(),
                    components: vec![],
                },
            ],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        match inverse {
            Command::Batch { commands, .. } => {
                assert_eq!(commands.len(), 3);
                // First inverse should undo C (the last applied)
                match &commands[0] {
                    Command::DeleteEntity { id } => assert_eq!(id.as_str(), "C"),
                    _ => panic!("Wrong first inverse"),
                }
                match &commands[2] {
                    Command::DeleteEntity { id } => assert_eq!(id.as_str(), "A"),
                    _ => panic!("Wrong last inverse"),
                }
            }
            _ => panic!("Wrong inverse variant"),
        }
    }

    // ===== Roundtrip =====

    #[test]
    fn test_forward_inverse_roundtrip_create_entity() {
        let mut doc = empty_doc();
        let original_len = doc.entities.len();
        let cmd = Command::CreateEntity {
            id: StableId::new("ent_01"),
            name: "Foo".to_string(),
            components: vec![],
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities.len(), original_len + 1);
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities.len(), original_len);
        assert_eq!(doc, empty_doc());
    }

    #[test]
    fn test_forward_inverse_roundtrip_set_field() {
        let mut doc = empty_doc();
        doc.entities
            .push(entity_with_components("ent_01", "Foo", vec![transform2d(50.0, 50.0)]));
        let cmd = Command::SetComponentField {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Transform2D".to_string(),
            field_path: "translation.x".to_string(),
            value: json!(999.0),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].components[0].values["translation"]["x"], json!(999.0));
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities[0].components[0].values["translation"]["x"], json!(50.0));
    }

    #[test]
    fn test_forward_inverse_roundtrip_reparent() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("A", "A", vec![]));
        doc.entities.push(entity_with_components("B", "B", vec![]));
        let cmd = Command::ReparentEntity {
            entity_id: StableId::new("A"),
            old_parent: None,
            new_parent: Some(StableId::new("B")),
        };
        let inverse = apply(&mut doc, &cmd).unwrap();
        assert_eq!(doc.entities[0].parent, Some(StableId::new("B")));
        apply(&mut doc, &inverse).unwrap();
        assert_eq!(doc.entities[0].parent, None);
    }

    // ===== Validation leaves doc unchanged =====

    #[test]
    fn test_failed_validation_leaves_doc_unchanged() {
        let mut doc = empty_doc();
        doc.entities.push(entity_with_components("ent_01", "Foo", vec![]));
        let snapshot_before = doc.clone();
        let cmd = Command::AddComponent {
            entity_id: StableId::new("ent_01"),
            type_id: "editor.Bogus".to_string(),
            values: json!({}),
        };
        let _ = apply(&mut doc, &cmd);
        assert_eq!(doc, snapshot_before);
    }

    #[test]
    fn test_field_path_helper_simple() {
        let mut v = json!({"a": 1});
        let old = set_field_path(&mut v, "a", json!(99)).unwrap();
        assert_eq!(old, json!(1));
        assert_eq!(v["a"], json!(99));
    }

    #[test]
    fn test_field_path_helper_nested() {
        let mut v = json!({"a": {"b": {"c": 1}}});
        let old = set_field_path(&mut v, "a.b.c", json!(42)).unwrap();
        assert_eq!(old, json!(1));
        assert_eq!(v["a"]["b"]["c"], json!(42));
    }

    #[test]
    fn test_field_path_helper_missing() {
        let mut v = json!({"a": 1});
        let result = set_field_path(&mut v, "b", json!(42));
        assert!(matches!(result, Err(CommandError::FieldNotFound(_))));
    }
}