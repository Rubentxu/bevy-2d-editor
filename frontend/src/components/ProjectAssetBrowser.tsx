import { useState, useCallback, useRef } from "react";
import {
  SceneAssetCatalogEntry,
  exportAssetToBsn,
} from "../services/scene-assets";
import { importBsnAssetFromFile } from "../services/bsnImport";

interface Props {
  entries: SceneAssetCatalogEntry[];
  onCreate: (name: string, role: string) => Promise<void>;
  onRename: (assetId: string, newPath: string) => Promise<void>;
  onDuplicate: (assetId: string) => Promise<void>;
  onDelete: (assetId: string) => Promise<void>;
  onOpen: (assetId: string) => void;
  onPlaceInstance: (
    assetId: string,
    translation?: { x: number; y: number }
  ) => Promise<void>;
}

const ROLES = ["actor", "level", "ui", "fragment", "screen", "effect"] as const;
type Role = (typeof ROLES)[number];

export default function ProjectAssetBrowser({
  entries,
  onCreate,
  onRename,
  onDuplicate,
  onDelete,
  onOpen,
  onPlaceInstance,
}: Props) {
  const [roleFilter, setRoleFilter] = useState<string>("all");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [placingAssetId, setPlacingAssetId] = useState<string | null>(null);
  const [exportingAssetId, setExportingAssetId] = useState<string | null>(null);
  const bsnFileInputRef = useRef<HTMLInputElement>(null);

  const filteredEntries =
    roleFilter === "all"
      ? entries
      : entries.filter((e) => e.role === roleFilter);

  const handleRoleChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      setRoleFilter(e.target.value);
    },
    []
  );

  const handleCreate = useCallback(async () => {
    const name = window.prompt("Scene Asset name:");
    if (!name) return;

    const role = window.prompt(
      `Scene Asset role (${ROLES.join(", ")}):`,
      "actor"
    );
    if (!role || !ROLES.includes(role as Role)) {
      alert(`Invalid role. Using "actor" as default.`);
      await onCreate(name, "actor");
      return;
    }
    await onCreate(name, role);
  }, [onCreate]);

  const handleRenameStart = useCallback(
    (entry: SceneAssetCatalogEntry) => {
      setRenamingId(entry.asset_id);
      setRenameValue(entry.logical_path);
    },
    []
  );

  const handleRenameConfirm = useCallback(
    async (assetId: string) => {
      if (!renameValue.trim()) {
        setRenamingId(null);
        return;
      }
      try {
        await onRename(assetId, renameValue.trim());
      } catch (e) {
        console.error("Rename failed:", e);
      }
      setRenamingId(null);
      setRenameValue("");
    },
    [renameValue, onRename]
  );

  const handleRenameCancel = useCallback(() => {
    setRenamingId(null);
    setRenameValue("");
  }, []);

  const handleDuplicate = useCallback(
    async (assetId: string) => {
      try {
        await onDuplicate(assetId);
      } catch (e) {
        console.error("Duplicate failed:", e);
      }
    },
    [onDuplicate]
  );

  const handleDelete = useCallback(
    async (assetId: string) => {
      const entry = entries.find((e) => e.asset_id === assetId);
      if (!entry) return;
      const confirmed = window.confirm(
        `Delete Scene Asset "${entry.logical_path}"? This cannot be undone.`
      );
      if (!confirmed) return;
      try {
        await onDelete(assetId);
      } catch (e) {
        console.error("Delete failed:", e);
      }
    },
    [entries, onDelete]
  );

  const handleExportBsn = useCallback(async (assetId: string) => {
    setExportingAssetId(assetId);
    try {
      const bsnText = await exportAssetToBsn(assetId);
      const blob = new Blob([bsnText], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${assetId}.bsn`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error("[ProjectAssetBrowser] Export .bsn failed:", e);
      window.alert(`Export failed: ${e}`);
    } finally {
      setExportingAssetId(null);
    }
  }, []);

  const handleImportBsn = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      // Strip .bsn extension if present for use as default name
      const defaultName = file.name.replace(/\.bsn$/i, "");
      const name = window.prompt("Asset name:", defaultName);
      if (!name || !name.trim()) {
        // User cancelled or gave empty name
        return;
      }

      try {
        const entryJson = await importBsnAssetFromFile(name.trim(), file);
        const entry = JSON.parse(entryJson);
        // Notify parent so it can refresh the catalog and open the new asset
        if ((window as any).refreshAssetCatalog) {
          (window as any).refreshAssetCatalog();
        }
        window.alert(`Imported "${name}" successfully. You can open it from the asset list.`);
      } catch (err) {
        console.error("[ProjectAssetBrowser] Import .bsn failed:", err);
        window.alert(`Import failed: ${err}`);
      } finally {
        // Reset the input so the same file can be re-selected
        if (bsnFileInputRef.current) {
          bsnFileInputRef.current.value = "";
        }
      }
    },
    []
  );

  const handlePlaceInstance = useCallback(
    async (assetId: string) => {
      // Show translation dialog (S1, E5)
      const translationStr = window.prompt(
        "Translation (optional, e.g. {x:100, y:200} or leave empty):"
      );
      let translation: { x: number; y: number } | undefined;
      if (translationStr && translationStr.trim()) {
        try {
          translation = JSON.parse(translationStr);
        } catch {
          // If parsing fails, try simple x,y format
          const match = translationStr.match(/x\s*:\s*([-\d.]+)\s*,\s*y\s*:\s*([-\d.]+)/i);
          if (match) {
            translation = {
              x: parseFloat(match[1]),
              y: parseFloat(match[2]),
            };
          }
        }
      }
      setPlacingAssetId(assetId);
      try {
        await onPlaceInstance(assetId, translation);
      } catch (e) {
        console.error("Place instance failed:", e);
      } finally {
        setPlacingAssetId(null);
      }
    },
    [onPlaceInstance]
  );

  return (
    <div className="project-asset-browser" data-testid="project-asset-browser">
      <div className="browser-header">
        <h2>Project Assets</h2>
        <button
          onClick={handleCreate}
          data-testid="create-asset-btn"
          className="primary"
        >
          + Create Scene Asset
        </button>
        <button
          onClick={() => bsnFileInputRef.current?.click()}
          data-testid="import-bsn-btn"
          className="secondary"
          title="Import a .bsn file"
        >
          Import .bsn
        </button>
        <input
          ref={bsnFileInputRef}
          type="file"
          accept=".bsn"
          style={{ display: "none" }}
          onChange={handleImportBsn}
          data-testid="bsn-file-input"
        />
      </div>

      <div className="browser-filters">
        <label>
          Filter by role:
          <select
            value={roleFilter}
            onChange={handleRoleChange}
            data-testid="role-filter-select"
          >
            <option value="all">All</option>
            {ROLES.map((role) => (
              <option key={role} value={role}>
                {role.charAt(0).toUpperCase() + role.slice(1)}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="browser-content">
        {filteredEntries.length === 0 ? (
          <div className="empty-state" data-testid="asset-browser-empty">
            <p>No Scene Assets found.</p>
            <p>
              {roleFilter === "all"
                ? "Create your first Scene Asset to get started."
                : `No assets with role "${roleFilter}".`}
            </p>
          </div>
        ) : (
          <table className="asset-table" data-testid="asset-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Role</th>
                <th>Version</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredEntries.map((entry) => (
                <tr key={entry.asset_id} data-testid={`asset-row-${entry.asset_id}`}>
                  <td className="asset-name">
                    {renamingId === entry.asset_id ? (
                      <input
                        type="text"
                        value={renameValue}
                        onChange={(e) => setRenameValue(e.target.value)}
                        onBlur={() => handleRenameConfirm(entry.asset_id)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter")
                            handleRenameConfirm(entry.asset_id);
                          if (e.key === "Escape") handleRenameCancel();
                        }}
                        autoFocus
                        data-testid="rename-input"
                      />
                    ) : (
                      <span data-testid="asset-name">{entry.logical_path}</span>
                    )}
                  </td>
                  <td>
                    <span
                      className="role-badge"
                      data-testid="asset-role-badge"
                    >
                      {entry.role}
                    </span>
                  </td>
                  <td className="version">
                    <span data-testid="asset-version">v{entry.current_version}</span>
                  </td>
                  <td className="actions">
                    <button
                      onClick={() => onOpen(entry.asset_id)}
                      data-testid="asset-open-btn"
                      title="Open for editing"
                    >
                      Open
                    </button>
                    <button
                      onClick={() => handlePlaceInstance(entry.asset_id)}
                      data-testid="asset-place-btn"
                      disabled={placingAssetId === entry.asset_id}
                      title="Place instance in scene"
                    >
                      {placingAssetId === entry.asset_id ? "Placing..." : "Place Instance"}
                    </button>
                    <button
                      onClick={() => handleRenameStart(entry)}
                      data-testid="asset-rename-btn"
                      disabled={renamingId !== null}
                      title="Rename asset"
                    >
                      Rename
                    </button>
                    <button
                      onClick={() => handleDuplicate(entry.asset_id)}
                      data-testid="asset-duplicate-btn"
                      title="Duplicate asset"
                    >
                      Duplicate
                    </button>
                    <button
                      onClick={() => handleDelete(entry.asset_id)}
                      data-testid="asset-delete-btn"
                      className="danger"
                      title="Delete asset"
                    >
                      Delete
                    </button>
                    <button
                      onClick={() => handleExportBsn(entry.asset_id)}
                      data-testid="asset-export-bsn-btn"
                      disabled={exportingAssetId === entry.asset_id}
                      title="Export as .bsn file"
                    >
                      {exportingAssetId === entry.asset_id
                        ? "Exporting…"
                        : "Export .bsn"}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
