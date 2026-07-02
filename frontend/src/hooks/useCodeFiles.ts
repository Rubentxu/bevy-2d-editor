import { useEffect, useState, useCallback } from "react";
import {
  SourceFile,
  listSourceFiles,
  readSourceFile,
  writeSourceFile,
  createSourceFile,
  deleteSourceFile,
} from "../services/code-files";

const DEFAULT_FILES: SourceFile[] = [];

/**
 * React hook for source file state and operations.
 *
 * Manages:
 * - File list (list of all source files)
 * - Current file id and content
 * - Dirty state (content has unsaved edits)
 * - Error state (last operation error)
 * - 500ms polling refresh for file list
 *
 * Does NOT manage undo/redo — source files are raw text edits without
 * an operation log (per design.md §Scope discipline).
 */
export function useCodeFiles() {
  const [files, setFiles] = useState<SourceFile[]>(DEFAULT_FILES);
  const [currentId, setCurrentId] = useState<string | null>(null);
  const [content, setContentState] = useState<string>("");
  const [dirty, setDirty] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Refresh the file list from the backend.
   */
  const refresh = useCallback(async () => {
    try {
      const list = await listSourceFiles();
      setFiles(list);
      setError(null);
    } catch (e) {
      console.error("useCodeFiles: refresh failed:", e);
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  /**
   * Open a source file by id, loading its content from OPFS.
   * @param id - The source file's id (path)
   */
  const open = useCallback(async (id: string) => {
    try {
      const result = await readSourceFile(id);
      if (!result.ok) {
        setError(result.error);
        return;
      }
      setCurrentId(id);
      setContentState(result.value);
      setDirty(false);
      setError(null);
    } catch (e) {
      console.error("useCodeFiles: open failed:", e);
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  /**
   * Save the current content to OPFS.
   */
  const save = useCallback(async () => {
    if (!currentId) return;
    try {
      const result = await writeSourceFile(currentId, content);
      if (!result.ok) {
        setError(result.error);
        return;
      }
      setDirty(false);
      setError(null);
    } catch (e) {
      console.error("useCodeFiles: save failed:", e);
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [currentId, content]);

  /**
   * Create a new source file and open it.
   * @param name - Display name (e.g., "main.rs")
   */
  const create = useCallback(
    async (name: string) => {
      try {
        const id = await createSourceFile(name);
        await refresh();
        await open(id);
        setError(null);
      } catch (e) {
        console.error("useCodeFiles: create failed:", e);
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [refresh, open]
  );

  /**
   * Update the in-memory content (marks dirty).
   * @param s - New content string
   */
  const setContent = useCallback((s: string) => {
    setContentState(s);
    setDirty(true);
  }, []);

  /**
   * Delete a source file by id.
   * If the deleted file is currently open, resets state.
   * @param id - The source file's id
   */
  const deleteFile = useCallback(
    async (id: string) => {
      try {
        await deleteSourceFile(id);
        if (id === currentId) {
          setCurrentId(null);
          setContentState("");
          setDirty(false);
        }
        await refresh();
        setError(null);
      } catch (e) {
        console.error("useCodeFiles: delete failed:", e);
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [currentId, refresh]
  );

  // Poll for file list every 500ms (same cadence as useScenes)
  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, 500);
    return () => clearInterval(interval);
  }, [refresh]);

  return {
    // State
    files,
    currentId,
    content,
    dirty,
    error,

    // Operations
    open,
    save,
    create,
    setContent,
    delete: deleteFile,
    refresh,
  };
}
