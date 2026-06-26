# Hito 0 Specification — Bevy 2D Scene Editor MVP

> **Status:** Accepted (all architectural decisions closed)
> **Vision:** [CONTEXT.md](../CONTEXT.md) · **Key decision:** [ADR-0001](./adr/0001-scene-document-json-as-source-of-truth.md)
> **Research basis:** `/tmp/opencode/cursor-research-2026-06-25.md`

---

## 1. Overview

Hito 0 is the first milestone of the Bevy 2D Editor. It validates that a browser-based scene editor for Bevy 2D games is technically viable and produces data that a real Bevy application can consume.

Hito 0 is **not** the full product vision. The vision is a Cursor-like AI IDE for Bevy 2D games. Hito 0 is the foundation: a scene editor whose data model is designed to be AI-friendly from day 1, so that future milestones (AI copilot, transactional agent, code awareness, full IDE) can be built on top without a rewrite.

---

## 2. Product Vision Context

```
Hito 0: Scene editor foundation (this spec)
    ↓
Hito 1: Reflection/schema layer + AI scene copilot (Ask + Plan)
    ↓
Hito 2: Transactional scene agent (Apply with approval)
    ↓
Hito 3: Playtest/verification loop
    ↓
Hito 4: Code awareness (Rust indexing, LSP)
    ↓
Hito 5: Hybrid local/cloud runtime (rust-analyzer, cargo)
    ↓
Hito 6: Cursor-like Bevy IDE (code editor, multi-agent, Git)
```

Hito 0 must produce a substrate that evolves through this roadmap without rewrites. The 4 AI-friendly constraints (Section 6) exist for this reason.

---

## 3. Scope

### 3.1 In Scope

| Area | What ships |
|---|---|
| **SceneDocument** | JSON source of truth with stable IDs, schemas, operation log |
| **Entities** | Create, edit, move, delete, rename, reparent |
| **Hierarchy** | Parent/child as canonical scene data |
| **Components** | 5 built-in editor components (Section 7) |
| **Inspector** | Field editing driven by Component Schema Registry |
| **Viewport** | Bevy WASM canvas, pan, zoom, grid, selection, move, basic gizmo |
| **Undo/Redo** | Semantic command log, gesture-batched |
| **Save/Load** | OPFS persistence, roundtrip without data loss |
| **DynamicScene Export** | Materialize SceneDocument into a Bevy-compatible scene |

### 3.2 Out of Scope (deferred to later milestones)

- AI features of any kind (Plan, Diff, Apply, chat, agent)
- User-defined component schemas (custom `game.*` components)
- Project panel (multi-scene browsing, asset browser)
- Asset pipeline (import, preview, thumbnails)
- Tilemap, colliders, paths, spawn zones
- Code editor, LSP, Rust intelligence
- Server-side anything
- Entity Template editing UI (templates exist as data but no visual editor)
- Scene variants / room inheritance
- Multiplayer / collaboration

---

## 4. Success Criteria

Hito 0 is done when all 5 criteria pass:

1. **Visual editing works:** create, edit, and move entities in the viewport without errors
2. **Save/load roundtrip:** save a scene with 50+ entities, reload, and verify zero data loss
3. **Bevy integration:** a real Bevy app (separate from the editor) loads the exported scene and renders it correctly
4. **Undo/redo:** works on create, delete, move, rename, reparent, and property changes
5. **AI-friendly core:** all 4 constraints (Section 6) are implemented and tested

---

## 5. Architecture

### 5.1 Runtime

**100% web-only.** No server-side component in Hito 0.

```
React App
  ├── Shell / Layout
  ├── Viewport (Bevy canvas host)
  ├── Hierarchy Panel
  ├── Inspector Panel
  └── Command Bridge
          │
          ▼
Rust/WASM Editor Core
  ├── SceneDocument (JSON source of truth)
  ├── Command System
  ├── Undo/Redo (Operation Log)
  ├── Selection State
  ├── Component Schema Registry
  ├── Validation
  ├── Save/Load (OPFS)
  ├── DynamicScene Export
  └── Bevy App / Viewport
          │
          ▼
Bevy ECS Preview World
  ├── Render entities
  ├── Editor camera
  ├── Gizmos
  ├── Picking
  └── Visual feedback
```

### 5.2 Persistence

**OPFS (Origin Private File System).** Browser-native, filesystem-like, supports binary assets for future milestones. Sync access via worker broker pattern.

**OPFS directory structure:**

```
project.json              ← Project metadata, registry reference
scenes/                   ← SceneDocument files (*.scene.json)
schemas/                  ← Component Schema Registry entries (*.schema.json)
assets/                   ← Asset files (images, future: spritesheets, fonts)
entities/                 ← Entity Template files (*.template.json)
.editor/                  ← Editor state (selection, viewport, preferences)
```

ECS concepts (Entities, Components, Systems) shape the conceptual architecture and UI navigation, but not all map 1:1 to physical folders. `systems/` is conceptual until code/runtime editing becomes first-class.

### 5.3 Communication Model

**Unidirectional command queue.** React ↔ Rust/WASM.

- React sends **typed commands** to the core (never mutates document state directly)
- Core processes commands, applies to SceneDocument, emits **snapshots** back
- React owns **only UI state** (panel layout, form input, transient interaction)
- React **never** owns document state

The command surface doubles as the future AI agent tool API.

```ts
// React → WASM (command)
editor.command({
  type: "SetComponentField",
  entityId: "ent_01J...",
  component: "editor.Transform2D",
  field: "translation",
  value: { x: 200, y: 64 }
});

// WASM → React (snapshot)
const snapshot = editor.getHierarchySnapshot();
```

---

## 6. Data Model — AI-Friendly Core

The SceneDocument is designed with 4 mandatory constraints so it can evolve into AI-assisted editing without refactor.

### 6.1 SceneDocument

**Format:** JSON (see [ADR-0001](./adr/0001-scene-document-json-as-source-of-truth.md))

The editor owns a custom JSON document. Bevy's `DynamicScene` and RON are **export targets**, not the primary model.

```json
{
  "version": "0.1",
  "scene_id": "scene_01J...",
  "name": "level_01",
  "entities": [
    {
      "id": "ent_01J...",
      "name": "PlayerSpawn",
      "parent": null,
      "components": [
        {
          "type_id": "editor.Transform2D",
          "values": { "translation": { "x": 128, "y": 64 }, "rotation": 0, "scale": { "x": 1, "y": 1 } }
        }
      ]
    }
  ]
}
```

### 6.2 Constraint 1: Stable IDs

**Hybrid:** opaque immutable ID + human-readable name.

- `id`: opaque, immutable, never reused (e.g., ULID)
- `name`: human-readable, mutable, for UX and search
- Renaming an entity **never** mutates its `id`
- References between entities use `id`, never `name`
- Entity Templates use **local IDs** internally; instantiation mints fresh global IDs

### 6.3 Constraint 2: Component Schemas with Metadata

**Global registry.** Schemas live in `schemas/` and are referenced by `type_id`.

Each schema includes:
- `type_id`: stable, namespaced identifier (e.g., `editor.Transform2D`)
- `display_name`: human-readable label
- `fields`: field definitions with type, default, constraints
- `version`: schema version for future migrations

Component Instances carry **only values**, referencing the schema:

```json
{
  "type_id": "editor.Transform2D",
  "values": { "translation": { "x": 128, "y": 64 }, "rotation": 0, "scale": { "x": 1, "y": 1 } }
}
```

### 6.4 Constraint 3: Reversible Operation Log

**Semantic commands**, not raw JSON diffs.

Command types:
- `CreateEntity`
- `DeleteEntity`
- `AddComponent`
- `RemoveComponent`
- `SetComponentField`
- `ReparentEntity`
- `InstantiateEntityTemplate`
- `RenameEntity`

Each command records: authorship, timestamp, rationale (for future agent auditing), and is fully reversible.

**Granularity:** interactive gestures (e.g., dragging an entity in the viewport) are batched into a single history entry, not per-frame deltas.

### 6.5 Constraint 4: DynamicScene Export

The editor materializes SceneDocument data into a Bevy-compatible `DynamicScene` via an export adapter. This is tested as part of Success Criterion 3 (a real Bevy app loads and renders the exported scene).

The export adapter maps:
- `editor.Transform2D` → Bevy `Transform` (local to parent)
- `editor.Sprite2D` → Bevy sprite components (with asset loading)
- `editor.Name` → Bevy `Name`
- `editor.Visible` / `editor.Locked` → **not exported** (editorial-only metadata)

### 6.6 Entity Hierarchy

**Canonical scene data.** Parent/child is part of the document, not just visual grouping.

- Each entity has an optional `parent` field referencing a stable ID
- Transforms are **local to parent** (matches Bevy's `Transform` + `GlobalTransform` model)
- Reparenting preserves world-space position (using `GlobalTransform::reparented_to()` pattern)
- Hierarchy is export-relevant: the DynamicScene Export preserves parent/child structure

### 6.7 Entity Template

A reusable editor-owned artifact stored in `entities/`. Can instantiate a **tree of Entities** (not just one).

- Has an explicit **root Entity**
- Internal entities use **local template IDs**
- On instantiation, the editor generates **fresh global stable IDs** in the Scene
- Template-local IDs never leak as Scene stable IDs

### 6.8 Asset References

**Logical Project paths** (Defold-inspired).

- Example: `assets/characters/player.png`
- Human-readable, inspectable in JSON
- Rename/move operations must go through the editor, which **automatically rewrites all references**
- Renaming outside the editor is not supported (same policy as Defold)
- Internal caches/indexes may exist later but never replace document-level truth

### 6.9 Forward Compatibility Policy

When a schema changes and a Component Instance has fields the new schema doesn't recognize:

- **Load:** preserve all unknown fields
- **Validate:** mark unknown fields as orphaned/invalid
- **Inspector:** show orphaned fields in a compatibility section (read-only)
- **Save:** keep orphaned fields unless the user explicitly confirms a cleanup/migration
- **Never** auto-delete unknown data

---

## 7. Built-in Components

Hito 0 ships exactly 5 editor components:

| Component | Fields | Exports to Bevy? |
|---|---|---|
| `editor.Name` | `name: string` | Yes → `Name` |
| `editor.Transform2D` | `translation: Vec2`, `rotation: f32`, `scale: Vec2` | Yes → `Transform` |
| `editor.Sprite2D` | `asset: AssetReference`, `color: Color`, `anchor: Anchor` | Yes → sprite bundle |
| `editor.Visible` | `visible: bool` | No (editorial only) |
| `editor.Locked` | `locked: bool` | No (editorial only) |

Each component has a full schema entry in the Component Schema Registry from day 1.

---

## 8. Reference Model Summary

| Concept | Reference format | Mutable? |
|---|---|---|
| Entity → Entity | Stable ID (opaque) | No (ID immutable) |
| Component Instance → Schema | `type_id` (string) | No (type binding) |
| Component → Asset | Logical Project path | Path can change via editor refactor |
| Entity Template → Scene | Instantiation (new IDs minted) | One-shot, not a live link |

---

## 9. Runtime Interaction (resolved)

### 9.1 Preview World Sync

**Selective rebuild on command commit.** Scene entities are fully rebuilt from the SceneDocument each time a command is confirmed. Editor entities (camera, gizmos, grid) persist across rebuilds. A `stable_id ↔ Bevy Entity` lookup table regenerates each rebuild for selection and picking. No incremental sync in Hito 0.

During interactive gestures (e.g., dragging an entity), the viewport updates visually without a command commit. Rebuild fires only on gesture end.

### 9.2 Selection Model

**Multi-selection from day 1.** Click selects, shift+click adds, drag on empty space performs box select. Selection state lives in the Rust core as a Bevy Resource, emits snapshots to React, and restores on undo/redo.

### 9.3 Canvas Rendering

**Single Bevy WASM instance renders everything inside the canvas.** Scene entities, grid, gizmos, selection highlights, and box select overlay all live in the same Bevy world. React never touches the canvas.

Editor state (`SelectionState`, `GridConfig`, `SnapConfig`) lives as Bevy Resources. SceneDocument, operation log, and schema registry live outside the World as Rust structs owned by the editor core.

### 9.4 Viewport Interaction (Hito 0)

| Feature | Behavior |
|---|---|
| Pan | middle-click + drag, or space + click-drag |
| Zoom | mouse wheel, centered on cursor |
| Grid | configurable spacing, toggle on/off |
| Snap | grid-based, toggle with key |
| Move gizmo | X/Y arrows on selected entity |
| Selection | outline on selected entity, box select with semi-transparent rect |

Excluded from Hito 0: rotation gizmo, scale gizmo, multi-gizmo, pivot editing.

### 9.5 DynamicScene Export Mapping

| SceneDocument field | Bevy runtime |
|---|---|
| `editor.Name.values.name` | `Name` |
| `editor.Transform2D.values.translation` | `Transform.translation` (z=0) |
| `editor.Transform2D.values.rotation` | `Transform.rotation` (z-axis) |
| `editor.Transform2D.values.scale` | `Transform.scale` (z=1) |
| `editor.Sprite2D.values.asset` | `Sprite` + `Handle<Image>` (loaded by path) |
| `editor.Sprite2D.values.color` | `Sprite.color` |
| `editor.Sprite2D.values.anchor` | Computed `Transform` offset |
| Entity `parent` | `ChildOf(parent_entity)` |
| `editor.Visible` / `editor.Locked` | Not exported (editorial only) |

Error rules:
- Missing asset → warning, entity exports without sprite (export does not fail)
- Unknown schema → skip component, warning
- Unknown field → preserved in document, not exported

---

## 10. Decision Log

All decisions captured during the grilling session, persisted in Engram memory:

| # | Decision | Topic key |
|---|---|---|
| 1 | Vision: Cursor-like AI IDE for Bevy 2D | `product/vision` |
| 2 | Hito 0 = scene editor MVP + AI-friendly core | `product/hito-0` |
| 3 | Success criterion: functional validation | `product/hito-0-success-criterion` |
| 4 | Persistence: OPFS | `product/hito-0-persistence` |
| 5 | Runtime: web-only | `product/hito-0-runtime` |
| 6 | SceneDocument: JSON source of truth | `architecture/scene-document-format` |
| 7 | ADR-0001 written | `adr/0001-scene-document-json` |
| 8 | Stable IDs: hybrid opaque + name | `architecture/entity-id-strategy` |
| 9 | Schemas: global registry | `architecture/component-schema-registry` |
| 10 | Project structure: Defold-inspired | `architecture/project-structure-inspiration` |
| 11 | ECS concepts: conceptual before physical | `architecture/ecs-conceptual-vs-physical-layout` |
| 12 | Entity Template: canonical term | `language/entity-template-term` |
| 13 | Entity Template: instantiates trees | `architecture/entity-template-instantiation` |
| 14 | Assets: Defold-like paths | `architecture/asset-reference-format` |
| 15 | Reference model: paths + stable IDs | `architecture/reference-model` |
| 16 | Operation Log: semantic commands | `architecture/operation-log-model` |
| 17 | Operation granularity: gesture-batched | `architecture/operation-log-granularity` |
| 18 | Hierarchy: canonical scene data | `architecture/entity-hierarchy-model` |
| 19 | Transform: local to parent | `architecture/transform-model` |
| 20 | Built-in components: 5 editor components | `architecture/hito-0-builtin-components` |
| 21 | React-WASM: unidirectional command queue | `architecture/react-wasm-bridge` |
| 22 | Hito 0 adds entities/ folder | `architecture/project-layout-entities-folder` |
| 23 | Preview world: selective rebuild on commit | `architecture/preview-world-sync` |
| 24 | Multi-selection + single Bevy renderer | `architecture/selection-and-rendering` |
| 25 | Viewport: pan/zoom/grid/snap/move gizmo | `architecture/hito-0-viewport` |
| 26 | DynamicScene Export mapping + error rules | `architecture/dynamic-scene-export-mapping` |

---

## 11. References

- [CONTEXT.md](../CONTEXT.md) — Project glossary and canonical language
- [ADR-0001](./adr/0001-scene-document-json-as-source-of-truth.md) — SceneDocument JSON as source of truth
- [Especificación-de-idea.md](../Especificación-de-idea.md) — Original v0.1 spec (superseded by this document for Hito 0)
- Research synthesis: `/tmp/opencode/cursor-research-2026-06-25.md` — Cursor AI and agent landscape research
