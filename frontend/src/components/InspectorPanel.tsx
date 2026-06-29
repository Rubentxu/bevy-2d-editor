import { useEffect, useState } from "react";
import { SceneDocument } from "../hooks/useSceneState";
import { SceneInstance } from "../services/scene-assets";
import ComponentCard from "./ComponentCard";
import AddComponentButton from "./AddComponentButton";
import SchemaAuthoringPanel from "./SchemaAuthoringPanel";

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
}: Props) {
  const entity = scene?.entities.find((e) => e.id === selectedId) ?? null;
  const [nameDraft, setNameDraft] = useState(entity?.name ?? "");
  const [showSchemaPanel, setShowSchemaPanel] = useState(false);
  const [schemaRefreshKey, setSchemaRefreshKey] = useState(0);

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
          {entity.components.length === 0 && (
            <div className="panel-empty">No components</div>
          )}
          {entity.components.map((c) => (
            <ComponentCard
              key={c.type_id}
              component={c}
              entityId={entity.id}
              onCommit={(fieldPath, value) => onSetField(entity.id, c.type_id, fieldPath, value)}
              onRemove={() => onRemoveComponent(entity.id, c.type_id)}
            />
          ))}
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
    </div>
  );
}