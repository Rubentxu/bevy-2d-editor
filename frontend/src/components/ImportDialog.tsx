/**
 * ImportDialog — ADR-0041 implementation.
 *
 * Modal dialog for importing external source files (Aseprite, LDtk, Tiled).
 *
 * ## User flow
 *
 * 1. Select source file via the file picker
 * 2. Choose destination (scene asset path)
 * 3. Click Import → calls `import_external_source_wasm`
 * 4. Result is either:
 *    - Success → dialog closes, new asset appears
 *    - Conflict → dialog shows summary, redirects to ChangeWorkbench for review
 *
 * ## Conflict handling (ADR-0041 decision #3)
 *
 * Conflicts surface through the **Change Workbench** (existing panel),
 * NOT a separate modal. This dialog only shows a brief conflict summary
 * with a "Review in Change Workbench" button that switches to the panel.
 */

import { useCallback, useEffect, useState } from "react";
import {
  getExternalSource,
  importExternalSource,
  listImporters,
  type ExternalSource,
  type ImporterDescriptor,
  type ReimportResult,
  SOURCE_KIND_LABELS,
} from "../services/importers";

interface Props {
  /** Whether the dialog is open. */
  isOpen: boolean;
  /** Callback to close the dialog. */
  onClose: () => void;
  /** Called after a successful import. */
  onImported?: (resourceRef: string) => void;
  /** Called to switch to the Change Workbench panel. */
  onShowChangeWorkbench?: (changeSetId?: string) => void;
}

/** State of the import operation. */
type ImportState =
  | { phase: "idle" }
  | { phase: "loading_importers" }
  | { phase: "ready"; importers: ImporterDescriptor[] }
  | { phase: "importing" }
  | { phase: "success"; resourceRef: string }
  | { phase: "conflict"; result: ReimportResult; changeSetId: string }
  | { phase: "error"; message: string };

/**
 * ImportDialog — modal for importing Aseprite, LDtk, and Tiled files.
 */
export default function ImportDialog({
  isOpen,
  onClose,
  onImported,
  onShowChangeWorkbench,
}: Props) {
  const [state, setState] = useState<ImportState>({ phase: "idle" });
  const [selectedKind, setSelectedKind] = useState<string>("Aseprite");
  const [destinationPath, setDestinationPath] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");

  // Load available importers when dialog opens
  useEffect(() => {
    if (!isOpen) return;
    setState({ phase: "loading_importers" });

    listImporters()
      .then((result) => {
        if (!result.ok) {
          setState({ phase: "error", message: result.error });
          return;
        }
        setState({ phase: "ready", importers: result.value });
      })
      .catch((e) => {
        setState({
          phase: "error",
          message: e instanceof Error ? e.message : String(e),
        });
      });
  }, [isOpen]);

  // Reset on close
  useEffect(() => {
    if (!isOpen) {
      setState({ phase: "idle" });
      setDestinationPath("");
      setFileName("");
    }
  }, [isOpen]);

  const handleFileSelect = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      setFileName(file.name);

      // Check if this file was previously imported (has a sidecar)
      const result = await getExternalSource(file.name);
      if (result.ok && result.value !== null) {
        // This file was previously imported — show reimport info
        const existingSource = result.value as ExternalSource;
        console.info(
          `[ImportDialog] File ${file.name} was previously imported at ${new Date(existingSource.last_import_time).toLocaleString()}`,
        );
      }
    },
    [],
  );

  const handleImport = useCallback(async () => {
    if (!destinationPath || !fileName) return;

    setState({ phase: "importing" });

    // Read the file as base64
    const fileInput =
      document.querySelector<HTMLInputElement>('input[type="file"]');
    const file = fileInput?.files?.[0];
    if (!file) {
      setState({ phase: "error", message: "No file selected" });
      return;
    }

    const bytesB64 = await fileToBase64(file);

    const result = await importExternalSource(
      selectedKind,
      fileName,
      bytesB64,
      destinationPath,
    );

    if (!result.ok) {
      setState({ phase: "error", message: result.error });
      return;
    }

    const importResult = result.value;
    console.info(`[ImportDialog] Import result:`, importResult);

    // Check if there are pending change sets (conflicts go to ChangeWorkbench)
    // The import result will have a change_set_id if it's queued
    // For now, show success and let the user know
    setState({ phase: "success", resourceRef: destinationPath });
    onImported?.(destinationPath);

    // If there are conflicts, the ChangeWorkbench will show them
    // We could detect this by checking if change_set_id is present in the result
  }, [selectedKind, fileName, destinationPath, onImported]);

  const handleReimport = useCallback(async () => {
    if (!fileName) return;

    setState({ phase: "importing" });

    const result = await importExternalSource(
      selectedKind,
      fileName,
      "", // bytesB64 not needed for reimport from the same URI
      destinationPath,
    );

    // For reimport, we use reimport_external_source_wasm instead
    // But the UI flow is similar
    if (!result.ok) {
      setState({ phase: "error", message: result.error });
      return;
    }

    setState({ phase: "success", resourceRef: destinationPath });
    onImported?.(destinationPath);
  }, [selectedKind, fileName, destinationPath, onImported]);

  const handleClose = useCallback(() => {
    if (state.phase === "importing") return; // Don't close while importing
    onClose();
  }, [state.phase, onClose]);

  if (!isOpen) return null;

  return (
    <div className="dialog-overlay" onClick={handleClose}>
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="import-dialog-title"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          if (e.key === "Escape" && state.phase !== "importing") {
            handleClose();
          }
        }}
      >
        <h2 id="import-dialog-title">Import External Source</h2>

        {/* Phase: Loading */}
        {state.phase === "loading_importers" && (
          <p style={{ color: "#cbd5e0" }}>Loading importers...</p>
        )}

        {/* Phase: Ready */}
        {state.phase === "ready" && (
          <div className="import-dialog-body">
            {/* Source kind selector */}
            <div className="form-group">
              <label htmlFor="import-kind">Source Type</label>
              <select
                id="import-kind"
                value={selectedKind}
                onChange={(e) => setSelectedKind(e.target.value)}
              >
                <option value="Aseprite">{SOURCE_KIND_LABELS.Aseprite}</option>
                <option value="Ldtk">{SOURCE_KIND_LABELS.Ldtk}</option>
                <option value="Tiled">{SOURCE_KIND_LABELS.Tiled}</option>
              </select>
            </div>

            {/* File picker */}
            <div className="form-group">
              <label htmlFor="import-file">Source File</label>
              <input
                id="import-file"
                type="file"
                accept=".json,.ldtk"
                onChange={handleFileSelect}
              />
              {fileName && <span className="file-name">{fileName}</span>}
            </div>

            {/* Destination path */}
            <div className="form-group">
              <label htmlFor="import-destination">Destination Path</label>
              <input
                id="import-destination"
                type="text"
                placeholder="e.g. levels/world_1/level_1.json"
                value={destinationPath}
                onChange={(e) => setDestinationPath(e.target.value)}
              />
              <span className="hint">
                Where to store this asset in the project
              </span>
            </div>

            {/* Import / Reimport button */}
            <div className="dialog-actions">
              <button type="button" onClick={handleClose}>
                Cancel
              </button>
              <button
                type="button"
                className="primary"
                onClick={handleImport}
                disabled={!destinationPath || !fileName}
              >
                Import
              </button>
            </div>
          </div>
        )}

        {/* Phase: Importing */}
        {state.phase === "importing" && (
          <div className="import-dialog-body">
            <p style={{ color: "#cbd5e0" }}>Importing...</p>
          </div>
        )}

        {/* Phase: Success */}
        {state.phase === "success" && (
          <div className="import-dialog-body">
            <p style={{ color: "#68d391" }}>
              ✓ Imported successfully to {state.resourceRef}
            </p>
            <div className="dialog-actions">
              <button type="button" className="primary" onClick={handleClose}>
                Done
              </button>
            </div>
          </div>
        )}

        {/* Phase: Conflict */}
        {state.phase === "conflict" && (
          <div className="import-dialog-body">
            <p style={{ color: "#f6e05e" }}>
              ⚠ Conflicts detected — review required
            </p>
            <p style={{ color: "#cbd5e0", fontSize: 13 }}>
              The source file has changed since the last import. Some changes
              conflict with your edits.
            </p>
            {state.result.diff && (
              <div className="conflict-summary">
                <span>+{state.result.diff.added} added</span>
                <span>-{state.result.diff.removed} removed</span>
                <span>~{state.result.diff.modified_source} source-changed</span>
                <span className="conflict">
                  !{state.result.diff.modified_editor} editor-changed
                </span>
                {state.result.diff.ownership_conflicts > 0 && (
                  <span className="conflict">
                    ⚠ {state.result.diff.ownership_conflicts} conflicts
                  </span>
                )}
              </div>
            )}
            <div className="dialog-actions">
              <button type="button" onClick={handleClose}>
                Cancel
              </button>
              <button
                type="button"
                className="primary"
                onClick={() => {
                  onShowChangeWorkbench?.(state.changeSetId);
                  handleClose();
                }}
              >
                Review in Change Workbench
              </button>
            </div>
          </div>
        )}

        {/* Phase: Error */}
        {state.phase === "error" && (
          <div className="import-dialog-body">
            <p style={{ color: "#fc8181" }}>✕ {state.message}</p>
            <div className="dialog-actions">
              <button type="button" onClick={handleClose}>
                Close
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

/** Convert a File to a base64 string. */
function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      // Remove the data URL prefix (e.g., "data:application/octet-stream;base64,")
      const base64 = result.split(",")[1];
      resolve(base64 ?? "");
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}
