/**
 * useSceneComponents — Hito 4 Order 7 PR2.
 *
 * React hook wrapping the SceneComponent service. Provides:
 * - list(): refreshes the in-memory list of SceneComponent schemas
 * - create(schema): creates a new SceneComponent schema via WASM
 * - bind(typeId, sceneAssetId | null): updates an existing schema's binding
 *
 * Cache strategy: the hook maintains an in-memory list and invalidates it
 * on every mutation. Consumers can opt to disable the auto-fetch via
 * `enabled: false` if they want to call list() on demand.
 */

import { useCallback, useEffect, useState } from "react";
import type { ComponentSchema } from "../types/schema";
import {
  bindSceneToSchema,
  createSceneComponent,
  listSceneComponentSchemas,
} from "../services/scene-components";

interface State {
  schemas: ComponentSchema[];
  loading: boolean;
  error: string | null;
}

export interface UseSceneComponentsResult extends State {
  refresh: () => Promise<void>;
  create: (schema: ComponentSchema) => Promise<string>;
  bind: (typeId: string, sceneAssetId: string | null) => Promise<void>;
}

export function useSceneComponents(
  options: { enabled?: boolean } = {}
): UseSceneComponentsResult {
  const { enabled = true } = options;
  const [state, setState] = useState<State>({
    schemas: [],
    loading: false,
    error: null,
  });

  const refresh = useCallback(async () => {
    setState((s) => ({ ...s, loading: true, error: null }));
    try {
      const schemas = await listSceneComponentSchemas();
      setState({ schemas, loading: false, error: null });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setState((s) => ({ ...s, loading: false, error: msg }));
    }
  }, []);

  const create = useCallback(
    async (schema: ComponentSchema): Promise<string> => {
      const typeId = await createSceneComponent(schema);
      await refresh();
      return typeId;
    },
    [refresh]
  );

  const bind = useCallback(
    async (typeId: string, sceneAssetId: string | null): Promise<void> => {
      await bindSceneToSchema(typeId, sceneAssetId);
      await refresh();
    },
    [refresh]
  );

  useEffect(() => {
    if (enabled) {
      void refresh();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled]);

  return { ...state, refresh, create, bind };
}
