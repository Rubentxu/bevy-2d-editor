import { useEffect, useState } from "react";
import { SceneDocument } from "../hooks/useSceneState";
import {
  SceneInstance,
  OverrideIssue,
  ResyncReport,
  ResolvedEntity,
  ComponentOverrideStatus,
  FieldOverrideEntry,
  validateOverrides,
  getResyncReports,
  effectiveValues,
  overrideFieldStatus,
  revertOverride,
  parseInstanceChild,
  fetchAssetForInstance,
} from "../services/scene-assets";
import ComponentCard from "./ComponentCard";
import AddComponentButton from "./AddComponentButton";
import SchemaAuthoringPanel from "./SchemaAuthoringPanel";
import RuntimePreviewInspector from "./RuntimePreviewInspector";

interface Props {
  scene: SceneDocument | null;
  selectedId: string | null;
  onRename: (entityId: string, newName: string) => void;
  onSetField: (entityId: string, typeId: string, fieldPath: string, value: any) => void;
  onRemoveComponent: (entityId: string, typeId: string) => void;
  onAddComponent: (entityId: string, typeId: string) => void;
  // Scene Instance operations (PR3)
  instances?: Record<string, SceneInstance>;
  onRemoveInstance?: (instanceId: string) => Promise<void>;
  onReplaceInstanceAsset?: (instanceId: string, newAssetId: string) => Promise<void>;
  assetEntries?: Array<{ asset_id: string; logical_path: string }>;
  // Jump to source (rust-source-integration)
  onJumpToSource?: (typeId: string) => void;
}

/**
 * Renders a single Scene Instance row with remove and replace actions.
 */
function InstanceRow({
  instance,
  onRemove,
  onReplace,
  assetEntries,
}: {
  instance: SceneInstance;
  onRemove: () => void;
  onReplace: () => void;
  assetEntries?: Array<{ asset_id: string; logical_path: string }>;
}) {
  const isBroken = instance.asset_version_seen === 0;
  return (
    <div
      key={instance.instance_id}
      className={`instance-row ${isBroken ? "instance-broken" : ""}`}
      data-testid={`instance-row-${instance.instance_id}`}
    >
      <span className="instance-id" data-testid={`instance-id-${instance.instance_id}`}>
        {instance.instance_id.slice(0, 12)}...
      </span>
      <span className="instance-asset" data-testid={`instance-asset-${instance.instance_id}`}>
        {instance.asset_ref}
      </span>
      {isBroken && (
        <span
          className="instance-broken-badge"
          data-testid={`instance-broken-${instance.instance_id}`}
          title="Asset version mismatch — instance may be broken"
        >
          BROKEN
        </span>
      )}
      <div className="instance-actions">
        <button
          onClick={onReplace}
          data-testid={`instance-replace-btn-${instance.instance_id}`}
          disabled={!assetEntries || assetEntries.length === 0}
          title="Replace with different asset"
        >
          Replace
        </button>
        <button
          onClick={onRemove}
          data-testid={`instance-remove-btn-${instance.instance_id}`}
          className="danger"
          title="Remove instance from scene"
        >
          Remove
        </button>
      </div>
    </div>
  );
}

export default function InspectorPanel({
  scene,
  selectedId,
  onRename,
  onSetField,
  onRemoveComponent,
  onAddComponent,
  instances = {},
  onRemoveInstance,
  onReplaceInstanceAsset,
  assetEntries = [],
  onJumpToSource,
}: Props) {
  const entity = scene?.entities.find((e) => e.id === selectedId) ?? null;
  const [nameDraft, setNameDraft] = useState(entity?.name ?? "");
  const [showSchemaPanel, setShowSchemaPanel] = useState(false);
  const [schemaRefreshKey, setSchemaRefreshKey] = useState(0);
  const [overrideIssues, setOverrideIssues] = useState<OverrideIssue[]>([]);
  const [resyncReports, setResyncReports] = useState<Array<[string, ResyncReport]>>([]);
  const [showOverrideDetails, setShowOverrideDetails] = useState(false);
  // Phase 6: Effective values + override indicators for instance entities
  const [resolvedEntity, setResolvedEntity] = useState<ResolvedEntity | null>(null);
  const [fieldOverrideIndex, setFieldOverrideIndex] = useState<FieldOverrideEntry[]>([]);

  // Load override issues, resync reports, effective values, and field override status
  // when a scene instance entity is selected (Phase 6.2, 6.3)
  useEffect(() => {
    const parsed = entity ? parseInstanceChild(entity.id) : null;
    if (!entity || !parsed) {
      setOverrideIssues([]);
      setResyncReports([]);
      setResolvedEntity(null);
      setFieldOverrideIndex([]);
      return;
    }
    const instId = parsed.instance_id;
    const instance = instances[instId];
    if (!instance) {
      setOverrideIssues([]);
      setResyncReports([]);
      setResolvedEntity(null);
      setFieldOverrideIndex([]);
      return;
    }
    // Delegate the 4-step pipeline (asset → effective values → override index →
    // validate) to refreshInstanceState so handleRevertField stays in sync
    // (W-N3 useEffect dup, COUP-R5-02).
    refreshInstanceState(instance, parsed.local_id);

    // Load resync reports
    (async () => {
      try {
        const reports = await getResyncReports();
        // Filter to only this instance
        setResyncReports(reports.filter(([id]) => id === instId));
      } catch {
        setResyncReports([]);
      }
    })();
  }, [entity?.id, instances]);

  /** Whether the selected entity belongs to a Scene Instance. */
  const isInstanceEntity = !!(entity && parseInstanceChild(entity.id) !== null);

  /**
   * 4-step pipeline: load asset → resolve effective values → fetch override index.
   * Used by both the initial-load useEffect and handleRevertField to keep them
   * in sync. Extracted from duplication (W-N3 useEffect dup, COUP-R5-02).
   *
   * All callers pass `instance` and `localId`; the helper handles the
   * fetchAssetForInstance + null-check + per-call setState cascade.
   */
  const refreshInstanceState = async (instance: SceneInstance, localId: string) => {
    const asset = await fetchAssetForInstance(instance);
    if (!asset) {
      setOverrideIssues([]);
      setResolvedEntity(null);
      setFieldOverrideIndex([]);
      return;
    }

    // Load effective values (Phase 6.2)
    try {
      const resolved = await effectiveValues(instance, asset);
      const matching = resolved.entities[localId];
      setResolvedEntity(matching ?? null);
    } catch {
      setResolvedEntity(null);
    }

    // Load field override index (Phase 6.3)
    try {
      const index = await overrideFieldStatus(instance);
      setFieldOverrideIndex(index);
    } catch {
      setFieldOverrideIndex([]);
    }

    // Validate overrides
    try {
      const issues = await validateOverrides(instance, asset);
      setOverrideIssues(issues);
    } catch {
      setOverrideIssues([]);
    }
  };

  useEffect(() => {
    setNameDraft(entity?.name ?? "");
  }, [entity?.id, entity?.name]);

  const instanceList = Object.values(instances);

  const handleRemoveInstance = async (instanceId: string) => {
    if (!onRemoveInstance) return;
    const confirmed = window.confirm(
      `Remove this Scene Instance? This cannot be undone.`
    );
    if (!confirmed) return;
    try {
      await onRemoveInstance(instanceId);
    } catch (e) {
      console.error("Remove instance failed:", e);
    }
  };

  const handleReplaceInstance = async (instanceId: string) => {
    if (!onReplaceInstanceAsset || assetEntries.length === 0) return;
    const newAssetId = window.prompt(
      `Replace with which asset?\n\nAvailable assets:\n${assetEntries
        .map((e) => `${e.asset_id}: ${e.logical_path}`)
        .join("\n")}\n\nEnter asset_id:`
    );
    if (!newAssetId || !newAssetId.trim()) return;
    // Validate that the asset exists
    const exists = assetEntries.some((e) => e.asset_id === newAssetId.trim());
    if (!exists) {
      alert(`Asset "${newAssetId}" not found.`);
      return;
    }
    try {
      await onReplaceInstanceAsset(instanceId, newAssetId.trim());
    } catch (e) {
      console.error("Replace instance failed:", e);
    }
  };

  if (!scene) {
    return (
      <div className="panel inspector" data-testid="inspector-panel">
        <h2>Inspector</h2>
        <div className="panel-empty">No scene loaded</div>
      </div>
    );
  }

  // Show InstanceList section when there are instances OR when no entity selected
  const showInstanceList = instanceList.length > 0 || !entity;

  // Extract selected instance ID if entity is a scene instance child
  const selectedInstanceId = parseInstanceChild(entity?.id ?? "")?.instance_id ?? null;
  const selectedInstance = selectedInstanceId ? instances[selectedInstanceId] : null;

  // Build per-field override status map from fieldOverrideIndex (Phase 6.3)
  // Key: "component_type_id:field_name", Value: ComponentOverrideStatus
  const fieldOverrideStatusMap: Record<string, ComponentOverrideStatus> = {};
  for (const entry of fieldOverrideIndex) {
    const fieldName = entry.field_path[entry.field_path.length - 1];
    const key = `${entry.component_type_id}:${fieldName}`;
    fieldOverrideStatusMap[key] = entry.status;
  }

  // Phase 6.5: Revert a field override, then re-poll effective values + override status
  const handleRevertField = async (typeId: string, fieldPath: string) => {
    if (!selectedInstance || !entity) return;
    const localId = parseInstanceChild(entity.id)?.local_id;
    if (!localId) return;
    try {
      await revertOverride(selectedInstance.instance_id, localId, typeId, [fieldPath]);
      // Re-use the same 4-step pipeline as the initial-load useEffect (W-N3 dup fix).
      await refreshInstanceState(selectedInstance, localId);
    } catch (e) {
      console.error("Revert override failed:", e);
    }
  };

  // Compute override status summary
  const overrideCounts = selectedInstance
    ? {
        active: selectedInstance.component_overrides.filter((p) => p.status === "active").length,
        stale: selectedInstance.component_overrides.filter((p) => p.status === "stale").length,
        orphaned: selectedInstance.component_overrides.filter((p) => p.status === "orphaned").length
          + selectedInstance.orphaned_component_overrides.filter((p) => p.status === "orphaned").length,
        conflict: selectedInstance.component_overrides.filter((p) => p.status === "conflict").length
          + selectedInstance.orphaned_component_overrides.filter((p) => p.status === "conflict").length,
      }
    : null;

  // Phase 6.4: Use resolvedEntity.components when instance entity selected
  const componentsToRender = isInstanceEntity && resolvedEntity
    ? resolvedEntity.components
    : entity?.components ?? [];

  // Phase 6.6: Check if resync warning banner should be shown
  const showResyncWarning = resyncReports.some(
    ([, report]) => report.stale > 0 || report.conflict > 0
  );
  const totalProblemCount = resyncReports.reduce(
    (sum, [, report]) => sum + report.stale + report.conflict,
    0
  );

  return (
    <div className="panel inspector" data-testid="inspector-panel">
      <h2>Inspector</h2>
      {entity && (
        <>
          <input
            type="text"
            className="entity-name"
            value={nameDraft}
            onChange={(e) => setNameDraft(e.target.value)}
            onBlur={() => {
              if (nameDraft !== entity.name) {
                onRename(entity.id, nameDraft);
              }
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                (e.target as HTMLInputElement).blur();
              }
            }}
            data-testid={`entity-name-${entity.id}`}
          />
          {componentsToRender.length === 0 && (
            <div className="panel-empty">No components</div>
          )}
          {componentsToRender.map((c) => {
            // Build per-component field override status lookup for this component
            const componentFieldStatus: Record<string, ComponentOverrideStatus> = {};
            for (const [key, status] of Object.entries(fieldOverrideStatusMap)) {
              const [typeId, fieldName] = key.split(":");
              if (typeId === c.type_id) {
                componentFieldStatus[fieldName] = status;
              }
            }
            return (
              <ComponentCard
                key={c.type_id}
                component={c}
                entityId={entity.id}
                onCommit={(fieldPath, value) => onSetField(entity.id, c.type_id, fieldPath, value)}
                onRemove={() => onRemoveComponent(entity.id, c.type_id)}
                fieldOverrideStatus={isInstanceEntity ? componentFieldStatus : undefined}
                onRevertField={isInstanceEntity ? (fieldPath) => handleRevertField(c.type_id, fieldPath) : undefined}
                onJumpToSource={onJumpToSource ? () => onJumpToSource(c.type_id) : undefined}
              />
            );
          })}
          <AddComponentButton key={schemaRefreshKey} entityId={entity.id} onAdd={(typeId) => onAddComponent(entity.id, typeId)} />
          <div className="inspector-actions">
            <button
              type="button"
              className="new-schema-btn"
              onClick={() => setShowSchemaPanel(true)}
            >
              + New Schema
            </button>
          </div>
          {/* Phase 6.6: Resync warning banner.
              The button previously opened a dedicated Override/Resync Workbench
              which was never implemented. Per-field revert (via ComponentCard's
              revert button) is the current resolution path. Tracked as future
              enhancement for a full workbench UX. */}
          {isInstanceEntity && showResyncWarning && (
            <div className="resync-warning-banner" data-testid="resync-warning-banner">
              <span className="resync-warning-icon">⚠️</span>
              <span className="resync-warning-text">
                {totalProblemCount} override{totalProblemCount !== 1 ? "s" : ""} need review (use per-field revert)
              </span>
            </div>
          )}
          {/* Component Override Summary (override-resync-workbench) */}
          {overrideCounts && (
            <div className="override-summary" data-testid="override-summary">
              {/* Phase 6.4: Normalized "Overrides" section header with badges */}
              <h4 className="overrides-section-header">Overrides</h4>
              <div className="override-counts">
                {overrideCounts.active > 0 && (
                  <span className="override-count active" title="Active component overrides">
                    {overrideCounts.active} active
                  </span>
                )}
                {overrideCounts.stale > 0 && (
                  <span className="override-count stale" title="Component overrides on renamed/removed fields">
                    {overrideCounts.stale} stale
                  </span>
                )}
                {overrideCounts.orphaned > 0 && (
                  <span className="override-count orphaned" title="Orphaned component overrides — entity removed from asset">
                    {overrideCounts.orphaned} orphaned
                  </span>
                )}
                {overrideCounts.conflict > 0 && (
                  <span className="override-count conflict" title="Type conflict component overrides">
                    {overrideCounts.conflict} conflict
                  </span>
                )}
              </div>
              {/* Resync reports for this instance */}
              {resyncReports.length > 0 && (
                <div className="resync-reports">
                  <span className="resync-label">Resync:</span>
                  {resyncReports.map(([id, report]) => (
                    <span key={id} className="resync-report" data-testid={`resync-${id}`}>
                      {report.active}a {report.stale}s {report.orphaned}o {report.conflict}c
                    </span>
                  ))}
                </div>
              )}
              {/* Component override issues details */}
              {overrideIssues.length > 0 && (
                <button
                  type="button"
                  className="override-issues-toggle"
                  onClick={() => setShowOverrideDetails(!showOverrideDetails)}
                >
                  {overrideIssues.length} issue{overrideIssues.length !== 1 ? "s" : ""}{" "}
                  {showOverrideDetails ? "▲" : "▼"}
                </button>
              )}
              {showOverrideDetails && overrideIssues.length > 0 && (
                <ul className="override-issues-list" data-testid="override-issues-list">
                  {overrideIssues.map((issue, i) => (
                    <li
                      key={i}
                      className={`override-issue override-issue-${issue.code}`}
                      data-testid={`override-issue-${i}`}
                    >
                      <code className="override-issue-code">{issue.code}</code>
                      <span className="override-issue-message">{issue.message}</span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </>
      )}
      {!entity && (
        <div className="panel-empty">Select an entity</div>
      )}
      {showSchemaPanel && (
        <SchemaAuthoringPanel
          mode="create"
          onClose={() => setShowSchemaPanel(false)}
          onSaved={() => {
            setShowSchemaPanel(false);
            setSchemaRefreshKey((k) => k + 1);
          }}
        />
      )}
      {/* Scene Instances Section (PR3) */}
      {showInstanceList && (
        <div className="instance-list" data-testid="instance-list">
          <h3>Scene Instances</h3>
          {instanceList.length === 0 ? (
            <div className="panel-empty">No instances</div>
          ) : (
            instanceList.map((inst) => (
              <InstanceRow
                key={inst.instance_id}
                instance={inst}
                onRemove={() => handleRemoveInstance(inst.instance_id)}
                onReplace={() => handleReplaceInstance(inst.instance_id)}
                assetEntries={assetEntries}
              />
            ))
          )}
        </div>
      )}

      {/* Runtime Preview tab — live Bevy preview inspection */}
      <div className="preview-tab-section" data-testid="preview-tab-section">
        <RuntimePreviewInspector />
      </div>
    </div>
  );
}
