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
import ComponentEditor from "./ComponentEditor";
import SchemaAuthoringPanel from "./SchemaAuthoringPanel";
import RuntimePreviewInspector from "./RuntimePreviewInspector";

interface Props {
  scene: SceneDocument | null;
  selectedId: string | null;
  onRename: (entityId: string, newName: string) => void;
  onSetField: (
    entityId: string,
    typeId: string,
    fieldPath: string,
    value: any,
  ) => void;
  onRemoveComponent: (entityId: string, typeId: string) => void;
  onAddComponent: (entityId: string, typeId: string) => void;
  // v0.82 P2 (ADR-0025): when more than one id is selected, the
  // inspector swaps to a multi-edit view that calls this when a
  // homogeneous field is committed. Without it, the multi-select view
  // remains read-only (still useful for inspection / Mixed markers
  // but no commits go through).
  onSetFieldOnMultiple?: (
    entityIds: string[],
    typeId: string,
    fieldPath: string,
    value: any,
  ) => void;
  // v0.82 P2 (ADR-0025): authoritative multi-select set. When its
  // size is > 1 the inspector renders the multi-edit view. When
  // exactly 1, that id is treated as the primary subject.
  selectedIds?: Set<string>;
  // Scene Instance operations (PR3)
  instances?: Record<string, SceneInstance>;
  onRemoveInstance?: (instanceId: string) => Promise<void>;
  onReplaceInstanceAsset?: (
    instanceId: string,
    newAssetId: string,
  ) => Promise<void>;
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
      <span
        className="instance-id"
        data-testid={`instance-id-${instance.instance_id}`}
      >
        {instance.instance_id.slice(0, 12)}...
      </span>
      <span
        className="instance-asset"
        data-testid={`instance-asset-${instance.instance_id}`}
      >
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

/**
 * Serialize any value to a stable string for divergence detection.
 * We deliberately ignore key order — JSON.stringify is fine because all
 * component values are emitted by the Rust serializer with sorted keys
 * (see editor-core `ComponentValue` ordering in v0.78). For arbitrary
 * user input this is still adequate for an equality check, which is the
 * only contract `aggregateField` relies on.
 */
function valueKey(v: any): string {
  if (v === null || v === undefined) return "null";
  if (typeof v === "number" || typeof v === "string" || typeof v === "boolean")
    return JSON.stringify(v);
  return JSON.stringify(v);
}

/**
 * Compute aggregated state for a single field across N entities.
 * Returns either a single homogeneous value or `{ kind: "mixed" }`.
 */
type FieldAggregate =
  | { kind: "homogeneous"; value: any }
  | { kind: "mixed"; sampleValues: any[] };

function aggregateField(values: any[]): FieldAggregate {
  if (values.length === 0) return { kind: "mixed", sampleValues: [] };
  const first = values[0];
  const k = valueKey(first);
  for (let i = 1; i < values.length; i++) {
    if (valueKey(values[i]) !== k) {
      // Cap the sample list at 3 to keep the tooltip readable.
      return {
        kind: "mixed",
        sampleValues: values.slice(0, 3),
      };
    }
  }
  return { kind: "homogeneous", value: first };
}

/**
 * MultiInspector: ADR-0025 F10. Renders the inspector body when 2+
 * entities are selected. For each component type that ALL selected
 * entities own, render a card with aggregated fields. Homogeneous
 * fields get the standard editor; divergent fields show a "— Mixed"
 * pill that, when activated, opens an overwrite input which dispatches
 * a single SetComponentFieldOnMultiple command.
 */
function MultiInspector({
  scene,
  selectedIds,
  onSetFieldOnMultiple,
}: {
  scene: SceneDocument;
  selectedIds: Set<string>;
  onSetFieldOnMultiple?: (
    entityIds: string[],
    typeId: string,
    fieldPath: string,
    value: any,
  ) => void;
}) {
  const ids = Array.from(selectedIds);
  const entities = ids
    .map((id) => scene.entities.find((e) => e.id === id))
    .filter((e): e is NonNullable<typeof e> => e !== undefined);

  // Component types that every selected entity owns (intersection).
  const commonTypeIds = entities.length
    ? entities[0].components
        .map((c) => c.type_id)
        .filter((typeId) =>
          entities.every((e) => e.components.some((c) => c.type_id === typeId)),
        )
    : [];

  return (
    <section
      className="inspector-multi"
      data-testid="inspector-multi"
      data-entity-count={entities.length}
      data-common-components={commonTypeIds.length}
    >
      <header
        className="inspector-multi-header"
        data-testid="inspector-multi-header"
      >
        <span className="inspector-multi-title">
          {entities.length} entities selected · {commonTypeIds.length}{" "}
          {commonTypeIds.length === 1 ? "component" : "components"} in common
        </span>
        {!onSetFieldOnMultiple && (
          <span
            className="inspector-multi-readonly"
            title="Multi-edit dispatcher not wired"
          >
            (read-only)
          </span>
        )}
      </header>
      {entities.length === 0 && (
        <div className="panel-empty">No matching entities in current scene</div>
      )}
      {entities.length > 0 && commonTypeIds.length === 0 && (
        <div
          className="panel-empty panel-empty-cta"
          data-testid="inspector-multi-no-common"
        >
          Selected entities share no components
        </div>
      )}
      {commonTypeIds.map((typeId) => {
        const comps = entities
          .map((e) => e.components.find((c) => c.type_id === typeId)!)
          .map((c) => c.values);
        // Field key set = union of all fields across the entities
        // (rarely different, but defensive). We render one row per
        // field present on the first entity; entities missing the
        // field would have been filtered by the commonTypeIds step
        // anyway (the intersection is by component presence only).
        const fieldSet = new Set<string>();
        for (const values of comps) {
          for (const k of Object.keys(values)) fieldSet.add(k);
        }
        const fields = Array.from(fieldSet);
        return (
          <div
            key={typeId}
            className="component-card multi"
            data-testid={`component-${typeId}`}
          >
            <header>
              <span className="type-id">{typeId}</span>
              <span className="multi-entity-count" title="Entities sharing this component">
                ×{entities.length}
              </span>
            </header>
            {fields.map((fieldPath) => {
              const valuesAcross = comps.map((v) => v[fieldPath]);
              const agg = aggregateField(valuesAcross);
              return (
                <MultiFieldRow
                  key={fieldPath}
                  fieldPath={fieldPath}
                  aggregate={agg}
                  disabled={!onSetFieldOnMultiple}
                  onCommit={(newValue) =>
                    onSetFieldOnMultiple?.(ids, typeId, fieldPath, newValue)
                  }
                />
              );
            })}
          </div>
        );
      })}
    </section>
  );
}

/**
 * MultiFieldRow: ADR-0025 F10. Per-field row inside a multi-inspector
 * component card. If the field is homogeneous across the selection,
 * renders the standard editor seeded with the common value. If
 * divergent, shows a "— Mixed" pill; clicking it reveals a single
 * overwrite input whose commit goes through onSetFieldOnMultiple.
 *
 * The editor is intentionally minimal here (number/text/checkbox/
 * JSON fallback) — we don't try to splice in Vec2/Color/Anchor
 * composite editors because the "Mixed" UX is meant to be a clear
 * "overwrite-all" affordance, not a per-axis diff view. A future
 * enhancement could split Vec2 axes into independent mixed markers.
 */
function MultiFieldRow({
  fieldPath,
  aggregate,
  disabled,
  onCommit,
}: {
  fieldPath: string;
  aggregate: FieldAggregate;
  disabled: boolean;
  onCommit: (newValue: any) => void;
}) {
  const [overriding, setOverriding] = useState(false);
  const [text, setText] = useState("");

  if (aggregate.kind === "homogeneous" && !overriding) {
    // Delegate to the standard editor for the common value.
    return (
      <div
        className="field-row multi homogeneous"
        data-testid={`field-row-${fieldPath}`}
        data-field-state="homogeneous"
      >
        <ComponentEditor
          fieldPath={fieldPath}
          value={aggregate.value}
          onCommit={disabled ? () => undefined : onCommit}
        />
      </div>
    );
  }

  if (!overriding) {
    // At this point we know the aggregate is mixed (the homogeneous
    // case returned above), so the `mixed` branch is the only one
    // that reaches here. Narrow explicitly so TS knows
    // `sampleValues` exists on the union.
    const sampleTooltip =
      aggregate.kind === "mixed"
        ? aggregate.sampleValues
            .map((v: any) => valueKey(v))
            .join(", ")
        : "";
    return (
      <div
        className="field-row multi mixed"
        data-testid={`field-row-${fieldPath}`}
        data-field-state="mixed"
      >
        <span className="field-label">{fieldPath}</span>
        <button
          type="button"
          className="mixed-pill"
          onClick={() => {
            if (disabled) return;
            setOverriding(true);
            setText(
              aggregate.kind === "homogeneous"
                ? String(aggregate.value ?? "")
                : "",
            );
          }}
          title={sampleTooltip ? `Sample values: ${sampleTooltip}` : "Mixed"}
          data-testid={`mixed-pill-${fieldPath}`}
        >
          — Mixed
        </button>
      </div>
    );
  }

  // Overriding: let the user type a value to write to all selected entities.
  return (
    <div
      className="field-row multi override"
      data-testid={`field-row-${fieldPath}`}
      data-field-state="overriding"
    >
      <span className="field-label">{fieldPath}</span>
      <input
        type="text"
        className="multi-override-input"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onBlur={() => {
          // Try JSON first; fall back to raw string.
          let parsed: any = text;
          try {
            parsed = JSON.parse(text);
          } catch {
            // keep raw string
          }
          onCommit(parsed);
          setOverriding(false);
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            (e.target as HTMLInputElement).blur();
          } else if (e.key === "Escape") {
            setOverriding(false);
          }
        }}
        autoFocus
        data-testid={`multi-override-${fieldPath}`}
      />
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
  onSetFieldOnMultiple,
  selectedIds,
  instances = {},
  onRemoveInstance,
  onReplaceInstanceAsset,
  assetEntries = [],
  onJumpToSource,
}: Props) {
  const entity = scene?.entities.find((e) => e.id === selectedId) ?? null;
  const [nameDraft, setNameDraft] = useState(entity?.name ?? "");
  const [componentQuery, setComponentQuery] = useState("");
  const [showSchemaPanel, setShowSchemaPanel] = useState(false);
  const [schemaRefreshKey, setSchemaRefreshKey] = useState(0);
  const [overrideIssues, setOverrideIssues] = useState<OverrideIssue[]>([]);
  const [resyncReports, setResyncReports] = useState<
    Array<[string, ResyncReport]>
  >([]);
  const [showOverrideDetails, setShowOverrideDetails] = useState(false);
  // Phase 6: Effective values + override indicators for instance entities
  const [resolvedEntity, setResolvedEntity] = useState<ResolvedEntity | null>(
    null,
  );
  const [fieldOverrideIndex, setFieldOverrideIndex] = useState<
    FieldOverrideEntry[]
  >([]);

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
  const refreshInstanceState = async (
    instance: SceneInstance,
    localId: string,
  ) => {
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
      `Remove this Scene Instance? This cannot be undone.`,
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
        .join("\n")}\n\nEnter asset_id:`,
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
  const selectedInstanceId =
    parseInstanceChild(entity?.id ?? "")?.instance_id ?? null;
  const selectedInstance = selectedInstanceId
    ? instances[selectedInstanceId]
    : null;

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
      await revertOverride(selectedInstance.instance_id, localId, typeId, [
        fieldPath,
      ]);
      // Re-use the same 4-step pipeline as the initial-load useEffect (W-N3 dup fix).
      await refreshInstanceState(selectedInstance, localId);
    } catch (e) {
      console.error("Revert override failed:", e);
    }
  };

  // Compute override status summary
  const overrideCounts = selectedInstance
    ? {
        active: selectedInstance.component_overrides.filter(
          (p) => p.status === "active",
        ).length,
        stale: selectedInstance.component_overrides.filter(
          (p) => p.status === "stale",
        ).length,
        orphaned:
          selectedInstance.component_overrides.filter(
            (p) => p.status === "orphaned",
          ).length +
          selectedInstance.orphaned_component_overrides.filter(
            (p) => p.status === "orphaned",
          ).length,
        conflict:
          selectedInstance.component_overrides.filter(
            (p) => p.status === "conflict",
          ).length +
          selectedInstance.orphaned_component_overrides.filter(
            (p) => p.status === "conflict",
          ).length,
      }
    : null;

  // Phase 6.4: Use resolvedEntity.components when instance entity selected
  const componentsToRender =
    isInstanceEntity && resolvedEntity
      ? resolvedEntity.components
      : (entity?.components ?? []);
  const normalizedComponentQuery = componentQuery.trim().toLowerCase();
  const visibleComponents = componentsToRender.filter((component) =>
    component.type_id.toLowerCase().includes(normalizedComponentQuery),
  );

  // Phase 6.6: Check if resync warning banner should be shown
  const showResyncWarning = resyncReports.some(
    ([, report]) => report.stale > 0 || report.conflict > 0,
  );
  const totalProblemCount = resyncReports.reduce(
    (sum, [, report]) => sum + report.stale + report.conflict,
    0,
  );

  return (
    <div className="panel inspector" data-testid="inspector-panel">
      <h2>Inspector</h2>
      <input
        type="search"
        className="panel-search"
        data-testid="inspector-search"
        placeholder="Search components…"
        aria-label="Search components"
        value={componentQuery}
        onChange={(event) => setComponentQuery(event.target.value)}
      />
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
          {visibleComponents.length === 0 && (
            <div className="panel-empty">
              {componentsToRender.length === 0
                ? "No components"
                : "No matching components"}
            </div>
          )}
          {visibleComponents.map((c) => {
            // Build per-component field override status lookup for this component
            const componentFieldStatus: Record<
              string,
              ComponentOverrideStatus
            > = {};
            for (const [key, status] of Object.entries(
              fieldOverrideStatusMap,
            )) {
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
                onCommit={(fieldPath, value) =>
                  onSetField(entity.id, c.type_id, fieldPath, value)
                }
                onRemove={() => onRemoveComponent(entity.id, c.type_id)}
                fieldOverrideStatus={
                  isInstanceEntity ? componentFieldStatus : undefined
                }
                onRevertField={
                  isInstanceEntity
                    ? (fieldPath) => handleRevertField(c.type_id, fieldPath)
                    : undefined
                }
                onJumpToSource={
                  onJumpToSource ? () => onJumpToSource(c.type_id) : undefined
                }
              />
            );
          })}
          <AddComponentButton
            key={schemaRefreshKey}
            entityId={entity.id}
            onAdd={(typeId) => onAddComponent(entity.id, typeId)}
          />
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
            <div
              className="resync-warning-banner"
              data-testid="resync-warning-banner"
            >
              <span className="resync-warning-icon">⚠️</span>
              <span className="resync-warning-text">
                {totalProblemCount} override{totalProblemCount !== 1 ? "s" : ""}{" "}
                need review (use per-field revert)
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
                  <span
                    className="override-count active"
                    title="Active component overrides"
                  >
                    {overrideCounts.active} active
                  </span>
                )}
                {overrideCounts.stale > 0 && (
                  <span
                    className="override-count stale"
                    title="Component overrides on renamed/removed fields"
                  >
                    {overrideCounts.stale} stale
                  </span>
                )}
                {overrideCounts.orphaned > 0 && (
                  <span
                    className="override-count orphaned"
                    title="Orphaned component overrides — entity removed from asset"
                  >
                    {overrideCounts.orphaned} orphaned
                  </span>
                )}
                {overrideCounts.conflict > 0 && (
                  <span
                    className="override-count conflict"
                    title="Type conflict component overrides"
                  >
                    {overrideCounts.conflict} conflict
                  </span>
                )}
              </div>
              {/* Resync reports for this instance */}
              {resyncReports.length > 0 && (
                <div className="resync-reports">
                  <span className="resync-label">Resync:</span>
                  {resyncReports.map(([id, report]) => (
                    <span
                      key={id}
                      className="resync-report"
                      data-testid={`resync-${id}`}
                    >
                      {report.active}a {report.stale}s {report.orphaned}o{" "}
                      {report.conflict}c
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
                  {overrideIssues.length} issue
                  {overrideIssues.length !== 1 ? "s" : ""}{" "}
                  {showOverrideDetails ? "▲" : "▼"}
                </button>
              )}
              {showOverrideDetails && overrideIssues.length > 0 && (
                <ul
                  className="override-issues-list"
                  data-testid="override-issues-list"
                >
                  {overrideIssues.map((issue, i) => (
                    <li
                      key={i}
                      className={`override-issue override-issue-${issue.code}`}
                      data-testid={`override-issue-${i}`}
                    >
                      <code className="override-issue-code">{issue.code}</code>
                      <span className="override-issue-message">
                        {issue.message}
                      </span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </>
      )}
      {/* v0.82 P2 (ADR-0025 F10): multi-edit view when >1 ids selected */}
      {scene && selectedIds && selectedIds.size > 1 && (
        <MultiInspector
          scene={scene}
          selectedIds={selectedIds}
          onSetFieldOnMultiple={onSetFieldOnMultiple}
        />
      )}
      {!entity && (
        <div
          className="panel-empty panel-empty-cta"
          data-testid="inspector-empty-cta"
        >
          <div className="panel-empty-title">No entity selected</div>
          <div className="panel-empty-subtitle">
            Click an entity in the Hierarchy to inspect it
          </div>
        </div>
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
