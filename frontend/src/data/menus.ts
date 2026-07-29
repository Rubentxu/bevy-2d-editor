import type { ReactNode } from "react";

export interface MenuItem {
  label: string;
  shortcut?: string;
  disabled?: boolean;
  onClick?: () => void;
  submenu?: MenuItem[];
  testId?: string;
  separator?: boolean;
}

export interface MenuHandlers {
  handleNewScene: () => void;
  handleSave: () => void;
  handleSaveAs: () => void;
  handleLoad: () => void;
  handleExportRust: () => void;
  handleUndo: () => void;
  handleRedo: () => void;
  handleDeleteEntity: () => void;
  handleToggleAI: () => void;
  handleToggleValidationCenter: () => void;
  handleToggleTileset: () => void;
  handleToggleAutoLayer: () => void;
  handleOpenLogic: () => void;
  handleOpenCode: () => void;
  handleTogglePlay: () => void;
  handleOpenCheatSheet: () => void;
  handleWelcomeTour: () => void;
  handleAbout: () => void;
  // Phase E — dock toggles + reset layout wired by App.tsx
  handleToggleLeftDock?: () => void;
  handleToggleOutlineDock?: () => void;
  handleTogglePropertiesDock?: () => void;
  handleToggleFullscreen?: () => void;
  handleResetLayout?: () => void;
  // v0.81 Tier 1b — workspace presets. Both handlers are optional so the
  // menu still works in surface-level previews (tests, snapshots).
  handleApplyPreset?: (presetId: string) => void;
  handleSaveWorkspacePreset?: () => void;
  setTheme: (theme: "dark" | "light") => void;
  selectedEntityId: string | null;
  editorMode: "scene" | "asset-authoring" | "logic" | "code" | "play";
}

const separator = (): MenuItem => ({ label: "", separator: true });
const todo = (label: string) => () =>
  console.warn(`[menu] TODO: wire ${label}`);

export function createMenuConfig(
  handlers: MenuHandlers,
): Record<string, MenuItem[]> {
  const sceneMode = handlers.editorMode === "scene";

  return {
    File: [
      {
        label: "New Scene",
        shortcut: "Ctrl+N",
        onClick: handlers.handleNewScene,
      },
      {
        label: "Save Scene",
        shortcut: "Ctrl+S",
        onClick: handlers.handleSave,
        testId: "save-btn",
      },
      {
        label: "Save Scene As…",
        shortcut: "Ctrl+Shift+S",
        onClick: handlers.handleSaveAs,
      },
      {
        label: "Load Project",
        shortcut: "Ctrl+O",
        onClick: handlers.handleLoad,
        testId: "load-btn",
      },
      separator(),
      {
        label: "Export Rust…",
        shortcut: "Ctrl+E",
        onClick: handlers.handleExportRust,
        testId: "export-rs-btn",
      },
      separator(),
      { label: "Quit (browser)", disabled: true },
    ],
    Edit: [
      {
        label: "Undo",
        shortcut: "Ctrl+Z",
        onClick: handlers.handleUndo,
        testId: "undo-btn",
      },
      {
        label: "Redo",
        shortcut: "Ctrl+Y / Ctrl+Shift+Z",
        onClick: handlers.handleRedo,
        testId: "redo-btn",
      },
      separator(),
      {
        label: "Cut",
        shortcut: "Ctrl+X",
        disabled: true,
        onClick: todo("Cut"),
      },
      {
        label: "Copy",
        shortcut: "Ctrl+C",
        disabled: true,
        onClick: todo("Copy"),
      },
      {
        label: "Paste",
        shortcut: "Ctrl+V",
        disabled: true,
        onClick: todo("Paste"),
      },
      {
        label: "Duplicate",
        shortcut: "Ctrl+D",
        disabled: true,
        onClick: todo("Duplicate"),
      },
      {
        label: "Delete",
        shortcut: "Del",
        disabled: !handlers.selectedEntityId,
        onClick: handlers.handleDeleteEntity,
      },
      separator(),
      {
        label: "Find",
        shortcut: "Ctrl+F",
        disabled: true,
        onClick: todo("Find"),
      },
    ],
    View: [
      {
        label: "Toggle Assets",
        shortcut: "F6",
        onClick: handlers.handleToggleLeftDock ?? todo("Toggle Assets"),
        testId: "menu-toggle-assets",
      },
      {
        label: "Toggle Outline",
        shortcut: "F8",
        onClick: handlers.handleToggleOutlineDock ?? todo("Toggle Outline"),
        testId: "menu-toggle-outline",
      },
      {
        label: "Toggle Properties",
        shortcut: "Shift+F8",
        onClick: handlers.handleTogglePropertiesDock ?? todo("Toggle Properties"),
        testId: "menu-toggle-properties",
      },
      { label: "Toggle Tools", shortcut: "F7", onClick: todo("Toggle Tools") },
      separator(),
      {
        label: "Fullscreen Viewport",
        shortcut: "F9",
        onClick: handlers.handleToggleFullscreen ?? todo("Fullscreen Viewport"),
        testId: "menu-fullscreen",
      },
      separator(),
      {
        label: "Reset Layout",
        onClick: handlers.handleResetLayout ?? todo("Reset Layout"),
        testId: "menu-reset-layout",
      },
      // ── Workspace presets (v0.81 Tier 1b) ──────────────────────────────
      // Built-in layouts target common game genres (2D Platformer, Top-Down
      // RPG, FPS) plus a Minimal layout for full-screen preview.
      // `handleApplyPreset` / `handleSaveWorkspacePreset` are wired by
      // App.tsx → useDockResize so the layout actually shifts; the menu
      // itself just dispatch the click.
      {
        label: "Workspace",
        submenu: [
          {
            label: "Default",
            onClick: () => handlers.handleApplyPreset?.("default"),
            testId: "menu-preset-default",
          },
          {
            label: "2D Platformer",
            onClick: () => handlers.handleApplyPreset?.("2d-platformer"),
            testId: "menu-preset-2d-platformer",
          },
          {
            label: "Top-Down RPG",
            onClick: () => handlers.handleApplyPreset?.("top-down-rpg"),
            testId: "menu-preset-top-down-rpg",
          },
          {
            label: "FPS",
            onClick: () => handlers.handleApplyPreset?.("fps"),
            testId: "menu-preset-fps",
          },
          {
            label: "Minimal",
            onClick: () => handlers.handleApplyPreset?.("minimal"),
            testId: "menu-preset-minimal",
          },
          { label: "", separator: true },
          {
            label: "Save Current as Preset…",
            onClick:
              handlers.handleSaveWorkspacePreset ??
              todo("Save Current as Preset…"),
            testId: "menu-preset-save",
          },
        ],
        testId: "menu-view-workspace",
      },
      {
        label: "Theme",
        submenu: [
          { label: "Dark", onClick: () => handlers.setTheme("dark") },
          { label: "Light", onClick: () => handlers.setTheme("light") },
        ],
      },
    ],
    Tools: [
      {
        label: "AI Assistant",
        shortcut: "Ctrl+Shift+A",
        onClick: handlers.handleToggleAI,
      },
      {
        label: "Validation Center",
        onClick: handlers.handleToggleValidationCenter,
      },
      { label: "Schema Authoring", onClick: todo("Schema Authoring") },
      {
        label: "Tileset Panel",
        onClick: handlers.handleToggleTileset,
      },
      {
        label: "Auto Layer Panel",
        onClick: handlers.handleToggleAutoLayer,
      },
      separator(),
      {
        label: "Logic Editor",
        disabled: !sceneMode,
        onClick: handlers.handleOpenLogic,
        testId: "menu-item-logic-editor",
      },
      {
        label: "Code Editor",
        disabled: !sceneMode,
        onClick: handlers.handleOpenCode,
        testId: "menu-item-code-editor",
      },
      separator(),
      {
        label: "Project Asset Browser",
        shortcut: "Ctrl+P",
        onClick: todo("Project Asset Browser"),
      },
    ],
    Run: [
      {
        label: handlers.editorMode === "play" ? "Stop" : "Play",
        shortcut: "Ctrl+P",
        onClick: handlers.handleTogglePlay,
        testId: handlers.editorMode === "play" ? "stop-btn" : "play-btn",
      },
      { label: "Pause", disabled: true, onClick: todo("Pause") },
      { label: "Step", disabled: true, onClick: todo("Step") },
    ],
    Help: [
      {
        label: "Cheat Sheet",
        shortcut: "?",
        onClick: handlers.handleOpenCheatSheet,
      },
      { label: "About", onClick: handlers.handleAbout },
      { label: "Welcome Tour", onClick: handlers.handleWelcomeTour },
    ],
  };
}

export const menuConfig: Record<string, MenuItem[]> = {};

export interface MenuDropdownProps {
  label: string;
  items?: MenuItem[];
  children?: ReactNode;
  open: boolean;
  onOpen: () => void;
  onClose: () => void;
  testId: string;
  /** Bounding rect of the trigger button, used to position the portaled dropdown with position:fixed. */
  anchorRect?: DOMRect;
}
