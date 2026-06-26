# Spec: SceneDocument + Component Schema Registry

> Change: `scene-document` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `scene-document`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):**
  - `scene-document-model` — SceneDocument, Entity, ComponentInstance data types and lossless JSON roundtrip
  - `component-schema-registry` — global registry with 5 seed schemas (Name, Transform2D, Sprite2D, Visible, Locked)
- **Source proposal:** [`docs/sddk/scene-document/proposal.md`](../sddk/scene-document/proposal.md)
- **Source explore:** [`docs/sddk/scene-document/explore-report.md`](../sddk/scene-document/explore-report.md)
- **Authoritative references:**
  - [Hito 0 §6.1–6.9 (Data Model — AI-Friendly Core)](../../hito-0-spec.md)
  - [Hito 0 §7 (Built-in Components)](../../hito-0-spec.md)
  - [ADR-0001 — SceneDocument JSON as source of truth](../../adr/0001-scene-document-json-as-source-of-truth.md)
  - [ADR-0002 — Single Bevy renders the canvas](../../adr/0002-single-bevy-renders-canvas.md)

---

## §2. Capability: `scene-document-model`

### Requirement: SceneDocument serializes to valid JSON

The system MUST serialize a `SceneDocument` into a JSON document containing the top-level fields `version`, `scene_id`, `name`, and `entities`.

#### Scenario: Serialize a populated SceneDocument

- GIVEN a `SceneDocument` with `version: "0.1"`, a `scene_id`, a `name`, and 1+ entities
- WHEN serialized to JSON
- THEN the output is valid JSON
- AND contains the top-level fields `version`, `scene_id`, `name`, `entities`

#### Scenario: Serialize an empty scene

- GIVEN an empty `SceneDocument` (no entities)
- WHEN serialized
- THEN the output is valid JSON with an empty `entities` array
- AND deserializes back to an empty `SceneDocument`

### Requirement: SceneDocument deserializes without silent data loss

The system MUST parse a valid JSON string into a `SceneDocument` and populate every field present in the input.

#### Scenario: Deserialize a well-formed scene

- GIVEN a valid JSON string matching the `SceneDocument` schema
- WHEN deserialized into a `SceneDocument`
- THEN all fields (`version`, `scene_id`, `name`, entities, ids, components, values) are populated correctly
- AND no data is silently dropped

### Requirement: Lossless JSON roundtrip

The system MUST preserve every field of a `SceneDocument` across `serialize → deserialize`.

#### Scenario: Roundtrip preserves entities, hierarchy, components, and values

- GIVEN a `SceneDocument` with multiple entities, components, and a parent/child hierarchy
- WHEN serialized to JSON then deserialized into a new `SceneDocument`
- THEN the two documents are deeply equal (id, name, parent, components, values)
- AND Stable IDs are byte-identical
- AND parent references are preserved

### Requirement: Stable IDs are immutable across rename

The system MUST keep an Entity's Stable ID unchanged when its name (or any other mutable field) changes.

#### Scenario: Renaming an entity does not mutate its id

- GIVEN an Entity with id `ent_01J...` and name "Player"
- WHEN the entity is renamed to "PlayerSpawn"
- THEN the id remains `ent_01J...` unchanged
- AND only the `name` field differs in the resulting document

### Requirement: Stable IDs are opaque and value-comparable

The system MUST treat Stable IDs as opaque, value-comparable identifiers that cannot be confused with human-readable names.

#### Scenario: IDs are opaque and compare by value

- GIVEN two entities with different Stable IDs
- WHEN stored in collections or compared
- THEN IDs are opaque strings (not Bevy Entity indices, not names)
- AND equality is by opaque value, not by index or name

### Requirement: Editor-owned types serialize as JSON objects

The system MUST serialize editor-owned value types (`Vec2`, `Color`, `Anchor`) using stable JSON object shapes — never arrays, never hex strings.

#### Scenario: Vec2, Color, and Anchor shapes in JSON

- GIVEN an entity with `editor.Transform2D` and `editor.Sprite2D` instances
- WHEN serialized
- THEN `translation` and `scale` appear as `Vec2` objects `{ "x": ..., "y": ... }`
- AND `color` appears as a `Color` object `{ "r": ..., "g": ..., "b": ..., "a": ... }`
- AND `anchor` appears as a string enum (e.g., `"Center"`, `"BottomLeft"`)

### Requirement: Forward compatibility — unknown fields are preserved

The system MUST preserve fields that exist on a Component Instance but are unknown to the current schema; the system MUST NOT auto-delete them.

#### Scenario: Unknown field is preserved and flagged orphaned

- GIVEN a JSON scene with a Component Instance whose `values` contains a field not declared in the current schema
- WHEN loaded into a `SceneDocument`
- THEN the unknown field is preserved in the deserialized structure
- AND validation marks it as orphaned (not silently deleted)

### Requirement: Document versioning is preserved

The system MUST preserve and surface the `version` field of a `SceneDocument`.

#### Scenario: Version field survives roundtrip

- GIVEN a JSON scene with `version: "0.1"`
- WHEN loaded
- THEN the `version` field is preserved in the deserialized document
- AND the `SceneDocument` type is version-aware (the field is part of the structure, not discarded)

### Requirement: Component Instance structure

The system MUST serialize each Component Instance as an object with a namespaced `type_id` string and a `values` object.

#### Scenario: Instance serializes with namespaced type_id

- GIVEN an entity with a Component Instance of type `editor.Transform2D`
- WHEN serialized
- THEN the instance has `type_id = "editor.Transform2D"` (string) and `values` (object)
- AND `type_id` is a string, not an integer index

---

## §3. Capability: `component-schema-registry`

### Requirement: Registry is seeded with 5 built-in schemas

The system MUST initialize the Component Schema Registry with exactly 5 built-in schemas: `editor.Name`, `editor.Transform2D`, `editor.Sprite2D`, `editor.Visible`, `editor.Locked`.

#### Scenario: Built-in schemas are present

- GIVEN the registry initialized with built-in seeds
- WHEN queried for all schemas
- THEN exactly 5 schemas exist with the expected `type_id`s

### Requirement: Schema lookup by `type_id`

The system MUST return the schema for a known `type_id` and MUST return `None` (not panic) for an unknown one.

#### Scenario: Known `type_id` returns its schema

- GIVEN the registry with seeded schemas
- WHEN `get_schema("editor.Transform2D")` is called
- THEN the Transform2D schema is returned with its fields (translation, rotation, scale)

#### Scenario: Unknown `type_id` returns None

- GIVEN the registry with seeded schemas
- WHEN `get_schema("editor.NonExistent")` is called
- THEN `None` is returned and no panic occurs

### Requirement: Each schema declares typed fields

The system MUST store field definitions (type, default value, constraints) for every schema field.

#### Scenario: Transform2D fields are defined

- GIVEN the Transform2D schema
- WHEN inspected
- THEN it declares fields `translation: Vec2`, `rotation: f32`, `scale: Vec2`
- AND each field exposes type, default value, and constraints where applicable

### Requirement: Name schema has a single string field

The system MUST define `editor.Name` with a single field `name: string` defaulting to `""`.

#### Scenario: Name schema defaults

- GIVEN the Name schema
- WHEN inspected
- THEN it has a single field `name: string` with default value `""`

### Requirement: Sprite2D schema includes an AssetReference

The system MUST define `editor.Sprite2D` with fields `asset`, `color`, `anchor`, where `asset` is an `AssetReference` (logical path string).

#### Scenario: Sprite2D asset reference is a logical path string

- GIVEN the Sprite2D schema
- WHEN inspected
- THEN it has fields `asset: AssetReference`, `color: Color`, `anchor: Anchor`
- AND `asset` is a logical Project path string (e.g., `"assets/characters/player.png"`), not a runtime handle

### Requirement: Visible and Locked are editorial-only

The system MUST flag `editor.Visible` and `editor.Locked` as not exporting to Bevy (editorial metadata).

#### Scenario: Visible and Locked export flag

- GIVEN the Visible and Locked schemas
- WHEN inspected
- THEN each declares a single `bool` field
- AND both are flagged as editorial-only (not exported to Bevy)

### Requirement: Registry is a single global instance per editor session

The system MUST expose one registry instance per editor session, scoped outside the Bevy World.

#### Scenario: Single registry instance, outside the Bevy World

- GIVEN the editor core is initialized
- WHEN the registry is accessed multiple times
- THEN the same registry instance is returned each time
- AND it lives outside the Bevy World (per ADR-0002)

---

## §4. Out-of-Scope Behaviors (explicit non-goals)

The following behaviors are NOT part of this change and MUST NOT be implemented here:

- Command system / Operation Log / undo-redo (Hito 0 §6.4 — separate change)
- OPFS persistence (Hito 0 §5.2 — separate change)
- DynamicScene Export adapter (Hito 0 §6.5 — separate change)
- Hierarchy / Inspector UI panels (separate change)
- User-defined schemas (`game.*` components — out of scope for Hito 0 per §3.2)

---

## §5. Acceptance Criteria

1. Every §2 scenario passes via Rust unit tests.
2. Every §3 scenario passes via Rust unit tests.
3. A JSON roundtrip test for a scene with 1+ entities is implemented and passing.
4. Spike migration: the Bevy `setup()` reads the `SceneDocument` and spawns the sprite from it (no hardcoded sprite).
5. A new Playwright test validates a scene with entities renders.
6. WASM builds cleanly with the new `serde` / `serde_json` dependencies.

---

## §6. Test Plan

| Section | Scenarios | Test type | Rough count |
|---|---|---|---|
| §2.1, §2.6, §2.7, §2.9 | Serialize + shapes + version | Rust unit (`document.rs`) | 4 |
| §2.2, §2.3, §2.8 | Deserialize + roundtrip + unknown fields | Rust unit (`document.rs`) | 3 |
| §2.4, §2.5 | ID immutability + opacity | Rust unit (`document.rs`) | 2 |
| §2.10 | Instance structure | Rust unit (`document.rs`) | 1 |
| §3.1, §3.4, §3.5, §3.6, §3.7 | Seeded schemas + field defs | Rust unit (`schema.rs`) | 5 |
| §3.2, §3.3 | Lookup hit + miss | Rust unit (`schema.rs`) | 2 |
| §3.8 | Global singleton | Rust unit (`schema.rs`) | 1 |
| Spike migration | Scene with entities renders | Playwright E2E (extend `engine.spec.ts`) | 1 |
| **Total** | | | **~19 tests** |

Dev cycle: `cargo test` (Rust units) + `just wasm` (build) + `just test` (Playwright E2E).