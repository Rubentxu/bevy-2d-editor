# Proposal: Component Schema Authoring

## Intent

Hito 0 §3.2 requires users to define custom `game.*` component schemas through the editor. The data layer (schema-registry-persistence) already exposes `register_schema`/`save_schema`/`delete_schema`/`list_schemas`, but there is no UI to create or edit schemas — users can only consume the 5 hardcoded `editor.*` built-ins. This change closes that gap: a form-driven authoring panel that lets users build, persist, and delete user-defined component schemas, making them immediately available in the AddComponent dropdown.

## Scope

### In Scope
- "New Schema" panel/dialog accessible from the editor
- Schema metadata: type_id (`game.*` enforced, `editor.*` rejected), display_name, `exports_to_bevy` toggle
- Field builder: add/remove/reorder fields; each with name, `FieldType` picker, default value, optional constraints
- Constraint editor per field (Min/Max/NonEmpty where applicable to the type)
- Save → `register_schema(json)` + `save_schema(type_id)`; appears in AddComponentButton immediately
- Delete user-defined schema (built-ins protected; rejects `editor.*`)
- Inline validation + error states (duplicate type_id, invalid prefix, empty field name)

### Out of Scope
- Schema versioning/migration beyond existing `version: "0.1"` (future)
- Schema inheritance/composition (future)
- Editing a schema's fields once entities already use it (data-migration concern, deferred)
- Drag-based reorder (button-based up/down only)

## Capabilities

> CONTRACT with sddk-spec. `openspec/specs/` currently contains only `entity-reparent-dnd`; the persistence capabilities are not yet archived.

### New Capabilities
- `schema-authoring`: Form-driven UI to create, edit, and delete user-defined `game.*` component schemas — fields with types, defaults, and constraints — persisted via the schema-registry-persistence layer.

### Modified Capabilities
None. (`AddComponentButton` already reads `list_schemas()` dynamically, so user-defined schemas surface automatically once registered — no spec-level change.)

## Approach

A modal panel (`SchemaAuthoringPanel.tsx`) opened from an inspector/toolbar entry point. It uses existing `engine-bridge.ts` functions — **no new wasm surface**. Each field row renders a value editor matching the picked `FieldType` for the default value (reusing the per-type editing patterns already in `ComponentEditor`). On save: build a `ComponentSchema` JSON → `register_schema` (validates prefix + uniqueness) → `save_schema` (persists to OPFS). Delete confirms → `delete_schema` + `unregister_schema`. Single-user, no concurrency.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/components/SchemaAuthoringPanel.tsx` | New | Modal: metadata + field builder + constraint editor |
| `frontend/src/components/SchemaFieldRow.tsx` | New | Per-field row: name, type picker, default editor, constraints |
| `frontend/src/components/InspectorPanel.tsx` | Modified | Add "New Schema" entry point |
| `frontend/src/components/AddComponentButton.tsx` | Modified | Disable delete affordance on `editor.*` built-ins |
| `frontend/src/engine-bridge.ts` | None | Already exposes needed functions |
| `frontend/tests/schema-authoring.spec.ts` | New | Playwright E2E: create, save, appears in dropdown, delete |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Invalid schema JSON reaches register_schema | Med | Client-side validation before call; inline errors |
| Deleting a schema used by entities leaves dangling refs | Med | Confirm dialog; document AddComponent fails if schema gone |
| Constraint UI: which types allow which constraints | Low | Map constraints to FieldType; disable inapplicable |
| Reorder UX confusion | Low | Up/down buttons, not drag |

## Rollback Plan

Frontend-only change. Revert the 2 new components + `InspectorPanel`/`AddComponentButton` edits. No backend/wasm changes, so revert is a single clean PR.

## Dependencies

- Requires: schema-registry-persistence change (register/save/delete/list) — already landed.
- Existing: React, `engine-bridge.ts`, `ComponentEditor` per-type value editors.

## Success Criteria

- [ ] Open "New Schema" panel from the inspector
- [ ] Create a schema with 3+ fields of different FieldTypes, with defaults + constraints
- [ ] `game.*` type_id accepted; `editor.*` rejected with inline error
- [ ] Save → schema appears in AddComponentButton dropdown without reload
- [ ] Reload page → schema persists (via `load_project` auto-restore)
- [ ] Delete a user schema removes it; built-in deletion blocked
- [ ] All existing Playwright + Rust tests pass (no regression)
