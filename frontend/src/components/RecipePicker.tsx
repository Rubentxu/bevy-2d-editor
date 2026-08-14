/**
 * RecipePicker — recipe-first entry surface for Logic Workflow v2.
 *
 * Shown FIRST when entering logic mode instead of jumping straight to a blank graph.
 * Users can:
 *   - Pick a built-in or user-authored recipe to scaffold a new logic graph
 *   - Opt into "Start from blank graph" instead
 *
 * The component emits the selected recipe asset_id (or null for blank) via onSelect.
 */

import { useEffect, useState, useCallback } from "react";
import {
  listLogicGraphAssets,
  type LogicGraphCatalogEntry,
} from "../services/logic-graphs";

interface Props {
  /** Called when user picks a recipe asset_id, or null for "blank graph". */
  onSelect: (assetId: string | null) => void;
  /** Called when user explicitly requests to see the graph editor without a recipe. */
  onStartBlank?: () => void;
}

interface RecipeGroup {
  label: string;
  entries: LogicGraphCatalogEntry[];
}

export default function RecipePicker({ onSelect, onStartBlank }: Props) {
  const [groups, setGroups] = useState<RecipeGroup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const entries = await listLogicGraphAssets();
        // Group: built-in recipes first, then user graphs
        const builtin = entries.filter((e) => e.builtin);
        const user = entries.filter((e) => !e.builtin);
        const g: RecipeGroup[] = [];
        if (builtin.length)
          g.push({ label: "Built-in Recipes", entries: builtin });
        if (user.length) g.push({ label: "My Graphs", entries: user });
        setGroups(g);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const handlePick = useCallback(
    (assetId: string) => {
      onSelect(assetId);
    },
    [onSelect],
  );

  return (
    <div
      className="recipe-picker"
      data-testid="recipe-picker"
      style={{
        padding: 16,
        height: "100%",
        overflowY: "auto",
        background: "#fafafa",
      }}
    >
      <header className="recipe-picker-header">
        <h2 data-testid="recipe-picker-title">Choose a Recipe</h2>
        <p
          className="recipe-picker-subtitle"
          data-testid="recipe-picker-subtitle"
        >
          Pick a starting pattern or start from scratch
        </p>
      </header>

      {loading && (
        <div
          className="recipe-picker-loading"
          data-testid="recipe-picker-loading"
        >
          Loading recipes…
        </div>
      )}

      {error && (
        <div
          className="recipe-picker-error"
          data-testid="recipe-picker-error"
          role="alert"
        >
          {error}
        </div>
      )}

      {!loading && !error && groups.length === 0 && (
        <div className="recipe-picker-empty" data-testid="recipe-picker-empty">
          No recipes found. Start from blank graph.
        </div>
      )}

      {groups.map((group) => (
        <section
          key={group.label}
          className="recipe-group"
          data-testid={`recipe-group-${group.label}`}
        >
          <h3 className="recipe-group-label">{group.label}</h3>
          <ul className="recipe-list">
            {group.entries.map((entry) => (
              <li key={entry.asset_id} className="recipe-item">
                <button
                  type="button"
                  className="recipe-btn"
                  onClick={() => handlePick(entry.asset_id)}
                  data-testid={`recipe-btn-${entry.asset_id}`}
                  title={entry.logical_path}
                >
                  <span className="recipe-name">
                    {entry.logical_path.split("/").pop()}
                  </span>
                  <span className="recipe-path">{entry.logical_path}</span>
                  {entry.builtin && (
                    <span
                      className="recipe-builtin-badge"
                      data-testid={`recipe-builtin-${entry.asset_id}`}
                    >
                      built-in
                    </span>
                  )}
                </button>
              </li>
            ))}
          </ul>
        </section>
      ))}

      <div className="recipe-picker-blank" data-testid="recipe-picker-blank">
        <button
          type="button"
          className="recipe-blank-btn"
          onClick={() => {
            onStartBlank?.();
            onSelect(null);
          }}
          data-testid="recipe-blank-btn"
        >
          Start from blank graph
        </button>
      </div>
    </div>
  );
}
