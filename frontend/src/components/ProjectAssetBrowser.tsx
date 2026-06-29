import { useState, useCallback } from "react";
import { SceneAssetCatalogEntry } from "../services/scene-assets";

interface Props {
  entries: SceneAssetCatalogEntry[];
  onCreate: (name: string, role: string) => Promise<void>;
  onRename: (assetId: string, newPath: string) => Promise<void>;
  onDuplicate: (assetId: string) => Promise<void>;
  onDelete: (assetId: string) => Promise<void>;
  onOpen: (assetId: string) => void;
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
}: Props) {
  const [roleFilter, setRoleFilter] = useState<string>("all");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

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
