interface Props {
  logicalPath: string;
  unsavedCount: number;
  onSave: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}

/**
 * AssetUnsavedChangesDialog: dirty-guard dialog for Scene Asset authoring.
 *
 * Per design D4 / spec S12:
 * - Title: "Unsaved Scene Asset Changes"
 * - Body: "Scene Asset **{logicalPath}** has {unsavedCount} unsaved edit(s). Save before leaving authoring mode?"
 * - Distinct testids from scene UnsavedChangesDialog (constraint C-3)
 */
export default function AssetUnsavedChangesDialog({
  logicalPath,
  unsavedCount,
  onSave,
  onDiscard,
  onCancel,
}: Props) {
  return (
    <div className="dialog-overlay" data-testid="asset-unsaved-dialog">
      <div className="dialog">
        <h3>Unsaved Scene Asset Changes</h3>
        <p>
          Scene Asset <strong>{logicalPath}</strong> has {unsavedCount} unsaved
          edit(s). Save before leaving authoring mode?
        </p>
        <div className="dialog-actions">
          <button
            className="primary"
            data-testid="asset-unsaved-save-btn"
            onClick={onSave}
          >
            Save and Leave
          </button>
          <button
            className="danger"
            data-testid="asset-unsaved-discard-btn"
            onClick={onDiscard}
          >
            Discard and Leave
          </button>
          <button
            data-testid="asset-unsaved-cancel-btn"
            onClick={onCancel}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
