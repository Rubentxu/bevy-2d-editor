# Suggested CONTEXT.md Glossary Additions

Merge these terms into the authoritative repository `CONTEXT.md`.

## Semantic Editor Model
The representation-independent editor-owned domain state. JSON, BSN and Bevy runtime scenes are representations/projections, not the authority.

## EditorSession
Explicit application-level owner of mutable project/editing state for one active editor session. Replaces scattered implicit global stores.

## Transaction Kernel
Shared application infrastructure for validation, batches, inverse/rollback, history, effects and ChangeSet application. It does not erase domain-specific command types.

## ChangeSet
A reviewable group of typed semantic operations with origin, actor, rationale, affected resources, validation, semantic diff, runtime/build effects and approval policy.

## Change Workbench
Unified review/apply/rollback surface for non-trivial ChangeSets from humans, recipes, agents, imports, migrations, plugins and runtime apply-back.

## Editor Capability
A stable user/tool action exposed by the application layer, such as `PlaceSceneInstance`, `ExtractSelectionAsSceneAsset` or `ReimportExternalSource`. Capabilities can coordinate multiple bounded contexts.

## World Workspace
Spatial/topological authoring surface for arranging and connecting existing Level Scene Assets. It does not replace level content documents.

## Recipe
Versioned authoring-time workflow that converts intent and parameters into a validated ChangeSet. Distinct from runtime Logic Bricks.

## External Source
Provenance record linking externally authored data (Aseprite/LDtk/Tiled, etc.) to editor resources for semantic reimport and conflict handling.

## Runtime Delta
An observed play-mode difference between authoring baseline and runtime value. It is transient until selected through Runtime Apply-Back.

## Scope of Change
Explicit target level for a semantic edit: instance, selected instances, definition, component default or supported source definition.

## Editor Extension
A capability-limited contribution to actions, validators, recipes, importers, inspectors, panels or diagnostics. Extensions do not receive unrestricted mutable EditorSession access.
