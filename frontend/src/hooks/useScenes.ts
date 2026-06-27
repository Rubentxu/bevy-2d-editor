import { useEffect, useState, useCallback } from "react";
import { listScenesExtended, getCurrentSceneId } from "../services/scenes";

export interface SceneInfo {
  id: string;
  name: string;
  is_dirty: boolean;
  is_active: boolean;
}

const DEFAULT_SCENES: SceneInfo[] = [];

/**
 * React hook for multi-scene state.
 * Polls list_scenes_extended() every 500ms (same cadence as useLogState).
 * Returns { scenes, currentId, refresh }.
 */
export function useScenes() {
  const [scenes, setScenes] = useState<SceneInfo[]>(DEFAULT_SCENES);
  const [currentId, setCurrentId] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [sceneList, id] = await Promise.all([
        listScenesExtended(),
        getCurrentSceneId(),
      ]);
      setScenes(sceneList);
      setCurrentId(id);
    } catch (e) {
      console.error("useScenes refresh failed:", e);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 500);
    return () => clearInterval(interval);
  }, [refresh]);

  return { scenes, currentId, refresh };
}
