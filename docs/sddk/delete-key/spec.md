# Spec: Delete Key Shortcut

> Change: `delete-key` · Phase: sddk-spec (draft) · Path: A-lite

## §1. Spec Metadata

- **Change:** `delete-key`
- **Phase:** spec (draft, awaiting design)
- **Path:** A-lite
- **Capabilities (NEW):** None
- **Capabilities (MODIFIED):** `keyboard-shortcuts` — adds non-modifier `Delete`/`Backspace` gesture routed to the existing `DeleteEntity` command
- **Source proposal:** [`docs/sddk/delete-key/proposal.md`](./proposal.md)
- **Authoritative references:**
  - [`docs/sddk/keyboard-shortcuts/spec.md`](../keyboard-shortcuts/spec.md) — listener structure, input-guard pattern
  - [`docs/sddk/command-system/spec.md`](../command-system/spec.md) — `DeleteEntity` semantics
  - [`docs/sddk/operation-log/spec.md`](../operation-log/spec.md) — `can_undo` gating
  - [`docs/sddk/ui-panels/spec.md`](../ui-panels/spec.md) — Hierarchy and Inspector panels
  - [CONTEXT.md](../../CONTEXT.md) — Entity, Stable ID, Operation Log

---

## §2. Capability: `keyboard-shortcuts` — Delete Key Event Handling

### Requirement: Delete or Backspace dispatches `DeleteEntity` for the selected Entity

The system MUST dispatch the existing `DeleteEntity` command with the selected Entity's Stable ID when the user presses `Delete` or `Backspace` without any modifier key, while focus is not inside an `<input>`, `<textarea>`, or `[contenteditable="true"]` element. The native browser delete/back behavior MUST be suppressed.

#### Scenario: Delete key removes the selected entity

- GIVEN an Entity `e1` is selected in the Hierarchy panel
- AND focus is on the editor surface (no input/textarea/contenteditable focused)
- WHEN the user presses `Delete`
- THEN `window.dispatch_command` is called with `{ command: { type: "DeleteEntity", id: "<stable_id_of_e1>" }, metadata: { authorship: "keyboard", timestamp: 0 } }`
- AND `e.preventDefault()` is invoked before dispatch

#### Scenario: Backspace key removes the selected entity

- GIVEN an Entity `e1` is selected
- WHEN the user presses `Backspace`
- THEN `dispatch_command` is called with `DeleteEntity { id: "<stable_id_of_e1>" }`
- AND the browser back navigation is NOT triggered

#### Scenario: No-op when no entity is selected

- GIVEN `selectedEntityId` is `null`
- WHEN the user presses `Delete` or `Backspace`
- THEN `dispatch_command` is NOT called
- AND no error toast appears

#### Scenario: No-op when focus is in an input

- GIVEN focus is in an `<input>` inside the Inspector
- AND an Entity `e1` is selected
- WHEN the user presses `Delete` inside that input
- THEN `dispatch_command` is NOT called
- AND the browser's native character deletion runs

#### Scenario: No-op when focus is in a textarea

- GIVEN focus is in a `<textarea>`
- WHEN the user presses `Backspace`
- THEN `dispatch_command` is NOT called

#### Scenario: No-op when focus is in contenteditable

- GIVEN focus is in `[contenteditable="true"]`
- WHEN the user presses `Delete`
- THEN `dispatch_command` is NOT called

### Requirement: Deletion is recorded in the Operation Log

The system MUST treat a successful Delete-key dispatch as a reversible Operation Log entry.

#### Scenario: Undo restores the deleted entity

- GIVEN an Entity `e1` was deleted via the Delete key
- WHEN the user presses `Ctrl+Z`
- THEN the Operation Log applies the inverse of `DeleteEntity`
- AND `e1` is restored in the SceneDocument and Hierarchy panel
- AND `can_undo` was `true` immediately after the delete

---

## §3. Capability: `keyboard-shortcuts` — Visual Feedback

### Requirement: Hierarchy and Inspector panels reflect post-deletion state

After a successful delete via keyboard, the Hierarchy panel MUST no longer render the deleted Entity, and the Inspector panel MUST clear its selection.

#### Scenario: Deleted entity disappears from Hierarchy

- GIVEN a SceneDocument with Entities `e1` and `e2`, `e1` selected
- WHEN the user presses `Delete`
- THEN the Hierarchy panel renders only `e2`
- AND the Inspector panel shows no selection

### Requirement: No-op deletion produces no visual change

When `selectedEntityId` is `null`, pressing Delete/Backspace MUST NOT change the Hierarchy panel, the Inspector panel, or any error indicator.

#### Scenario: Pressing Delete with no selection leaves UI unchanged

- GIVEN no entity is selected
- WHEN the user presses `Delete`
- THEN the Hierarchy panel renders the same content as before
- AND no error toast appears

---

## §4. Capability: `keyboard-shortcuts` — Screenshot Verification (E2E)

### Requirement: Delete key produces a non-zero screenshot diff vs. baseline

A Playwright E2E test MUST load a scene with at least one Entity, select it, capture a baseline screenshot, press `Delete`, capture a post-action screenshot, and assert the pixel diff between the two is non-zero. Baseline MUST live under `frontend/tests/baselines/`.

#### Scenario: Delete changes the visual scene

- GIVEN a scene with at least one Entity is loaded via `load_scene_json`
- AND `e1` is selected in the Hierarchy panel
- AND a baseline screenshot is written to `frontend/tests/baselines/delete-key-before.png`
- WHEN the user presses `Delete` while the editor surface is focused
- THEN a post-action screenshot is captured to `frontend/tests/baselines/delete-key-after.png`
- AND the pixel diff between the two screenshots has a non-zero changed-pixel count

---

## §5. Out-of-Scope Behaviors

- Any Rust/WASM change (`DeleteEntity` already shipped in `command.rs` L28-30)
- Multi-select deletion (single selection only, Hito 0 scope)
- Customizable keybindings or confirmation dialog
- Delete via context menu or toolbar button
- Separate E2E for Backspace, no-selection, and input-guard paths (covered by TypeScript unit tests)

---

## §6. Acceptance Criteria

1. Delete and Backspace dispatch `DeleteEntity { id }` for the selected Entity when focus is on the editor surface.
2. The `dispatch_command` envelope uses `id` (matches `command.rs` L29), not `entity_id`.
3. Pressing Delete/Backspace with no selection is a no-op (no dispatch, no error).
4. Pressing Delete/Backspace while focus is in `<input>`, `<textarea>`, or `[contenteditable]` is a no-op (native behavior preserved).
5. After deletion, the Hierarchy panel no longer shows the entity and the Inspector selection is cleared.
6. The Operation Log records the delete (`can_undo = true` afterward; Ctrl+Z restores the entity).
7. `e.preventDefault()` runs on the matched gesture (suppresses browser back navigation on Backspace).
8. Existing undo/redo shortcuts continue to work — no regression in the `keyboard-shortcuts` capability.
9. A Playwright test selects an entity, captures `delete-key-before.png`, presses Delete, captures `delete-key-after.png`, and asserts a non-zero pixel diff.

---

## §7. Test Plan

| Section | Scenarios | Test type | Count |
|---|---|---|---|
| §2.1 Delete/Backspace gesture | dispatch + envelope shape | TS unit (hook) | 2 |
| §2.1 No selection / input-guard | no-op paths | TS unit | 4 |
| §2.2 Operation Log | undo restores deleted entity | TS unit + Playwright | 1 |
| §3 Visual feedback | hierarchy + inspector update | Playwright | 1 |
| §3 No-op visual | no UI change | Playwright | 1 |
| §4 Screenshot diff | pixelmatch non-zero | Playwright (`pixelmatch`) | 1 |
| **Total** | | | **~10 tests** |

Dev cycle: `just wasm && cd frontend && npx playwright test`.

Baselines under `frontend/tests/baselines/`:
- `delete-key-before.png` — entity selected, scene ready
- `delete-key-after.png` — after one Delete press

---

## §8. Notes on Implementation Surfaces (non-binding)

- Listener reuses the existing `window.keydown` registration. The early-return on `!modKey` must be restructured to also route no-modifier presses to `onDelete(id)`.
- Input-guard predicate (`e.target.closest("input,textarea,[contenteditable=\"true\"]")`) runs first — unchanged.
- `e.preventDefault()` runs only after gesture match and `selectedEntityId !== null`, before invoking `onDelete`.
- `handleDeleteEntity` in `App.tsx` calls `dispatch(...)`, refreshes the scene, then `setSelectedEntityId(null)`.
- Field name is `id` (verified at `command.rs` L29), not `entity_id`.
