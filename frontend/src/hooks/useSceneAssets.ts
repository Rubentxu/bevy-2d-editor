import { useEffect, useState, useCallback } from "react";
import {
  SceneAssetCatalogEntry,
  AssetLogState,
  SceneAssetDocument,
  SceneInstance,
  listSceneAssets,
  getSceneAssetCatalogJson,
  openSceneAsset,
  closeSceneAsset,
  getAssetDocumentJson,
  dispatchAssetCommand,
  undoAsset,
  redoAsset,
  getAssetLogState,
  saveSceneAsset,
  createSceneAsset,
  renameSceneAsset,
  duplicateSceneAsset,
  deleteSceneAsset,
  placeSceneInstance,
  removeSceneInstance,
  replaceSceneInstanceAsset,
  getSceneInstances,
} from "../services/scene-assets";

const DEFAULT_ENTRIES: SceneAssetCatalogEntry[] = [];
const DEFAULT_DOC: SceneAssetDocument | null = null;
const DEFAULT_LOG_STATE: AssetLogState = {
  size: 0,
  can_undo: false,
  can_redo: false,
  cursor: 0,
  dirty: false,
};
const DEFAULT_INSTANCES: Record<string, SceneInstance> = {};

/**
 * React hook for Scene Asset state and operations.
 *
 * Manages:
 * - Catalog entries (list of all Scene Assets)
 * - Current asset document (when one is open)
 * - Operation log state (for undo/redo/save dirty flag)
 * - 500ms polling refresh for catalog
 */
export function useSceneAssets() {
  const [entries, setEntries] =
    useState<SceneAssetCatalogEntry[]>(DEFAULT_ENTRIES);
  const [assetDoc, setAssetDoc] = useState<SceneAssetDocument | null>(
    DEFAULT_DOC,
  );
  const [activeAssetId, setActiveAssetId] = useState<string | null>(null);
  const [logState, setLogState] = useState<AssetLogState>(DEFAULT_LOG_STATE);
  const [instances, setInstances] =
    useState<Record<string, SceneInstance>>(DEFAULT_INSTANCES);
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  /**
   * Refresh the catalog entries from the backend.
   */
  const refreshCatalog = useCallback(async () => {
    try {
      const catalog = await getSceneAssetCatalogJson();
      setEntries(catalog);
    } catch (e) {
      console.error("useSceneAssets: refreshCatalog failed:", e);
    }
  }, []);

  /**
   * Refresh the asset log state.
   */
  const refreshLogState = useCallback(async () => {
    try {
      const state = await getAssetLogState();
      setLogState(state);
    } catch (e) {
      console.error("useSceneAssets: refreshLogState failed:", e);
    }
  }, []);

  /**
   * Refresh the scene instances from the backend.
   *
   * Quietly tolerates the "No scene loaded" condition that occurs on every
   * fresh page load before the user has loaded a scene — the polling effect
   * fires before any user action. Other failures remain as `console.error`.
   */
  const refreshInstances = useCallback(async () => {
    try {
      const inst = await getSceneInstances();
      setInstances(inst);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      if (msg.includes("No scene loaded")) {
        console.debug(
          "useSceneAssets: refreshInstances skipped (no scene loaded yet)",
        );
      } else {
        console.error("useSceneAssets: refreshInstances failed:", e);
      }
    }
  }, []);

  /**
   * Full refresh - catalog + log state + instances.
   */
  const refresh = useCallback(async () => {
    await Promise.all([
      refreshCatalog(),
      refreshLogState(),
      refreshInstances(),
    ]);
  }, [refreshCatalog, refreshLogState, refreshInstances]);

  /**
   * Open a Scene Asset for editing.
   * @param assetId - The asset's stable ID
   */
  const open = useCallback(
    async (assetId: string) => {
      try {
        await openSceneAsset(assetId);
        const docJson = await getAssetDocumentJson();
        const doc = JSON.parse(docJson) as SceneAssetDocument;
        setAssetDoc(doc);
        setActiveAssetId(assetId);
        await refreshLogState();
      } catch (e) {
        console.error("useSceneAssets: open failed:", e);
        throw e;
      }
    },
    [refreshLogState],
  );

  /**
   * Close the current Scene Asset (no save).
   */
  const close = useCallback(() => {
    closeSceneAsset();
    setAssetDoc(null);
    setActiveAssetId(null);
    setLogState(DEFAULT_LOG_STATE);
  }, []);

  /**
   * Dispatch an AssetCommand to the open asset.
   * @param command - The AssetCommand (without envelope)
   * @returns The result from the backend
   */
  const dispatch = useCallback(
    async (command: object): Promise<string> => {
      const envelope = {
        command,
        metadata: { authorship: "user", timestamp: Date.now() },
      };
      const result = await dispatchAssetCommand(JSON.stringify(envelope));
      // Refresh doc and log state after dispatch
      const docJson = await getAssetDocumentJson();
      setAssetDoc(JSON.parse(docJson));
      await refreshLogState();
      return result;
    },
    [refreshLogState],
  );

  /**
   * Undo the last asset command.
   */
  const undo = useCallback(async () => {
    await undoAsset();
    const docJson = await getAssetDocumentJson();
    setAssetDoc(JSON.parse(docJson));
    await refreshLogState();
  }, [refreshLogState]);

  /**
   * Redo the next asset command.
   */
  const redo = useCallback(async () => {
    await redoAsset();
    const docJson = await getAssetDocumentJson();
    setAssetDoc(JSON.parse(docJson));
    await refreshLogState();
  }, [refreshLogState]);

  /**
   * Save the current asset (body-first, then catalog).
   */
  const save = useCallback(async () => {
    await saveSceneAsset();
    await refreshLogState();
  }, [refreshLogState]);

  // ── CRUD operations ────────────────────────────────────────────────────────

  /**
   * Create a new Scene Asset.
   */
  const create = useCallback(
    async (name: string, role: string) => {
      await createSceneAsset(name, role);
      await refreshCatalog();
    },
    [refreshCatalog],
  );

  /**
   * Rename a Scene Asset.
   */
  const rename = useCallback(
    async (assetId: string, newPath: string) => {
      await renameSceneAsset(assetId, newPath);
      await refreshCatalog();
    },
    [refreshCatalog],
  );

  /**
   * Duplicate a Scene Asset.
   * NOTE: 1-arity (assetId only) per constraint C-1.
   */
  const duplicate = useCallback(
    async (assetId: string) => {
      await duplicateSceneAsset(assetId);
      await refreshCatalog();
    },
    [refreshCatalog],
  );

  /**
   * Delete a Scene Asset.
   */
  const deleteAsset = useCallback(
    async (assetId: string) => {
      await deleteSceneAsset(assetId);
      // If we had this asset open, close it
      if (assetId === activeAssetId) {
        close();
      }
      await refreshCatalog();
    },
    [activeAssetId, close, refreshCatalog],
  );

  // ── Scene Instance operations (PR3) ──────────────────────────────────────────

  /**
   * Place a Scene Asset as a new Scene Instance in the active scene.
   * @param assetId - The asset's stable ID from the catalog
   * @param translation - Optional translation {x, y}
   */
  const placeInstance = useCallback(
    async (assetId: string, translation?: { x: number; y: number }) => {
      await placeSceneInstance(assetId, translation);
      // Refresh instances after placing
      await refreshInstances();
    },
    [refreshInstances],
  );

  /**
   * Remove a Scene Instance from the active scene.
   * @param instanceId - The instance's stable ID
   */
  const removeInstance = useCallback(
    async (instanceId: string) => {
      await removeSceneInstance(instanceId);
      // Refresh instances after removing
      await refreshInstances();
    },
    [refreshInstances],
  );

  /**
   * Replace the asset of an existing Scene Instance.
   * @param instanceId - The instance's stable ID
   * @param newAssetId - The new asset's stable ID
   */
  const replaceInstanceAsset = useCallback(
    async (instanceId: string, newAssetId: string) => {
      await replaceSceneInstanceAsset(instanceId, newAssetId);
      // Refresh instances after replacing
      await refreshInstances();
    },
    [refreshInstances],
  );

  /**
   * Force a refresh trigger (increments counter to force re-render).
   */
  const forceRefresh = useCallback(() => {
    setRefreshTrigger((n) => n + 1);
  }, []);

  // Poll for catalog + log state + instances every 500ms
  useEffect(() => {
    refreshCatalog();
    refreshLogState();
    refreshInstances();

    const interval = setInterval(() => {
      refreshCatalog();
      refreshLogState();
      refreshInstances();
    }, 500);

    return () => clearInterval(interval);
  }, [refresh, refreshCatalog, refreshLogState, refreshInstances]);

  return {
    // Catalog state
    entries,
    refreshCatalog,

    // Active asset state
    assetDoc,
    activeAssetId,

    // Log state
    logState,
    dirty: logState.dirty,

    // Asset operations
    open,
    close,
    dispatch,
    undo,
    redo,
    save,

    // CRUD
    create,
    rename,
    duplicate,
    deleteAsset: deleteAsset,

    // Scene Instances (PR3)
    instances,
    refreshInstances,
    placeInstance,
    removeInstance,
    replaceInstanceAsset,

    // Force refresh
    forceRefresh,
  };
}
