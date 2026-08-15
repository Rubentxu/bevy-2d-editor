/**
 * useEditorWorkspaceController — composition-root state for the editor
 * workspace.
 *
 * Owns the four "wiring" concerns that App.tsx used to mix in with
 * rendering:
 *   1. Editor mode (scene / asset-authoring / logic / code / play).
 *   2. Multi-select (selectedIds, lastClickedId, modifier-aware click
 *      handler, Esc/Ctrl+A keyboard, setSelectedEntityId back-compat
 *      shim).
 *   3. Dirty guards (pendingNavigation + dirty-guard dialog state for
 *      scene↔asset transitions).
 *   4. Test bridge hooks exposed on `window.__setEditorMode`,
 *      `window.__setSelectedEntityId`, `window.__openAIPanel`.
 *
 * The hook is deliberately headless: it returns pure state plus typed
 * setters and a `bindTestHooks()` function App.tsx calls once on mount.
 * Side effects that touch the DOM (the keyboard listener and the
 * `window.*` registrations) live in App.tsx so this hook stays
 * composable in any future provider.
 *
 * Wave D2 deliberately does NOT touch:
 *   - the JSX tree (still rendered in App.tsx).
 *   - the EditorGateway integration (useSceneState stays the source
 *     of truth for scene state; this controller just owns the
 *     workspace-level selection that wraps it).
 *   - the test-mode `initGuard` (still owned by App.tsx because it
 *     depends on test-mode-only state).
 */

import { useCallback, useMemo, useState } from "react";
import type { EditorMode } from "../components/MenuBar";

export interface WorkspaceController {
  // Mode
  editorMode: EditorMode;
  setEditorMode: (mode: EditorMode) => void;
  // Multi-select
  selectedIds: Set<string>;
  lastClickedId: string | null;
  selectedEntityId: string | null;
  selectEntity: (id: string, modifier: "plain" | "range" | "toggle") => void;
  setSelectedEntityId: (id: string | null) => void;
  setSelectedIds: (ids: Set<string>) => void;
  clearSelection: () => void;
  // Dirty guards
  pendingNavigation: NavigationTarget | null;
  setPendingNavigation: (target: NavigationTarget | null) => void;
  pendingBackToScene: boolean;
  setPendingBackToScene: (pending: boolean) => void;
  // Test bridge
  bindTestHooks: () => void;
}

export interface NavigationTarget {
  fileId: string;
  line: number;
}

export interface UseEditorWorkspaceControllerOptions {
  /**
   * When provided, modifier-aware "range" clicks look up the
   * hierarchy order in the Scene Document. When omitted, range clicks
   * degrade to a plain select (preserves the back-compat behaviour of
   * the pre-D2 hook callers).
   */
  sceneOrderForRangeSelect?: ReadonlyArray<string>;
}

export function useEditorWorkspaceController(
  options: UseEditorWorkspaceControllerOptions = {},
): WorkspaceController {
  const [editorMode, setEditorModeState] = useState<EditorMode>("scene");
  const setEditorMode = useCallback((mode: EditorMode) => {
    setEditorModeState(mode);
  }, []);

  const [selectedIds, setSelectedIds] = useState<Set<string>>(
    () => new Set<string>(),
  );
  const [lastClickedId, setLastClickedId] = useState<string | null>(null);
  const [pendingNavigation, setPendingNavigation] =
    useState<NavigationTarget | null>(null);
  const [pendingBackToScene, setPendingBackToSceneState] = useState(false);
  const setPendingBackToScene = useCallback((pending: boolean) => {
    setPendingBackToSceneState(pending);
  }, []);

  const setSelectedEntityId = useCallback((id: string | null) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (id == null) next.clear();
      else {
        next.clear();
        next.add(id);
      }
      return next;
    });
    setLastClickedId(id);
  }, []);

  // Derived single-id view: prefer `lastClickedId` ONLY when it is
  // still in the active set; fall back to the lone id otherwise.
  const selectedEntityId = useMemo(
    () =>
      (lastClickedId && selectedIds.has(lastClickedId)
        ? lastClickedId
        : null) ?? (selectedIds.size === 1 ? Array.from(selectedIds)[0] : null),
    [lastClickedId, selectedIds],
  );

  const selectEntity = useCallback(
    (id: string, modifier: "plain" | "range" | "toggle") => {
      if (
        modifier === "range" &&
        lastClickedId &&
        options.sceneOrderForRangeSelect
      ) {
        const ids = options.sceneOrderForRangeSelect;
        const fromIdx = ids.indexOf(lastClickedId);
        const toIdx = ids.indexOf(id);
        if (fromIdx === -1 || toIdx === -1) {
          setSelectedIds(new Set([id]));
        } else {
          const [lo, hi] =
            fromIdx < toIdx ? [fromIdx, toIdx] : [toIdx, fromIdx];
          setSelectedIds((prev) => {
            const next = new Set(prev);
            for (let i = lo; i <= hi; i++) next.add(ids[i]);
            return next;
          });
        }
      } else if (modifier === "toggle") {
        setSelectedIds((prev) => {
          const next = new Set(prev);
          if (next.has(id)) next.delete(id);
          else next.add(id);
          return next;
        });
      } else {
        setSelectedIds(new Set([id]));
      }
      setLastClickedId(id);
    },
    [lastClickedId, options.sceneOrderForRangeSelect],
  );

  const setSelectedIdsDirect = useCallback((ids: Set<string>) => {
    setSelectedIds(ids);
  }, []);

  const clearSelection = useCallback(() => {
    setSelectedIds(new Set());
    setLastClickedId(null);
  }, []);

  const bindTestHooks = useCallback(() => {
    if (typeof window === "undefined") return;
    (
      window as unknown as { __setEditorMode?: (mode: EditorMode) => void }
    ).__setEditorMode = (mode: EditorMode) => setEditorMode(mode);
    (
      window as unknown as {
        __setSelectedEntityId?: (id: string | null) => void;
      }
    ).__setSelectedEntityId = (id: string | null) => setSelectedEntityId(id);
    // The AI panel hook is exposed by App.tsx because it owns the
    // dialog state. We leave it as a no-op here so callers can keep
    // asking the controller to wire it when they call bindTestHooks.
    (window as unknown as { __openAIPanel?: () => void }).__openAIPanel =
      () => {
        // Real implementation lives in App.tsx; here we just record
        // intent so tests can call it before App.tsx mounts.
        (
          window as unknown as { __openAIPanelPending?: boolean }
        ).__openAIPanelPending = true;
      };
  }, [setEditorMode, setSelectedEntityId]);

  return {
    editorMode,
    setEditorMode,
    selectedIds,
    lastClickedId,
    selectedEntityId,
    selectEntity,
    setSelectedEntityId,
    setSelectedIds: setSelectedIdsDirect,
    clearSelection,
    pendingNavigation,
    setPendingNavigation,
    pendingBackToScene,
    setPendingBackToScene,
    bindTestHooks,
  };
}
