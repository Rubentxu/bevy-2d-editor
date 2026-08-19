import { callBridge, bridgeReady } from "./bridge-call";
/**
 * Thin wrappers around window.auto_layer_* WASM bindings (auto-layer-generation PR2).
 * All functions wait for the engine to be ready before invoking.
 *
 * NOTE: Uses "AutoLayer" terminology per the Bevy 2D Editor domain language.
 * No prefab/EntityTemplate/blueprint/archetype terms allowed.
 */

// ── AutoLayer Domain Types ────────────────────────────────────────────────────

/**
 * One cell in a 3x3 auto-tiling pattern.
 * - `filled`: matches any non-empty tile in the source layer
 * - `empty`: matches an empty cell in the source layer
 * - `any`: wildcard — matches regardless of source cell state
 */
export type PatternCell = "filled" | "empty" | "any";

/**
 * A 3x3 neighborhood pattern for auto-tiling.
 *
 * The center cell [1][1] is always ignored — it is the cell being evaluated,
 * not part of the pattern context.
 *
 * Layout (row-major):
 *   [0][0] [0][1] [0][2]
 *   [1][0] [1][1] [1][2]   ← center [1][1] is ignored during matching
 *   [2][0] [2][1] [2][2]
 */
export type Pattern3x3 = [
  [PatternCell, PatternCell, PatternCell],
  [PatternCell, PatternCell, PatternCell],
  [PatternCell, PatternCell, PatternCell],
];

/**
 * One auto-tiling rule.
 * Rules are evaluated in declaration order inside `regenerate()`. The first
 * rule whose pattern matches the 3x3 neighborhood wins — later rules are not
 * evaluated for that cell.
 */
export interface AutoRule {
  /** The 3x3 pattern to match against the source layer neighborhood. */
  pattern: Pattern3x3;
  /** Tiles to emit when this rule matches. */
  output: TileRefPayload[];
  /** Optional probability [0.0, 1.0]. If absent, the rule always fires. */
  chance?: number;
}

/**
 * A tile reference used in auto-layer rule output.
 * Mirrors the Rust `TileRef` from `tileset.rs`.
 */
export interface TileRefPayload {
  tileset_id: string;
  local_index: number;
}

// ── Engine-ready guard ───────────────────────────────────────────────────────

async function waitForEngine(): Promise<void> {
  await bridgeReady();
}

// ── AutoLayer WASM wrappers ───────────────────────────────────────────────────

/**
 * Check whether an AutoLayer's cached tile grid is stale.
 *
 * An AutoLayer cache is stale when the source TileLayer has been modified
 * (paint/erase) since the cache was last built.
 *
 * @param assetRef  - Logical path of the scene asset (e.g. "levels/world1")
 * @param layerId   - The AutoLayer's stable id string
 * @returns `true` if the cached grid needs regeneration, `false` if up-to-date
 */
export async function isAutoLayerStale(
  assetRef: string,
  layerId: string,
): Promise<boolean> {
  await waitForEngine();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return await callBridge("is_auto_layer_stale_wasm", assetRef, layerId);
}

/**
 * Regenerate an AutoLayer's cached tile grid from its source TileLayer.
 *
 * This operation is recorded in the asset operation log so it can be undone
 * and redone via `undoAsset()` / `redoAsset()`.
 *
 * @param assetRef - Logical path of the scene asset
 * @param layerId  - The AutoLayer's stable id string
 * @returns The updated SceneAssetDocument JSON string
 */
export async function regenerateAutoLayer(
  assetRef: string,
  layerId: string,
): Promise<string> {
  await waitForEngine();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return await callBridge("regenerate_auto_layer_wasm", assetRef, layerId);
}

/**
 * Add an AutoRule to an AutoLayer (direct mutation, bypasses operation log).
 *
 * @param assetRef - Logical path of the scene asset
 * @param layerId  - The AutoLayer's stable id string
 * @param rule     - The AutoRule to add
 * @returns The updated SceneAssetDocument JSON string
 */
export async function addAutoRule(
  assetRef: string,
  layerId: string,
  rule: AutoRule,
): Promise<string> {
  await waitForEngine();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return await callBridge(
    "add_auto_rule_wasm",
    assetRef,
    layerId,
    JSON.stringify(rule),
  );
}

/**
 * Replace an AutoRule in an AutoLayer at the given index (direct mutation).
 *
 * @param assetRef   - Logical path of the scene asset
 * @param layerId    - The AutoLayer's stable id string
 * @param ruleIndex  - Index of the rule to replace
 * @param rule       - The new AutoRule
 * @returns The updated SceneAssetDocument JSON string
 * @throws if ruleIndex is out of bounds
 */
export async function updateAutoRule(
  assetRef: string,
  layerId: string,
  ruleIndex: number,
  rule: AutoRule,
): Promise<string> {
  await waitForEngine();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return await callBridge(
    "update_auto_rule_wasm",
    assetRef,
    layerId,
    ruleIndex,
    JSON.stringify(rule),
  );
}

/**
 * Remove an AutoRule from an AutoLayer at the given index (direct mutation).
 *
 * @param assetRef   - Logical path of the scene asset
 * @param layerId    - The AutoLayer's stable id string
 * @param ruleIndex  - Index of the rule to remove
 * @returns The updated SceneAssetDocument JSON string
 * @throws if ruleIndex is out of bounds
 */
export async function removeAutoRule(
  assetRef: string,
  layerId: string,
  ruleIndex: number,
): Promise<string> {
  await waitForEngine();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return await callBridge(
    "remove_auto_rule_wasm",
    assetRef,
    layerId,
    ruleIndex,
  );
}
