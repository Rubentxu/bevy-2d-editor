import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

export type ToastSeverity = "success" | "warning" | "error" | "info";

export interface Toast {
  id: string;
  message: string;
  severity: ToastSeverity;
  /** Timestamp the toast was created — drives auto-dismiss. */
  createdAt: number;
}

export interface ToastContextValue {
  toasts: Toast[];
  addToast: (message: string, severity?: ToastSeverity) => string;
  dismissToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

const AUTO_DISMISS_MS = 5_000;
const MAX_VISIBLE = 3;

/**
 * Phase 5 — Toast provider + pub-sub context.
 *
 * Any component in the tree can call `useToasts()` and dispatch toasts
 * without prop-drilling. The `Toasts` view component subscribes to the
 * same context to render the queue.
 *
 * - Max 3 visible at once; FIFO eviction when a 4th arrives.
 * - Auto-dismiss after 5s. Manual dismiss via the × button on the row.
 */
export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const counter = useRef(0);

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const addToast = useCallback(
    (message: string, severity: ToastSeverity = "info") => {
      counter.current += 1;
      const id = `toast_${counter.current}_${Date.now()}`;
      const toast: Toast = { id, message, severity, createdAt: Date.now() };
      setToasts((prev) => {
        const next = [...prev, toast];
        // Cap visible at MAX_VISIBLE — drop oldest first.
        if (next.length > MAX_VISIBLE) next.shift();
        return next;
      });
      return id;
    },
    [],
  );

  const value = useMemo<ToastContextValue>(
    () => ({ toasts, addToast, dismissToast }),
    [toasts, addToast, dismissToast],
  );

  // Auto-dismiss loop — check every 500ms; expired toasts are removed.
  useEffect(() => {
    if (toasts.length === 0) return;
    const interval = window.setInterval(() => {
      const cutoff = Date.now() - AUTO_DISMISS_MS;
      setToasts((prev) => prev.filter((t) => t.createdAt > cutoff));
    }, 500);
    return () => window.clearInterval(interval);
  }, [toasts.length]);

  return (
    <ToastContext.Provider value={value}>{children}</ToastContext.Provider>
  );
}

/**
 * Read the toast queue + actions. Throws if used outside a `<ToastProvider>`.
 */
export function useToasts(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToasts must be used inside a <ToastProvider>");
  }
  return ctx;
}
