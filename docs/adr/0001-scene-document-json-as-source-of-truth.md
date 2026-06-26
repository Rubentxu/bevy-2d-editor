# SceneDocument uses JSON as source of truth, not RON or DynamicScene

## Status

Accepted

## Context

The Bevy 2D Editor must store scene data that includes editor-owned metadata (stable IDs, human-readable names, operation log rationale, schema references, validation state) that does not exist in Bevy's runtime types. Bevy natively serializes scenes to RON via `DynamicScene`, and `DynamicScene` can round-trip through `Reflect`-derived components.

The question is whether the editor's authoritative document should be:

- **A.** A custom JSON document owned by the editor, with export adapters to `DynamicScene`/RON when Bevy integration is needed.
- **B.** RON, because Bevy already speaks RON and it feels more "native."
- **C.** `DynamicScene` itself, serialized directly, so the editor and runtime share the same model.

## Decision

We chose **Option A**: the `SceneDocument` is a custom JSON document owned by the editor. Bevy's `DynamicScene` and RON are export/integration targets, not the primary editing model.

## Considered Options

### Option A — Custom JSON document (chosen)

The editor owns a stable JSON format with:

- Stable entity IDs separate from Bevy runtime `Entity` values
- Component schemas with metadata (display name, fields, defaults, version, constraints)
- Component instances that reference schemas by `type_id` and carry only field values
- An operation log of semantic commands with authorship and rationale
- Human-readable asset references as logical Project paths
- Validation results and unknown-field preservation for forward compatibility

**Pros:**

- The editor document carries metadata Bevy does not model (stable IDs, editor names, operation history, schema references).
- JSON is the most widely tooled format for inspection, debugging, AI tooling, and browser-native parsing.
- The editor is not coupled to Bevy's serialization format, which changes between Bevy versions.
- Schema-driven validation and field-level editing are first-class.
- Future AI agents can read, diff, and propose changes to a structured document more naturally than to reflected RON.

**Cons:**

- An export adapter to `DynamicScene` must be maintained.
- Component schemas must be kept in sync with Rust types in the user's Bevy project.
- The editor is not "pure Bevy" in storage, which surprises people who expect RON everywhere.

### Option B — RON as primary format

Use Bevy's native RON serialization for scenes.

**Rejected because:** RON does not natively carry editor metadata (stable IDs, operation log, schema references, validation state). You would either extend RON with non-standard fields (fragile) or maintain a parallel metadata layer (which is effectively Option A with extra steps). RON is also harder to parse and inspect in browser tooling than JSON.

### Option C — DynamicScene as the primary model

The editor operates directly on Bevy's `DynamicScene` and serializes from there.

**Rejected because:** `DynamicScene` is a runtime type. It uses `Entity` IDs that are not stable across runs. It does not carry editor metadata. Coupling the editor's authoritative model to `DynamicScene` would make undo/redo, stable references, AI tooling, and schema evolution significantly harder. The research synthesis explicitly recommended: "Do not make Bevy's runtime serialization the editor's authoritative domain model."

## Consequences

- The editor maintains a `DynamicScene Export` adapter that materializes `SceneDocument` data into a Bevy-compatible runtime scene. This adapter must be versioned and tested.
- Component schemas in the `Component Schema Registry` are the contract between editor data and Bevy runtime types. When a schema changes, the editor preserves unknown fields (see forward-compatibility policy) rather than silently dropping data.
- The `SceneDocument` JSON format must be versioned. Old scenes must remain loadable; migrations are explicit, not automatic.
- Future AI agents operate on the JSON document and its semantic command log, not on raw Bevy runtime state. This is by design.
- Bevy version upgrades affect the export adapter, not the editor's core document model. This isolates editor stability from Bevy churn.
