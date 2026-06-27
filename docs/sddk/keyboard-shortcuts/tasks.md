# Tasks: Keyboard Shortcuts (Ctrl+Z / Ctrl+Y)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~150–200 (1 new hook ~60 lines + App.tsx +3 lines + 1 new spec ~100 lines) |
| 400-line budget risk | Low |
| Chained PRs recommended | No |
| Suggested split | single PR |
| Delivery strategy | single-pr |
| Chain strategy | stacked-to-main |

Decision needed before apply: No
Chained PRs recommended: No
Chain strategy: stacked-to-main
400-line budget risk: Low

## Phase 1: Hook implementation

- [ ] 1.1 Create `frontend/src/hooks/useKeyboardShortcuts.ts` with `useKeyboardShortcuts({ onUndo, onRedo, logState })` signature
- [ ] 1.2 In `useKeyboardShortcuts.ts`: input-guard via `e.target instanceof HTMLElement && e.target.closest('input,textarea,[contenteditable="true"]')` returns early
- [ ] 1.3 In `useKeyboardShortcuts.ts`: `useEffect` registers `window.addEventListener('keydown', handler)`; return removes it on unmount
- [ ] 1.4 In `useKeyboardShortcuts.ts`: handler matches `(metaKey||ctrlKey) && key==='z' && !shiftKey` → undo; same with `shiftKey` → redo; `(metaKey||ctrlKey) && key==='y' && !shiftKey` → redo; calls `e.preventDefault()` first, then checks `logState.can_undo`/`can_redo` before invoking callbacks

## Phase 2: Registration

- [ ] 2.1 In `frontend/src/App.tsx`: add import `import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";` near the existing hook imports (top of file)
- [ ] 2.2 In `frontend/src/App.tsx`: call `useKeyboardShortcuts({ onUndo: handleUndo, onRedo: handleRedo, logState });` after `const logState = useLogState();` (L21)

## Phase 3: Playwright E2E

- [ ] 3.1 Create `frontend/tests/baselines/` directory for the reference screenshot
- [ ] 3.2 Create `frontend/tests/keyboard-shortcuts.spec.ts` with one test: `page.goto('/')`, wait for topbar + `window.dispatch_command`
- [ ] 3.3 In `keyboard-shortcuts.spec.ts`: dispatch an `AddEntity` command via `window.dispatch_command`, wait for hierarchy panel to render the new entity
- [ ] 3.4 In `keyboard-shortcuts.spec.ts`: take baseline screenshot of `hierarchy-panel`, click canvas to blur, fire `Control+KeyZ` via `page.keyboard.press`, assert entity count via `get_scene_snapshot` decreased by 1, take second screenshot
- [ ] 3.5 In `keyboard-shortcuts.spec.ts`: assert `expect(after).not.toEqual(before)` via `expect.poll` with screenshot diff (or pixelmatch) confirming visual change

## Phase 4: Verification

- [ ] 4.1 Run `cd frontend && npm run build:wasm` — must succeed (no Rust source changes, but ensures WASM still builds with no regressions)
- [ ] 4.2 Run `cd frontend && npx playwright test keyboard-shortcuts.spec.ts` — must pass
- [ ] 4.3 Run `cd frontend && npx tsc --noEmit` — must pass (type-check the new hook)