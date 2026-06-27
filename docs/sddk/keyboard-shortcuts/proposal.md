# Proposal: Keyboard Shortcuts for Undo/Redo

## Intent
Editors without keyboard undo/redo feel broken. The `operation-log` cycle shipped `undo()`/`redo()` and `useLogState()` (can_undo/can_redo), and `App.tsx` already wires button handlers — but only mouse clicks trigger them. This change adds standard editor shortcuts (Ctrl/Cmd+Z, Ctrl+Y, Ctrl/Cmd+Shift+Z) so undo/redo matches user expectations. It closes the gap explicitly deferred in `docs/sddk/operation-log/spec.md §4`.

## Scope

### In Scope
- `useKeyboardShortcuts` React hook listening to `window` `keydown`
- Detect: Ctrl/Cmd+Z (undo); Ctrl+Y **and** Ctrl/Cmd+Shift+Z (redo)
- Gate on `can_undo`/`can_redo` from existing `useLogState()`
- Ignore when focus is in `<input>`, `<textarea>`, or `contenteditable`
- `preventDefault` to suppress browser native undo
- One Playwright E2E (action → Ctrl+Z → screenshot diff)

### Out of Scope
- Any Rust/WASM change (verified: bindings already exist, engine-bridge.ts L74-76, L173-190)
- Customizable keybindings / settings UI
- Shortcuts beyond undo/redo (save, copy, etc.)
- Menu/tooltip hints displaying the shortcut

## Capabilities

> Contract with sddk-spec. Existing capabilities researched in `docs/sddk/operation-log/spec.md`.

### New Capabilities
- `keyboard-shortcuts`: browser keydown → undo/redo trigger with input-guard and can_undo/can_redo gating

### Modified Capabilities
- None. The `undo-redo` capability's requirements (cursor movement, inverse application, truncation semantics) are unchanged — this change only adds a new *trigger* for already-specified behavior.

## Approach
Add `frontend/src/hooks/useKeyboardShortcuts.ts`. It accepts `onUndo`/`onRedo` callbacks plus a `LogState`, registers one `keydown` listener on `window` in `useEffect`, cleans up on unmount. Register once in `App.tsx` passing the **existing** `handleUndo`/`handleRedo` and `logState` — no logic duplication, no new WASM call per keypress.

Matching: `(metaKey || ctrlKey) && key==='z' && !shiftKey` → undo; with `shiftKey` → redo; `(metaKey || ctrlKey) && key==='y'` → redo. Input guard runs first: `e.target.closest('input,textarea,[contenteditable="true"]')` → return. Then `preventDefault`, then check `can_undo`/`can_redo` before invoking the callback.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `frontend/src/hooks/useKeyboardShortcuts.ts` | New | The hook |
| `frontend/src/App.tsx` | Modified | Register hook (import + 1 call, L21 area) |
| `frontend/tests/keyboard-shortcuts.spec.ts` | New | Playwright E2E |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Shortcut fires while typing in inspector inputs | Med | `closest()` input-guard before any action |
| Mac Cmd vs Ctrl mismatch | Low | Match both `metaKey` and `ctrlKey` |
| `useLogState` 500ms poll lags rapid keypresses | Low | Gating is best-effort; WASM `undo()` is safe to call anyway (returns `Err(NothingToUndo)`, caught by `handleUndo`) |
| Refocus/selection state after undo | Low | Reuses `handleUndo` which already clears `selectedEntityId` |

## Rollback Plan
Delete `useKeyboardShortcuts.ts`, remove the import + 1 registration line in `App.tsx`, delete the test file. No data migration, no Rust rebuild. The undo/redo **buttons** remain functional throughout — zero user-visible regression.

## Dependencies
- Existing: `undo()`, `redo()`, `get_log_state()` WASM bindings (engine-bridge.ts L74-76, L173-190)
- Existing: `useLogState()` hook polling every 500ms (useLogState.ts)
- Existing: `handleUndo`/`handleRedo` in App.tsx (L34-53)

## Success Criteria
- [ ] Ctrl+Z and Cmd+Z undo (Playwright-verified)
- [ ] Ctrl+Y **and** Ctrl/Cmd+Shift+Z redo
- [ ] Shortcut ignored when focus is in an input/textarea/contenteditable
- [ ] No-op (no error toast) when `can_undo`/`can_redo` is false
- [ ] Browser native undo suppressed via `preventDefault`
- [ ] Existing undo/redo buttons still work — no regression
- [ ] Playwright screenshot diff confirms visual state change after Ctrl+Z
