# Spec: Command System

> Change: `command-system` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `command-system`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `command-system` — typed Command enum, CommandProcessor, reversibility, validation, wasm_bindgen dispatch
  - `command-batching` — gesture grouping via BatchCommand wrapper
- **Source proposal:** [`docs/sddk/command-system/proposal.md`](../command-system/proposal.md)
- **Source explore:** [`docs/sddk/command-system/explore-report.md`](../command-system/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §5.3 (Communication Model)](../../hito-0-spec.md)
  - [Hito 0 §6.2-6.7 (Data Model)](../../hito-0-spec.md)
  - [Hito 0 §6.4 (Reversible Operation Log)](../../hito-0-spec.md)
  - [ADR-0001 — JSON source of truth](../../adr/0001-scene-document-json-as-source-of-truth.md)
  - [ADR-0002 — Single Bevy renders the canvas](../../adr/0002-single-bevy-renders-canvas.md)
  - [ADR-0003 — Forward compat via serde_json::Value](../../adr/0003-forward-compat-via-serde-json-value.md)

---

## §2. Capability: `command-system`

### Requirement: Command enum defines 8 semantic types

The system MUST define a `Command` enum with exactly the 8 variants from Hito 0 §6.4: `CreateEntity`, `DeleteEntity`, `AddComponent`, `RemoveComponent`, `SetComponentField`, `ReparentEntity`, `InstantiateEntityTemplate`, `RenameEntity`.

#### Scenario: All 8 command variants exist
- GIVEN the `Command` enum
- WHEN its variants are enumerated
- THEN exactly 8 variants exist matching the spec list

### Requirement: Each command carries metadata

The system MUST associate each command with `CommandMetadata` containing authorship, timestamp, and rationale for future agent auditing (§6.4).

#### Scenario: Command metadata is required and serializable
- GIVEN a `Command` instance
- WHEN serialized to JSON
- THEN `authorship`, `timestamp`, and `rationale` fields are present
- AND all three are preserved across roundtrip

### Requirement: CreateEntity adds a new entity to the document

The system MUST add a new `Entity` to `SceneDocument.entities` when `CreateEntity { id, name, components }` is applied. The `id` MUST be unique within the document.

#### Scenario: CreateEntity adds a fresh entity
- GIVEN a SceneDocument with 0 entities
- WHEN `CreateEntity { id: "ent_new", name: "Foo", components: [] }` is applied
- THEN the document has 1 entity
- AND that entity's id is `ent_new`

#### Scenario: CreateEntity with duplicate id is rejected
- GIVEN a SceneDocument containing entity `ent_existing`
- WHEN `CreateEntity { id: "ent_existing", ... }` is applied
- THEN the command fails with `CommandError::DuplicateId`
- AND the document is unchanged

### Requirement: DeleteEntity removes an entity and all child references

The system MUST remove an entity and set `parent: None` on all its direct children (which become root entities) when `DeleteEntity { id }` is applied.

#### Scenario: DeleteEntity removes a leaf entity
- GIVEN a SceneDocument with one entity
- WHEN `DeleteEntity { id: that_entity_id }` is applied
- THEN the document has 0 entities

#### Scenario: DeleteEntity reparents children to root
- GIVEN a parent entity with one child
- WHEN `DeleteEntity { id: parent_id }` is applied
- THEN the parent is removed
- AND the child has `parent: None`

#### Scenario: DeleteEntity on non-existent id fails
- GIVEN a SceneDocument with no entity `ent_missing`
- WHEN `DeleteEntity { id: "ent_missing" }` is applied
- THEN the command fails with `CommandError::EntityNotFound`

### Requirement: AddComponent attaches a new component instance

The system MUST append a `ComponentInstance { type_id, values }` to `entity.components` when `AddComponent { entity_id, type_id, values }` is applied. The `type_id` MUST exist in the `ComponentSchemaRegistry`.

#### Scenario: AddComponent with valid schema succeeds
- GIVEN a SceneDocument and the registry with `editor.Transform2D`
- WHEN `AddComponent { entity_id, type_id: "editor.Transform2D", values: {...} }` is applied
- THEN the entity has one more component
- AND the component's `type_id` is `"editor.Transform2D"`

#### Scenario: AddComponent with unknown schema is rejected
- GIVEN the registry without `editor.Bogus`
- WHEN `AddComponent { type_id: "editor.Bogus", ... }` is applied
- THEN the command fails with `CommandError::UnknownSchema`
- AND the document is unchanged

#### Scenario: AddComponent preserves unknown fields
- GIVEN a component instance with an unknown field in `values`
- WHEN `AddComponent` is applied
- THEN the unknown field is preserved in the document (per ADR-0003)

### Requirement: RemoveComponent detaches a component instance

The system MUST remove the `ComponentInstance` with the matching `type_id` from the entity when `RemoveComponent { entity_id, type_id }` is applied.

#### Scenario: RemoveComponent removes existing instance
- GIVEN an entity with `editor.Transform2D`
- WHEN `RemoveComponent { type_id: "editor.Transform2D" }` is applied
- THEN the entity no longer has that component

#### Scenario: RemoveComponent on absent component is a no-op
- GIVEN an entity without `editor.Sprite2D`
- WHEN `RemoveComponent { type_id: "editor.Sprite2D" }` is applied
- THEN the command succeeds (no error)
- AND the document is unchanged

### Requirement: SetComponentField mutates one field of one component

The system MUST update one field within `ComponentInstance.values` when `SetComponentField { entity_id, type_id, field_path, value }` is applied. The `field_path` is a dotted string (e.g., `"translation.x"`).

#### Scenario: SetComponentField with simple path updates one field
- GIVEN an entity with `editor.Transform2D.translation = {x: 0, y: 0}`
- WHEN `SetComponentField { field_path: "translation.x", value: 100 }` is applied
- THEN `translation.x` is 100
- AND `translation.y` is still 0

#### Scenario: SetComponentField with nested path drills into objects
- GIVEN an entity with `editor.Sprite2D.color = {r: 1, g: 1, b: 1, a: 1}`
- WHEN `SetComponentField { field_path: "color.r", value: 0.5 }` is applied
- THEN `color.r` is 0.5
- AND other color components are unchanged

#### Scenario: SetComponentField on missing field path fails
- GIVEN an entity with `editor.Transform2D`
- WHEN `SetComponentField { field_path: "nonexistent.field" }` is applied
- THEN the command fails with `CommandError::FieldNotFound`

### Requirement: ReparentEntity moves an entity under a new parent

The system MUST set `entity.parent` to the new value when `ReparentEntity { entity_id, old_parent, new_parent }` is applied. The command MUST capture the `old_parent` at validation time so inverse generation is mechanical. Cycles MUST be rejected.

#### Scenario: ReparentEntity to a valid parent succeeds
- GIVEN two root entities A and B
- WHEN `ReparentEntity { entity_id: A, new_parent: B }` is applied
- THEN A's `parent` is B's id
- AND inverse `ReparentEntity { entity_id: A, new_parent: None }` is produced

#### Scenario: ReparentEntity creating a cycle is rejected
- GIVEN a hierarchy A → B → C
- WHEN `ReparentEntity { entity_id: A, new_parent: C }` is applied
- THEN the command fails with `CommandError::WouldCreateCycle`
- AND A's parent is unchanged

### Requirement: RenameEntity changes only the name field

The system MUST update `entity.name` when `RenameEntity { entity_id, old_name, new_name }` is applied. The `id` MUST NOT change (per §6.2).

#### Scenario: RenameEntity updates name but preserves id
- GIVEN an entity with `id: "ent_01", name: "Player"`
- WHEN `RenameEntity { new_name: "PlayerSpawn" }` is applied
- THEN `name` is `"PlayerSpawn"`
- AND `id` is still `"ent_01"`

### Requirement: InstantiateEntityTemplate stubs tree creation

The system MUST validate `InstantiateEntityTemplate { template_id, target_parent }` and emit a placeholder inverse. Full tree instantiation is deferred (stub scope).

#### Scenario: InstantiateEntityTemplate validates template_id exists
- GIVEN the registry has no template `tmpl_missing`
- WHEN `InstantiateEntityTemplate { template_id: "tmpl_missing" }` is applied
- THEN the command fails with `CommandError::TemplateNotFound`

### Requirement: Every command is reversible

The system MUST produce an inverse `Command` for every applied command. Applying forward then inverse MUST restore the original document.

#### Scenario: Forward+inverse roundtrip restores document
- GIVEN a SceneDocument D
- WHEN a command C is applied producing inverse I and new document D'
- AND I is applied to D'
- THEN the resulting document equals D

### Requirement: Validation runs before mutation

The system MUST validate commands against the `ComponentSchemaRegistry` and document state before applying any mutation. Validation failures MUST leave the document unchanged.

#### Scenario: Failed validation leaves document untouched
- GIVEN a SceneDocument D
- WHEN an invalid command is dispatched
- THEN the command fails
- AND D is unchanged (byte-equal)

---

## §3. Capability: `command-batching`

### Requirement: BatchCommand groups multiple commands atomically

The system MUST accept `BatchCommand { label: String, commands: Vec<Command> }` and apply all commands as a single atomic history entry. If any command fails, the entire batch fails and the document is unchanged.

#### Scenario: BatchCommand applies all commands in order
- GIVEN a SceneDocument D
- WHEN `BatchCommand { commands: [CreateEntity, AddComponent, SetComponentField] }` is applied
- THEN all three commands are applied
- AND the resulting document reflects all three changes

#### Scenario: BatchCommand atomic rollback on failure
- GIVEN a SceneDocument D
- WHEN a `BatchCommand` is dispatched where the second command would fail validation
- THEN the entire batch fails
- AND D is unchanged

### Requirement: BatchCommand inverse is a BatchCommand with inverses reversed

The system MUST produce a batch inverse where each command is replaced by its individual inverse and the order is reversed.

#### Scenario: Batch inverse undoes each command in reverse order
- GIVEN a batch [A, B, C] applied to document D
- WHEN the batch inverse is generated
- THEN it contains [inverse(C), inverse(B), inverse(A)]
- AND applying the inverse restores D

---

## §4. Out-of-Scope Behaviors (explicit non-goals)

- Operation Log persistence (storage of past commands) — separate change
- Undo/redo UI or keyboard shortcuts — separate change
- Asset loading pipeline — defer until sprite asset is needed
- Full Entity Template tree instantiation — stub this cycle
- React UI panel integration (Hierarchy/Inspector) — separate change

---

## §5. Acceptance Criteria

1. Every §2 scenario passes via Rust unit tests.
2. Every §3 scenario passes via Rust unit tests.
3. Forward+inverse roundtrip test for each of the 8 commands.
4. Batch atomicity test passes.
5. `dispatch_command` wasm_bindgen accepts JSON, applies, returns JSON.
6. Bevy preview world rebuilds after a successful command.
7. WASM builds cleanly.
8. Existing Playwright tests still pass (LinearBus untouched).

---

## §6. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2 (command-system) | All 8 commands + reversibility + validation | Rust unit (`command.rs`, `processor.rs`) | ~25 |
| §3 (command-batching) | Batch apply + atomicity + inverse | Rust unit (`processor.rs`) | ~5 |
| Bevy rebuild | SceneDocument mutation triggers entity respawn | Rust integration (`lib.rs`) | 1 |
| E2E | dispatch_command from JS mutates scene | Playwright (`engine.spec.ts`) | 1 |
| **Total** | | | **~32 tests** |

Dev cycle: `cargo test --lib` (Rust units) + `just wasm` (build) + `just test` (Playwright E2E).