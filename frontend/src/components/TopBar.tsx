import { LogState } from "../hooks/useLogState";

type EditorMode = "scene" | "asset-authoring" | "logic";

interface Props {
  editorMode?: EditorMode;
  onOpenAssets?: () => void;
  onBackToScene?: () => void;
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
  error: string | null;
  onDismissError: () => void;
}

export default function TopBar({
  editorMode = "scene",
  onOpenAssets,
  onBackToScene,
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
  error,
  onDismissError,
}: Props) {
  const isAssetAuthoring = editorMode === "asset-authoring";

  return (
    <div className="topbar" data-testid="topbar">
      <h1>Bevy 2D Editor</h1>

      {/* Scene mode buttons — hidden in asset authoring mode */}
      {!isAssetAuthoring && (
        <>
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
