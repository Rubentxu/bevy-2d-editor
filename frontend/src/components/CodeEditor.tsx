import { useCallback, useEffect, useMemo, useState, useRef } from "react";
import CodeMirror, { Extension, EditorView } from "@uiw/react-codemirror";
import { rust } from "@codemirror/lang-rust";
import { vscodeDark } from "@uiw/codemirror-theme-vscode";

import { useCodeFiles } from "../hooks/useCodeFiles";
import type { SourceFile } from "../services/code-files";
import PromptDialog from "./PromptDialog";
import ConfirmDialog from "./ConfirmDialog";
import type { NavigationTarget } from "../types/navigation";

/**
 * Props for CodeEditor — extends the hook's state with navigation support.
 */
export interface CodeEditorProps {
  /** Jump-to-source navigation target from App.tsx pendingNavigation state. */
  navigationTarget?: NavigationTarget | null;
  /** Called when the editor has scrolled to the navigation target. */
  onEditorReady?: () => void;
}

/**
 * Code editor surface — Rust syntax-highlighted CodeMirror 6 backed by
 * the Project's OPFS source store.
 *
 * Mirrors the LogicGraphEditor single-component grain (palette + canvas in one).
 * File list panel on the left; CodeMirror on the right.
 *
 * Save trigger: Ctrl+S (Windows/Linux) / Cmd+S (Mac).
 * Load/save failures surface as a dismissible error bar at the top of the editor.
 */
export default function CodeEditor({
  navigationTarget,
  onEditorReady,
}: CodeEditorProps = {}) {
  const {
    files,
    currentId,
    content,
    dirty,
    error,
    open,
    save,
    create,
    setContent,
    delete: deleteFile,
  } = useCodeFiles();

  // EditorView ref for programmatic scroll navigation
  const viewRef = useRef<EditorView | null>(null);

  // Scroll to target line when navigationTarget changes
  useEffect(() => {
    if (!navigationTarget || !viewRef.current) return;
    if (viewRef.current.state.doc.lines < navigationTarget.line) return;
    try {
      const lineStart = viewRef.current.state.doc.line(
        navigationTarget.line,
      ).from;
      viewRef.current.dispatch({
        selection: { anchor: lineStart },
        effects: EditorView.scrollIntoView(lineStart, { y: "center" }),
      });
      onEditorReady?.();
    } catch {
      // Line number out of range — ignore
    }
  }, [navigationTarget, onEditorReady]);

  // Error toast visibility
  const [errorVisible, setErrorVisible] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  // T3.2 — dialog state (replaces window.prompt/confirm)
  const [createFileDialogOpen, setCreateFileDialogOpen] = useState(false);
  const [deleteFileId, setDeleteFileId] = useState<string | null>(null);
  const [deleteFileName, setDeleteFileName] = useState("");

  // Show error toast whenever the hook reports an error.
  useEffect(() => {
    if (error) {
      setErrorMessage(error);
      setErrorVisible(true);
    }
  }, [error]);

  const dismissError = useCallback(() => {
    setErrorVisible(false);
    setErrorMessage(null);
  }, []);

  // Ctrl+S / Cmd+S → save
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        void save();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [save]);

  // onChange fires on every keystroke. Skip no-op programmatic syncs
  // by comparing against the hook's current content.
  const handleChange = useCallback(
    (value: string) => {
      if (value === content) return;
      setContent(value);
    },
    [setContent, content],
  );

  // Extensions for CodeMirror: Rust language + VS Code dark theme.
  const extensions = useMemo<Extension[]>(() => [rust()], []);

  const handleCreateFileSubmit = useCallback(
    async (name: string) => {
      setCreateFileDialogOpen(false);
      try {
        await create(name);
      } catch (e) {
        console.error("CodeEditor: create failed:", e);
      }
    },
    [create],
  );

  const handleDeleteFile = useCallback(
    (id: string) => {
      const file = files.find((f) => f.id === id);
      setDeleteFileId(id);
      setDeleteFileName(file?.name ?? id);
    },
    [files],
  );

  const handleDeleteFileConfirm = useCallback(async () => {
    if (!deleteFileId) return;
    await deleteFile(deleteFileId);
    setDeleteFileId(null);
    setDeleteFileName("");
  }, [deleteFileId, deleteFile]);

  const handleDeleteFileCancel = useCallback(() => {
    setDeleteFileId(null);
    setDeleteFileName("");
  }, []);

  const handleNewClick = useCallback(() => {
    setCreateFileDialogOpen(true);
  }, []);

  // ── Render ────────────────────────────────────────────────────────────────

  if (files.length === 0) {
    return (
      <div
        data-testid="code-editor"
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          height: "100%",
          color: "#888",
          fontFamily: "var(--font-mono, monospace)",
          fontSize: 14,
          gap: 12,
        }}
      >
        <span>No source files — click + to create one</span>
        <button
          onClick={handleNewClick}
          style={{ padding: "4px 12px", cursor: "pointer" }}
        >
          + Create one
        </button>
      </div>
    );
  }

  return (
    <div
      style={{ display: "flex", height: "100%", width: "100%" }}
      data-testid="code-editor"
    >
      {/* ── File list panel ────────────────────────────────────────────── */}
      <div
        style={{
          width: 200,
          borderRight: "1px solid #3c3c3c",
          display: "flex",
          flexDirection: "column",
          background: "#1e1e1e",
          color: "#cccccc",
          fontSize: 13,
        }}
      >
        <div
          style={{
            padding: "8px 8px 4px",
            fontWeight: 600,
            fontSize: 11,
            textTransform: "uppercase",
            color: "#888",
            letterSpacing: "0.05em",
          }}
        >
          Source Files
        </div>

        {/* File list */}
        <div style={{ flex: 1, overflowY: "auto" }}>
          {files.map((file: SourceFile) => (
            <FileListItem
              key={file.id}
              file={file}
              isActive={file.id === currentId}
              onOpen={() => void open(file.id)}
              onDelete={() => void handleDeleteFile(file.id)}
            />
          ))}
        </div>

        {/* New file button */}
        <div style={{ padding: 8, borderTop: "1px solid #3c3c3c" }}>
          <button
            onClick={handleNewClick}
            style={{
              width: "100%",
              padding: "4px 8px",
              cursor: "pointer",
              background: "#2d2d2d",
              color: "#cccccc",
              border: "1px solid #3c3c3c",
              borderRadius: 3,
              fontSize: 12,
            }}
          >
            + New File
          </button>
        </div>
      </div>

      {/* ── Editor panel ───────────────────────────────────────────────── */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>
        {/* Error toast */}
        {errorVisible && errorMessage && (
          <div
            style={{
              padding: "6px 12px",
              background: "#c0392b",
              color: "#fff",
              fontSize: 12,
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              cursor: "pointer",
            }}
            onClick={dismissError}
            title="Click to dismiss"
          >
            <span>{errorMessage}</span>
            <span style={{ opacity: 0.7 }}>✕</span>
          </div>
        )}

        {/* Status bar */}
        {currentId && (
          <div
            style={{
              padding: "4px 12px",
              background: "#252526",
              borderBottom: "1px solid #3c3c3c",
              fontSize: 11,
              color: dirty ? "#ce9178" : "#6a9955",
              fontFamily: "var(--font-mono, monospace)",
            }}
          >
            {files.find((f) => f.id === currentId)?.name ?? currentId}
            {dirty ? " • unsaved" : ""}
          </div>
        )}

        {/* CodeMirror editor */}
        {currentId ? (
          <div style={{ flex: 1, overflow: "hidden" }}>
            <CodeMirror
              value={content}
              height="100%"
              extensions={extensions}
              theme={vscodeDark}
              onChange={handleChange}
              style={{ height: "100%" }}
              basicSetup={true}
              onCreateEditor={(view) => {
                viewRef.current = view;
              }}
            />
          </div>
        ) : (
          <div
            style={{
              flex: 1,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              color: "#666",
              fontFamily: "var(--font-mono, monospace)",
              fontSize: 14,
            }}
          >
            Select a file from the list
          </div>
        )}
      </div>

      {/* T3.2 — in-app dialogs replacing window.prompt/confirm */}
      {createFileDialogOpen && (
        <PromptDialog
          title="New Source File"
          label="File name"
          placeholder="main.rs"
          defaultValue="main.rs"
          onConfirm={handleCreateFileSubmit}
          onCancel={() => setCreateFileDialogOpen(false)}
        />
      )}

      {deleteFileId && (
        <ConfirmDialog
          title="Delete Source File"
          message={`Delete "${deleteFileName}"? This cannot be undone.`}
          confirmLabel="Delete"
          onConfirm={handleDeleteFileConfirm}
          onCancel={handleDeleteFileCancel}
          danger
        />
      )}
    </div>
  );
}

// ── FileListItem ────────────────────────────────────────────────────────────

interface FileListItemProps {
  file: SourceFile;
  isActive: boolean;
  onOpen: () => void;
  onDelete: () => void;
}

function FileListItem({ file, isActive, onOpen, onDelete }: FileListItemProps) {
  return (
    <div
      onClick={onOpen}
      style={{
        padding: "6px 8px",
        cursor: "pointer",
        background: isActive ? "#37373d" : "transparent",
        color: isActive ? "#e0e0e0" : "#cccccc",
        borderLeft: isActive ? "2px solid #007acc" : "2px solid transparent",
        fontSize: 13,
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        userSelect: "none",
      }}
    >
      <span
        style={{
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          flex: 1,
        }}
        title={file.name}
      >
        {file.name}
      </span>
      <button
        onClick={(e) => {
          e.stopPropagation();
          onDelete();
        }}
        title={`Delete ${file.name}`}
        style={{
          background: "none",
          border: "none",
          color: "#888",
          cursor: "pointer",
          padding: "0 2px",
          fontSize: 11,
          lineHeight: 1,
          flexShrink: 0,
          marginLeft: 4,
        }}
      >
        ✕
      </button>
    </div>
  );
}
