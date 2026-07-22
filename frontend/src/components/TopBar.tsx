import { useState } from "react";
import { LogState } from "../hooks/useLogState";
import { useHotReloadStatus } from "../hooks/useHotReloadStatus";
import { useTheme } from "../hooks/useTheme";
import ToolbarGroup from "./ToolbarGroup";
import TooltipButton from "./TooltipButton";

type EditorMode = "scene" | "asset-authoring" | "logic" | "code" | "play";

interface Props {
  editorMode?: EditorMode;
  onOpenAssets?: () => void;
  onBackToScene?: () => void;
  onOpenLogic?: () => void;
  onOpenCode?: () => void;
  logState: LogState;
  onUndo: () => void;
  onRedo: () => void;
  onSave: () => void;
  onLoad: () => void;
  onExportRust: () => void;
  onToggleAI: () => void;
  aiPanelOpen: boolean;
  onToggleValidationCenter: () => void;
  validationCenterOpen: boolean;
  onToggleTileset: () => void;
  tilesetPanelOpen: boolean;
  onToggleAutoLayer: () => void;
  autoLayerPanelOpen: boolean;
  onTogglePlay?: () => void;
}

export default function TopBar({
  editorMode = "scene",
  onOpenAssets,
  onBackToScene,
  onOpenLogic,
  onOpenCode,
  logState,
  onUndo,
  onRedo,
  onSave,
  onLoad,
  onExportRust,
  onToggleAI,
  aiPanelOpen,
  onToggleValidationCenter,
  validationCenterOpen,
  onToggleTileset,
  tilesetPanelOpen,
  onToggleAutoLayer,
  autoLayerPanelOpen,
  onTogglePlay,
}: Props) {
  const isAssetAuthoring = editorMode === "asset-authoring";
  const isPlayMode = editorMode === "play";
  const [isRefreshing, setIsRefreshing] = useState(false);
  const { lastReloadedAt, inFlightSaves, refresh } = useHotReloadStatus();
  const { theme, toggleTheme } = useTheme();

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refresh();
    } finally {
      setIsRefreshing(false);
    }
  };

  return (
    <div className="topbar" data-testid="topbar">
      <h1>🎮 Bevy 2D Editor</h1>
      <button
        type="button"
        className="theme-toggle-btn"
        onClick={toggleTheme}
        title={
          theme === "dark" ? "Switch to light theme" : "Switch to dark theme"
        }
        aria-label={
          theme === "dark" ? "Switch to light theme" : "Switch to dark theme"
        }
        data-testid="theme-toggle-btn"
      >
        {theme === "dark" ? "☀️" : "🌙"}
      </button>

      <div className="toolbar-groups">
        <ToolbarGroup label="Mode" data-testid="toolbar-group-mode">
          <TooltipButton
            icon="▣ Scene"
            label="Scene editor"
            onClick={onBackToScene ?? (() => {})}
            disabled={!onBackToScene && editorMode !== "scene"}
            active={editorMode === "scene"}
            testId="open-scene-btn"
          />
          <TooltipButton
            icon="⚡ Logic"
            label="Open Logic Graph Editor"
            onClick={onOpenLogic ?? (() => {})}
            disabled={!onOpenLogic}
            active={editorMode === "logic"}
            testId="open-logic-btn"
          />
          <TooltipButton
            icon="📝 Code"
            label="Open Code Editor"
            onClick={onOpenCode ?? (() => {})}
            disabled={!onOpenCode}
            active={editorMode === "code"}
            testId="open-code-btn"
          />
          {isAssetAuthoring && (
            <TooltipButton
              icon="← Back to Scene"
              label="Return to scene editor"
              onClick={onBackToScene ?? (() => {})}
              disabled={!onBackToScene}
              testId="back-to-scene-btn"
            />
          )}
        </ToolbarGroup>

        <ToolbarGroup label="Edit" data-testid="toolbar-group-edit">
          <TooltipButton
            icon="↶ Undo"
            label="Undo"
            shortcut="Ctrl+Z"
            onClick={onUndo}
            disabled={!logState.can_undo}
            testId="undo-btn"
          />
          <TooltipButton
            icon="↷ Redo"
            label="Redo"
            shortcut="Ctrl+Shift+Z"
            onClick={onRedo}
            disabled={!logState.can_redo}
            testId="redo-btn"
          />
          <TooltipButton
            icon="Save"
            label={isAssetAuthoring ? "Save asset" : "Save scene"}
            shortcut="Ctrl+S"
            onClick={onSave}
            testId="save-btn"
          />
          <TooltipButton
            icon="Load Project"
            label="Load project"
            shortcut="Ctrl+O"
            onClick={onLoad}
            testId="load-btn"
          />
          <TooltipButton
            icon="📥 Export .rs"
            label="Export scene as Rust code"
            shortcut="Ctrl+E"
            onClick={onExportRust}
            testId="export-rs-btn"
          />
        </ToolbarGroup>

        <ToolbarGroup label="Tools" data-testid="toolbar-group-tools">
          <TooltipButton
            icon={isRefreshing ? "…" : "↻"}
            label="Force hot-reload"
            shortcut="Ctrl+R"
            onClick={handleRefresh}
            disabled={inFlightSaves > 0 || isRefreshing}
            testId="topbar-refresh"
          />
          <TooltipButton
            icon="✨ AI"
            label="AI panel"
            onClick={onToggleAI}
            active={aiPanelOpen}
            testId="ai-panel-btn"
          />
          <TooltipButton
            icon="✅ Validation"
            label="Validation Center"
            shortcut="Ctrl+;"
            onClick={onToggleValidationCenter}
            active={validationCenterOpen}
            testId="validation-center-btn"
          />
          <TooltipButton
            icon="🏠 Tileset"
            label="Tileset Panel"
            shortcut="Ctrl+T"
            onClick={onToggleTileset}
            active={tilesetPanelOpen}
            testId="tileset-panel-btn"
          />
          <TooltipButton
            icon="🔄 Auto Layer"
            label="Auto Layer Panel"
            shortcut="Ctrl+L"
            onClick={onToggleAutoLayer}
            active={autoLayerPanelOpen}
            testId="auto-layer-panel-btn"
          />
          {onOpenAssets && (
            <TooltipButton
              icon="Assets"
              label="Open asset browser"
              onClick={onOpenAssets}
              testId="open-assets-btn"
            />
          )}
        </ToolbarGroup>

        <ToolbarGroup label="Run" data-testid="toolbar-group-run">
          {(editorMode === "scene" || isPlayMode) && onTogglePlay && (
            <TooltipButton
              icon={isPlayMode ? "⏹ Stop" : "▶ Play"}
              label={
                isPlayMode
                  ? "Stop preview and return to editor"
                  : "Start preview"
              }
              shortcut="Ctrl+P"
              onClick={onTogglePlay}
              active={isPlayMode}
              testId={isPlayMode ? "stop-btn" : "play-btn"}
            />
          )}
        </ToolbarGroup>
      </div>

      {lastReloadedAt != null && (
        <span data-testid="hot-reload-badge">
          Hot-reload: {lastReloadedAt.toLocaleTimeString()}
        </span>
      )}
      <span className="status" data-testid="log-status">
        {isAssetAuthoring ? "Asset" : "Scene"}: {logState.size}
      </span>
    </div>
  );
}
