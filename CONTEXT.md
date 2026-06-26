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

**Entity Template**:
A reusable editor-owned template stored in the Project that can instantiate one or more Entities with predefined Component Instances.
_Avoid_: Prefab, archetype, runtime entity

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

Dev: "Then when we need runtime integration, we generate a DynamicScene Export instead of treating Bevy serialization as the source of truth."

Domain Expert: "Right. The Bevy 2D Editor owns the editing model; Bevy consumes an export."
