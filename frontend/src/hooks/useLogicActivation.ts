/**
 * useLogicActivation — typed hook for the Logic Graph log state from WASM.
 *
 * Provides a polling interface to the `window.get_logic_log_state()` WASM bridge,
 * returning a typed `LogicLogState`. Falls back to `null` gracefully when WASM
 * is not yet ready (matching the pattern in useLogicGraph.ts:147-155).
 *
 * PR4 correction: replaces the inline `(window as any).get_logic_log_state()` cast
 * in RuntimePreviewInspector.tsx with a typed, tested abstraction (design.md:131).
 *
 * NOTE: The WASM function `get_logic_log_state()` returns undo/redo metadata
 * (size, can_undo, can_redo, cursor) — not an entries array. The hook types
 * reflect the actual WASM return shape.
 */

import { useCallback, useEffect, useState } from "react";

export interface LogicLogState {
  size: number;
  can_undo: boolean;
  can_redo: boolean;
  cursor: number;
}

interface UseLogicActivationOptions {
  /** Polling interval in milliseconds (default: 1000). */
  pollIntervalMs?: number;
}

/**
 * Hook that polls `window.get_logic_log_state()` and returns a typed snapshot.
 *
 * Returns `{ snapshot: null, refresh }` when WASM is not yet ready.
 * After a successful poll, `snapshot` is a `LogicLogState`.
 *
 * @example
 * const { snapshot, refresh } = useLogicActivation({ pollIntervalMs: 2000 });
 * if (snapshot) {
 *   console.log("Logic log size:", snapshot.size, "can_undo:", snapshot.can_undo);
 * }
 */
export function useLogicActivation(options: UseLogicActivationOptions = {}): {
  snapshot: LogicLogState | null;
  refresh: () => Promise<void>;
} {
  const { pollIntervalMs = 1000 } = options;

  const [snapshot, setSnapshot] = useState<LogicLogState | null>(null);

  const refresh = useCallback(async () => {
    try {
      const stateJson = await (window as any).get_logic_log_state();
      setSnapshot(JSON.parse(stateJson) as LogicLogState);
    } catch {
      // WASM not ready or function not exposed — leave snapshot as null.
      // This matches the graceful-fallback pattern in useLogicGraph.ts:147-155.
      setSnapshot(null);
    }
  }, []);

  useEffect(() => {
    refresh();
    const id = setInterval(refresh, pollIntervalMs);
    return () => clearInterval(id);
  }, [refresh, pollIntervalMs]);

  return { snapshot, refresh };
}
