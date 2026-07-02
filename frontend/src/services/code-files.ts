/**
 * Thin wrappers around window.source_files_* WASM bindings.
 * All functions wait for the engine to be ready before invoking.
 *
 * Source files are raw `.rs` text stored in OPFS `sources/` directory.
 * Owned by WASM editor-core, not a frontend-only service.
 *
 * Uses the canonical `OpfsResult<T>` envelope from `types/opfs.ts`
 * (per `design.md` §Interfaces/Contracts). On-the-wire JSON from WASM
 * always matches the broad optional-field shape; we narrow to a
 * discriminated union at this service boundary for type safety.
 */

import type { OpfsResult } from "../types/opfs";

export interface SourceFile {
  id: string;
  path: string;
  name: string;
}

async function waitForEngine(): Promise<void> {
  let attempts = 0;
  while (
    typeof (window as any).list_source_files !== "function" &&
    attempts < 50
  ) {
    await new Promise((r) => setTimeout(r, 100));
    attempts++;
  }
}

/**
 * Parse a WASM response that may arrive as a string (JSON) or already-parsed object.
 * The Rust bindings sometimes return Promise<JsValue> with the JSON-serialized string,
 * sometimes the deserialized object directly depending on the serde path.
 */
function parseOpfs<T>(raw: unknown): OpfsResult<T> {
  if (typeof raw === "string") {
    return JSON.parse(raw) as OpfsResult<T>;
  }
  return raw as OpfsResult<T>;
}

/**
 * List all source files in the project's OPFS source store.
 * @returns Array of SourceFile metadata (id, path, name).
 */
export async function listSourceFiles(): Promise<SourceFile[]> {
  await waitForEngine();
  const parsed = parseOpfs<SourceFile[]>((window as any).list_source_files());
  if (!parsed.ok) throw new Error(parsed.error);
  return parsed.value!;
}

/**
 * Read the content of a source file by id.
 * @param id - The source file's id (which equals its path, e.g. "src/main")
 * @returns Object with ok:true and value:content string, or ok:false with error.
 */
export async function readSourceFile(
  id: string
): Promise<{ ok: true; value: string } | { ok: false; error: string }> {
  await waitForEngine();
  const parsed = parseOpfs<string>((window as any).read_source_file(id));
  if (parsed.ok) return { ok: true, value: parsed.value! };
  return { ok: false, error: parsed.error! };
}

/**
 * Write content to a source file by id.
 * Creates the file if it doesn't exist, overwrites if it does.
 * @param id - The source file's id (which equals its path, e.g. "src/main")
 * @param content - The new file content.
 * @returns Object with ok:true on success, or ok:false with error.
 */
export async function writeSourceFile(
  id: string,
  content: string
): Promise<{ ok: true } | { ok: false; error: string }> {
  await waitForEngine();
  const parsed = parseOpfs<null>((window as any).write_source_file(id, content));
  if (parsed.ok) return { ok: true };
  return { ok: false, error: parsed.error! };
}

/**
 * Create a new source file.
 * @param name - The display name for the file (e.g., "main.rs").
 *                The file path will be derived from the name.
 * @returns The id (path) of the created file.
 * @throws Error if the file already exists or creation fails.
 */
export async function createSourceFile(name: string): Promise<string> {
  await waitForEngine();
  // Derive path from name: "main.rs" -> "main", "src/lib.rs" -> "src/lib"
  const path = name.endsWith(".rs") ? name.slice(0, -3) : name;
  const parsed = parseOpfs<SourceFile>(
    (window as any).create_source_file(path, name)
  );
  if (!parsed.ok) throw new Error(parsed.error!);
  return parsed.value!.id;
}

/**
 * Delete a source file by id.
 * @param id - The source file's id (which equals its path).
 * @throws Error if the delete fails (e.g., file not found).
 */
export async function deleteSourceFile(id: string): Promise<void> {
  await waitForEngine();
  const parsed = parseOpfs<null>((window as any).delete_source_file(id));
  if (!parsed.ok) throw new Error(parsed.error!);
}
