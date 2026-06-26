/**
 * OPFS (Origin Private File System) wrapper for the Bevy 2D Editor.
 *
 * Provides async file operations under a `bevy-2d-editor` namespace.
 * All functions return `{ok: boolean, value?: T, error?: string}` for
 * uniform error handling from Rust wasm_bindgen externs.
 */

const OPFS_ROOT_NAME = "bevy-2d-editor";

interface OpfsResult<T = void> {
  ok: boolean;
  value?: T;
  error?: string;
}

async function getRoot(): Promise<FileSystemDirectoryHandle | null> {
  if (!navigator.storage?.getDirectory) {
    return null;
  }
  const root = await navigator.storage.getDirectory();
  return root.getDirectoryHandle(OPFS_ROOT_NAME, { create: true });
}

async function getSubdir(
  segments: string[],
  createDirs: boolean = true
): Promise<FileSystemDirectoryHandle | null> {
  const root = await getRoot();
  if (!root) return null;
  let dir = root;
  for (const segment of segments) {
    try {
      const opts = createDirs ? { create: true } : { create: false };
      dir = await dir.getDirectoryHandle(segment, opts);
    } catch (e) {
      if (e instanceof DOMException && e.name === "NotFoundError") {
        return null;
      }
      throw e;
    }
  }
  return dir;
}

export async function opfsSaveFile(
  path: string,
  contents: string
): Promise<OpfsResult> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter((s) => s.length > 0);
    const filename = segments.pop();
    if (!filename) {
      return { ok: false, error: "Invalid path" };
    }
    const dir = await getSubdir(segments, true);
    if (!dir) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const fileHandle = await dir.getFileHandle(filename, { create: true });
    const writable = await fileHandle.createWritable();
    await writable.write(contents);
    await writable.close();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

export async function opfsLoadFile(path: string): Promise<OpfsResult<string>> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter((s) => s.length > 0);
    const filename = segments.pop();
    if (!filename) {
      return { ok: false, error: "Invalid path" };
    }
    const dir = await getSubdir(segments, false);
    if (!dir) {
      return { ok: false, error: "File not found" };
    }
    const fileHandle = await dir.getFileHandle(filename, { create: false });
    const file = await fileHandle.getFile();
    const text = await file.text();
    return { ok: true, value: text };
  } catch (e) {
    if (e instanceof DOMException && e.name === "NotFoundError") {
      return { ok: false, error: "File not found" };
    }
    return { ok: false, error: String(e) };
  }
}

export async function opfsListFiles(path: string): Promise<OpfsResult<string[]>> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter((s) => s.length > 0);
    const dir = await getSubdir(segments, false);
    if (!dir) {
      return { ok: true, value: [] };
    }
    const files: string[] = [];
    for await (const [name, handle] of dir.entries()) {
      if (handle.kind === "file") files.push(name);
    }
    return { ok: true, value: files };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}

export async function opfsExists(path: string): Promise<boolean> {
  try {
    if (!navigator.storage?.getDirectory) {
      return false;
    }
    const segments = path.split("/").filter((s) => s.length > 0);
    const filename = segments.pop();
    if (!filename) return false;
    const dir = await getSubdir(segments, false);
    if (!dir) return false;
    await dir.getFileHandle(filename, { create: false });
    return true;
  } catch {
    return false;
  }
}

export async function opfsDeleteFile(path: string): Promise<OpfsResult> {
  try {
    if (!navigator.storage?.getDirectory) {
      return { ok: false, error: "OPFS unavailable" };
    }
    const segments = path.split("/").filter((s) => s.length > 0);
    const filename = segments.pop();
    if (!filename) return { ok: false, error: "Invalid path" };
    const dir = await getSubdir(segments, false);
    if (!dir) return { ok: false, error: "File not found" };
    await dir.removeEntry(filename);
    return { ok: true };
  } catch (e) {
    if (e instanceof DOMException && e.name === "NotFoundError") {
      return { ok: false, error: "File not found" };
    }
    return { ok: false, error: String(e) };
  }
}