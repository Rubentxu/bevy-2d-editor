/**
 * Thin wrappers around window.logic_graph_* WASM bindings.
 * All functions wait for the engine to be ready before invoking.
 *
 * NOTE: Uses "Logic Graph" terminology per spec §Logic Bricks.
 * Mirrors the scene-assets.ts service pattern.
 */

import type { LogicGraphAsset } from "../hooks/useLogicGraph";

/**
 * Logic graph catalog entry — lightweight metadata for the browser list.
 */
export interface LogicGraphCatalogEntry {
  asset_id: string;
  logical_path: string;
  builtin: boolean;
  created_at: number;
  updated_at: number;
}

async function waitForEngine(): Promise<void> {
  let attempts = 0;
  while (
    typeof (window as any).list_logic_graph_assets !== "function" &&
    attempts < 50
  ) {
    await new Promise((r) => setTimeout(r, 100));
    attempts++;
  }
}

/**
 * List all logic graph assets from the catalog.
 * @returns JSON array of LogicGraphCatalogEntry
 */
export async function listLogicGraphAssets(): Promise<LogicGraphCatalogEntry[]> {
  await waitForEngine();
  const result = (window as any).list_logic_graph_assets();
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Open a Logic Graph asset from OPFS into the active graph slot.
 * @param assetId - The asset's stable ID
 * @returns JSON string of the LogicGraphAsset body
 */
export async function openLogicGraphAsset(assetId: string): Promise<string> {
  await waitForEngine();
  return (window as any).open_logic_graph_asset(assetId);
}

/**
 * Get the currently-active LogicGraphAsset as JSON.
 * @returns JSON string of the active graph
 */
export async function getActiveLogicGraph(): Promise<LogicGraphAsset | null> {
  await waitForEngine();
  try {
    const result = (window as any).get_logic_graph();
    return result ? JSON.parse(result) : null;
  } catch {
    return null;
  }
}
