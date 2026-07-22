import { useMemo, useState } from "react";
import { useScenes } from "../hooks/useScenes";
import { useTheme } from "../hooks/useTheme";
import type { LogState } from "../hooks/useLogState";
import { createMenuConfig } from "../data/menus";
import MenuDropdown from "./Menu/MenuDropdown";

export type EditorMode =
  "scene" | "asset-authoring" | "logic" | "code" | "play";

export interface MenuBarProps {
  editorMode?: EditorMode;
  onOpenAssets?: () => void;
  onBackToScene?: () => void;
  onOpenLogic?: () => void;
  onOpenCode?: () => void;
  logState: LogState;
  onUndo: () => void;
  onRedo: () => void;
  onSave: () => void;
  onSaveAs?: () => void;
  onLoad: () => void;
  onExportRust: () => void;
  onNewScene?: () => void;
  onDeleteEntity?: () => void;
  selectedEntityId?: string | null;
  onToggleAI: () => void;
  aiPanelOpen: boolean;
  onToggleValidationCenter: () => void;
  validationCenterOpen: boolean;
  onToggleTileset: () => void;
  tilesetPanelOpen: boolean;
  onToggleAutoLayer: () => void;
  autoLayerPanelOpen: boolean;
  onTogglePlay?: () => void;
  onOpenSearch?: () => void;
  onOpenCheatSheet?: () => void;
  onWelcomeTour?: () => void;
}

export default function MenuBar({
  editorMode = "scene",
  onOpenLogic,
  onOpenCode,
  logState,
  onUndo,
  onRedo,
  onSave,
  onSaveAs,
  onLoad,
  onExportRust,
  onNewScene,
  onDeleteEntity,
  selectedEntityId = null,
  onToggleAI,
  onToggleValidationCenter,
  onToggleTileset,
  onToggleAutoLayer,
  onTogglePlay,
  onOpenSearch,
  onOpenCheatSheet,
  onWelcomeTour,
}: MenuBarProps) {
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const { scenes } = useScenes();
  const { theme, setTheme, toggleTheme } = useTheme();
  const projectName = scenes[0]?.name ?? "Untitled";

  const menus = useMemo(
    () =>
      createMenuConfig({
        handleNewScene:
          onNewScene ?? (() => console.warn("[menu] TODO: wire New Scene")),
        handleSave: onSave,
        handleSaveAs: onSaveAs ?? onSave,
        handleLoad: onLoad,
        handleExportRust: onExportRust,
        handleUndo: onUndo,
        handleRedo: onRedo,
        handleDeleteEntity: onDeleteEntity ?? (() => {}),
        handleToggleAI: onToggleAI,
        handleToggleValidationCenter: onToggleValidationCenter,
        handleToggleTileset: onToggleTileset,
        handleToggleAutoLayer: onToggleAutoLayer,
        handleOpenLogic: onOpenLogic ?? (() => {}),
        handleOpenCode: onOpenCode ?? (() => {}),
        handleTogglePlay: onTogglePlay ?? (() => {}),
        handleOpenCheatSheet: onOpenCheatSheet ?? (() => {}),
        handleWelcomeTour:
          onWelcomeTour ??
          (() => console.warn("[menu] TODO: wire Welcome Tour")),
        setTheme,
        selectedEntityId,
        editorMode,
      }),
    [
      editorMode,
      onDeleteEntity,
      onExportRust,
      onLoad,
      onNewScene,
      onOpenCheatSheet,
      onOpenCode,
      onOpenLogic,
      onRedo,
      onSave,
      onSaveAs,
      onToggleAI,
      onToggleAutoLayer,
      onTogglePlay,
      onToggleTileset,
      onToggleValidationCenter,
      onUndo,
      onWelcomeTour,
      selectedEntityId,
      setTheme,
    ],
  );

  return (
    <nav
      className="menubar"
      data-testid="menubar"
      aria-label="Application menu"
    >
      <div className="menubar-brand">
        🎮 {projectName}
        <h1 className="visually-hidden">Bevy 2D Editor</h1>
      </div>
      <div className="menubar-menus" data-testid="topbar">
        <div className="menubar-legacy-actions" aria-label="Quick actions">
          <div data-testid="toolbar-group-mode">
            <button
              type="button"
              onClick={() => {
                if (!onOpenLogic || !onOpenCode) return;
                onOpenLogic();
                window.setTimeout(onOpenCode, 0);
              }}
              disabled={!onOpenLogic || !onOpenCode}
            >
              Logic Code
            </button>
          </div>
          <div data-testid="toolbar-group-edit">
            <button
              type="button"
              data-testid="undo-btn"
              title="Undo (Ctrl+Z)"
              onClick={onUndo}
            >
              Undo
            </button>
            <button
              type="button"
              data-testid="redo-btn"
              title="Redo (Ctrl+Shift+Z)"
              onClick={onRedo}
            >
              Redo
            </button>
            <button
              type="button"
              data-testid="save-btn"
              title="Save Scene (Ctrl+S)"
              onClick={onSave}
            >
              Save
            </button>
            <button
              type="button"
              data-testid="load-btn"
              title="Load Project (Ctrl+O)"
              onClick={onLoad}
            >
              Load
            </button>
          </div>
          <div data-testid="toolbar-group-tools">
            <button
              type="button"
              data-testid="topbar-refresh"
              title="Force hot-reload"
              onClick={() => window.location.reload()}
            >
              ↻
            </button>
            <button type="button" onClick={onToggleAI}>
              AI
            </button>
          </div>
          <div data-testid="toolbar-group-run">
            <button
              type="button"
              data-testid={editorMode === "play" ? "stop-btn" : "play-btn"}
              onClick={onTogglePlay}
            >
              {editorMode === "play" ? "Stop" : "Play"}
            </button>
          </div>
        </div>
        {Object.entries(menus).map(([label, items]) => (
          <MenuDropdown
            key={label}
            label={label}
            items={items}
            open={openMenu === label}
            onOpen={() => setOpenMenu(label)}
            onClose={() => setOpenMenu(null)}
            testId={`menu-${label.toLowerCase()}`}
          />
        ))}
      </div>
      <div className="menubar-actions">
        <button
          type="button"
          className="menubar-icon-button theme-toggle-btn"
          onClick={toggleTheme}
          aria-label={
            theme === "dark" ? "Switch to light theme" : "Switch to dark theme"
          }
          title={
            theme === "dark" ? "Switch to light theme" : "Switch to dark theme"
          }
          data-testid="theme-toggle-btn"
        >
          {theme === "dark" ? "☀️" : "🌙"}
        </button>
        <button
          type="button"
          className="menubar-search-button"
          onClick={onOpenSearch}
          aria-label="Open command palette"
          title="Search commands (Ctrl+K)"
        >
          🔍 <span>Cmd+K</span>
        </button>
        <button
          type="button"
          className={`menubar-play-button${editorMode === "play" ? " active" : ""}`}
          onClick={onTogglePlay}
          data-testid={editorMode === "play" ? "stop-btn" : "play-btn"}
        >
          {editorMode === "play" ? "■ Stop" : "▶ Play"}
        </button>
        <span className="menubar-status" data-testid="log-status">
          {logState.size}
        </span>
      </div>
    </nav>
  );
}
