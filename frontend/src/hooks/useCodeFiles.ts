import { useEffect, useState, useCallback, useRef } from "react";
import {
  SourceFile,
  listSourceFiles,
  readSourceFile,
  writeSourceFile,
  createSourceFile,
  deleteSourceFile,
} from "../services/code-files";

/**
 * React hook for source file state and operations.
 *
 * Manages:
 * - File list (list of all source files)
 * - Current file id and content
 * - Dirty state (content has unsaved edits, computed against last-saved snapshot)
 * - Error state (last operation error)
 * - 500ms polling refresh for file list
 *
 * Does NOT manage undo/redo — source files are raw text edits without
 * an operation log (per design.md §Scope discipline).
 *
 * Race-condition guards:
 * - `setContent` only marks dirty when content actually changes (vs last-saved snapshot)
 * - `save` captures current id at invocation; only clears dirty if no keystroke during await
 * - `deleteFile` reads current id from a ref (not closure) to avoid stale read on rapid switch
 * - `refresh` does NOT clear errors from other operations (toast flicker prevention)
 */
export function useCodeFiles() {
  const [files, setFiles] = useState<SourceFile[]>([]);
  const [currentId, setCurrentId] = useState<string | null>(null);
  const [content, setContentState] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  // lastSavedContent: snapshot of content at the last successful save (or open).
  // dirty is derived: content !== lastSavedContent. Avoids "always dirty" bug
  // when CM6 onChange fires on programmatic state sync.
  const [lastSavedContent, setLastSavedContent] = useState<string>("");
  const dirty = content !== lastSavedContent;

  // currentIdRef lets async callbacks (deleteFile) read the LATEST id rather
  // than the closure-captured one. Prevents the stale-closure race when the
  // user rapidly opens a different file after triggering a delete.
  const currentIdRef = useRef<string | null>(null);
  useEffect(() => {
    currentIdRef.current = currentId;
  }, [currentId]);

  /**
   * Refresh the file list from the backend. Does NOT clear errors.
   */
  const refresh = useCallback(async () => {
    try {
      const list = await listSourceFiles();
      setFiles(list);
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
      setLastSavedContent(result.value);
      setError(null);
    } catch (e) {
      console.error("useCodeFiles: open failed:", e);
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  /**
   * Save the current content to OPFS.
   * Captures currentId at invocation time to avoid stale-closure race
   * if the user switches files during the in-flight save.
   */
  const save = useCallback(async () => {
    const idAtCall = currentIdRef.current;
    if (!idAtCall) return;
    try {
      const result = await writeSourceFile(idAtCall, content);
      if (!result.ok) {
        setError(result.error);
        return;
      }
      // Only update lastSavedContent (which clears dirty) if id hasn't
      // changed during the await. Prevents overwriting the new file's
      // dirty state with the old file's saved state.
      if (currentIdRef.current === idAtCall) {
        setLastSavedContent(content);
      }
      setError(null);
    } catch (e) {
      console.error("useCodeFiles: save failed:", e);
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [content]);

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
        // Note: create does NOT call refresh() on error — stale error state
        // is preferred over flicker from a failed refresh (per refresh guard).
      }
    },
    [refresh, open]
  );

  /**
   * Update the in-memory content. Dirty is derived (see top of hook),
   * so no-op setContent calls do not mark dirty.
   * @param s - New content string
   */
  const setContent = useCallback((s: string) => {
    setContentState(s);
  }, []);

  /**
   * Delete a source file by id.
   * If the deleted file is currently open, resets state.
   * Uses currentIdRef to avoid stale-closure on rapid file switch.
   * @param id - The source file's id
   */
  const deleteFile = useCallback(
    async (id: string) => {
      try {
        await deleteSourceFile(id);
        if (id === currentIdRef.current) {
          setCurrentId(null);
          setContentState("");
          setLastSavedContent("");
        }
        await refresh();
        setError(null);
      } catch (e) {
        console.error("useCodeFiles: delete failed:", e);
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [refresh]
  );

  // Initial fetch + 500ms polling refresh (same cadence as useScenes).
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
