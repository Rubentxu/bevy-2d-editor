import { useEffect, useState } from "react";
import { SceneDocument } from "../hooks/useSceneState";
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
}

export default function InspectorPanel({
  scene,
  selectedId,
  onRename,
  onSetField,
  onRemoveComponent,
  onAddComponent,
}: Props) {
  const entity = scene?.entities.find((e) => e.id === selectedId) ?? null;
  const [nameDraft, setNameDraft] = useState(entity?.name ?? "");
  const [showSchemaPanel, setShowSchemaPanel] = useState(false);
  const [schemaRefreshKey, setSchemaRefreshKey] = useState(0);

  useEffect(() => {
    setNameDraft(entity?.name ?? "");
  }, [entity?.id, entity?.name]);

  if (!scene) {
    return (
      <div className="panel inspector" data-testid="inspector-panel">
        <h2>Inspector</h2>
        <div className="panel-empty">No scene loaded</div>
      </div>
    );
  }
  if (!entity) {
    return (
      <div className="panel inspector" data-testid="inspector-panel">
        <h2>Inspector</h2>
        <div className="panel-empty">Select an entity</div>
      </div>
    );
  }

  return (
    <div className="panel inspector" data-testid="inspector-panel">
      <h2>Inspector</h2>
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
    </div>
  );
}