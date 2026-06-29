import { LogState } from "../hooks/useLogState";

type EditorMode = "scene" | "asset-authoring";

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
