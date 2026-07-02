import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import CodeMirror, { Extension } from "@uiw/react-codemirror";
import { rust } from "@codemirror/lang-rust";
import { vscodeDark } from "@uiw/codemirror-theme-vscode";

import { useCodeFiles } from "../hooks/useCodeFiles";
import type { SourceFile } from "../services/code-files";

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
export default function CodeEditor() {
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

  // Error toast visibility
  const [errorVisible, setErrorVisible] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

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

  // Imperative handle to the CodeMirror EditorView.
  // Used to programatically set content when the selected file changes
  // without triggering the onChange handler.
  const editorViewRef = useRef<{ view?: import("@codemirror/view").EditorView }>(
    {}
  );

  // Track the last content we synced FROM the hook INTO the editor.
  // If incoming content differs from this, it is an external change (file open)
  // and we must NOT mark dirty when CM6's onChange fires.
  const lastSyncedContentRef = useRef<string>("");

  // When the hook's content changes (file opened or saved), sync it into
  // the editor DOM imperatively so CM6's onChange does not fire.
  useEffect(() => {
    const view = editorViewRef.current.view;
    if (!view) return;
    if (content === lastSyncedContentRef.current) return;

    lastSyncedContentRef.current = content;
    view.dispatch({
      changes: {
        from: 0,
        to: view.state.doc.length,
        insert: content,
      },
    });
  }, [content]);

  const handleChange = useCallback(
    (value: string) => {
      // This fires on every keystroke in CM6. The first time we see a
      // change that matches the hook's current content, it is a no-op from
      // the programmatic sync above — skip it to avoid spurious dirty flag.
      if (value === lastSyncedContentRef.current) return;
      lastSyncedContentRef.current = value;
      setContent(value);
    },
    [setContent]
  );

  // Extensions for CodeMirror: Rust language + VS Code dark theme.
  const extensions = useMemo<Extension[]>(() => [rust()], []);

  const handleCreateFile = useCallback(async () => {
    const name = window.prompt("File name:", "main.rs");
    if (!name) return;
    try {
      await create(name);
    } catch (e) {
      console.error("CodeEditor: create failed:", e);
    }
  }, [create]);

  const handleDeleteFile = useCallback(
    async (id: string) => {
      // eslint-disable-next-line no-alert
      if (!window.confirm(`Delete "${files.find((f) => f.id === id)?.name}"?`))
        return;
      await deleteFile(id);
    },
    [files, deleteFile]
  );

  const handleNewClick = useCallback(async () => {
    await handleCreateFile();
  }, [handleCreateFile]);

  // ── Render ────────────────────────────────────────────────────────────────

  if (files.length === 0) {
    return (
      <div
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
        <span>No source files yet.</span>
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
    <div style={{ display: "flex", height: "100%", width: "100%" }}>
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
              ref={editorViewRef as any}
              style={{ height: "100%" }}
              basicSetup={{
                lineNumbers: true,
                highlightActiveLineGutter: true,
                highlightSpecialChars: true,
                foldGutter: true,
                drawSelection: true,
                dropCursor: true,
                allowMultipleSelections: true,
                indentOnInput: true,
                syntaxHighlighting: true,
                bracketMatching: true,
                closeBrackets: true,
                autocompletion: true,
                rectangularSelection: true,
                crosshairCursor: true,
                highlightActiveLine: true,
                highlightSelectionMatches: true,
                closeBracketsKeymap: true,
                defaultKeymap: true,
                searchKeymap: true,
                historyKeymap: true,
                foldKeymap: true,
                completionKeymap: true,
                lintKeymap: true,
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
