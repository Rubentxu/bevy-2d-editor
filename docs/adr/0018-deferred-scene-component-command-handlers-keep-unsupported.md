# ADR-0018: Deferred SceneComponent Command Handlers Remain Unsupported

## Status

Accepted (2026-07-20) — Hito 7 (`scene-component-authoring-ux` PR1)

## Context

Hito 4 Order 7 (`scene-component-authoring`, ADR-0016, v0.75.0) introduced
three new variants on the editor `Command` enum so the AI proxy could
author SceneComponent schemas:

- `Command::CreateSceneComponent { schema }`
- `Command::UpdateSceneComponentFields { type_id, fields }`
- `Command::BindSceneToSchema { type_id, scene_asset_id }`

ADR-0016 deliberately left the `apply` handlers for these three variants
as `CommandError::Unsupported`, noting that "specialized apply handlers
will be added in a follow-up PR when the frontend needs them." The intent
was to keep `processor.rs` honest: routing the variants through the
generic `apply` / `undo` pipeline would silently mutate backend state
without UI parity.

The `scene-component-authoring-ux` change (`docs/sddk/scene-component-authoring-ux/`)
now needs to surface these operations in the UI: catalog-backed picker,
inline validation, and a "Place Instance" entry point. The PR1 design
file originally explored adding `command_scene_component::apply_create`
/ `apply_update` / `apply_bind` handlers so the UI could share the same
OperationLog path as every other editor command — including
reversible undo/redo semantics.

Two realities make that path unattractive right now:

1. **The handler module does not exist.** `processor.rs:215-229` and
   `processor.rs:423-435` return
   `CommandError::Unsupported("CreateSceneComponent must be applied via
   command_scene_component::apply_create", …)`. There is no
   `command_scene_component` module under `crates/editor-core/src/`
   (confirmed by `ls crates/editor-core/src/ | grep command_scene`).
   Implementing the three handlers means re-deriving the schema
   registry, the SceneAssetCatalog state, and the OperationLog state
   shape from scratch.

2. **Direct WASM exports already work.** `create_scene_component`,
   `bind_scene_to_schema`, and `list_scene_component_schemas` are
   exported by `crates/editor-core/src/lib.rs` and wrapped by
   `frontend/src/services/scene-components.ts`. The AI proxy uses
   `dispatch_command`, not the direct exports, but the frontend authoring
   UX can stay on the direct path without losing any functionality
   observed in the UI (catalog read, validation feedback, place instance).

3. **Undo parity is already partial.** Placement uses
   `Command::PlaceInstance`, which is fully reversible. The
   `create`/`update`/`bind` variants in the original AI schema do not need
   to be reversible from the authoring UI — the existing schema save
   path is `register_schema` + `save_schema`, both non-reversible writes,
   and the SceneComponent UX inherits that contract.

## Decision

**Keep `processor.rs:215-229, 423-435` returning
`CommandError::Unsupported` for the three SceneComponent variants. The
`scene-component-authoring-ux` UX is implemented entirely on top of the
existing direct WASM exports.**

Concretely:

- `getSceneAssetCatalog` in `frontend/src/services/scene-components.ts`
  is a thin re-export of `getSceneAssetCatalogJson` from
  `frontend/src/services/scene-assets.ts`. No new Rust export is added.
- `validateSceneComponentDraft` is a pure-TS function over the catalog
  and `get_validation_issues_wasm` results — no new WASM bridge.
- `placeSceneComponentInstance` (added in PR2) wraps
  `place_scene_instance` from `frontend/src/services/scene-assets.ts`,
  reusing `Command::PlaceInstance` for undo parity.
- The `command_scene_component::apply_*` module is **not** introduced.
  ADR-0016's "follow-up PR" is explicitly closed by this ADR.

## Considered Options

### Option A — Implement `command_scene_component::apply_*` and route the UI through it

- **Pros**: Unifies the AI dispatch path (`dispatch_command`) and the UI
  authoring path under the same OperationLog; guarantees the same
  applied-state representation regardless of which surface invoked the
  change; future auditability.
- **Cons**: Re-derives three handlers (~300–500 LOC) without a current
  user-visible win. Undo parity for `create`/`update`/`bind` is not
  needed by the authoring UX (which is a save, not a transient
  operation). Each handler would have to mirror schema-registry,
  catalog, and asset-binding invariants that already live in
  `schema.rs`, `scene_assets.rs`, and `wasm_scene_instance.rs`. Adding
  the module also means PR1 grows by enough work to threaten the 400-line
  budget. **Rejected.**

### Option B — Keep `CommandError::Unsupported` and build the UX on direct WASM exports (chosen)

- **Pros**: PR1 stays focused on picker + validation; no new Rust
  surface; AI dispatch path is unaffected (still returns
  `Unsupported` for these three variants, which is correct because the
  AI does not currently dispatch them either — see
  `FORBIDDEN_AI_COMMANDS` and the AI schema in
  `crates/ai-proxy/data/propose_commands_schema.json`).
- **Cons**: Two ways to express the same SceneComponent authoring
  operation exist in the codebase — the AI schema lists `create`/
  `update`/`bind` as `Unsupported`, while the UI uses
  `create_scene_component` etc. directly. The split is acceptable today
  because the AI does not currently exercise these variants, but a
  future change that adds AI-driven SceneComponent authoring may need
  to revisit this ADR.

### Option C — Drop the three variants from the `Command` enum entirely

- **Pros**: Removes the misleading "must be applied via
  command_scene_component::apply_*" error message. Reflects the actual
  state (these variants exist for AI-schema completeness but are never
  dispatched).
- **Cons**: Forces an AI-schema change in the same cycle, expanding
  scope into `crates/ai-proxy/`. The `Command` enum is also used by the
  existing dispatch pipeline for serialization shapes. **Rejected for
  this PR; tracked as a follow-up question.**

## Consequences

### Positive

- PR1 of `scene-component-authoring-ux` ships with no new Rust code
  and stays within the chained-PR budget.
- AI dispatch path remains honest: `Unsupported` is the truthful answer
  for these variants because no handler exists.
- Future authors who look for `command_scene_component::apply_*` will
  find this ADR and the explicit decision to defer, rather than being
  confused by stale "must be applied via" error messages.

### Negative / Risks

- The UI and AI paths for SceneComponent authoring diverge: UI uses
  direct exports, AI uses `dispatch_command` with `Unsupported`. If the
  AI starts dispatching these variants before the handlers land, the
  error message will surface to AI users. Mitigation: the AI schema in
  `crates/ai-proxy/data/propose_commands_schema.json` does not currently
  emit these variants, so the risk is latent rather than active.
- If undo parity becomes required for SceneComponent authoring (e.g. an
  "Undo last SceneComponent edit" button), this decision will need to
  be reversed and the three handlers added.

## When to revisit

- When the AI proxy begins dispatching `CreateSceneComponent`,
  `UpdateSceneComponentFields`, or `BindSceneToSchema` (the AI schema
  currently does not).
- When a future UX surface requires reversible SceneComponent authoring
  operations.
- When `processor.rs` is split into per-domain modules (the "follow-up
  PR" mentioned in ADR-0016) and the handler module becomes cheap to add
  alongside the split.

## References

- ADR-0016 (`docs/adr/0016-scene-component-authoring.md`) — origin of
  the `CommandError::Unsupported` decision.
- `crates/editor-core/src/processor.rs:215-229, 423-435` — current
  handler stubs.
- `crates/editor-core/src/lib.rs` — direct WASM exports
  `create_scene_component`, `bind_scene_to_schema`,
  `list_scene_component_schemas`.
- `frontend/src/services/scene-components.ts` — UX-side wrappers.
- `frontend/src/services/scene-assets.ts` — `getSceneAssetCatalogJson`,
  `placeSceneInstance`.
- `docs/sddk/scene-component-authoring-ux/spec.md`, `design.md`,
  `tasks.md` — PR1 scoped to S1–S7 (catalog picker + validation +
  empty-state) with Place Instance and E2E follow-ups on PR2/PR3.
