/**
 * Thin wrappers around window.asset_files_* WASM bindings.
 * All functions wait for the engine to be ready before invoking.
 *
 * Asset files are binary textures stored in OPFS `resources/` directory.
 * Owned by WASM editor-core, not a frontend-only service.
 *
 * Uses the canonical `OpfsResult<T>` envelope from `types/opfs.ts`.
 */

import type { OpfsResult } from "../types/opfs";
import { emit, inFlightSaveCounter } from "./hot-reload";

export type AssetFileKind = "Texture" | "Audio" | "Font";

export interface AssetFile {
  id: string;
  path: string;
  name: string;
  kind: AssetFileKind;
  mime_type: string;
  size_bytes: number;
}

async function waitForEngine(): Promise<void> {
  let attempts = 0;
  while (
    typeof (window as any).list_asset_files !== "function" &&
    attempts < 50
  ) {
    await new Promise((r) => setTimeout(r, 100));
    attempts++;
  }
}

function parseOpfs<T>(raw: unknown): OpfsResult<T> {
  if (typeof raw === "string") {
    return JSON.parse(raw) as OpfsResult<T>;
  }
  return raw as OpfsResult<T>;
}

/**
 * List all asset files in the `resources/` directory.
 * @returns Array of AssetFile metadata.
 * @throws Error if the WASM engine is unavailable or the operation fails.
 */
export async function listAssetFiles(): Promise<AssetFile[]> {
  await waitForEngine();
  const parsed = parseOpfs<AssetFile[]>((window as any).list_asset_files());
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.value!;
}

/**
 * Import a binary asset file into OPFS.
 * Creates the binary file and its metadata sidecar.
 * @param name - File name (e.g. "player.png")
 * @param mimeType - MIME type (e.g. "image/png")
 * @param bytes - Raw file bytes
 * @returns Created AssetFile on success.
 */
export async function importAssetFile(
  name: string,
  mimeType: string,
  bytes: Uint8Array,
): Promise<AssetFile> {
  await waitForEngine();
  inFlightSaveCounter.incr();
  try {
    const jsBytes = new Uint8Array(bytes);
    const parsed = parseOpfs<AssetFile>(
      (window as any).import_asset_file(name, mimeType, jsBytes),
    );
    if (!parsed.ok) throw new Error(parsed.error);
    emit({ type: "hot-reload-asset", assetId: name });
    return parsed.value!;
  } finally {
    inFlightSaveCounter.decr();
  }
}

/**
 * Read the raw bytes of an asset file by id.
 * @param id - The asset file id (which equals its path, e.g. "player.png")
 * @returns Uint8Array bytes on success.
 */
export async function readAssetFileBytes(id: string): Promise<Uint8Array> {
  await waitForEngine();
  const parsed = parseOpfs<{ kind: "ok"; value: number[] }>(
    (window as any).read_asset_file_bytes(id),
  );
  if (!parsed.ok) throw new Error(parsed.error);
  // value is a JSON-serialized array of bytes from serde_json
  const arr = parsed.value as unknown as number[];
  return new Uint8Array(arr);
}

/**
 * Delete an asset file and its metadata sidecar from OPFS.
 * @param id - The asset file id.
 */
export async function deleteAssetFile(id: string): Promise<void> {
  await waitForEngine();
  inFlightSaveCounter.incr();
  try {
    const parsed = parseOpfs<null>((window as any).delete_asset_file(id));
    if (!parsed.ok) throw new Error(parsed.error);
    emit({ type: "hot-reload-asset", assetId: id });
  } finally {
    inFlightSaveCounter.decr();
  }
}
