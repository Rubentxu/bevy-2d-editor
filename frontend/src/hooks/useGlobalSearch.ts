import { useCallback, useEffect, useRef, useState } from "react";
import { useScenes } from "./useScenes";
import { useSceneAssets } from "./useSceneAssets";
import { useCodeFiles } from "./useCodeFiles";
import { useAssetFiles } from "./useAssetFiles";

export type GlobalSearchResultType =
  | "scene"
  | "entity"
  | "scene-asset"
  | "source-file"
  | "asset-file"
  | "command";

export interface GlobalSearchResult {
  type: GlobalSearchResultType;
  id: string;
  label: string;
  path: string;
  /** StableId of the entity this result refers to (entity type only). */
  entityId?: string;
  /** Optional handler invoked when user clicks the result. */
  onClick?: () => void;
}

interface IndexSources {
  scenes: ReturnType<typeof useScenes>["scenes"];
  sceneAssets: ReturnType<typeof useSceneAssets>["entries"];
  sourceFiles: ReturnType<typeof useCodeFiles>["files"];
  assetFiles: ReturnType<typeof useAssetFiles>["files"];
}

const MAX_RESULTS = 50;

/**
 * v0.82 — Global search index (Phase B: entity + command implemented).
 *
 * Builds a flat, in-memory index from the existing React hooks:
 * - Scenes (`useScenes`)
 * - Scene Assets (`useSceneAssets`)
 * - Source files (`useCodeFiles`)
 * - Asset files (`useAssetFiles`)
 *
 * Entity search within the active scene is supported via the WASM engine's
 * `list_scene_entities` export when the engine is ready.
 * Command results are populated by the consumer (SearchTab / CommandPalette)
 * via the `commandResults` prop — this hook merges them into the result stream.
 *
 * Search is case-insensitive substring matching, with prefix hits ranked
 * above mid-string hits. Results are capped at `MAX_RESULTS` to keep the
 * list scannable in the bottom dock.
 */
export function useGlobalSearch() {
  const { scenes } = useScenes();
  const { entries: sceneAssets } = useSceneAssets();
  const { files: sourceFiles } = useCodeFiles();
  const { files: assetFiles } = useAssetFiles();

  const [results, setResults] = useState<GlobalSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  /** Consumer-supplied command results (e.g. from CommandPalette history). */
  const [commandResults, setCommandResults] = useState<GlobalSearchResult[]>([]);

  // Mirror the polled collections into a ref so `search` can read the
  // latest snapshot without depending on them in its useCallback dep array
  // (the underlying hooks poll every 500ms/5s, so a direct dep would
  // re-fire search continuously while the user types).
  const indexRef = useRef<IndexSources>({
    scenes,
    sceneAssets,
    sourceFiles,
    assetFiles,
  });
  useEffect(() => {
    indexRef.current = { scenes, sceneAssets, sourceFiles, assetFiles };
  }, [scenes, sceneAssets, sourceFiles, assetFiles]);

  const search = useCallback(
    async (query: string): Promise<GlobalSearchResult[]> => {
      const q = query.trim().toLowerCase();
      if (q.length < 1) {
        setResults([]);
        setLoading(false);
        return [];
      }
      setLoading(true);
      try {
        const {
          scenes: s,
          sceneAssets: a,
          sourceFiles: f,
          assetFiles: af,
        } = indexRef.current;
        const out: GlobalSearchResult[] = [];

        // Scenes — match by name
        for (const scene of s) {
          if (scene.name.toLowerCase().includes(q)) {
            out.push({
              type: "scene",
              id: scene.id,
              label: scene.name,
              path: `Scene · ${scene.id}`,
            });
          }
        }

        // Scene Assets — match by logical_path or role
        for (const asset of a) {
          const haystack = `${asset.logical_path} ${asset.role}`.toLowerCase();
          if (haystack.includes(q)) {
            out.push({
              type: "scene-asset",
              id: asset.asset_id,
              label: asset.logical_path,
              path: `Scene Asset · ${asset.role}`,
            });
          }
        }

        // Source Files — match by name or path
        for (const file of f) {
          const haystack = `${file.path} ${file.name}`.toLowerCase();
          if (haystack.includes(q)) {
            out.push({
              type: "source-file",
              id: file.id,
              label: file.name || file.path,
              path: file.path,
            });
          }
        }

        // Asset Files — match by name, mime, or kind
        for (const file of af) {
          const haystack =
            `${file.name} ${file.mime_type} ${file.kind}`.toLowerCase();
          if (haystack.includes(q)) {
            out.push({
              type: "asset-file",
              id: file.id,
              label: file.name,
              path: file.path,
            });
          }
        }

        // Entity search — WASM provides list_scene_entities for the active scene.
        // We only search within the currently active scene.
        const activeScene = s.find((sc) => sc.is_active);
        if (activeScene) {
          await searchEntitiesInScene(q, activeScene.id, out);
        }

        // Merge command results (already filtered by query via SearchTab)
        out.push(...commandResults.filter((r) => r.label.toLowerCase().includes(q)));

        // Sort: prefix matches before substring; then alphabetically.
        out.sort((a, b) => {
          const aPrefix = a.label.toLowerCase().startsWith(q) ? 0 : 1;
          const bPrefix = b.label.toLowerCase().startsWith(q) ? 0 : 1;
          if (aPrefix !== bPrefix) return aPrefix - bPrefix;
          return a.label.localeCompare(b.label);
        });

        const capped = out.slice(0, MAX_RESULTS);
        setResults(capped);
        return capped;
      } finally {
        setLoading(false);
      }
    },
    // `search` is intentionally stable — it reads from indexRef and commandResults
    // instead of depending on the polled collections directly.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [commandResults],
  );

  // Reset loading flag when the underlying index changes mid-flight so the
  // UI doesn't get stuck on "Searching…" if a stale promise resolves last.
  useEffect(() => {
    setLoading(false);
  }, [scenes, sceneAssets, sourceFiles, assetFiles]);

  return { results, loading, search, setCommandResults };
}

// ── Entity search helper ──────────────────────────────────────────────────────

interface SceneEntity {
  stable_id: string;
  local_id: string;
  name: string;
}

async function searchEntitiesInScene(
  query: string,
  sceneId: string,
  out: GlobalSearchResult[],
): Promise<void> {
  if (typeof window === "undefined") return;
  const fn = (window as any).list_scene_entities;
  if (typeof fn !== "function") return;
  try {
    const raw = fn(sceneId);
    const entities: SceneEntity[] =
      typeof raw === "string" ? JSON.parse(raw) : raw;
    for (const entity of entities) {
      if (entity.name.toLowerCase().includes(query)) {
        out.push({
          type: "entity",
          id: entity.stable_id,
          entityId: entity.stable_id,
          label: entity.name,
          path: `Entity · ${entity.local_id}`,
        });
      }
    }
  } catch {
    // WASM not ready or scene has no entities — skip silently.
  }
}
