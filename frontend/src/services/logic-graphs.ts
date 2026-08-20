/**
 * Thin wrappers around window.logic_graph_* WASM bindings.
 * All functions wait for the engine to be ready before invoking.
 *
 * NOTE: Uses "Logic Graph" terminology per spec §Logic Bricks.
 * Mirrors the scene-assets.ts service pattern.
 */

import type { LogicGraphAsset } from "../hooks/useLogicGraph";
import { callBridge, bridgeReady } from "./bridge-call";

/**
 * Result of a successful bind operation.
 */
export interface BindResult {
  binding_id: string;
}

/**
 * Error returned when binding fails.
 */
export interface BindError {
  code: string;
  message: string;
}

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
  await bridgeReady();
}

/**
 * List all logic graph assets from the catalog.
 * @returns JSON array of LogicGraphCatalogEntry
 */
export async function listLogicGraphAssets(): Promise<
  LogicGraphCatalogEntry[]
> {
  await waitForEngine();
  const result = await callBridge<LogicGraphCatalogEntry[]>(
    "list_logic_graph_assets",
  );
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Open a Logic Graph asset from OPFS into the active graph slot.
 * @param assetId - The asset's stable ID
 * @returns JSON string of the LogicGraphAsset body
 */
export async function openLogicGraphAsset(assetId: string): Promise<string> {
  await waitForEngine();
  return await callBridge<string>("open_logic_graph_asset", assetId);
}

/**
 * Get the currently-active LogicGraphAsset as JSON.
 * @returns JSON string of the active graph
 */
export async function getActiveLogicGraph(): Promise<LogicGraphAsset | null> {
  await waitForEngine();
  try {
    const result = await callBridge<string>("get_logic_graph");
    return result ? JSON.parse(result) : null;
  } catch {
    return null;
  }
}

/**
 * Bind a LogicGraphAsset (by recipe/asset ID) to a Scene Instance.
 *
 * @param sceneInstanceId - Stable ID of the Scene Instance
 * @param recipeId        - Asset ID of the LogicGraphAsset recipe
 * @param fieldOverrides  - Optional map of field_path → override value
 * @returns binding_id on success
 * @throws BindError on failure
 */
export async function bindLogicInstance(
  sceneInstanceId: string,
  recipeId: string,
  fieldOverrides: Record<string, unknown> = {},
): Promise<string> {
  await waitForEngine();
  const result = await callBridge<string>(
    "bind_logic_graph_to_instance_wasm",
    sceneInstanceId,
    recipeId,
    fieldOverrides,
  );
  // Result is a binding_id string on success
  if (!result || result.startsWith("Err")) {
    throw new Error(
      result ?? "bind_logic_graph_to_instance_wasm returned empty",
    );
  }
  return result;
}

/**
 * Unbind (remove) a logic binding from a Scene Instance.
 *
 * @param sceneInstanceId - Stable ID of the Scene Instance
 * @param bindingId      - Binding ID to remove
 */
export async function unbindLogicInstance(
  sceneInstanceId: string,
  bindingId: string,
): Promise<void> {
  await waitForEngine();
  const result = await callBridge<string>(
    "unbind_logic_graph_from_instance_wasm",
    sceneInstanceId,
    bindingId,
  );
  if (!result || result.startsWith("Err")) {
    throw new Error(
      result ?? "unbind_logic_graph_from_instance_wasm returned empty",
    );
  }
}

/**
 * Set (or update) a field override on an existing logic binding.
 *
 * @param bindingId - Binding ID to update
 * @param fieldPath - Dot-separated field path within the bound graph
 * @param value     - New override value
 */
export async function setLogicFieldOverride(
  bindingId: string,
  fieldPath: string,
  value: unknown,
): Promise<void> {
  await waitForEngine();
  const result = await callBridge<string>(
    "set_binding_field_override_wasm",
    bindingId,
    fieldPath,
    value,
  );
  if (!result || result.startsWith("Err")) {
    throw new Error(result ?? "set_binding_field_override_wasm returned empty");
  }
}
