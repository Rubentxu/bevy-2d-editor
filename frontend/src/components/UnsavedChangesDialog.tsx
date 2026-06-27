interface Props {
  sourceName: string;
  onSave: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}

export default function UnsavedChangesDialog({ sourceName, onSave, onDiscard, onCancel }: Props) {
  return (
    <div className="dialog-overlay" data-testid="unsaved-dialog">
      <div className="dialog">
        <h3>Unsaved Changes</h3>
        <p>
          Scene <strong>{sourceName}</strong> has unsaved changes.
        </p>
        <div className="dialog-actions">
          <button
            className="primary"
            data-testid="unsaved-save-btn"
            onClick={onSave}
          >
            Save
          </button>
          <button
            className="danger"
            data-testid="unsaved-discard-btn"
            onClick={onDiscard}
          >
            Discard
          </button>
          <button
            data-testid="unsaved-cancel-btn"
            onClick={onCancel}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
