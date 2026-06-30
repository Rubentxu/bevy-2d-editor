# Bevy 2D Editor

Browser-based editor for building Bevy 2D game scenes that is intended to evolve into a Cursor-like IDE for Bevy games. This context captures the project language so future design and implementation work stays consistent.

## Language

**Bevy 2D Editor**:
The product as a whole: a browser-based editor for Bevy 2D games that starts as a scene editor and evolves toward a Cursor-like IDE with AI assistance.
_Avoid_: tool, app, engine editor, IDE (when you mean only the current MVP)

**Hito 0**:
The first milestone, focused on validating the scene editor foundation through functional criteria before AI editing or code editing features arrive.
_Avoid_: final MVP, full product, beta

**Project**:
The top-level editable workspace stored in browser persistence, containing scenes and shared definitions needed by the editor.
_Avoid_: repo, game, workspace (when you mean the editor-managed unit)

**SceneDocument**:
The editor's source-of-truth document for a scene, stored as stable JSON and owned by the editor rather than by Bevy runtime serialization.
_Avoid_: RON file, DynamicScene, runtime world

**Entity**:
A logical object inside a scene with an immutable stable ID and a separate human-facing name or slug.
_Avoid_: Bevy Entity, object, node

**Scene Asset**:
A reusable Bevy-aligned scene composition stored in the Project and intended to converge with Bevy Scene Notation (`.bsn`) assets. A Scene Asset can describe one Entity, a hierarchy of Entities, a reusable fragment, a UI composition, or a full level. Use a role/kind such as `actor`, `fragment`, `screen`, `level`, `ui`, or `effect` instead of creating separate prefab/collection concepts.
_Avoid_: prefab, blueprint, Entity Template, DynamicScene Export, Defold-style split between GameObject/Collection/Factory/Proxy

**Level Scene Asset**:
A Scene Asset with role `level` used as the editor-owned unit for level design. Level-specific data such as tile/object/semantic/generated layers belongs inside the Level Scene Asset unless future research proves it breaks the Scene Asset contract.
_Avoid_: LevelDocument, tilemap document, map file, room document, separate level asset type

**Level Layer**:
An explicit layer inside a Level Scene Asset that organizes level-design data by purpose, such as placed Scene Instances, tile painting, semantic IntGrid-like cells, or generated auto-layer output.
_Avoid_: raw tilemap layer (when the layer is not visual tiles), render layer (when you mean authoring data), physics layer (unless it specifically controls collision filtering)

**Scene Instance Layer**:
A Level Layer whose purpose is to organize placed Scene Instances inside a Level Scene Asset, such as actors, props, spawn points, pickups, doors, checkpoints, and triggers. A Scene Instance belongs to exactly one Scene Instance Layer; the layer owns the placement rather than merely referencing a global instance list.
_Avoid_: Object Layer, Entity Layer, GameObject layer, instance collection

**Scene Instance**:
A placed use of a Scene Asset inside a SceneDocument, represented as an asset reference plus instance-owned Component Instances and explicit local patches/overrides, with references and Stable IDs owned by the editor.
_Avoid_: prefab instance, cloned template, deep copy, Bevy Entity

**Component Override**:
A non-destructive patch applied by a Scene Instance to a specific Component Instance on an asset-local Entity inside the referenced Scene Asset. Component identity is explicit (`component_type_id`) and field paths only address fields inside that component.
_Avoid_: property override, prefab override, opaque patch, field path that hides the component type

**Project Asset Browser**:
The editor UI surface for browsing Project-level assets such as scenes, schemas, Scene Assets, and future level-design assets. It is not a filesystem browser and must show editor-owned logical assets, not raw OPFS implementation details.
_Avoid_: file explorer, asset folder, OS browser

**Scene Asset Authoring Mode**:
The isolated editing mode for a Scene Asset document. Changes affect the Scene Asset definition and later propagate to Scene Instances through explicit resync workflows.
_Avoid_: prefab mode (unless comparing with Unity), template editor, scene clone editor

**Override / Resync Workbench**:
The UI and workflow for inspecting, applying, reverting, rebinding, and resetting Scene Instance overrides after the referenced Scene Asset changes.
_Avoid_: merge conflict dialog, auto-fix panel, prefab override window

**Validation Center**:
The project-wide issue panel for broken references, missing schemas, invalid paths, export warnings, override conflicts, dirty scenes, and invalid AI proposals.
_Avoid_: console (when you mean structured project health), error list (when warnings/suggestions also apply)

**Runtime Preview Inspector**:
The read-only UI for understanding how editor-owned data appears in the Bevy preview world, including ephemeral runtime mappings, preview metrics, and provenance.
_Avoid_: Bevy inspector (when you mean editor-owned preview diagnostics), runtime source of truth

**Entity Template**:
Legacy/transitional term for reusable entity compositions. Prefer Scene Asset for future-facing design aligned with Bevy's BSN roadmap.
_Avoid_: prefab, blueprint, archetype, runtime entity

**Stable ID**:
The opaque, immutable identifier used to reference an Entity across saves, undo/redo, and future agent operations.
_Avoid_: name, slug, label

**Component Schema Registry**:
The project-global catalog of component types and their field definitions, used by the inspector and validation.
_Avoid_: per-scene schema, per-entity schema, inline component definition

**Component Instance**:
The values attached to an Entity for one component type, referring back to a schema in the Component Schema Registry.
_Avoid_: full schema copy, ad hoc props

**Asset Reference**:
The logical Project path used by editor-owned data to point at an asset, such as `assets/characters/player.png`.
_Avoid_: runtime handle, opaque asset id, absolute filesystem path

**Operation Log**:
The reversible history of typed editor commands, used for undo/redo and future agent auditing.
_Avoid_: raw event stream, UI history

**DynamicScene Export**:
The adapter that materializes editor-owned scene data into a Bevy-compatible runtime scene representation.
_Avoid_: source of truth, primary scene model

## Example Dialogue

Dev: "For Hito 0, the SceneDocument stays as JSON and each Entity keeps its own stable ID."

Domain Expert: "Good. The Entity name can change, but the Stable ID must not, otherwise references and undo break."

Dev: "And the inspector reads field definitions from the Component Schema Registry, not from each Entity."

Domain Expert: "Exactly. Component Instances carry values; schemas stay global to the Project."

Dev: "When we define reusable content, we call it a Scene Asset, not a prefab or Entity Template."

Domain Expert: "Exactly. Scene Asset keeps us aligned with Bevy's BSN roadmap, while Scene Instance describes a placed use inside a SceneDocument."

Dev: "Then when we need runtime integration, we generate a DynamicScene Export instead of treating Bevy serialization as the source of truth."

Domain Expert: "Right. The Bevy 2D Editor owns the editing model; Bevy consumes an export."
