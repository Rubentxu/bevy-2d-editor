/**
 * useHotReloadStatus — React hook for hot-reload UI state.
 *
 * Subscribes to hot-reload events to track:
 * - lastReloadedAt: timestamp of the most recent reload
 * - inFlightSaves: number of pending save operations
 *
 * refresh() triggers a full reload after all in-flight saves settle.
 */

import { useEffect, useState, useCallback, useRef } from "react";
import { subscribe, inFlightSaveCounter } from "../services/hot-reload";
import { forceReload } from "../engine-bridge";

export interface HotReloadStatus {
  lastReloadedAt: Date | null;
  inFlightSaves: number;
  refresh: () => void;
}

/**
 * Hook tracking hot-reload state for UI binding.
 *
 * - Subscribes to both `hot-reload-source` and `hot-reload-asset` events.
 * - Updates `lastReloadedAt` when either event fires.
 * - Polls `inFlightSaveCounter.value` every 100ms for reactive updates.
 * - `refresh()` waits for in-flight saves to settle then calls `forceReload()`.
 */
export function useHotReloadStatus(): HotReloadStatus {
  const [lastReloadedAt, setLastReloadedAt] = useState<Date | null>(null);
  const [inFlightSaves, setInFlightSaves] = useState(0);
  const refreshRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    // Subscribe to source reload events
    const unsubSource = subscribe("hot-reload-source", () => {
      setLastReloadedAt(new Date());
    });

    // Subscribe to asset reload events
    const unsubAsset = subscribe("hot-reload-asset", () => {
      setLastReloadedAt(new Date());
    });

    // Poll in-flight save counter every 100ms
    const poll = setInterval(() => {
      setInFlightSaves(inFlightSaveCounter.value);
    }, 100);

    return () => {
      unsubSource();
      unsubAsset();
      clearInterval(poll);
    };
  }, []);

  const refresh = useCallback(async () => {
    // Wait until in-flight saves settle
    while (inFlightSaveCounter.value > 0) {
      await new Promise((r) => setTimeout(r, 100));
    }
    forceReload();
    setLastReloadedAt(new Date());
  }, []);

  refreshRef.current = refresh;

  return { lastReloadedAt, inFlightSaves, refresh };
}
