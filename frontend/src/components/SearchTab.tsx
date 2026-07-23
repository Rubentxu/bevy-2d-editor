import { useEffect, useState } from "react";
import {
  useGlobalSearch,
  type GlobalSearchResult,
} from "../hooks/useGlobalSearch";

const TYPE_ICONS: Record<GlobalSearchResult["type"], string> = {
  scene: "🎬",
  entity: "🧩",
  "scene-asset": "🎨",
  "source-file": "📄",
  "asset-file": "🖼️",
  command: "⚙️",
};

const DEBOUNCE_MS = 150;

/**
 * v0.81 Tier 1 — Global Search tab.
 *
 * A debounced input drives `useGlobalSearch().search`; the hook returns
 * ranked results from the in-memory index of scenes, scene assets, source
 * files, and asset files. Click handlers are reserved for future tiers
 * (v0.82 navigation wiring) — the `onClick` field is on each result but
 * is currently undefined for all indexed types.
 */
export default function SearchTab() {
  const [query, setQuery] = useState("");
  const { results, loading, search } = useGlobalSearch();

  useEffect(() => {
    // Debounce so rapid typing doesn't thrash the index on every keystroke.
    const handle = setTimeout(() => {
      void search(query);
    }, DEBOUNCE_MS);
    return () => clearTimeout(handle);
  }, [query, search]);

  return (
    <section
      className="search-tab"
      data-testid="bottom-tabpanel-search"
      aria-label="Global search"
    >
      <input
        className="bottom-dock-search-input"
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="Search scenes, assets, source files…"
        aria-label="Global search"
        data-testid="global-search-input"
      />
      {loading ? (
        <p className="bottom-dock-empty" data-testid="global-search-loading">
          Searching…
        </p>
      ) : query.length === 0 ? (
        <p className="bottom-dock-empty" data-testid="global-search-helper">
          Type to search scenes, scene assets, source files, and asset files.
        </p>
      ) : results.length === 0 ? (
        <p className="bottom-dock-empty" data-testid="global-search-empty">
          No results for "{query}".
        </p>
      ) : (
        <ul className="bottom-dock-results" data-testid="global-search-results">
          {results.map((result) => (
            <li
              key={`${result.type}:${result.id}`}
              className="bottom-dock-result-item"
              data-testid={`global-search-result-${result.type}-${result.id}`}
              onClick={result.onClick}
              role={result.onClick ? "button" : undefined}
            >
              <span className="bottom-dock-result-icon" aria-hidden="true">
                {TYPE_ICONS[result.type]}
              </span>
              <span className="bottom-dock-result-label">{result.label}</span>
              <small className="bottom-dock-result-path">{result.path}</small>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
