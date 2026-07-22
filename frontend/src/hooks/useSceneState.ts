import { useCallback, useEffect, useState } from "react";
import { dispatchCommand, getSceneSnapshot } from "../engine-bridge";

export interface SceneDocument {
  version: string;
  scene_id: string;
  name: string;
  entities: Array<{
    id: string;
    name: string;
    parent?: string;
    components: Array<{ type_id: string; values: any }>;
  }>;
}

export interface DispatchResult {
  inverse?: any;
  snapshot?: SceneDocument;
  error?: string;
}

/**
 * React hook for scene state management.
 * Provides:
 * - `scene`: current SceneDocument or null
 * - `refresh()`: re-reads from WASM
 * - `dispatch(envelope)`: dispatches a command, refreshes state from response
 */
export function useSceneState() {
  const [scene, setScene] = useState<SceneDocument | null>(null);

  const refresh = useCallback(async () => {
    try {
      // Wait for engine to be ready (window.get_scene_snapshot may not be set yet)
      let attempts = 0;
      while (
        typeof (window as any).get_scene_snapshot !== "function" &&
        attempts < 50
      ) {
        await new Promise((r) => setTimeout(r, 100));
        attempts += 1;
      }
      const snap = await getSceneSnapshot();
      setScene(snap);
    } catch (e) {
      console.error("useSceneState.refresh failed:", e);
    }
  }, []);

  const dispatch = useCallback(
    async (envelope: object): Promise<DispatchResult> => {
      try {
        // Wait for engine ready
        let attempts = 0;
        while (
          typeof (window as any).dispatch_command !== "function" &&
          attempts < 50
        ) {
          await new Promise((r) => setTimeout(r, 100));
          attempts += 1;
        }
        const resultJson = await dispatchCommand(envelope);
        const parsed = JSON.parse(resultJson);
        if (parsed.snapshot) {
          setScene(parsed.snapshot);
        }
        return parsed;
      } catch (e) {
        const msg = String(e);
        console.error("useSceneState.dispatch failed:", e);
        return { error: msg };
      }
    },
    [],
  );

  useEffect(() => {
    refresh();
    // Poll periodically to catch external state changes (e.g., load_scene_json from outside React)
    const interval = setInterval(refresh, 500);
    return () => clearInterval(interval);
  }, [refresh]);

  return { scene, refresh, dispatch };
}
