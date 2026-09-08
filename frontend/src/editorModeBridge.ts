/**
 * Single source of truth for the `editorMode` test-bridge surface.
 *
 * Background:
 *   - The editor exposes a programmatic mode setter
 *     `window.__setEditorMode` (declared in `useEditorWorkspaceController`'s
 *     `bindTestHooks`).
 *   - P5 of `app-characterization.spec.ts` needs a reader to assert
 *     the composition-root boundary, so we expose
 *     `window.__getEditorMode` too.
 *   - Both bridge handles observe the same value, so they need to read
 *     / write through a single module-scope reference (a leaf module
 *     with no React imports, so we sidestep cyclic-import issues).
 *
 * Why a separate module?
 *   - `useEditorWorkspaceController` owns the React state (and should
 *     keep owning it — that's the composition-root contract).
 *   - `engine-bridge.ts` is what creates the `window.__getEditorMode`
 *     binding at startup.
 *   - Both need to talk to *the same value*, so we factor that value
 *     into this small shared module instead of duplicating it.
 */

export type EditorMode =
  | "scene"
  | "asset-authoring"
  | "code"
  | "logic"
  | "play"
  | "world";

let currentMode: EditorMode = "scene";

export function setEditorMode(mode: EditorMode): void {
  currentMode = mode;
}

export function getEditorMode(): EditorMode {
  return currentMode;
}
