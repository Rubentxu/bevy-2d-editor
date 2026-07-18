import { useState } from "react";
import { LogState } from "../hooks/useLogState";
import { useHotReloadStatus } from "../hooks/useHotReloadStatus";

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
  error: string | null;
  onDismissError: () => void;
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
  error,
  onDismissError,
}: Props) {
  const isAssetAuthoring = editorMode === "asset-authoring";
  const isPlayMode = editorMode === "play";
  const [isRefreshing, setIsRefreshing] = useState(false);
  const { lastReloadedAt, inFlightSaves, refresh } = useHotReloadStatus();

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
      <h1>Bevy 2D Editor</h1>

      {/* Scene mode buttons — hidden in asset authoring mode */}
      {!isAssetAuthoring && (
        <>
          <button
            onClick={onOpenLogic}
            data-testid="open-logic-btn"
            title="Open Logic Graph Editor"
          >
            ⚡ Logic
          </button>
          <button
            onClick={onOpenCode}
            data-testid="open-code-btn"
            title="Open Code Editor"
          >
            📝 Code
          </button>
          {/* Play/Stop button — only visible in scene and play modes */}
          {(editorMode === "scene" || isPlayMode) && onTogglePlay && (
            <button
              onClick={onTogglePlay}
              data-testid={isPlayMode ? "stop-btn" : "play-btn"}
              title={isPlayMode ? "Stop preview and return to editor" : "Start preview"}
            >
              {isPlayMode ? "⏹ Stop" : "▶ Play"}
            </button>
          )}
          {!isAssetAuthoring && lastReloadedAt != null && (
            <span data-testid="hot-reload-badge">
              Hot-reload: {lastReloadedAt.toLocaleTimeString()}
            </span>
          )}
          {!isAssetAuthoring && (
            <button
              onClick={handleRefresh}
              disabled={inFlightSaves > 0 || isRefreshing}
              data-testid="topbar-refresh"
              title="Force hot-reload"
            >
              ↻
            </button>
          )}
          <button
            onClick={onUndo}
            disabled={!logState.can_undo}
            data-testid="undo-btn"
            title="Undo (Ctrl+Z)"
          >
            ↶ Undo
          </button>
          <button
            onClick={onRedo}
            disabled={!logState.can_redo}
            data-testid="redo-btn"
            title="Redo (Ctrl+Shift+Z)"
          >
            ↷ Redo
          </button>
          <button onClick={onSave} data-testid="save-btn" title="Save scene">
            Save
          </button>
          <button onClick={onLoad} data-testid="load-btn" title="Load project (restores scenes + schemas)">
            Load Project
          </button>
          <button onClick={onExportRust} data-testid="export-rs-btn" title="Export scene as Rust code">
            📥 Export .rs
          </button>
          <button
            onClick={onToggleAI}
            data-testid="ai-panel-btn"
            title={aiPanelOpen ? "Close AI panel" : "Open AI panel"}
            className={aiPanelOpen ? "ai-btn active" : "ai-btn"}
          >
            ✨ AI
          </button>
          <button
            onClick={onToggleValidationCenter}
            data-testid="validation-center-btn"
            title={validationCenterOpen ? "Close Validation Center" : "Open Validation Center"}
            className={validationCenterOpen ? "vc-btn active" : "vc-btn"}
          >
            ✅ Validation
          </button>
          <button
            onClick={onToggleTileset}
            data-testid="tileset-panel-btn"
            title={tilesetPanelOpen ? "Close Tileset Panel" : "Open Tileset Panel"}
            className={tilesetPanelOpen ? "tileset-btn active" : "tileset-btn"}
          >
            🏠 Tileset
          </button>
          <button
            onClick={onToggleAutoLayer}
            data-testid="auto-layer-panel-btn"
            title={autoLayerPanelOpen ? "Close Auto Layer Panel" : "Open Auto Layer Panel"}
            className={autoLayerPanelOpen ? "auto-layer-btn active" : "auto-layer-btn"}
          >
            🔄 Auto Layer
          </button>
        </>
      )}

      {/* Asset authoring mode buttons */}
      {isAssetAuthoring && (
        <>
          <button
            onClick={onBackToScene}
            data-testid="back-to-scene-btn"
            title="Return to scene editor"
          >
            ← Back to Scene
          </button>
        </>
      )}

      {/* Always visible */}
      {error && (
        <span
          className="error"
          onClick={onDismissError}
          title="Click to dismiss"
          data-testid="topbar-error"
        >
          {error}
        </span>
      )}
      <span className="status" data-testid="log-status">
        {isAssetAuthoring ? "Asset" : "Scene"}: {logState.size}
      </span>
    </div>
  );
}
