/**
 * useAppCommandPalette — builds the Command Palette and Cheat Sheet catalogs
 * plus the cross-window command executor used by Global Search.
 *
 * The paletteCommands / serializablePaletteItems / executeCommandById /
 * cheatSheetGroups quartet used to be two useMemos + one useCallback in
 * AppInner (lines 1127–1471 of the pre-refactor App.tsx). They were lifted
 * into this hook so App.tsx stays under the ≤600 line composition-root
 * target.
 *
 * The hook is *behaviour-preserving*: every command calls the exact
 * handler the original code did, no stubs, no console-and-forget.
 */
import { useCallback, useMemo } from "react";
import type { PaletteCommand } from "../components/CommandPalette";
import type { ShortcutGroup as CheatSheetGroup } from "../components/CheatSheet";
import type { EditorMode } from "../components/MenuBar";
import type { SceneHandlers } from "./useSceneHandlers";

export interface CommandPaletteApi {
  paletteCommands: PaletteCommand[];
  serializablePaletteItems: {
    id: string;
    label: string;
    shortcut?: string;
    group: string;
  }[];
  executeCommandById: (commandId: string) => void;
  cheatSheetGroups: CheatSheetGroup[];
}

export interface CommandPaletteInputs {
  editorMode: EditorMode;
  selectedEntityId: string | null;
  handlers: SceneHandlers;
  setExportRustOpen: (open: boolean) => void;
  setCheatSheetOpen: (open: boolean) => void;
  setRenameRequestTick: (tick: number | ((t: number) => number)) => void;
  setEditorMode: (mode: EditorMode) => void;
  fitToContent: () => void;
  resetViewport: () => void;
}

export function useAppCommandPalette({
  editorMode,
  selectedEntityId,
  handlers,
  setExportRustOpen,
  setCheatSheetOpen,
  setRenameRequestTick,
  setEditorMode,
  fitToContent,
  resetViewport,
}: CommandPaletteInputs): CommandPaletteApi {
  // ── Command palette catalog (Phase 3.2) ───────────────────────────────────
  // Static list of >15 commands wired to existing App handlers. Built
  // every render but only when one of the dependencies changes (the
  // handlers are useCallback-wrapped so identity is stable).
  const paletteCommands = useMemo<PaletteCommand[]>(
    () => [
      // File
      {
        id: "file.save",
        label: "Save Scene",
        shortcut: "Ctrl+S",
        group: "File",
        action: handlers.handleSave,
      },
      {
        id: "file.load",
        label: "Load Project",
        group: "File",
        action: handlers.handleLoad,
      },
      {
        id: "file.export",
        label: "Export Rust",
        group: "File",
        action: () => setExportRustOpen(true),
      },
      {
        id: "file.new-scene",
        label: "New Scene",
        group: "File",
        action: () => handlers.handleNewScene(`scene_${Date.now()}`),
      },
      // Edit
      {
        id: "edit.undo",
        label: "Undo",
        shortcut: "Ctrl+Z",
        group: "Edit",
        action: () => {
          if (editorMode === "scene") void handlers.handleUndo();
          else void handlers.handleAssetUndo();
        },
      },
      {
        id: "edit.redo",
        label: "Redo",
        shortcut: "Ctrl+Shift+Z",
        group: "Edit",
        action: () => {
          if (editorMode === "scene") void handlers.handleRedo();
          else void handlers.handleAssetRedo();
        },
      },
      {
        id: "edit.delete",
        label: "Delete Selection",
        shortcut: "Del",
        group: "Edit",
        action: () => {
          if (selectedEntityId) void handlers.handleDeleteEntity(selectedEntityId);
        },
      },
      {
        id: "edit.new-entity",
        label: "New Entity",
        shortcut: "N",
        group: "Edit",
        action: () => {
          if (editorMode === "scene") void handlers.handleCreateEntity();
        },
      },
      {
        id: "edit.rename",
        label: "Rename Selected",
        shortcut: "F2",
        group: "Edit",
        action: () => setRenameRequestTick((t) => t + 1),
      },
      // View
      {
        id: "view.toggle-ai",
        label: "Toggle AI Panel",
        group: "View",
        action: handlers.handleToggleAI,
      },
      {
        id: "view.toggle-validation",
        label: "Toggle Validation Center",
        group: "View",
        action: handlers.handleToggleValidationCenter,
      },
      {
        id: "view.toggle-tileset",
        label: "Toggle Tileset",
        group: "View",
        action: handlers.handleToggleTileset,
      },
      {
        id: "view.toggle-autolayer",
        label: "Toggle Auto Layer",
        group: "View",
        action: handlers.handleToggleAutoLayer,
      },
      {
        id: "view.reset-viewport",
        label: "Reset Viewport",
        group: "View",
        action: () => resetViewport(),
      },
      {
        id: "view.fit-viewport",
        label: "Fit Viewport",
        shortcut: "F",
        group: "View",
        action: () => fitToContent(),
      },
      {
        id: "view.open-logic",
        label: "Open Logic Editor",
        group: "View",
        action: handlers.handleOpenLogic,
      },
      {
        id: "view.open-code",
        label: "Open Code Editor",
        group: "View",
        action: handlers.handleOpenCode,
      },
      {
        id: "view.open-browser",
        label: "Open Project Browser",
        group: "View",
        action: () => setEditorMode("asset-authoring"),
      },
      // Assets
      {
        id: "assets.create",
        label: "Create Scene Asset",
        group: "Assets",
        action: () => handlers.handleAssetCreate(`asset_${Date.now()}`, "actor"),
      },
      // Play
      {
        id: "play.toggle",
        label: "Play / Stop",
        group: "Play",
        action: handlers.handleTogglePlay,
      },
      // Help
      {
        id: "help.cheatsheet",
        label: "Show Cheat Sheet",
        shortcut: "?",
        group: "Help",
        action: () => setCheatSheetOpen(true),
      },
    ],
    [
      editorMode,
      selectedEntityId,
      handlers.handleSave,
      handlers.handleLoad,
      handlers.handleNewScene,
      handlers.handleUndo,
      handlers.handleAssetUndo,
      handlers.handleRedo,
      handlers.handleAssetRedo,
      handlers.handleDeleteEntity,
      handlers.handleCreateEntity,
      handlers.handleToggleAI,
      handlers.handleToggleValidationCenter,
      handlers.handleToggleTileset,
      handlers.handleToggleAutoLayer,
      resetViewport,
      fitToContent,
      handlers.handleOpenLogic,
      handlers.handleOpenCode,
      handlers.handleAssetCreate,
      handlers.handleTogglePlay,
      setExportRustOpen,
      setRenameRequestTick,
      setCheatSheetOpen,
      setEditorMode,
    ],
  );

  // ── Command palette executor for Global Search (CRITICAL ISSUE 2) ──────────
  // Expose command palette items as search results via window.
  // The serializable command metadata (id, label, shortcut, group) is returned;
  // the actual action is dispatched via __executeCommand(commandId).
  const serializablePaletteItems = useMemo(() => {
    return paletteCommands.map((cmd) => ({
      id: cmd.id,
      label: cmd.label,
      shortcut: cmd.shortcut,
      group: cmd.group,
    }));
  }, [paletteCommands]);

  // Command executor: looks up command by id and invokes its action.
  const executeCommandById = useCallback(
    (commandId: string) => {
      const cmd = paletteCommands.find((c) => c.id === commandId);
      if (cmd) {
        cmd.action();
      }
    },
    [paletteCommands],
  );

  // ── Cheat sheet shortcuts (Phase 3.3) ─────────────────────────────────────
  const cheatSheetGroups = useMemo<CheatSheetGroup[]>(
    () => [
      {
        title: "General",
        entries: [
          { keys: ["Ctrl", "K"], label: "Open command palette" },
          { keys: ["?"], label: "Open cheat sheet" },
        ],
      },
      {
        title: "Editing",
        entries: [
          { keys: ["Ctrl", "Z"], label: "Undo" },
          { keys: ["Ctrl", "Y"], label: "Redo" },
          { keys: ["Ctrl", "Shift", "Z"], label: "Redo (alternate)" },
          { keys: ["N"], label: "New entity" },
          { keys: ["F2"], label: "Rename selected entity" },
          { keys: ["Del"], label: "Delete selection" },
          { keys: ["Backspace"], label: "Delete selection" },
        ],
      },
      {
        title: "Viewport",
        entries: [
          { keys: ["F"], label: "Fit viewport to content" },
          { keys: ["Wheel"], label: "Zoom around cursor" },
          { keys: ["Space", "+ Drag"], label: "Pan canvas" },
        ],
      },
      {
        title: "Play",
        entries: [
          {
            keys: ["Space", "+ W/A/S/D"],
            label: "Move (gamepad in play mode)",
          },
        ],
      },
    ],
    [],
  );

  return {
    paletteCommands,
    serializablePaletteItems,
    executeCommandById,
    cheatSheetGroups,
  };
}
