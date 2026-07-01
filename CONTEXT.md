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
A Level Layer whose purpose is to organize placed Scene Instances inside a Level Scene Asset, such as actors, props, spawn points, pickups, doors, checkpoints, and triggers. A Scene Instance belongs to exactly one Scene Instance Layer; the layer owns the placement rather than merely referencing a global instance list. Each layer carries a `Layer ID`, a `Layer Kind`, a layer-level `name`, an `order`, and its placed Scene Instances.
_Avoid_: Object Layer, Entity Layer, GameObject layer, instance collection

**Scene Instance**:
A placed use of a Scene Asset inside a SceneDocument, represented as an asset reference plus instance-owned Component Instances and explicit local patches/overrides, with references and Stable IDs owned by the editor. A Scene Instance carries three distinct concept groups: (1) **asset components** in the referenced Scene Asset, (2) **instance components** owned by the placed occurrence itself (e.g. `editor.Transform2D` for placement), and (3) **Component Overrides** that non-destructively patch asset-local Entity components.
_Avoid_: prefab instance, cloned template, deep copy, Bevy Entity

**Component Override**:
A non-destructive patch applied by a Scene Instance to a specific Component Instance on an asset-local Entity inside the referenced Scene Asset. Component identity is explicit (`component_type_id`) and field paths only address fields inside that component.
_Avoid_: property override, prefab override, opaque patch, field path that hides the component type

**Override Count Badge**:
A UI element in the Inspector Panel showing per-status counts of component overrides for a selected Scene Instance. Displays `active | stale | orphaned | conflict` badges derived from the instance's `component_overrides` and `orphaned_component_overrides` vectors.
_Avoid_: override counter, status badge (when you mean this specific element)

**Per-field Override Indicator**:
A visual indicator rendered next to each field in a ComponentCard when the selected entity belongs to a Scene Instance. The indicator is a colored dot reflecting the override's status: Active=blue, Stale=warning (amber), Conflict=error (red), Orphaned=dimmed (grey). Status is computed via `override_field_status_wasm` and passed as `fieldOverrideStatus` prop to ComponentCard.
_Avoid_: override dot, status dot, field indicator

**Resync Warning Banner**:
A warning banner displayed in the Inspector Panel when `get_resync_reports` reports non-zero `stale` or `conflict` counts for the selected Scene Instance. Shows a count of problem overrides and an "Open Workbench" button that navigates to the Override / Resync Workbench. The button may use a placeholder href until the workbench is fully implemented.
_Avoid_: stale warning, conflict alert, override banner

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
The values attached to an Entity for one component type, referring back to a schema in the Component Schema Registry. When attached to a Scene Instance, it serves as an **instance component** (placement-time data such as `editor.Transform2D`).
_Avoid_: full schema copy, ad hoc props

**Asset Reference**:
The logical Project path used by editor-owned data to point at an asset, such as `assets/characters/player.png`.
_Avoid_: runtime handle, opaque asset id, absolute filesystem path

**Operation Log**:
The reversible history of typed editor commands, used for undo/redo and future agent auditing.
_Avoid_: raw event stream, UI history

**Layer ID**:
The opaque, immutable identifier used to reference a Level Layer (e.g., a `SceneInstanceLayer`) across saves, undo/redo, and future agent operations. Serializes as a plain string and never collides with another layer id within the same Level Scene Asset.
_Avoid_: name, slug, label

**Layer Kind**:
The soft-typed category of a `SceneInstanceLayer`: `actors`, `props`, `spawns`, `triggers`, `collision`, or `custom`. The kind is set at layer creation and is immutable afterwards to keep layered semantics stable for downstream tooling.
_Avoid_: hard enum without custom, vendor-specific layer type names

**Preview Metrics**:
Live runtime data of the Bevy preview world exposed to the JS-side inspector: frames per second, last frame time in milliseconds, and the total number of preview rebuilds. Updated by the Bevy `emit_events` and `rebuild_preview_world` systems.
_Avoid_: Bevy entity handles, raw renderer stats, platform-specific counters

**Preview Mapping**:
Per-instance runtime projection list. Each entry references a `StableId`, a `LocalId`, an `AssetReference`, and a component count. StableId-only on the editor side — Bevy Entity IDs are NOT exposed to the editor model.
_Avoid_: leaking Bevy entity indices, raw entity handles, transient world pointers

**DynamicScene Export**:
The adapter that materializes editor-owned scene data into a Bevy-compatible runtime scene representation.
_Avoid_: source of truth, primary scene model

**BSN Export**:
The process of converting a `SceneAssetDocument` into raw `.bsn` text via the `BsnExporter` trait. The output is `.bsn`-native syntax (no Rust `commands.spawn_scene_list(...)` wrapper, no `bsn_list![...]` macro, no Rust tuple commas in `Children`). The `EditorCoreBsnExporter` provides the working impl; when Bevy PR #23639 lands, a `BevyBsnExporter` swap-in will use Bevy's official writer. BSN Export is output-only in Hito 3; import (`.bsn` → `SceneAssetDocument`) is deferred.
_Avoid_: DynamicScene export (different format), `.bsn` asset import, round-trip conversion in Hito 3

## Logic Bricks (Behavior Authoring)

**Logic Bricks**:
The visual behavior authoring system for the Bevy 2D Editor. Users wire common 2D gameplay (jump, collision, health, timers, proximity) as node/edge graphs without leaving the editor and without a scripting VM. Behavior is evaluated by a compiled dispatch scheduler, not a dynamic script interpreter.
_Avoid_: Blueprint (when referring to this editor's system), visual scripting VM, event graph, scripting language

**LogicGraphAsset**:
The editor-owned reusable behavior document, symmetric to `SceneAssetDocument` but carrying `nodes: Vec<LogicNode>` and `edges: Vec<LogicEdge>` instead of entities. Stored as stable JSON. Has `SceneAssetRole::Logic`. Reuses the Scene Asset relationship idea — opaque IDs, typed edges, validation, stable JSON — but exposes explicit `LogicEdge` records for wiring. It is NOT a Bevy runtime scene and is NOT exported to `.bsn`.
_Avoid_: behavior tree, state machine asset, script file, logic component (when you mean the reusable asset)

**LogicInstance**:
A placed use of a `LogicGraphAsset` bound to exactly one Scene Instance in v1. Mirrors `SceneInstance` but binds behavior rather than scene composition. The binding references the target Scene Instance through editor-owned Stable IDs and is non-overridable in v1. Multi-entity addressing is future work and must not be implied by the term today.
_Avoid_: logic binding (when you mean the placed instance), behavior instance, script instance

**RustController**:
A `LogicNode` kind that references a `controller_id` resolved at runtime to a compiled `NodeEvaluator` trait impl. This is the Rust-native compiled extension point — the Unity/C#-like escape hatch — with no scripting and no user-authored Rust snippets inside graph nodes. v1: built-in controllers compiled into the editor only.
_Avoid_: script node, code node, dynamic controller, plugin controller (until the plugin system exists)

**Pattern Block**:
A curated, versioned `LogicGraphAsset` shipped as a built-in recipe for common 2D patterns (e.g. `recipes/platformer_jump`, `recipes/health_damage`). Reused via `LogicInstance` exactly like any user-authored graph. Also called a "Recipe".
_Avoid_: macro, template graph, preset (when you mean a shipped built-in recipe)

**Logic Evaluation Schedule**:
The event/change-driven dispatch scheduler in `editor-core` that evaluates projected `LogicInstance` graphs in preview. Sensors emit events; controllers and actuators run only when their inputs changed. It never iterates every graph every frame and executes no user text — it dispatches compiled bricks, not a VM.
_Avoid_: logic VM, script runtime, interpreter, evaluation loop (when you mean the scheduler)

**Sensor / Controller / Actuator**:
The three primary `LogicNode` roles in a Logic Bricks graph. **Sensor** nodes produce inputs (key press, collision, timer, health change, proximity). **Controller** nodes make decisions (if, gate, flip-flop, compare, math, sequence). **Actuator** nodes produce outputs (apply impulse, set animation, emit signal, spawn, destroy). Concrete behavior is selected by a `node_type_id`; the role only describes where the node sits in the Sensor → Controller → Actuator flow.
_Avoid_: input node / output node (when the BGE-inspired three-role taxonomy applies)

**LogicGraph**:
The node/edge graph data structure inside a `LogicGraphAsset`. Distinct from the asset (which carries metadata, version, path) — the graph is the behavioral content.
_Avoid_: logic map, node map, circuit

## Example Dialogue

Dev: "For Hito 0, the SceneDocument stays as JSON and each Entity keeps its own stable ID."

Domain Expert: "Good. The Entity name can change, but the Stable ID must not, otherwise references and undo break."

Dev: "And the inspector reads field definitions from the Component Schema Registry, not from each Entity."

Domain Expert: "Exactly. Component Instances carry values; schemas stay global to the Project."

Dev: "When we define reusable content, we call it a Scene Asset, not a prefab or Entity Template."

Domain Expert: "Exactly. Scene Asset keeps us aligned with Bevy's BSN roadmap, while Scene Instance describes a placed use inside a SceneDocument."

Dev: "Then when we need runtime integration, we generate a DynamicScene Export instead of treating Bevy serialization as the source of truth."

Domain Expert: "Right. The Bevy 2D Editor owns the editing model; Bevy consumes an export."
