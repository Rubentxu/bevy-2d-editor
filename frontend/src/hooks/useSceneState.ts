import { useCallback, useEffect, useState } from "react";
import { getEditorGateway } from "../services/EditorGateway";

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
 * React hook for scene state management. Routes through the typed
 * `EditorGateway` so the polling + window-globals dance is hidden
 * behind a single `whenReady()` promise. The 500 ms refresh poll
 * remains for backwards compatibility with external `load_scene_json`
 * mutations.
 */
export function useSceneState() {
  const [scene, setScene] = useState<SceneDocument | null>(null);
  const gateway = getEditorGateway();

  const refresh = useCallback(async () => {
    try {
      const result = await gateway.getSceneSnapshot();
      if (result.ok) {
        setScene((result.value as SceneDocument | null) ?? null);
      } else {
        console.error("useSceneState.refresh failed:", result.error);
      }
    } catch (e) {
      console.error("useSceneState.refresh failed:", e);
    }
  }, [gateway]);

  const dispatch = useCallback(
    async (envelope: object): Promise<DispatchResult> => {
      try {
        const parsed = await gateway.dispatchCommand(envelope);
        if (parsed.snapshot) {
          setScene(parsed.snapshot as SceneDocument);
        }
        return {
          inverse: parsed.inverse,
          snapshot: parsed.snapshot as SceneDocument | undefined,
          error: parsed.error,
        };
      } catch (e) {
        const msg = String(e);
        console.error("useSceneState.dispatch failed:", e);
        return { error: msg };
      }
    },
    [gateway],
  );

  useEffect(() => {
    refresh();
    // Poll periodically to catch external state changes (e.g., load_scene_json from outside React)
    const interval = setInterval(refresh, 500);
    return () => clearInterval(interval);
  }, [refresh]);

  return { scene, refresh, dispatch };
}
