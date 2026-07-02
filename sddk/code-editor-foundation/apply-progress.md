# Apply Progress: Code Editor Foundation (Hito 4 Order 1)

## PR Chain

| PR | Description | Branch | PR | Status |
|----|-------------|--------|----|--------|
| PR 1 | Foundation: Rust source_files + WASM + ADR-0012 | `feat/code-editor-foundation` | [#45](https://github.com/Rubentxu/bevy-2d-editor/pull/45) | ✅ MERGED v0.43.0 (1 debt-fix round) |
| PR 2 | Service + Hook layer | `feat/code-editor-foundation-pr2-service-hook` | [#46](https://github.com/Rubentxu/bevy-2d-editor/pull/46) | ✅ MERGED v0.43.0 (1 debt-fix round) |
| PR 3 | UI wiring (CodeEditor.tsx + App + TopBar) | `feat/code-editor-foundation-pr3-ui` | [#47](https://github.com/Rubentxu/bevy-2d-editor/pull/47) | ✅ MERGED (no fix cycle needed) |
| PR 4 | Tests + verification + debt cleanup | `feat/code-editor-foundation-pr4-tests` | (pending) | 🔲 NEXT → v0.44.0 |

## Completed Tasks

### Phase 2: Service + Hook (PR 2)

- [x] **2.1** `frontend/src/services/code-files.ts` — Created
  - `listSourceFiles()`, `readSourceFile()`, `writeSourceFile()`, `createSourceFile()`, `deleteSourceFile()` via `waitForEngine`
  - Mirrors `services/scenes.ts` pattern; reuses `OpfsResult<T>` contract
  - Exports `SourceFile { id, path, name }` interface
- [x] **2.2** `frontend/src/hooks/useCodeFiles.ts` — Created
  - State: `{files, currentId, content, dirty, error}`
  - 500ms polling via `setInterval` (same cadence as `useScenes`)
  - Actions: `open`, `save`, `create`, `setContent`, `delete`, `refresh`
  - Mirrors `hooks/useScenes.ts` grain; intentionally simpler than `useSceneAssets` (no undo/redo per design §Scope discipline)
- [x] **2.3** `frontend/src/engine-bridge.ts` — Modified (L202-209)
  - Added 5 `window.*` bindings: `list_source_files`, `read_source_file`, `write_source_file`, `create_source_file`, `delete_source_file`
  - Bridge calls to WASM exports, mirroring scene-asset block L202-224

### Phase 3: UI Wiring (PR 3)

- [x] **3.1** `frontend/package.json` — Modified
  - Added `@uiw/react-codemirror`, `@codemirror/lang-rust`, `@uiw/codemirror-theme-vscode`
  - `npm install` succeeded, 0 vulnerabilities
- [x] **3.2** `frontend/src/App.tsx` — Modified
  - Added `"code"` to `EditorMode` union (L28)
  - Added `editorMode === "code"` view branch rendering `<CodeEditor />`
  - Added `handleOpenCode` handler and `onOpenCode` prop wiring to TopBar
  - Added `import CodeEditor from "./components/CodeEditor"`
- [x] **3.3** `frontend/src/components/TopBar.tsx` — Modified
  - Added `"code"` to local `EditorMode` union (L3)
  - Added `onOpenCode` prop to Props interface
  - Added "Code" toolbar button (📝 Code) mirroring "Logic" entry
- [x] **3.4** `frontend/src/components/CodeEditor.tsx` — Created
  - File list panel (left sidebar, 200px): click to open, ✕ to delete, + New File button
  - `<CodeMirror>` with `rust()` extension + `vscodeDark` theme
  - Empty state: "No source files yet" + Create button when `files.length === 0`
  - "Select a file from the list" when `files.length > 0 && !currentId`
  - Error toast bar (red, dismiss on click) for load/save failures
  - Status bar showing file name + dirty indicator (amber "unsaved" / green clean)
  - Ctrl+S / Cmd+S keyboard shortcut → `save()`
  - Imperative `EditorView` sync via `useRef`: programmatic content update on file open skips `onChange` to avoid spurious dirty flag via `lastSyncedContentRef`

## Implementation Details

### `createSourceFile` API note
The WASM binding takes `(path, name)` two arguments, not just `name`.
The TS service derives `path` from `name` (stripping `.rs` extension) before calling `window.create_source_file(path, name)`.

### CM6 save trigger decision (design.md open question)
Ctrl+S / Cmd+S → `save()`. Explicit button-only was considered but rejected — muscle memory for Ctrl+S is universal.

### Dirty state guard
CM6's `onChange` fires on every keystroke. The hook's `lastSavedContent` guard (in `useCodeFiles`) already handles no-op detection. The `lastSyncedContentRef` in `CodeEditor` additionally prevents programmatic content sync from CM6 `onChange` from marking dirty.

### TypeScript verification
- `npx tsc --noEmit` → **0 errors**
- `cargo test -p editor-core --lib` → **383 passed, 0 failed**
- No ESLint config in project (no lint to run)

## Carried Debt (NOT fixing in PR 3)

| ID | Severity | Description | Deferred To |
|----|----------|-------------|-------------|
| M-1 | HIGH | `CodeEditor.tsx`: `basicSetup` 23-key enum → use bare `basicSetup: true` | PR 4 |
| M-2 | HIGH | `CodeEditor.tsx`: imperative `view.dispatch` + `lastSyncedContentRef` + ref cast is redundant (uses @uiw v4.23.0 `ExternalChange.of(true)`); **real UX bug** (cursor-loss on file open) | PR 4 |
| overeng-W1 | HIGH | `SourceFile.id == SourceFile.path` violates CONTEXT.md "Stable ID" term | Rust-side redesign (drop `id`, use `path` as natural key) |
| coupling-W1 | HIGH | `rename_scene_asset` / `delete_scene_asset` silently discard `js_delete_file` Result via `let _ = ...` pattern | Rust-side fix (2-line per caller) |
| overeng-PR2-2 | MEDIUM | `useCodeFiles.ts`: `runOp<T>(fn)` helper extract (~40 LOC reducible) | PR 4 |
| overeng-PR2-5/coupling-PR2-10 | MEDIUM | `code-files.ts`: add `@throws {Error}` JSDoc on `listSourceFiles`, `createSourceFile` | PR 4 |
| overeng-S1 | SUGGESTION | `SceneAssetDocument` uses inline types instead of shared `types.ts` | Future |
| overeng-S2 | SUGGESTION | `EffectiveValues` / `ResolvedScene` unification | Future |
| coupling-S1 | SUGGESTION | `fetchAssetForInstance` calls `openSceneAsset` as side effect | Future |
| Pre-existing | — | `wasm-pack` bug in `logic_evaluator.rs:1071,1101,1074` (blocks `npm run build` full chain; not PR 3 regression) | Hot-fix on main |

## Final State (after PR 3 release)

| Metric | Value |
|--------|-------|
| `cargo test -p editor-core --lib -- --test-threads=1` | **383/383 passing** on main |
| `npx tsc --noEmit` (full frontend) | **0 errors** on main |
| `npx vite build` (production bundle) | **981.88 kB / 315.20 kB gzip**, builds cleanly |
| `cargo build -p editor-core` | **0 errors**, 43 pre-existing warnings (no new) |
| PR 1 | merged `0c1a2b3` (squash), v0.43.0 tag `60f80aff7d` |
| PR 2 | merged `9e8ade8` (squash) |
| PR 3 | merged `c12de50` (squash) |
| Local feature branches | deleted (cleaned) |
| Remote feature branches | deleted via `--delete-branch` |

## SDDK Cycle Cost (cumulative for 3 PRs)

| PR | Apply | Verify | Debt | Fix | Archive | Release | Total | Debt-Fix Rounds |
|----|-------|--------|------|-----|---------|---------|-------|-----------------|
| PR 1 | ✓ | PASS_WITH_WARNINGS | FAIL (3 HIGH) | 1 round (PASS_WITH_WARNINGS) | ✓ | ✓ (v0.43.0) | 11 phases | 1 |
| PR 2 | ✓ | PASS_WITH_WARNINGS | FAIL (8 HIGH) | 1 round (PASS_WITH_WARNINGS) | ✓ | ✓ (no tag) | 6 phases | 1 |
| PR 3 | ✓ | PASS_WITH_WARNINGS | PASS_WITH_WARNINGS | 0 rounds | ✓ | ✓ (no tag) | 5 phases | 0 |
| **Total** | 3 | 3 PW | 2 FAIL + 1 PW | 2 rounds | 3 | 3 (1 tagged) | **22 phases** | **2 rounds** |

## Next Steps

- **PR 4**: Tests + verification + debt cleanup
  - `frontend/tests/code-editor.spec.ts` — Playwright E2E for all 11 spec scenarios
  - Rust unit tests in `crates/editor-core/src/source_files.rs`
  - Bundle size measurement (CM6-only delta vs <200KB gzipped budget per design)
  - 6 HIGH debt cleanups (M-1, M-2, overeng-W1, coupling-W1, overeng-PR2-2, overeng-PR2-5/coupling-PR2-10)
  - **Release v0.44.0** when landed
- After v0.44.0: Hito 4 Order 2 (`rust-source-integration` — scene↔source navigation)

---

*Apply executor: sddk-apply | Branch: feat/code-editor-foundation-pr3-ui | Commit: 1b5266a | Released: c12de50 (squash) | v0.43.0 covers PR 1+2+3; v0.44.0 lands with PR 4 | 2026-07-02*
