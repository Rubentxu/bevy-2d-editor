/**
 * SearchResultRow — shared row component for Global Search and Command Palette.
 * Renders a single search result with type icon, label, path, and an
 * optional action affordance.
 *
 * Used by:
 * - SearchTab (global search results)
 * - CommandPalette (command history / filtered commands)
 */

import type { GlobalSearchResult } from "../hooks/useGlobalSearch";

const TYPE_ICONS: Record<GlobalSearchResult["type"], string> = {
  scene: "🎬",
  entity: "🧩",
  "scene-asset": "🎨",
  "source-file": "📄",
  "asset-file": "🖼️",
  command: "⚙️",
};

interface SearchResultRowProps {
  result: GlobalSearchResult;
  isFocused?: boolean;
  /** Called when the row is clicked. */
  onClick?: (result: GlobalSearchResult) => void;
  /** Called when the user activates the row (Enter key / double-click). */
  onActivate?: (result: GlobalSearchResult) => void;
  testId?: string;
}

export default function SearchResultRow({
  result,
  isFocused = false,
  onClick,
  onActivate,
  testId,
}: SearchResultRowProps) {
  const icon = TYPE_ICONS[result.type] ?? "📌";

  const handleClick = () => onClick?.(result);
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onActivate?.(result);
    }
  };

  return (
    <li
      className={`search-result-row${isFocused ? " search-result-row--focused" : ""}`}
      data-testid={testId ?? `search-result-row-${result.type}-${result.id}`}
      role={onClick ? "button" : undefined}
      tabIndex={onClick ? 0 : undefined}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      aria-selected={isFocused}
    >
      <span className="search-result-row__icon" aria-hidden="true">
        {icon}
      </span>
      <span className="search-result-row__body">
        <span className="search-result-row__label">{result.label}</span>
        <span className="search-result-row__path">{result.path}</span>
      </span>
      {result.type === "command" && (
        <span className="search-result-row__badge">command</span>
      )}
    </li>
  );
}
