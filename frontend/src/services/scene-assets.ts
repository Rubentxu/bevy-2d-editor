/**
 * Thin wrappers around window.scene_asset_* WASM bindings.
 * All functions wait for the engine to be ready before invoking.
 *
 * NOTE: Uses "Scene Asset" terminology per spec S21.
 * NO prefab/EntityTemplate/blueprint/archetype terms allowed.
 */

export interface SceneAssetCatalogEntry {
  asset_id: string;
  logical_path: string;
  role: string;
  current_version: number;
}

export interface AssetLogState {
  size: number;
  can_undo: boolean;
  can_redo: boolean;
  cursor: number;
  dirty: boolean;
}

// ── Scene Asset Document Types ─────────────────────────────────────────────────

export interface ComponentInstance {
  type_id: string;
  values: Record<string, unknown>;
}

export interface SceneAssetEntity {
  local_id: string;
  local_path: string;
  name: string;
  components: ComponentInstance[];
}

export type RelationshipKind = "Child" | { custom: string };

export interface SceneAssetRelationship {
  from_local_id: string;
  to_local_id: string;
  kind: RelationshipKind;
  field_path?: string[];
}

export interface ExposedProperty {
  name: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any;
}

export interface SceneAssetMetadata {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

export interface SceneAssetDocument {
  asset_id: string;
  logical_path: string;
  role: string;
  version: number;
  entities: SceneAssetEntity[];
  relationships: SceneAssetRelationship[];
  exposed_properties: ExposedProperty[];
  metadata: SceneAssetMetadata;
}

async function waitForEngine(): Promise<void> {
  let attempts = 0;
  while (
    typeof (window as any).create_scene_asset !== "function" &&
    attempts < 50
  ) {
    await new Promise((r) => setTimeout(r, 100));
    attempts++;
  }
}

/**
 * Create a new Scene Asset.
 * @param name - Human-facing name (will be normalized to logical_path)
 * @param role - Asset role (e.g., "actor", "level", "ui")
 * @returns JSON string of SceneAssetCatalogEntry
 */
export async function createSceneAsset(
  name: string,
  role: string
): Promise<string> {
  await waitForEngine();
  return (window as any).create_scene_asset(name, role);
}

/**
 * Rename a Scene Asset (moves the file and updates catalog).
 * @param assetId - The asset's stable ID
 * @param newPath - New logical path (e.g., "characters/player")
 * @returns JSON string of updated SceneAssetCatalogEntry
 */
export async function renameSceneAsset(
  assetId: string,
  newPath: string
): Promise<string> {
  await waitForEngine();
  return (window as any).rename_scene_asset(assetId, newPath);
}

/**
 * Duplicate a Scene Asset.
 * NOTE: This is 1-arity (assetId only) per constraint C-1.
 * No suggested_name parameter - the backend generates a unique name.
 * @param assetId - The source asset's stable ID
 * @returns JSON string of new SceneAssetCatalogEntry
 */
export async function duplicateSceneAsset(assetId: string): Promise<string> {
  await waitForEngine();
  return (window as any).duplicate_scene_asset(assetId);
}

/**
 * Delete a Scene Asset (removes file and catalog entry).
 * @param assetId - The asset's stable ID
 */
export async function deleteSceneAsset(assetId: string): Promise<void> {
  await waitForEngine();
  return (window as any).delete_scene_asset(assetId);
}

/**
 * List Scene Assets, optionally filtered by role.
 * @param roleFilter - Optional role to filter by (e.g., "actor").
 *                      Pass undefined/null for all assets.
 * @returns JSON array of SceneAssetCatalogEntry
 */
export async function listSceneAssets(
  roleFilter?: string
): Promise<SceneAssetCatalogEntry[]> {
  await waitForEngine();
  const result = (window as any).list_scene_assets(roleFilter ?? null);
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Open a Scene Asset for editing (loads body into SCENE_ASSET_DOC).
 * @param assetId - The asset's stable ID
 * @returns JSON string of the asset's document body
 */
export async function openSceneAsset(assetId: string): Promise<string> {
  await waitForEngine();
  return (window as any).open_scene_asset(assetId);
}

/**
 * Close the currently open Scene Asset (drops SCENE_ASSET_DOC, resets log).
 * No file write occurs.
 */
export function closeSceneAsset(): void {
  if (typeof (window as any).close_scene_asset !== "function") {
    throw new Error("close_scene_asset not available");
  }
  (window as any).close_scene_asset();
}

/**
 * Get the current Scene Asset document JSON.
 * @returns JSON string of the active document
 * @throws if no asset is open
 */
export async function getAssetDocumentJson(): Promise<string> {
  await waitForEngine();
  return (window as any).get_asset_document_json();
}

/**
 * Get the Scene Asset catalog as JSON.
 * @returns JSON array of all SceneAssetCatalogEntry
 */
export async function getSceneAssetCatalogJson(): Promise<SceneAssetCatalogEntry[]> {
  await waitForEngine();
  const result = (window as any).get_scene_asset_catalog_json();
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Dispatch an AssetCommand to the active Scene Asset document.
 * @param cmdJson - JSON string of AssetCommand envelope
 * @returns JSON string of AssetCommandResult (inverse + snapshot)
 */
export async function dispatchAssetCommand(cmdJson: string): Promise<string> {
  await waitForEngine();
  return (window as any).dispatch_asset_command(cmdJson);
}

/**
 * Undo the last asset command.
 * @returns JSON string of the inverse command
 */
export async function undoAsset(): Promise<string> {
  await waitForEngine();
  return (window as any).undo_asset();
}

/**
 * Redo the next asset command.
 * @returns JSON string (empty on success)
 */
export async function redoAsset(): Promise<string> {
  await waitForEngine();
  return (window as any).redo_asset();
}

/**
 * Get the current asset operation log state.
 * @returns AssetLogState object with size, can_undo, can_redo, cursor, dirty
 */
export async function getAssetLogState(): Promise<AssetLogState> {
  await waitForEngine();
  const result = (window as any).get_asset_log_state();
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Save the current Scene Asset (body-first, then catalog update).
 * @returns JSON string of the saved path
 */
export async function saveSceneAsset(): Promise<string> {
  await waitForEngine();
  return (window as any).save_scene_asset();
}

// ── Scene Instance Types (PR3) ────────────────────────────────────────────────

/**
 * Override health per ADR-0005 §Overrides, §Versioning.
 */
export type OverrideStatus = "active" | "orphaned" | "stale" | "conflict";

/**
 * A single non-destructive patch on a placed Scene Instance.
 */
export interface OverridePatch {
  target_local_id: string;
  field_path: string[];
  value: unknown;
  status: OverrideStatus;
}

/**
 * A placed use of a Scene Asset: reference + patches, NOT a deep clone.
 * Per ADR-0005 §Overrides, §Versioning.
 */
export interface SceneInstance {
  instance_id: string;
  asset_ref: string;
  asset_version_seen: number;
  id_map: Record<string, string>;
  overrides: OverridePatch[];
  orphaned_overrides: OverridePatch[];
}

/**
 * Result of a scene instance command (place/remove/replace).
 */
export interface SceneInstanceCommandResult {
  inverse: object;
  snapshot: object;
}

// ── Scene Instance Operations (PR3) ──────────────────────────────────────────

/**
 * Place a Scene Asset as a new Scene Instance in the active scene.
 *
 * @param assetId - The asset's stable ID from the catalog
 * @param translationJson - Optional translation as {x: number, y: number}
 * @returns SceneInstanceCommandResult JSON
 */
export async function placeSceneInstance(
  assetId: string,
  translationJson?: { x: number; y: number }
): Promise<SceneInstanceCommandResult> {
  await waitForEngine();
  const result = (window as any).place_scene_instance(
    assetId,
    translationJson ? JSON.stringify(translationJson) : null
  );
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Remove a Scene Instance from the active scene.
 *
 * @param instanceId - The instance's stable ID
 * @returns SceneInstanceCommandResult JSON
 */
export async function removeSceneInstance(
  instanceId: string
): Promise<SceneInstanceCommandResult> {
  await waitForEngine();
  const result = (window as any).remove_scene_instance(instanceId);
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Replace the asset of an existing Scene Instance.
 *
 * @param instanceId - The instance's stable ID
 * @param newAssetId - The new asset's stable ID
 * @returns SceneInstanceCommandResult JSON
 */
export async function replaceSceneInstanceAsset(
  instanceId: string,
  newAssetId: string
): Promise<SceneInstanceCommandResult> {
  await waitForEngine();
  const result = (window as any).replace_scene_instance_asset(
    instanceId,
    newAssetId
  );
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Get all Scene Instances from the active scene.
 *
 * @returns Map of instance_id → SceneInstance
 */
export async function getSceneInstances(): Promise<
  Record<string, SceneInstance>
> {
  await waitForEngine();
  const result = (window as any).get_scene_instances();
  return typeof result === "string" ? JSON.parse(result) : result;
}
