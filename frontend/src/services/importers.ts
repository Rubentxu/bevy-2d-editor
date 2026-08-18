/**
 * Importers service — typed bindings for the importer WASM exports (ADR-0041).
 *
 * Provides a clean API surface for the ImportDialog and other UI components
 * that need to work with external source importers (Aseprite, LDtk, Tiled).
 *
 * ## WASM exports bound here
 *
 * 1. `register_importer_wasm(json)` — registers an importer from a JSON manifest
 * 2. `list_importers_wasm(kind?)` — lists all or filtered importers
 * 3. `import_external_source_wasm(kind, source_uri, bytes_b64, target)` — imports a file
 * 4. `reimport_external_source_wasm(source_uri)` — reimports an already-imported file
 * 5. `get_external_source_wasm(resource_ref)` — gets the provenance record
 */

import { waitForEditorReady } from "../utils/waitForEditorReady";

/** Window augmented with the WASM bridge. */
interface WindowWithBridge {
  __bridge?: Record<string, unknown>;
  __bevyEngineStarted?: boolean;
  // §8 External Source Importers WASM exports (ADR-0041 / v0.93)
  list_importers_wasm?: (kind?: string) => Promise<string>;
  register_importer_wasm?: (json: string) => Promise<string>;
  import_external_source_wasm?: (
    kind: string,
    source_uri: string,
    bytes_b64: string,
    target_resource_ref: string,
  ) => Promise<string>;
  reimport_external_source_wasm?: (source_uri?: string) => Promise<string>;
  get_external_source_wasm?: (resource_ref: string) => Promise<string>;
}

/** Read the WASM bridge from the window object. */
function readBridge(): WindowWithBridge | null {
  if (typeof window === "undefined") return null;
  return window as unknown as WindowWithBridge;
}

/** Result wrapper for importer operations. */
export type ImporterResult<T> =
  { ok: true; value: T } | { ok: false; error: string };

/** Descriptor for a registered importer. */
export interface ImporterDescriptor {
  id: string;
  kind: string;
  supported_versions: {
    min: { major: number; minor: number; patch: number };
    max: { major: number; minor: number; patch: number };
  };
  display_name: string;
}

/** Result of an import operation. */
export interface ImportResult {
  change_set_id: string;
  sidecar_path: string;
  parse_output: unknown;
}

/** Result of a reimport operation. */
export interface ReimportResult {
  status: "no-op" | "queued" | "auto-applied";
  source_uri: string;
  change_set_id?: string;
  diff?: {
    added: number;
    removed: number;
    modified_source: number;
    modified_editor: number;
    ownership_conflicts: number;
  };
}

/** Provenance record from a sidecar `.meta.json` file. */
export interface ExternalSource {
  kind: string;
  source_uri: string;
  fingerprint: string;
  importer_id: string;
  importer_version: { major: number; minor: number; patch: number };
  last_import_time: number;
  mappings: unknown[];
  ownership_rules: unknown[];
  schema_version: number;
}

/** Call a WASM export that takes no arguments. */
async function callNoArg<T>(
  fn: (() => Promise<string>) | undefined,
): Promise<ImporterResult<T>> {
  if (!fn) return { ok: false, error: "wasm export not available" };
  try {
    const result = await fn();
    try {
      return { ok: true, value: JSON.parse(result) as T };
    } catch {
      return { ok: true, value: result as unknown as T };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

/** Call a WASM export that takes one optional argument. */
async function callOneArg<T>(
  fn: ((arg: string | undefined) => Promise<string>) | undefined,
  arg: string | undefined,
): Promise<ImporterResult<T>> {
  if (!fn) return { ok: false, error: "wasm export not available" };
  try {
    const result = await fn(arg ?? undefined);
    try {
      return { ok: true, value: JSON.parse(result) as T };
    } catch {
      return { ok: true, value: result as unknown as T };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

/** Call a WASM export that takes multiple string arguments. */
async function callMulti<T>(
  fn: ((...args: string[]) => Promise<string>) | undefined,
  ...args: string[]
): Promise<ImporterResult<T>> {
  if (!fn) return { ok: false, error: "wasm export not available" };
  try {
    const result = await fn(...args);
    try {
      return { ok: true, value: JSON.parse(result) as T };
    } catch {
      return { ok: true, value: result as unknown as T };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

/**
 * List all registered importers, optionally filtered by kind.
 *
 * @param kind - Optional filter: "Aseprite", "Ldtk", "Tiled", or "Custom"
 */
export async function listImporters(
  kind?: string,
): Promise<ImporterResult<ImporterDescriptor[]>> {
  await waitForEditorReady();
  const w = readBridge();
  return callOneArg(w?.list_importers_wasm, kind);
}

/**
 * Register a new importer from a JSON manifest.
 *
 * @param manifest - JSON object with id, kind, supported_versions, display_name
 */
export async function registerImporter(
  manifest: Record<string, unknown>,
): Promise<ImporterResult<{ ok: boolean }>> {
  await waitForEditorReady();
  const w = readBridge();
  if (!w?.register_importer_wasm) {
    return { ok: false, error: "register_importer_wasm export not available" };
  }
  try {
    const result = await w.register_importer_wasm(JSON.stringify(manifest));
    try {
      return { ok: true, value: JSON.parse(result) as { ok: boolean } };
    } catch {
      return { ok: true, value: { ok: true } as { ok: boolean } };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

/**
 * Import an external source file (Aseprite, LDtk, Tiled).
 *
 * @param kind - Source kind: "Aseprite", "Ldtk", or "Tiled"
 * @param sourceUri - Path/URI of the source file
 * @param bytesB64 - Base64-encoded file bytes
 * @param target - Destination resource path in the project
 */
export async function importExternalSource(
  kind: string,
  sourceUri: string,
  bytesB64: string,
  target: string,
): Promise<ImporterResult<ImportResult>> {
  await waitForEditorReady();
  const w = readBridge();
  return callMulti(
    w?.import_external_source_wasm,
    kind,
    sourceUri,
    bytesB64,
    target,
  );
}

/**
 * Re-import an already-imported external source file.
 *
 * Detects changes and either auto-applies or queues for human review
 * via the Change Workbench.
 *
 * @param sourceUri - Path/URI of the source file
 */
export async function reimportExternalSource(
  sourceUri: string,
): Promise<ImporterResult<ReimportResult>> {
  await waitForEditorReady();
  const w = readBridge();
  return callOneArg(w?.reimport_external_source_wasm, sourceUri);
}

/**
 * Get the provenance record for an imported resource.
 *
 * Reads the sidecar `.meta.json` file associated with the resource.
 *
 * @param resourceRef - The logical path of the imported resource
 * @returns The ExternalSource record, or null if no sidecar exists
 */
export async function getExternalSource(
  resourceRef: string,
): Promise<ImporterResult<ExternalSource | null>> {
  await waitForEditorReady();
  const w = readBridge();
  if (!w?.get_external_source_wasm) {
    return {
      ok: false,
      error: "get_external_source_wasm export not available",
    };
  }
  try {
    const result = await w.get_external_source_wasm(resourceRef);
    if (result === "null") {
      return { ok: true, value: null };
    }
    try {
      return { ok: true, value: JSON.parse(result) as ExternalSource };
    } catch {
      return { ok: true, value: result as unknown as ExternalSource };
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}

/** Supported external source kinds. */
export type SourceKind = "Aseprite" | "Ldtk" | "Tiled";

/** Human-readable labels for source kinds. */
export const SOURCE_KIND_LABELS: Record<SourceKind, string> = {
  Aseprite: "Aseprite (.json + .png)",
  Ldtk: "LDtk (.ldtk)",
  Tiled: "Tiled (.json)",
};

/** File extension filters for each source kind. */
export const SOURCE_KIND_EXTENSIONS: Record<SourceKind, string[]> = {
  Aseprite: [".json"],
  Ldtk: [".ldtk"],
  Tiled: [".json"],
};
