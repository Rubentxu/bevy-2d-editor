import { useEffect, useState } from "react";
import {
  useGlobalSearch,
  type GlobalSearchResult,
} from "../hooks/useGlobalSearch";
import SearchResultRow from "./SearchResultRow";
import type { NavigationTarget } from "./CodeEditor";
import { sceneSwitch } from "../services/scenes";

const DEBOUNCE_MS = 150;

/**
 * Phase B — Global Search tab with actionable results (PR2 T2.5).
 *
 * A debounced input drives `useGlobalSearch().search`; the hook returns
 * ranked results from the in-memory index of scenes, scene assets, source
 * files, asset files, entities, and commands.
 *
 * Click handlers are fully wired (spec §3 `global-search`):
 * - scene         → switch to the scene
 * - entity        → focus the entity in the inspector (future)
 * - scene-asset   → open the asset
 * - source-file   → navigate to source file in code editor
 * - asset-file    → open asset file (external)
 * - command       → execute command
 */
interface SearchTabProps {
  /** Wired to App.tsx pendingNavigation → CodeEditor navigationTarget. */
  onSourceNavigate?: (target: NavigationTarget) => void;
}

export default function SearchTab({ onSourceNavigate }: SearchTabProps) {
  const [query, setQuery] = useState("");
  const [focusedIdx, setFocusedIdx] = useState(-1);
  const { results, loading, search, setCommandResults } = useGlobalSearch();

  // CRITICAL ISSUE 2: Initialize command results from the command palette.
  // This effect runs once on mount to populate command search results.
  useEffect(() => {
    const items = (window as any).__getCommandPaletteItems?.() ?? [];
    const commandResults: GlobalSearchResult[] = items.map(
      (item: { id: string; label: string; shortcut?: string; group: string }) => ({
        type: "command" as const,
        id: item.id,
        label: item.label,
        path: `${item.group} · ${item.shortcut ?? ""}`.trim(),
        onClick: () => (window as any).__executeCommand?.(item.id),
      }),
    );
    setCommandResults(commandResults);
  }, [setCommandResults]);

  useEffect(() => {
    const handle = setTimeout(() => {
      void search(query);
    }, DEBOUNCE_MS);
    return () => clearTimeout(handle);
  }, [query, search]);

  // Reset focused index when results change.
  useEffect(() => {
    setFocusedIdx(-1);
  }, [results]);

  const handleResultAction = async (result: GlobalSearchResult) => {
    switch (result.type) {
      case "scene":
        try {
          await sceneSwitch(result.id);
        } catch (e) {
          console.warn("[SearchTab] scene switch failed:", e);
        }
        break;

      case "entity":
        // T2.5: wire entity focus via window-exposed testing hook.
        if (result.entityId) {
          (window as any).__setSelectedEntityId?.(result.entityId);
        }
        break;

      case "scene-asset": {
        // T2.5: switch to asset-authoring mode before opening.
        // CRITICAL ISSUE 3: use App-owned useSceneAssets().open() via window hook
        // so React state (assetDoc, activeAssetId) is updated.
        (window as any).__setEditorMode?.("asset-authoring");
        try {
          await (window as any).__openSceneAssetFromSearch?.(result.id);
        } catch (e) {
          console.warn("[SearchTab] open scene asset failed:", e);
        }
        break;
      }

      case "source-file": {
        // T2.5: switch to code mode, then navigate to source file.
        (window as any).__setEditorMode?.("code");
        if (onSourceNavigate) {
          onSourceNavigate({ fileId: result.id, line: 1 });
        }
        break;
      }

      case "asset-file": {
        // Asset files are external resources — open in a new tab.
        // T2.5: result.path is now the stored file path (not a display string).
        const url = result.path.startsWith("http")
          ? result.path
          : `asset://opfs/${result.path}`;
        window.open(url, "_blank", "noopener");
        break;
      }

      case "command":
        if (result.onClick) result.onClick();
        break;
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusedIdx((i) => Math.min(i + 1, results.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusedIdx((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter" && focusedIdx >= 0) {
      e.preventDefault();
      void handleResultAction(results[focusedIdx]);
    }
  };

  return (
    <section
      className="search-tab"
      data-testid="bottom-tabpanel-search"
      aria-label="Global search"
      onKeyDown={handleKeyDown}
    >
      <input
        className="bottom-dock-search-input"
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search scenes, assets, source files…"
        aria-label="Global search"
        data-testid="global-search-input"
        autoFocus
      />
      {loading ? (
        <p className="bottom-dock-empty" data-testid="global-search-loading">
          Searching…
        </p>
      ) : query.length === 0 ? (
        <p className="bottom-dock-empty" data-testid="global-search-helper">
          Type to search scenes, scene assets, source files, asset files, entities, and commands.
        </p>
      ) : results.length === 0 ? (
        <p className="bottom-dock-empty" data-testid="global-search-empty">
          No results for "{query}".
        </p>
      ) : (
        <ul
          className="bottom-dock-results"
          data-testid="global-search-results"
          role="listbox"
          aria-label="Search results"
        >
          {results.map((result, idx) => (
            <SearchResultRow
              key={`${result.type}:${result.id}`}
              result={result}
              isFocused={idx === focusedIdx}
              onClick={() => void handleResultAction(result)}
              onActivate={() => void handleResultAction(result)}
              testId={`global-search-result-${result.type}-${result.id}`}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
