# Apply Progress: Code Editor Foundation (Hito 4 Order 1)

## PR Chain

| PR | Description | Branch | Status |
|----|-------------|--------|--------|
| PR 1 | Foundation: Rust source_files + WASM + ADR-0012 | `feat/code-editor-foundation` | Merged as v0.43.0 |
| **PR 2** | **Service + Hook layer** | **`feat/code-editor-foundation-pr2-service-hook`** | **Done** |
| PR 3 | UI wiring (CodeEditor.tsx + App + TopBar) | `feat/code-editor-foundation-pr3-ui` | Pending |
| PR 4 | Tests + verification | `feat/code-editor-foundation-pr4-tests` | Pending |

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

## Implementation Details

### `createSourceFile` API note
The WASM binding takes `(path, name)` two arguments, not just `name`.
The TS service derives `path` from `name` (stripping `.rs` extension) before calling `window.create_source_file(path, name)`.

### TypeScript verification
- `npx tsc --noEmit` → **0 errors**
- No ESLint config in project (no lint to run)

## Carried Debt (NOT fixing in PR 2)

| ID | Severity | Description | Filed Issue |
|----|----------|-------------|-------------|
| overeng-W1 | HIGH | `SourceFile.id == SourceFile.path` violates CONTEXT.md "Stable ID" term | Not filed — requires Rust API redesign |
| coupling-W1 | HIGH | `rename_scene_asset` (L3171) and `delete_scene_asset` (L3264) silently discard `js_delete_file` Result via `let _ = ...` | Not filed — separate Rust-side fix |
| overeng-S1 | SUGGESTION | `SceneAssetDocument` uses inline types instead of re-exporting from a shared `types.ts` | Deferred |
| overeng-S2 | SUGGESTION | `EffectiveValues` response shape could be unified with `ResolvedScene` | Deferred |
| coupling-S1 | SUGGESTION | `fetchAssetForInstance` calls `openSceneAsset` as a side effect | Deferred |

## Next Steps

- PR 3: UI wiring (`CodeEditor.tsx`, App mode, TopBar button)
- PR 4: Tests + verification
- After all PRs merged: `sddk-release` to push, PR, merge to main, tag v0.44.0

---

*Apply executor: sddk-apply | Branch: feat/code-editor-foundation-pr2-service-hook | Committed: 2026-07-02*
