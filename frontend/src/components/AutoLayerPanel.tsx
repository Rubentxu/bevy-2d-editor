import React, { useState, useEffect, useCallback } from "react";
import {
  type AutoRule,
  type Pattern3x3,
  type PatternCell,
  type TileRefPayload,
  regenerateAutoLayer,
  addAutoRule,
  updateAutoRule,
  removeAutoRule,
} from "../services/autoLayer";
import { listTilesets, type TilesetMetadata } from "../services/tilesets";
import { type AutoLayerPayload } from "../services/scene-assets";
import { useAutoLayerStale } from "../hooks/useAutoLayerStale";

interface Props {
  /** The AutoLayer being edited. */
  layer: AutoLayerPayload;
  /** Logical path of the scene asset this layer belongs to. */
  assetRef: string;
  /** Called when the layer is regenerated (cache updated). */
  onRegenerate?: () => void;
}

/** Index positions for the 8 surrounding cells in a 3x3 grid. */
const SURROUNDING_INDICES: [number, number][] = [
  [0, 0],
  [0, 1],
  [0, 2],
  [1, 0],
  [1, 2],
  [2, 0],
  [2, 1],
  [2, 2],
];

/** Display labels for each surrounding cell position. */
const CELL_LABELS: Record<string, string> = {
  "0,0": "TL",
  "0,1": "T",
  "0,2": "TR",
  "1,0": "L",
  "1,2": "R",
  "2,0": "BL",
  "2,1": "B",
  "2,2": "BR",
};

function buildDefaultPattern(): Pattern3x3 {
  return [
    ["any" as PatternCell, "any" as PatternCell, "any" as PatternCell],
    ["any" as PatternCell, "any" as PatternCell, "any" as PatternCell],
    ["any" as PatternCell, "any" as PatternCell, "any" as PatternCell],
  ];
}

export const AutoLayerPanel: React.FC<Props> = ({
  layer,
  assetRef,
  onRegenerate,
}) => {
  const [rules, setRules] = useState<AutoRule[]>(layer.rules ?? []);
  const [selectedRuleIndex, setSelectedRuleIndex] = useState<number>(0);
  const [tilesets, setTilesets] = useState<TilesetMetadata[]>([]);
  const [regenerating, setRegenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [stale, setStale, refreshStale] = useAutoLayerStale(assetRef, layer.id);

  // Current rule being edited
  const currentRule = rules[selectedRuleIndex] ?? {
    pattern: buildDefaultPattern(),
    output: [],
    chance: undefined,
  };

  // Load tilesets for the tile picker
  useEffect(() => {
    listTilesets().then(setTilesets).catch(console.error);
  }, []);

  const handleCellChange = useCallback(
    (row: number, col: number, value: PatternCell) => {
      if (row === 1 && col === 1) return; // center is always ignored
      const newPattern: Pattern3x3 = [
        [
          row === 0 && col === 0 ? value : currentRule.pattern[0][0],
          row === 0 && col === 1 ? value : currentRule.pattern[0][1],
          row === 0 && col === 2 ? value : currentRule.pattern[0][2],
        ],
        [
          row === 1 && col === 0 ? value : currentRule.pattern[1][0],
          currentRule.pattern[1][1], // center is always ignored
          row === 1 && col === 2 ? value : currentRule.pattern[1][2],
        ],
        [
          row === 2 && col === 0 ? value : currentRule.pattern[2][0],
          row === 2 && col === 1 ? value : currentRule.pattern[2][1],
          row === 2 && col === 2 ? value : currentRule.pattern[2][2],
        ],
      ];
      const updated = [...rules];
      updated[selectedRuleIndex] = { ...currentRule, pattern: newPattern };
      setRules(updated);
      persistRuleUpdate(selectedRuleIndex, updated[selectedRuleIndex]);
    },
    [currentRule, rules, selectedRuleIndex],
  );

  const handleOutputTileChange = useCallback(
    (tilesetId: string, localIndex: number) => {
      const newOutput: TileRefPayload[] = [
        { tileset_id: tilesetId, local_index: localIndex },
      ];
      const updated = [...rules];
      updated[selectedRuleIndex] = { ...currentRule, output: newOutput };
      setRules(updated);
      persistRuleUpdate(selectedRuleIndex, updated[selectedRuleIndex]);
    },
    [currentRule, rules, selectedRuleIndex],
  );

  const handleChanceChange = useCallback(
    (value: number) => {
      const updated = [...rules];
      updated[selectedRuleIndex] = {
        ...currentRule,
        chance: value === 100 ? undefined : value / 100,
      };
      setRules(updated);
      persistRuleUpdate(selectedRuleIndex, updated[selectedRuleIndex]);
    },
    [currentRule, rules, selectedRuleIndex],
  );

  const handleAddRule = useCallback(async () => {
    const newRule: AutoRule = {
      pattern: buildDefaultPattern(),
      output: [],
      chance: undefined,
    };
    try {
      await addAutoRule(assetRef, layer.id, newRule);
      const updated = [...rules, newRule];
      setRules(updated);
      setSelectedRuleIndex(updated.length - 1);
    } catch (e) {
      setError(`Failed to add rule: ${e}`);
    }
  }, [assetRef, layer.id, rules]);

  const handleRemoveRule = useCallback(async () => {
    if (rules.length <= 1) return;
    try {
      await removeAutoRule(assetRef, layer.id, selectedRuleIndex);
      const updated = rules.filter((_, i) => i !== selectedRuleIndex);
      setRules(updated);
      setSelectedRuleIndex(Math.min(selectedRuleIndex, updated.length - 1));
    } catch (e) {
      setError(`Failed to remove rule: ${e}`);
    }
  }, [assetRef, layer.id, rules, selectedRuleIndex]);

  const handleRegenerate = useCallback(async () => {
    setRegenerating(true);
    setError(null);
    try {
      await regenerateAutoLayer(assetRef, layer.id);
      setStale(false);
      onRegenerate?.();
    } catch (e) {
      setError(`Regeneration failed: ${e}`);
    } finally {
      setRegenerating(false);
    }
  }, [assetRef, layer.id, onRegenerate]);

  const persistRuleUpdate = async (index: number, rule: AutoRule) => {
    try {
      if (index < rules.length) {
        await updateAutoRule(assetRef, layer.id, index, rule);
      }
    } catch (e) {
      setError(`Failed to update rule: ${e}`);
    }
  };

  const currentOutput = currentRule.output[0];
  const selectedTileset = tilesets.find(
    (ts) => ts.id === (currentOutput?.tileset_id ?? layer.tileset_id),
  );
  const tilesetOptions = selectedTileset
    ? Array.from(
        {
          length:
            Math.floor(
              (selectedTileset.columns *
                selectedTileset.tile_height *
                selectedTileset.spacing) /
                selectedTileset.tile_width,
            ) ?? 256,
        },
        (_, i) => i,
      )
    : [];

  return (
    <div className="auto-layer-panel">
      <h3>Auto Layer: {layer.name}</h3>

      {/* Stale warning banner */}
      {stale && (
        <div className="stale-banner">
          <span>
            Layer is stale — source tiles changed. Regenerate to update.
          </span>
          <button onClick={handleRegenerate} disabled={regenerating}>
            {regenerating ? "Regenerating…" : "Regenerate"}
          </button>
        </div>
      )}

      {error && <div className="auto-layer-error">{error}</div>}

      {/* 3x3 Pattern Grid */}
      <div className="pattern-grid">
        {([0, 1, 2] as const).map((row) =>
          ([0, 1, 2] as const).map((col) => {
            const isCenter = row === 1 && col === 1;
            const value = currentRule.pattern[row][col];
            return (
              <div
                key={`${row}-${col}`}
                className={`pattern-cell ${isCenter ? "center" : ""} ${isCenter ? "center-disabled" : ""}`}
                title={
                  isCenter ? "Center (ignored)" : CELL_LABELS[`${row},${col}`]
                }
              >
                {isCenter ? (
                  <span className="center-label">—</span>
                ) : (
                  <select
                    value={value}
                    onChange={(e) =>
                      handleCellChange(row, col, e.target.value as PatternCell)
                    }
                  >
                    <option value="filled">Filled</option>
                    <option value="empty">Empty</option>
                    <option value="any">Any</option>
                  </select>
                )}
              </div>
            );
          }),
        )}
      </div>

      {/* Output Tile Picker */}
      <div className="tile-picker">
        <label>Output Tile</label>
        <select
          value={currentOutput?.tileset_id ?? layer.tileset_id}
          onChange={(e) => {
            const ts = tilesets.find((t) => t.id === e.target.value);
            if (ts && currentOutput) {
              handleOutputTileChange(e.target.value, currentOutput.local_index);
            }
          }}
        >
          <option value="">Select tileset…</option>
          {tilesets.map((ts) => (
            <option key={ts.id} value={ts.id}>
              {ts.name}
            </option>
          ))}
        </select>
        {selectedTileset && (
          <select
            value={currentOutput?.local_index ?? 0}
            onChange={(e) =>
              handleOutputTileChange(
                selectedTileset.id,
                parseInt(e.target.value, 10),
              )
            }
          >
            {tilesetOptions.slice(0, 256).map((i) => (
              <option key={i} value={i}>
                Tile {i}
              </option>
            ))}
          </select>
        )}
      </div>

      {/* Chance Slider */}
      <div className="chance-slider">
        <label>
          Chance:{" "}
          {currentRule.chance !== undefined
            ? `${Math.round((currentRule.chance ?? 0) * 100)}%`
            : "Always"}
        </label>
        <input
          type="range"
          min="0"
          max="100"
          value={
            currentRule.chance !== undefined
              ? Math.round((currentRule.chance ?? 0) * 100)
              : 100
          }
          onChange={(e) => handleChanceChange(parseInt(e.target.value, 10))}
        />
      </div>

      {/* Rule List */}
      <div className="rule-list">
        <label>Rules ({rules.length})</label>
        <ul>
          {rules.map((rule, i) => (
            <li
              key={i}
              className={i === selectedRuleIndex ? "selected" : ""}
              onClick={() => setSelectedRuleIndex(i)}
            >
              Rule {i + 1}
            </li>
          ))}
        </ul>
      </div>

      {/* Rule Actions */}
      <div className="rule-actions">
        <button onClick={handleAddRule}>+ Add Rule</button>
        <button onClick={handleRemoveRule} disabled={rules.length <= 1}>
          Remove Rule
        </button>
        <button
          onClick={handleRegenerate}
          disabled={regenerating}
          className="regen-btn"
        >
          {regenerating ? "Regenerating…" : "Regenerate"}
        </button>
      </div>
    </div>
  );
};
