import { LogState } from "../hooks/useLogState";

interface Props {
  logState: LogState;
  onUndo: () => void;
  onRedo: () => void;
  onSave: () => void;
  onLoad: () => void;
  error: string | null;
  onDismissError: () => void;
}

export default function TopBar({
  logState,
  onUndo,
  onRedo,
  onSave,
  onLoad,
  error,
  onDismissError,
}: Props) {
  return (
    <div className="topbar" data-testid="topbar">
      <h1>Bevy 2D Editor</h1>
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
      <button onClick={onLoad} data-testid="load-btn" title="Load project (restores scenes + schemas + templates)">
        Load Project
      </button>
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
        History: {logState.size}
      </span>
    </div>
  );
}