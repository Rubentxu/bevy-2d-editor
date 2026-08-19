import { callBridge, bridgeReady } from "./bridge-call";
/**
 * Thin wrappers around window.paint_tile / window.erase_tile WASM bindings
 * and tileset CRUD operations.
 *
 * NOTE: Uses "Tileset" terminology per level-design-tools spec.
 */

export interface TilesetMetadata {
  id: string;
  name: string;
  image_ref: string;
  tile_width: number;
  tile_height: number;
  columns: number;
  spacing: number;
}

async function waitForEngine(): Promise<void> {
  await bridgeReady();
}

/**
 * List all available tilesets.
 * @returns JSON array of TilesetMetadata
 */
export async function listTilesets(): Promise<TilesetMetadata[]> {
  await waitForEngine();
  const result = await callBridge<TilesetMetadata[]>("list_tilesets");
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Load a tileset by ID.
 * @returns Parsed tileset JSON
 */
export async function loadTileset(id: string): Promise<any> {
  await waitForEngine();
  const result = await callBridge("load_tileset", id);
  return typeof result === "string" ? JSON.parse(result) : result;
}

/**
 * Save a tileset (create or update).
 * @param tileset - The tileset object to save
 * @returns JSON string of saved TilesetAsset
 */
export async function saveTileset(tileset: any): Promise<string> {
  await waitForEngine();
  const result = await callBridge("save_tileset", JSON.stringify(tileset));
  return typeof result === "string" ? result : String(result);
}

/**
 * Delete a tileset by ID.
 */
export async function deleteTileset(id: string): Promise<void> {
  await waitForEngine();
  return await callBridge("delete_tileset", id);
}

/**
 * Create a new tileset and persist it.
 *
 * @returns The new tileset ID
 */
export async function createTileset(
  name: string,
  imageRef: string,
  tileWidth: number,
  tileHeight: number,
  columns: number,
  spacing: number = 0,
): Promise<string> {
  const tileset = {
    name,
    image_ref: imageRef,
    tile_width: tileWidth,
    tile_height: tileHeight,
    columns,
    spacing,
  };
  const result = await saveTileset(tileset);
  // parse the returned JSON to extract the id
  const parsed = typeof result === "string" ? JSON.parse(result) : result;
  return parsed.id ?? result;
}

/**
 * Paint a tile at the given grid coordinate.
 *
 * @param assetRef  - The asset's logical path
 * @param layerId   - The target layer ID
 * @param x         - Grid X coordinate
 * @param y         - Grid Y coordinate
 * @param tilesetId - Source tileset ID
 * @param localIndex - Index of the tile within the tileset
 */
export async function paintTile(
  assetRef: string,
  layerId: string,
  x: number,
  y: number,
  tilesetId: string,
  localIndex: number,
): Promise<void> {
  await waitForEngine();
  await await callBridge(
    "paint_tile",
    assetRef,
    layerId,
    x,
    y,
    tilesetId,
    localIndex,
  );
}

/**
 * Erase the tile at the given grid coordinate.
 *
 * @param assetRef - The asset's logical path
 * @param layerId  - The target layer ID
 * @param x        - Grid X coordinate
 * @param y        - Grid Y coordinate
 */
export async function eraseTile(
  assetRef: string,
  layerId: string,
  x: number,
  y: number,
): Promise<void> {
  await waitForEngine();
  await await callBridge("erase_tile", assetRef, layerId, x, y);
}
