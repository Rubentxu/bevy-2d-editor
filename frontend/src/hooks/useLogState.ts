import { useEffect, useState } from "react";
import { getLogState } from "../engine-bridge";

export interface LogState {
  size: number;
  can_undo: boolean;
  can_redo: boolean;
  cursor: number;
}

const DEFAULT_LOG_STATE: LogState = {
  size: 0,
  can_undo: false,
  can_redo: false,
  cursor: -1,
};

/**
 * React hook for operation log state (can_undo, can_redo, size).
 * Polls every 500ms. UI components use this to enable/disable undo/redo buttons.
 */
export function useLogState(): LogState {
  const [state, setState] = useState<LogState>(DEFAULT_LOG_STATE);

  useEffect(() => {
    const update = async () => {
      try {
        // Wait for engine to be ready
        let attempts = 0;
        while (typeof (window as any).get_log_state !== "function" && attempts < 50) {
          await new Promise((r) => setTimeout(r, 100));
          attempts += 1;
        }
        const s = await getLogState();
        setState(s);
      } catch (e) {
        console.error("useLogState failed:", e);
      }
    };
    update();
    const interval = setInterval(update, 500);
    return () => clearInterval(interval);
  }, []);

  return state;
}