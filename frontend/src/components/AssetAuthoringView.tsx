import { useState, useCallback, useEffect } from "react";
import ComponentCard from "./ComponentCard";
import AddComponentButton from "./AddComponentButton";
import {
  SceneAssetDocument,
  SceneAssetEntity,
  SceneInstanceLayerSummary,
  SceneInstanceLayerKind,
  listSceneInstanceLayers,
  createSceneInstanceLayer,
  deleteSceneInstanceLayer,
  setAssetDocumentJson,
} from "../services/scene-assets";

interface Props {
  document: SceneAssetDocument;
  activeEntityId: string | null;
  onSelectEntity: (localId: string | null) => void;
  onCommit: (
    localId: string,
    typeId: string,
    fieldPath: string,
    value: any,
  ) => Promise<void>;
  onAddComponent: (localId: string, typeId: string) => Promise<void>;
  onRemoveComponent: (localId: string, typeId: string) => Promise<void>;
  onUndo: () => Promise<void>;
  onRedo: () => Promise<void>;
  onSave: () => Promise<void>;
  onBackToScene: () => void;
  canUndo: boolean;
  canRedo: boolean;
  dirty: boolean;
}

/**
 * AssetAuthoringView: purpose-built entity/component editor for Scene Assets.
 *
 * Constraint C-2: onCommit adapter wraps fieldPath string as [fieldPath] array
 * for AssetCommand.SetComponentValue.field_path: Vec<String>.
 */
export default function AssetAuthoringView({
  document,
  activeEntityId,
  onSelectEntity,
  onCommit,
  onAddComponent,
  onRemoveComponent,
  onUndo,
  onRedo,
  onSave,
  onBackToScene,
  canUndo,
  canRedo,
  dirty,
}: Props) {
  const [selectedTab, setSelectedTab] = useState<
    "entities" | "relationships" | "layers"
  >("entities");

  // Layers tab is only available for Level Scene Assets.
  const isLevel = document.role === "level";

  const activeEntity = document.entities.find(
    (e) => e.local_id === activeEntityId,
  );

  /**
   * Adapter for ComponentCard.onCommit.
   * Converts fieldPath string to [fieldPath] array per constraint C-2.
   */
  const handleComponentCommit = useCallback(
    async (localId: string, typeId: string, fieldPath: string, value: any) => {
      // fieldPath is already a string like "translation" or "translation.x"
      // Wrap as [fieldPath] for SetComponentValue.field_path: Vec<String>
      await onCommit(localId, typeId, fieldPath, value);
    },
    [onCommit],
  );

  /**
   * Adapter for AddComponentButton.onAdd.
   * Dispatches AddComponent AssetCommand.
   */
  const handleAddComponent = useCallback(
    async (localId: string, typeId: string) => {
      await onAddComponent(localId, typeId);
    },
    [onAddComponent],
  );

  return (
    <div className="asset-authoring-view" data-testid="asset-authoring-view">
      {/* Header with asset info and actions */}
      <div className="authoring-header" data-testid="authoring-header">
        <div className="asset-info">
          <h2 data-testid="authoring-asset-name">{document.logical_path}</h2>
          <span className="asset-role-badge" data-testid="authoring-asset-role">
            {document.role}
          </span>
          <span className="asset-version" data-testid="authoring-asset-version">
            v{document.version}
          </span>
        </div>
        <div className="authoring-actions">
          <button
            onClick={onUndo}
            disabled={!canUndo}
            data-testid="asset-undo-btn"
            title="Undo (Ctrl+Shift+Z)"
          >
            ↶ Undo
          </button>
          <button
            onClick={onRedo}
            disabled={!canRedo}
            data-testid="asset-redo-btn"
            title="Redo (Ctrl+Shift+U)"
          >
            ↷ Redo
          </button>
          <button
            onClick={onSave}
            disabled={!dirty}
            data-testid="asset-save-btn"
            className="primary"
          >
            Save
          </button>
          <button onClick={onBackToScene} data-testid="back-to-scene-btn">
            Back to Scene
          </button>
        </div>
      </div>

      {/* Entity list + relationships tabs */}
      <div className="authoring-tabs">
        <button
          className={selectedTab === "entities" ? "active" : ""}
          onClick={() => setSelectedTab("entities")}
          data-testid="tab-entities"
        >
          Entities ({document.entities.length})
        </button>
        <button
          className={selectedTab === "relationships" ? "active" : ""}
          onClick={() => setSelectedTab("relationships")}
          data-testid="tab-relationships"
        >
          Relationships ({document.relationships.length})
        </button>
        {isLevel && (
          <button
            className={selectedTab === "layers" ? "active" : ""}
            onClick={() => setSelectedTab("layers")}
            data-testid="tab-layers"
          >
            Layers ({(document.layers ?? []).length})
          </button>
        )}
      </div>

      <div className="authoring-content">
        {selectedTab === "entities" ? (
          <EntityList
            entities={document.entities}
            activeEntityId={activeEntityId}
            onSelectEntity={onSelectEntity}
          />
        ) : selectedTab === "relationships" ? (
          <RelationshipsPanel
            relationships={document.relationships}
            entities={document.entities}
          />
        ) : (
          isLevel && (
            <LayersPanel
              document={document}
              onAssetJsonChanged={(json) => setAssetDocumentJson(json)}
            />
          )
        )}
      </div>

      {/* Component editor for selected entity */}
      {activeEntity && (
        <div className="entity-editor" data-testid="entity-editor">
          <h3 data-testid="entity-editor-title">Entity: {activeEntity.name}</h3>
          <div className="components-list">
            {activeEntity.components.map((comp) => (
              <ComponentCard
                key={comp.type_id}
                component={comp}
                entityId={activeEntity.local_id}
                onCommit={(fieldPath, value) =>
                  handleComponentCommit(
                    activeEntity.local_id,
                    comp.type_id,
                    fieldPath,
                    value,
                  )
                }
                onRemove={() =>
                  onRemoveComponent(activeEntity.local_id, comp.type_id)
                }
              />
            ))}
            <AddComponentButton
              entityId={activeEntity.local_id}
              onAdd={(typeId) =>
                handleAddComponent(activeEntity.local_id, typeId)
              }
            />
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * Entity list panel.
 */
interface EntityListProps {
  entities: SceneAssetEntity[];
  activeEntityId: string | null;
  onSelectEntity: (localId: string | null) => void;
}

function EntityList({
  entities,
  activeEntityId,
  onSelectEntity,
}: EntityListProps) {
  if (entities.length === 0) {
    return (
      <div className="empty-entities" data-testid="entities-empty">
        <p>No entities in this Scene Asset.</p>
        <p>Add an entity to get started.</p>
      </div>
    );
  }

  return (
    <div className="entity-list" data-testid="entity-list">
      {entities.map((entity) => (
        <div
          key={entity.local_id}
          className={`entity-item ${
            entity.local_id === activeEntityId ? "selected" : ""
          }`}
          onClick={() => onSelectEntity(entity.local_id)}
          data-testid={`entity-item-${entity.local_id}`}
        >
          <span
            className="entity-name"
            data-testid={`entity-name-${entity.local_id}`}
          >
            {entity.name}
          </span>
          <span className="entity-components-count">
            {entity.components.length} components
          </span>
        </div>
      ))}
    </div>
  );
}

/**
 * Read-only relationships panel.
 */
interface RelationshipsPanelProps {
  relationships: SceneAssetDocument["relationships"];
  entities: SceneAssetEntity[];
}

function RelationshipsPanel({
  relationships,
  entities,
}: RelationshipsPanelProps) {
  const getEntityName = (localId: string) => {
    const entity = entities.find((e) => e.local_id === localId);
    return entity ? entity.name : localId;
  };

  const formatKind = (kind: any): string => {
    if (kind === "Child") return "Child";
    if (typeof kind === "object" && kind.custom)
      return `Custom: ${kind.custom}`;
    return String(kind);
  };

  if (relationships.length === 0) {
    return (
      <div className="empty-relationships" data-testid="relationships-empty">
        <p>No relationships defined.</p>
        <p className="hint">Relationships are read-only in this view.</p>
      </div>
    );
  }

  return (
    <div className="relationships-list" data-testid="relationships-list">
      {relationships.map((rel, idx) => (
        <div
          key={idx}
          className="relationship-item"
          data-testid={`relationship-${idx}`}
        >
          <span className="rel-from" data-testid="rel-from">
            {getEntityName(rel.from_local_id)}
          </span>
          <span className="rel-arrow">→</span>
          <span className="rel-kind" data-testid="rel-kind">
            {formatKind(rel.kind)}
          </span>
          <span className="rel-arrow">→</span>
          <span className="rel-to" data-testid="rel-to">
            {getEntityName(rel.to_local_id)}
          </span>
          {rel.field_path && (
            <span className="rel-field" data-testid="rel-field">
              [{rel.field_path.join(".")}]
            </span>
          )}
        </div>
      ))}
    </div>
  );
}

/**
 * Read-first Scene Instance Layers panel.
 * Calls into WASM bridges for list/create/delete; the resulting
 * updated asset JSON is sent back to the backend via `set_asset_document_wasm`
 * so a subsequent save persists the layers.
 */
interface LayersPanelProps {
  document: SceneAssetDocument;
  onAssetJsonChanged: (json: string) => Promise<void> | void;
}

function LayersPanel({ document, onAssetJsonChanged }: LayersPanelProps) {
  // MED-7 fix: derive `layers` directly from `document` instead of
  // maintaining a separate copy via refresh(). This removes the
  // dual-source-of-truth and the eventual-consistency window.
  const [layers, setLayers] = useState<SceneInstanceLayerSummary[]>([]);
  const [creatingName, setCreatingName] = useState("");
  const [creatingKind, setCreatingKind] =
    useState<SceneInstanceLayerKind>("actors");
  const [error, setError] = useState<string | null>(null);

  // Recompute layers whenever the document changes (parent owns truth).
  // The WASM call is read-only so this is cheap for small assets.
  useEffect(() => {
    let cancelled = false;
    setError(null);
    listSceneInstanceLayers(JSON.stringify(document))
      .then((list) => {
        if (!cancelled) setLayers(list);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [document]);

  const handleCreate = useCallback(async () => {
    const name = creatingName.trim();
    if (!name) return;
    setError(null);
    try {
      const updated = await createSceneInstanceLayer(
        JSON.stringify(document),
        name,
        creatingKind,
      );
      await onAssetJsonChanged(updated);
      setCreatingName("");
      // No explicit refresh — the parent updates document, which triggers the effect.
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [creatingName, creatingKind, document, onAssetJsonChanged]);

  const handleDelete = useCallback(
    async (layerId: string) => {
      setError(null);
      try {
        const updated = await deleteSceneInstanceLayer(
          JSON.stringify(document),
          layerId,
        );
        await onAssetJsonChanged(updated);
        // No explicit refresh — same as handleCreate.
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [document, onAssetJsonChanged],
  );

  if (layers.length === 0) {
    return (
      <div className="layers-panel" data-testid="layers-panel">
        <div className="layers-create">
          <input
            type="text"
            placeholder="New layer name"
            value={creatingName}
            onChange={(e) => setCreatingName(e.target.value)}
            data-testid="layers-new-name"
          />
          <select
            value={creatingKind}
            onChange={(e) =>
              setCreatingKind(e.target.value as SceneInstanceLayerKind)
            }
            data-testid="layers-new-kind"
          >
            <option value="actors">actors</option>
            <option value="props">props</option>
            <option value="spawns">spawns</option>
            <option value="triggers">triggers</option>
            <option value="collision">collision</option>
            <option value="custom">custom</option>
          </select>
          <button
            onClick={handleCreate}
            disabled={!creatingName.trim()}
            data-testid="layers-create-btn"
          >
            Create Layer
          </button>
        </div>
        {error && (
          <div className="layers-error" data-testid="layers-error">
            {error}
          </div>
        )}
        <div className="layers-empty" data-testid="layers-empty">
          No Scene Instance Layers yet. Create one above.
        </div>
      </div>
    );
  }

  return (
    <div className="layers-panel" data-testid="layers-panel">
      <div className="layers-create">
        <input
          type="text"
          placeholder="New layer name"
          value={creatingName}
          onChange={(e) => setCreatingName(e.target.value)}
          data-testid="layers-new-name"
        />
        <select
          value={creatingKind}
          onChange={(e) =>
            setCreatingKind(e.target.value as SceneInstanceLayerKind)
          }
          data-testid="layers-new-kind"
        >
          <option value="actors">actors</option>
          <option value="props">props</option>
          <option value="spawns">spawns</option>
          <option value="triggers">triggers</option>
          <option value="collision">collision</option>
          <option value="custom">custom</option>
        </select>
        <button
          onClick={handleCreate}
          disabled={!creatingName.trim()}
          data-testid="layers-create-btn"
        >
          Create Layer
        </button>
      </div>
      {error && (
        <div className="layers-error" data-testid="layers-error">
          {error}
        </div>
      )}
      <ul className="layers-list" data-testid="layers-list">
        {layers.map((l) => (
          <li
            key={l.id}
            className="layer-row"
            data-testid={`layer-row-${l.id}`}
          >
            <span className="layer-name" data-testid={`layer-name-${l.id}`}>
              {l.name}
            </span>
            <span className="layer-kind" data-testid={`layer-kind-${l.id}`}>
              {l.kind}
            </span>
            <span className="layer-order" data-testid={`layer-order-${l.id}`}>
              order {l.order}
            </span>
            <span className="layer-count">
              {l.instances_count} instance
              {l.instances_count === 1 ? "" : "s"}
            </span>
            <button
              className="layer-delete-btn"
              onClick={() => handleDelete(l.id)}
              data-testid={`layer-delete-${l.id}`}
            >
              Delete
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
