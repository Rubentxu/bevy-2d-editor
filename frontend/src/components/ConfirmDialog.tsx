interface Props {
  title?: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
  danger?: boolean;
}

/**
 * In-app replacement for window.confirm (destructive).
 * Renders a centered dialog with Cancel + Confirm buttons.
 * Escape key cancels.
 */
export default function ConfirmDialog({
  title = "Confirm",
  message,
  confirmLabel = "OK",
  cancelLabel = "Cancel",
  onConfirm,
  onCancel,
  danger = false,
}: Props) {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      onCancel();
    }
  };

  return (
    <div
      className="dialog-overlay"
      data-testid="confirm-dialog"
      onClick={(e) => {
        if (e.target === e.currentTarget) onCancel();
      }}
      onKeyDown={handleKeyDown}
    >
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
      >
        {title && (
          <h3 id="confirm-dialog-title" style={{ marginBottom: 8 }}>
            {title}
          </h3>
        )}
        <p
          style={{
            color: "#cbd5e0",
            fontSize: 14,
            marginBottom: 20,
            lineHeight: 1.5,
          }}
        >
          {message}
        </p>
        <div className="dialog-actions">
          <button
            type="button"
            onClick={onCancel}
            data-testid="confirm-dialog-cancel-btn"
          >
            {cancelLabel}
          </button>
          <button
            type="button"
            className={danger ? "danger" : "primary"}
            onClick={onConfirm}
            data-testid="confirm-dialog-confirm-btn"
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
