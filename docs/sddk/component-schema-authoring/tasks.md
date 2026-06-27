# Tasks: Component Schema Authoring

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~650-800 (2 new components + 2 modified + CSS + E2E) |
| 400-line budget risk | High |
| Chained PRs recommended | No (single-PR constraint from task brief) |
| Suggested split | Single PR — exceeds budget but user-mandated |
| Delivery strategy | single-pr |
| Chain strategy | size-exception |

Decision needed before apply: Yes
Chained PRs recommended: No
Chain strategy: size-exception
400-line budget risk: High

### Size-Exception Rationale

Frontend-only change with no Rust/wasm surface delta; tests + new components + small panel edits + CSS cluster together as one cohesive feature. Splitting would create artificial coupling between a half-wired panel and the rest. Approve as single PR or split per `Suggested Work Units`.

### Suggested Work Units (alternative split)

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | SchemaFieldRow + types/CSS scaffolding | PR 1 (optional split) | ~200 LOC |
| 2 | SchemaAuthoringPanel (create/edit/delete) | PR 2 | ~350 LOC |
| 3 | InspectorPanel + AddComponentButton wiring + E2E | PR 3 | ~150 LOC |

## Phase 1: Foundation

- [ ] 1.1 Create `frontend/src/components/SchemaFieldRow.tsx` — per-field row with name input, FieldType picker (String/F32/Bool/Vec2/Color/Anchor/AssetReference), default-value editor matching the picked type, and a constraint editor (Min/Max for F32; NonEmpty for String; disabled otherwise); accepts onChange/onRemove/onMoveUp/onMoveDown props.
- [ ] 1.2 Append CSS rules to `frontend/src/styles.css` for `.schema-authoring-panel` (modal overlay), `.schema-field-row`, `.schema-constraints`, `.schema-error`, and the "New Schema" button in `.inspector-actions`.

## Phase 2: Core Authoring Panel

- [ ] 2.1 Create `frontend/src/components/SchemaAuthoringPanel.tsx` — props `{mode: "create"|"edit", initial?: ComponentSchema, onClose, onSaved}`; maintains draft state `{type_id, display_name, exports_to_bevy, fields: DraftField[]}`; renders metadata inputs + a field list rendered by `SchemaFieldRow`.
- [ ] 2.2 Wire validation in `SchemaAuthoringPanel.tsx` — reject `type_id` missing `game.` prefix; reject `editor.*` (call `(window as any).is_builtin_type`); reject empty `display_name`; reject empty/duplicate field names; surface inline errors and disable Save.
- [ ] 2.3 Implement Save handler in `SchemaAuthoringPanel.tsx` — build `ComponentSchema` JSON (version "0.1") → `(window as any).register_schema(json)` → `(window as any).save_schema(type_id)`; on `save_schema` failure keep in-memory registration and surface error per spec §"Persistence failure".
- [ ] 2.4 Implement Delete handler in `SchemaAuthoringPanel.tsx` — show confirm dialog → `(window as any).unregister_schema(type_id)` + `(window as any).delete_schema(type_id)`; reject/hide affordance when `is_builtin_type(type_id)` is true.
- [ ] 2.5 Implement Cancel handler in `SchemaAuthoringPanel.tsx` — close without invoking register/save/delete (per spec §"Cancel Discards Draft").

## Phase 3: Integration

- [ ] 3.1 Add "New Schema" entry button to `frontend/src/components/InspectorPanel.tsx` — opens `SchemaAuthoringPanel` in create mode (visible when an entity is selected); pass an `onSaved` callback that no-ops for now.
- [ ] 3.2 Add edit affordance to `frontend/src/components/AddComponentButton.tsx` — render a small edit icon next to each dropdown item where `is_builtin_type(s) === false`; clicking opens `SchemaAuthoringPanel` in edit mode (loads via `(window as any).load_schema(s)`).

## Phase 4: Testing

- [ ] 4.1 Create `frontend/tests/schema-authoring.spec.ts` — E2E: (a) open panel via Inspector, create `game.PlayerHealth` with 3 fields of different types, save, assert it appears in AddComponent dropdown without reload; (b) reject `mySchema` (no prefix) and `editor.Transform2D` (builtin) with inline errors; (c) delete user schema via AddComponent edit affordance, assert removed from dropdown; (d) reload page → schema persists via `load_project`.

## Verification (post-apply)

- [ ] V.1 Run `pnpm tsc --noEmit` and `pnpm test` — no regressions.
- [ ] V.2 Run `pnpm playwright test schema-authoring.spec.ts` — all 4 scenarios pass.
- [ ] V.3 Manual smoke: create schema → add to entity → reload → schema still present in dropdown.