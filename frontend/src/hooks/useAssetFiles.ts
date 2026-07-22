import { useEffect, useState, useCallback } from "react";
import {
  AssetFile,
  listAssetFiles,
  importAssetFile,
  deleteAssetFile,
} from "../services/asset-files";

/**
 * React hook for asset file state and operations.
 *
 * Manages:
 * - File list (all asset files in the `resources/` OPFS directory)
 * - Loading and error state
 * - Import and delete operations
 *
 * Polling refresh every 5s to pick up external OPFS changes.
 */
export function useAssetFiles() {
  const [files, setFiles] = useState<AssetFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const list = await listAssetFiles();
      setFiles(list);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load asset files");
    }
  }, []);

  useEffect(() => {
    setLoading(true);
    refresh().finally(() => setLoading(false));

    const interval = setInterval(refresh, 5000);
    return () => clearInterval(interval);
  }, [refresh]);

  const importFile = useCallback(
    async (file: File): Promise<AssetFile> => {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const asset = await importAssetFile(file.name, file.type, bytes);
      await refresh();
      return asset;
    },
    [refresh],
  );

  const removeFile = useCallback(
    async (id: string) => {
      await deleteAssetFile(id);
      await refresh();
    },
    [refresh],
  );

  return {
    files,
    loading,
    error,
    refresh,
    importFile,
    deleteFile: removeFile,
  };
}
