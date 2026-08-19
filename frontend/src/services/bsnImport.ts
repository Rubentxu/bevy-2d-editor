import { callBridge, bridgeReady } from "./bridge-call";
/**
 * BSN file import service.
 * Wraps the `import_bsn_asset_wasm` WASM binding for importing
 * `.bsn` text files produced by EditorCoreBsnExporter as new Scene Assets.
 */

/**
 * Import a `.bsn` file and create a new Scene Asset from it.
 *
 * @param name - Logical path / name for the new asset (e.g. "characters/player")
 * @param bsnText - Raw `.bsn` file content
 * @returns JSON string of the new SceneAssetCatalogEntry
 * @throws Error with parse details if the `.bsn` text is malformed
 */
export async function importBsnAsset(
  name: string,
  bsnText: string,
): Promise<string> {
  const result = await await callBridge("import_bsn_asset_wasm", name, bsnText);
  if (typeof result !== "string") {
    throw new Error("Unexpected WASM return type for import_bsn_asset_wasm");
  }
  return result;
}

/**
 * Import a `.bsn` file from a File object and create a new Scene Asset.
 *
 * @param name - Logical path / name for the new asset
 * @param file - The File object from an <input type="file"> picker
 * @returns JSON string of the new SceneAssetCatalogEntry
 */
export function importBsnAssetFromFile(
  name: string,
  file: File,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = async (e) => {
      const text = e.target?.result;
      if (typeof text !== "string") {
        reject(new Error("Failed to read file as text"));
        return;
      }
      try {
        const json = await importBsnAsset(name, text);
        resolve(json);
      } catch (err) {
        reject(err);
      }
    };
    reader.onerror = () => reject(new Error("FileReader error"));
    reader.readAsText(file);
  });
}

/**
 * Import a `.bsn` text string into a SceneAssetDocument (raw parse, no asset creation).
 * Use `importBsnAsset` for the full import-to-asset flow.
 *
 * @param bsnText - Raw `.bsn` file content
 * @returns JSON string of the resulting SceneAssetDocument
 * @throws Error with parse details if the `.bsn` text is malformed
 */
export async function importBsnTextToDocument(
  bsnText: string,
): Promise<string> {
  const result = await callBridge("import_bsn_text_to_asset_wasm", bsnText);
  if (typeof result !== "string") {
    throw new Error(
      "Unexpected WASM return type for import_bsn_text_to_asset_wasm",
    );
  }
  return result;
}
