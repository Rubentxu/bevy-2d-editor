/**
 * useAppShortcuts — wires the keyboard shortcut bus to the editor's
 * mode-aware handlers.
 *
 * The useKeyboardShortcuts call used to be a 25-line inline block in
 * AppInner that re-derived which handler each shortcut should call
 * based on the current editor mode. This hook keeps the same
 * derivation logic inside App.tsx's composition root but factors the
 * noise out.
 */
import { useKeyboardShortcuts } from "./useKeyboardShortcuts";
import type { EditorMode } from "../components/MenuBar";
import type { SceneHandlers } from "./useSceneHandlers";

export interface AppShortcutsInputs {
  editorMode: EditorMode;
  handlers: SceneHandlers;
  selectedIds: Set<string>;
  selectedEntityId: string | null;
  logState: {
    dirty?: boolean;
    size: number;
    can_undo: boolean;
    can_redo: boolean;
    cursor: number;
  };
  assetLogState: {
    dirty?: boolean;
    size: number;
    can_undo: boolean;
    can_redo: boolean;
    cursor: number;
  };
  setCommandPaletteOpen: (open: boolean) => void;
  setCheatSheetOpen: (open: boolean) => void;
  setRenameRequestTick: (tick: number | ((t: number) => number)) => void;
  fitToContent: () => void;
  dock: {
    toggleBottom: () => void;
    toggleLeft: () => void;
    toggleOutline: () => void;
    toggleProperties: () => void;
  };
  fullscreen: { toggle: () => void };
}

export function useAppShortcuts({
  editorMode,
  handlers,
  selectedIds,
  selectedEntityId,
  logState,
  assetLogState,
  setCommandPaletteOpen,
  setCheatSheetOpen,
  setRenameRequestTick,
  fitToContent,
  dock,
  fullscreen,
}: AppShortcutsInputs): void {
  useKeyboardShortcuts({
    enabled: editorMode !== "play",
    onUndo:
      editorMode === "scene" ? handlers.handleUndo : handlers.handleAssetUndo,
    onRedo:
      editorMode === "scene" ? handlers.handleRedo : handlers.handleAssetRedo,
    // v0.82 P2 (ADR-0025): route Delete/Backspace through the multi-
    // delete sink when more than one id is selected. The hook keeps
    // a single-id fallback for the legacy single-select flow.
    onDeleteEntities:
      editorMode === "scene" && selectedIds.size > 1
        ? (ids) => void handlers.handleDeleteEntities(ids)
        : undefined,
    selectedIds,
    logState: editorMode === "scene" ? logState : assetLogState,
    selectedEntityId,
    onDeleteEntity: handlers.handleDeleteEntity,
    onCreateEntity:
      editorMode === "scene" ? handlers.handleCreateEntity : undefined,
    onOpenCommandPalette: () => setCommandPaletteOpen(true),
    onOpenCheatSheet: () => setCheatSheetOpen(true),
    onRenameSelected: () => setRenameRequestTick((t) => t + 1),
    onFitViewport: () => fitToContent(),
    onToggleBottomDock: dock.toggleBottom,
    onToggleLeftDock: dock.toggleLeft,
    onToggleOutlineDock: dock.toggleOutline,
    onTogglePropertiesDock: dock.toggleProperties,
    onToggleFullscreen: fullscreen.toggle,
  });
}
