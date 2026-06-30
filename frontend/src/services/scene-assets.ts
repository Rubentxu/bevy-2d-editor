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
  /** Level Layers (only meaningful when role === "level"). */
  layers?: LevelLayerPayload[];
}

export interface LevelLayerPayload {
  kind: "scene_instance";
  id: string;
  name: string;
  layer_kind: SceneInstanceLayerKind;
  order: number;
  instances: SceneInstance[];
}

// ── Runtime Preview Inspector types (runtime-preview-inspector) ───────────

/**
 * Live preview metrics. Updated by Bevy's emit_events and rebuild_preview_world
 * systems. Read by the JS-side inspector via WASM bridge.
 */
export interface PreviewMetrics {
  fps: number;
  frame_time_ms: number;
  rebuild_count: number;
}

/**
 * One entry in the preview entity mapping list. StableId-only on the editor
 * side; no Bevy Entity ID is exposed to JS.
 */
export interface PreviewMappingEntry {
  stable_id: string;
  local_id: string;
  asset_ref: string;
  component_count: number;
}

/**
 * Per-instance provenance detail. Returned by getPreviewProvenance.
 */
export interface PreviewProvenance {
  stable_id: string;
  local_id: string;
  asset_ref: string;
  components: string[];
  is_from_instance: boolean;
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
 * Component override health per ADR-0005 §Overrides, §Versioning and ADR-0009.
 */
export type ComponentOverrideStatus = "active" | "orphaned" | "stale" | "conflict";

/**
 * A single non-destructive component field patch on a placed Scene Instance.
 */
export interface ComponentOverride {
  target_local_id: string;
  component_type_id: string;
  field_path: string[];
  value: unknown;
  status: ComponentOverrideStatus;
}

/**
 * A placed use of a Scene Asset: reference + instance components + overrides,
 * NOT a deep clone.
 * Per ADR-0005 §Overrides, §Versioning; ADR-0009; level-design-layers-research.
 */
export interface SceneInstance {
  instance_id: string;
  asset_ref: string;
  asset_version_seen: number;
  id_map: Record<string, string>;
  /** Components owned by this placed occurrence (placement-time data). */
  instance_components: ComponentInstance[];
  component_overrides: ComponentOverride[];
  orphaned_component_overrides: ComponentOverride[];
}

/**
 * Soft-typed Scene Instance Layer category (level-design-layers-research).
 */
export type SceneInstanceLayerKind =
  | "actors"
  | "props"
  | "spawns"
  | "triggers"
  | "collision"
  | "custom";

/**
 * Read-side summary of a Scene Instance Layer (instances vector is omitted
 * for list-affordance UI; full payload lives on the asset document).
 */
export interface SceneInstanceLayerSummary {
  id: string;
  name: string;
  kind: SceneInstanceLayerKind;
  order: number;
  instances_count: number;
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

/**
 * Read `instance_components` for a given placed `instance_id`.
 * Returns `null` if no instance with that id is loaded.
 */
export async function getInstanceComponents(
  instanceId: string,
): Promise<ComponentInstance[] | null> {
  await waitForEngine();
  const result = (window as any).get_instance_components_wasm(instanceId);
  if (result === null || result === undefined) return null;
  return typeof result === "string" ? JSON.parse(result) : result;
}

// ── Override / Resync WASM wrappers ──────────────────────────────────────────

/**
 * Issue found during override validation.
 * Codes: missing_entity, missing_component, duplicate_field, missing_field, type_conflict.
 */
export interface OverrideIssue {
  code: string;
  patch: ComponentOverride;
  message: string;
}

/**
 * Summary of a resync operation — counts of what happened to overrides.
 */
export interface ResyncReport {
  active: number;
  orphaned: number;
  stale: number;
  conflict: number;
  rebound: number;
}

/**
 * Resolved scene: asset entities with overrides merged in (read-only).
 */
export interface ResolvedEntity {
  local_id: string;
  local_path: string;
  name: string;
  components: ComponentInstance[];
}

export interface ResolvedScene {
  entities: Record<string, ResolvedEntity>;
  id_map: Record<string, string>;
  minted_stable_ids: string[];
  unresolved: ComponentOverride[];
}

/**
 * Validate a SceneInstance's overrides against an asset.
 * @returns Array of OverrideIssue objects (empty if all overrides are valid).
 */
export async function validateOverrides(
  instance: SceneInstance,
  asset: SceneAssetDocument
): Promise<OverrideIssue[]> {
  await waitForEngine();
  const result = (window as any).validate_overrides_wasm(
    JSON.stringify(instance),
    JSON.stringify(asset)
  );
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Compute effective values: read-only merge of asset + active overrides.
 * @returns ResolvedScene with merged component values.
 */
export async function effectiveValues(
  instance: SceneInstance,
  asset: SceneAssetDocument
): Promise<ResolvedScene> {
  await waitForEngine();
  const result = (window as any).effective_values_wasm(
    JSON.stringify(instance),
    JSON.stringify(asset)
  );
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Try to rebind an orphaned patch to a new asset.
 * @returns LocalId string if match found, null otherwise.
 */
export async function tryRebind(
  orphanedPatch: ComponentOverride,
  asset: SceneAssetDocument
): Promise<string | null> {
  await waitForEngine();
  const result = (window as any).try_rebind_wasm(
    JSON.stringify(orphanedPatch),
    JSON.stringify(asset)
  );
  const parsed = typeof result === "string" ? JSON.parse(result) : result;
  return parsed === null ? null : parsed;
}

/**
 * Drain accumulated resync reports from the last scene load / replace operation.
 * @returns Array of [instance_id, ResyncReport] tuples.
 */
export async function getResyncReports(): Promise<
  Array<[string, ResyncReport]>
> {
  await waitForEngine();
  const result = (window as any).get_resync_reports();
  return typeof result === "string" ? JSON.parse(result) : result;
}

// ── Scene Instance Layer WASM wrappers (scene-instance-layer) ─────────────

/**
 * List Scene Instance Layers of a Scene Asset document.
 * @returns Array of layer summaries (id, name, kind, order, instances_count).
 */
export async function listSceneInstanceLayers(
  assetJson: string
): Promise<SceneInstanceLayerSummary[]> {
  await waitForEngine();
  const result = (window as any).list_scene_instance_layers_wasm(assetJson);
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Create a Scene Instance Layer in the asset document. Returns the updated
 * asset JSON string (caller is responsible for merging it back into editor state).
 * @throws on unknown kind
 */
export async function createSceneInstanceLayer(
  assetJson: string,
  name: string,
  kind: SceneInstanceLayerKind
): Promise<string> {
  await waitForEngine();
  const result = (window as any).create_scene_instance_layer_wasm(
    assetJson,
    name,
    kind
  );
  if (typeof result === "string") return result;
  // Some WASM error paths return JsValue objects; coerce to string.
  return String(result);
}

/**
 * Delete a Scene Instance Layer by id. Returns the updated asset JSON string.
 * Unknown id is a no-op (the returned asset JSON is unchanged).
 */
export async function deleteSceneInstanceLayer(
  assetJson: string,
  layerId: string
): Promise<string> {
  await waitForEngine();
  const result = (window as any).delete_scene_instance_layer_wasm(
    assetJson,
    layerId
  );
  return typeof result === "string" ? result : String(result);
}

/**
 * Replace the in-memory Scene Asset document in the backend with the given
 * JSON. Returns no value on success.
 *
 * Used after `createSceneInstanceLayer` or `deleteSceneInstanceLayer` to
 * commit layer mutations back to the backend so the next
 * `saveSceneAsset()` persists them.
 */
export async function setAssetDocumentJson(assetJson: string): Promise<void> {
  await waitForEngine();
  (window as any).set_asset_document_wasm(assetJson);
}

// ── Runtime Preview Inspector wrappers (runtime-preview-inspector) ────────

/**
 * Read live preview metrics: fps, frame time in ms, total rebuild count.
 */
export async function getPreviewMetrics(): Promise<PreviewMetrics> {
  await waitForEngine();
  const result = (window as any).get_preview_metrics_wasm();
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Read the preview entity mapping list. StableId-only on the editor side.
 */
export async function getPreviewMapping(): Promise<PreviewMappingEntry[]> {
  await waitForEngine();
  const result = (window as any).get_preview_mapping_wasm();
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Read per-instance provenance detail. Returns `null` if the `stable_id`
 * is not currently projected.
 */
export async function getPreviewProvenance(
  stableId: string
): Promise<PreviewProvenance | null> {
  await waitForEngine();
  const result = (window as any).get_preview_provenance_wasm(stableId);
  if (result === null || result === undefined) return null;
  return typeof result === "string" ? JSON.parse(result) : result;
}

// ── BSN file export (bsn-file-export-research) ──────────────────────────────

/**
 * Export a `SceneAsset` by `asset_id` to `.bsn` text via the editor's
 * `EditorCoreBsnExporter`. Does NOT change the currently-open document.
 * The output is raw `.bsn` syntax suitable for hand-off to a Bevy runtime.
 *
 * @param assetId The `asset_id` of the SceneAsset to export
 * @returns The `.bsn` text as a string
 * @throws if the asset is not found or export fails
 */
export async function exportAssetToBsn(assetId: string): Promise<string> {
  await waitForEngine();
  const result = await (window as any).export_asset_to_bsn_wasm(assetId);
  if (typeof result === "string") return result;
  // Safety: coerce any error value to string.
  return String(result);
}
