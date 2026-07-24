import { useState, useCallback, useRef, useEffect } from "react";
import {
  SceneAssetCatalogEntry,
  exportAssetToBsn,
} from "../services/scene-assets";
import {
  listSceneComponentSchemas,
  placeSceneComponentInstance,
  StaleSceneComponentBindingError,
} from "../services/scene-components";
import { importBsnAssetFromFile } from "../services/bsnImport";
import ThumbnailCell from "./ThumbnailCell";

interface Props {
  entries: SceneAssetCatalogEntry[];
  onCreate: (name: string, role: string) => Promise<void>;
  onRename: (assetId: string, newPath: string) => Promise<void>;
  onDuplicate: (assetId: string) => Promise<void>;
  onDelete: (assetId: string) => Promise<void>;
  onOpen: (assetId: string) => void;
  onPlaceInstance: (
    assetId: string,
    translation?: { x: number; y: number },
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

  // Hito 7 (scene-component-authoring-ux PR2): for each asset row, surface
  // "Place Instance (SceneComponent)" when a registered SceneComponent schema
  // references this asset's id via `bound_scene_asset_ref`. The lookup is
  // on-demand — it runs once on mount and re-runs whenever `entries` changes
  // (parent re-render), so newly-saved schemas are picked up via the
  // existing useSceneAssets refresh path. When the bound id no longer
  // resolves (stale), the click handler raises a typed error instead of a
  // silent success.
  const [bindingsByAsset, setBindingsByAsset] = useState<
    Record<string, string>
  >({});

  const refreshBindings = useCallback(async () => {
    if (typeof (window as any).list_scene_component_schemas !== "function") {
      return;
    }
    try {
      const schemas = await listSceneComponentSchemas();
      const next: Record<string, string> = {};
      for (const s of schemas) {
        const ref = s.bound_scene_asset_ref;
        if (typeof ref === "string" && ref.length > 0) {
          // Keep the first schema binding if multiple schemas share the same
          // asset (rare — schemas should be 1:1 with bound assets). The
          // click handler resolves and rejects stale refs at place time.
          next[ref] = s.type_id;
        }
      }
      setBindingsByAsset(next);
    } catch (e) {
      // list_scene_component_schemas may not be available in some contexts;
      // tolerate and fall back to no bindings rather than crashing the browser.
      console.warn("[ProjectAssetBrowser] refreshBindings failed:", e);
    }
  }, []);

  // On-demand refresh: mount + whenever the parent refreshes the catalog
  // (`entries` identity changes). Avoids the 500ms setInterval PR2 used
  // before — the bindings stay in sync because `entries` is the parent
  // refresh trigger.
  useEffect(() => {
    refreshBindings();
  }, [refreshBindings, entries]);

  const filteredEntries =
    roleFilter === "all"
      ? entries
      : entries.filter((e) => e.role === roleFilter);

  const handleRoleChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      setRoleFilter(e.target.value);
    },
    [],
  );

  const handleCreate = useCallback(async () => {
    const name = window.prompt("Scene Asset name:");
    if (!name) return;

    const role = window.prompt(
      `Scene Asset role (${ROLES.join(", ")}):`,
      "actor",
    );
    if (!role || !ROLES.includes(role as Role)) {
      alert(`Invalid role. Using "actor" as default.`);
      await onCreate(name, "actor");
      return;
    }
    await onCreate(name, role);
  }, [onCreate]);

  const handleRenameStart = useCallback((entry: SceneAssetCatalogEntry) => {
    setRenamingId(entry.asset_id);
    setRenameValue(entry.logical_path);
  }, []);

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
    [renameValue, onRename],
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
    [onDuplicate],
  );

  const handleDelete = useCallback(
    async (assetId: string) => {
      const entry = entries.find((e) => e.asset_id === assetId);
      if (!entry) return;
      const confirmed = window.confirm(
        `Delete Scene Asset "${entry.logical_path}"? This cannot be undone.`,
      );
      if (!confirmed) return;
      try {
        await onDelete(assetId);
      } catch (e) {
        console.error("Delete failed:", e);
      }
    },
    [entries, onDelete],
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
        window.alert(
          `Imported "${name}" successfully. You can open it from the asset list.`,
        );
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
    [],
  );

  const handlePlaceInstance = useCallback(
    async (assetId: string) => {
      // Show translation dialog (S1, E5)
      const translationStr = window.prompt(
        "Translation (optional, e.g. {x:100, y:200} or leave empty):",
      );
      let translation: { x: number; y: number } | undefined;
      if (translationStr && translationStr.trim()) {
        try {
          translation = JSON.parse(translationStr);
        } catch {
          // If parsing fails, try simple x,y format
          const match = translationStr.match(
            /x\s*:\s*([-\d.]+)\s*,\s*y\s*:\s*([-\d.]+)/i,
          );
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
    [onPlaceInstance],
  );

  // Hito 7 (scene-component-authoring-ux PR2 / S5, S7): Place Instance
  // directly from the asset row when a SceneComponent schema binds to this
  // asset. Translation prompt stays identical to the plain path so behaviour
  // is consistent across entry points. The click handler dispatches into
  // `placeSceneComponentInstance` which performs the stale-ref check, so
  // even if a stale binding somehow lingers in `bindingsByAsset`, the user
  // sees a typed error instead of a silent success.
  const handlePlaceSceneComponentInstance = useCallback(
    async (assetId: string) => {
      const typeId = bindingsByAsset[assetId];
      if (!typeId) {
        window.alert(
          "No SceneComponent schema is currently bound to this asset. Save a SceneComponent schema first.",
        );
        return;
      }
      const translationStr = window.prompt(
        "Translation (optional, e.g. {x:100, y:200} or leave empty):",
      );
      let translation: { x: number; y: number } | undefined;
      if (translationStr && translationStr.trim()) {
        try {
          translation = JSON.parse(translationStr);
        } catch {
          const match = translationStr.match(
            /x\s*:\s*([-\d.]+)\s*,\s*y\s*:\s*([-\d.]+)/i,
          );
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
        await placeSceneComponentInstance(typeId, translation);
      } catch (e) {
        if (e instanceof StaleSceneComponentBindingError) {
          window.alert(e.message);
        } else {
          window.alert(
            `Place Instance failed: ${e instanceof Error ? e.message : String(e)}`,
          );
        }
      } finally {
        setPlacingAssetId(null);
      }
    },
    [bindingsByAsset],
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
            <p>
              {roleFilter === "all"
                ? "No Scene Assets yet — click + to create your first one"
                : `No Scene Assets with role "${roleFilter}".`}
            </p>
          </div>
        ) : (
          <table className="asset-table" data-testid="asset-table">
            <thead>
              <tr>
                <th>Preview</th>
                <th>Name</th>
                <th>Role</th>
                <th>Version</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredEntries.map((entry) => (
                <tr
                  key={entry.asset_id}
                  data-testid={`asset-row-${entry.asset_id}`}
                  draggable={true}
                  onDragStart={(e) => {
                    if (e.dataTransfer) {
                      e.dataTransfer.setData(
                        "application/x-bevy-asset-id",
                        entry.asset_id,
                      );
                      e.dataTransfer.setData("text/plain", entry.logical_path);
                      e.dataTransfer.effectAllowed = "copy";
                    }
                  }}
                >
                  <td className="asset-preview">
                    <ThumbnailCell
                      assetId={entry.asset_id}
                      resourcePath={entry.preview_resource ?? null}
                    />
                  </td>
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
                    <span className="role-badge" data-testid="asset-role-badge">
                      {entry.role}
                    </span>
                  </td>
                  <td className="version">
                    <span data-testid="asset-version">
                      v{entry.current_version}
                    </span>
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
                      {placingAssetId === entry.asset_id
                        ? "Placing..."
                        : "Place Instance"}
                    </button>
                    {/* Hito 7 (PR2 / S5, S7): when a SceneComponent schema binds
                        this asset, surface a separate entry point that
                        resolves through `placeSceneComponentInstance`. The
                        stale-ref check lives in the service, so we always
                        enable the button when a binding is known — if the
                        reference becomes stale before the click, the typed
                        error is surfaced as an alert. */}
                    {bindingsByAsset[entry.asset_id] && (
                      <button
                        onClick={() =>
                          handlePlaceSceneComponentInstance(entry.asset_id)
                        }
                        data-testid="asset-place-scene-component-btn"
                        disabled={placingAssetId === entry.asset_id}
                        title={
                          "Place instance via bound SceneComponent schema (" +
                          bindingsByAsset[entry.asset_id] +
                          ")"
                        }
                        className="primary"
                      >
                        {placingAssetId === entry.asset_id
                          ? "Placing..."
                          : "Place (SceneComponent)"}
                      </button>
                    )}
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
