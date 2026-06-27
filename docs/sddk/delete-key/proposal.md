# Proposal: Delete Key Shortcut for Entity Removal

## Intent
An editor where you select an entity and can't press Delete to remove it feels broken. The `keyboard-shortcuts` cycle wired Undo/Redo, and the `command-system` cycle already ships `DeleteEntity` (removes entity, reparents its children to root). `App.tsx` tracks the selected entity and exposes `dispatch()`. This change connects the last gap: **pressing Delete/Backspace removes the currently selected Entity via the existing `DeleteEntity` command**. No Rust changes.

## Scope

### In Scope
- Extend `useKeyboardShortcuts` to handle Delete/Backspace (non-modifier gesture)
- Gate on `selectedEntityId !== null`
- Dispatch `DeleteEntity { id }` through the existing `dispatch()` + `window.dispatch_command`
- Clear selection after deletion (`setSelectedEntityId(null)`)
- Input-guard: no-op when focus is in `<input>`, `<textarea>`, `[contenteditable="true"]`
- One Playwright E2E (select entity → Delete → entity gone from hierarchy)

### Out of Scope
- Any Rust/WASM change (`DeleteEntity` already in `command.rs` L28-30)
- Customizable keybindings or confirmation dialog
- Multi-select deletion (single selection only, Hito 0 scope)
- Delete via context menu or toolbar button

## Capabilities

> Contract with sddk-spec. Existing capabilities researched in `docs/sddk/keyboard-shortcuts/spec.md` and `docs/sddk/command-system/spec.md`.

### New Capabilities
- None

### Modified Capabilities
- `keyboard-shortcuts`: extend the existing window keydown listener to handle a **non-modifier** gesture (Delete/Backspace → `DeleteEntity`). The existing listener structure early-returns on `!modKey`; it must be restructured to also route bare-key presses. The input-focus guard and `preventDefault` patterns are reused unchanged.

> Note: `command-system`'s `DeleteEntity` requirement (remove entity, reparent children, `EntityNotFound` on missing id) is **unchanged** — this change only adds a UI trigger.

## Approach
Restructure `useKeyboardShortcuts` handler: keep the input-guard first (unchanged), then branch on `modKey`. If modifier: existing undo/redo logic. If **no modifier** and key is `Delete` or `Backspace`: `preventDefault`, check `selectedEntityId`, and call a new `onDelete(id)` callback.

The callback lives in `App.tsx` as `handleDeleteEntity`: it dispatches `{ type: "DeleteEntity", id }` (note: field is `id`, not `entity_id` — matches `command.rs` L28-30), refreshes scene, clears selection. Pass `selectedEntityId` + `onDelete` into the hook options.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/hooks/useKeyboardShortcuts.ts` | Modified | Add Delete/Backspace branch + `onDelete`/`selectedEntityId` options |
| `frontend/src/App.tsx` | Modified | Add `handleDeleteEntity`; pass new options to hook (L127) |
| `frontend/tests/delete-shortcut.spec.ts` | New | Playwright E2E |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Backspace triggers browser back navigation | Med | `preventDefault` before dispatch |
| Fires while typing in inspector input | Med | Reuse existing `closest()` input-guard |
| Stale `selectedEntityId` after external mutation | Low | Polling `useSceneState` + clear-on-undo/redo already handle this |
| Deleting parent entity unexpectedly orphans children | Low | `DeleteEntity` reparents children to root (specified behavior, not a bug) |

## Rollback Plan
Revert `useKeyboardShortcuts.ts` to undo/redo-only (remove Delete branch + options), remove `handleDeleteEntity` and the two options passed in `App.tsx` L127, delete the test file. No Rust rebuild, no data migration. Existing undo/redo shortcuts and the `DeleteEntity` command (dispatchable via other means) remain functional.

## Dependencies
- Existing: `DeleteEntity` variant in `Command` enum (`command.rs` L28-30)
- Existing: `dispatch_command` WASM binding + `useSceneState.dispatch()`
- Existing: `selectedEntityId` state in `App.tsx` L14
- Existing: input-guard pattern in `useKeyboardShortcuts.ts` L24-25

## Success Criteria
- [ ] Delete and Backspace remove the selected entity (Playwright-verified)
- [ ] No-op (no error) when no entity is selected
- [ ] No-op when focus is in input/textarea/contenteditable
- [ ] After deletion, Hierarchy no longer shows the entity; selection cleared
- [ ] Operation Log records the command (`can_undo` true after delete; Ctrl+Z restores)
- [ ] Browser back navigation not triggered on Backspace
- [ ] Existing undo/redo shortcuts still work — no regression
