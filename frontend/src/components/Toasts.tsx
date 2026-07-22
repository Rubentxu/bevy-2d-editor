import { useToasts, type ToastSeverity } from "../hooks/useToasts";

/**
 * Phase 5 — Toast stack view.
 *
 * Renders the current toast queue in the top-right corner. Each row is
 * icon + message + dismiss × button. The `.slide-up` animation from Phase 4
 * tokens gives the entrance a soft pop.
 *
 * This component is mounted once at the App root, inside the ToastProvider.
 */
export default function Toasts() {
  const { toasts, dismissToast } = useToasts();

  return (
    <div className="toast-stack" data-testid="toast-stack" aria-live="polite">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`toast toast-${toast.severity} slide-up`}
          role="status"
          data-testid={`toast-${toast.severity}`}
          data-toast-id={toast.id}
        >
          <span className="toast-icon" aria-hidden="true">
            {iconFor(toast.severity)}
          </span>
          <span className="toast-message">{toast.message}</span>
          <button
            type="button"
            className="toast-dismiss"
            onClick={() => dismissToast(toast.id)}
            aria-label="Dismiss notification"
            title="Dismiss"
            data-testid="toast-dismiss"
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}

function iconFor(severity: ToastSeverity): string {
  switch (severity) {
    case "success":
      return "✓";
    case "warning":
      return "⚠";
    case "error":
      return "✗";
    default:
      return "ℹ";
  }
}
