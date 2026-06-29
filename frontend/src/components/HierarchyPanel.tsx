import { useState, useEffect } from "react";
import { SceneDocument } from "../hooks/useSceneState";

interface Props {
  scene: SceneDocument | null;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onRename: (entityId: string, newName: string) => void;
}

/**
 * Compute indentation depth for an entity by walking parent chain.
 */
function entityDepth(entity: SceneDocument["entities"][number], allEntities: SceneDocument["entities"]): number {
  let depth = 0;
  let current = entity.parent;
  const visited = new Set<string>();
  while (current && !visited.has(current)) {
    visited.add(current);
    const parent = allEntities.find((e) => e.id === current);
    if (!parent) break;
    depth += 1;
    current = parent.parent;
  }
  return depth;
}

export default function HierarchyPanel({ scene, selectedId, onSelect, onRename }: Props) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  // Clear stale editingId if the entity disappears mid-edit
  useEffect(() => {
    if (editingId !== null && !scene?.entities.some((e) => e.id === editingId)) {
      setEditingId(null);
    }
  }, [editingId, scene]);

  const commitRename = (entity: SceneDocument["entities"][number]) => {
    if (editingId !== entity.id) return;
    const trimmed = editValue.trim();
    if (trimmed === "" || trimmed === entity.name) {
      setEditingId(null);
      return;
    }
    onRename(entity.id, trimmed);
    setEditingId(null);
  };

  const reparent = (entityId: string, newParent: string | null) => {
    const currentParent = scene!.entities.find((e) => e.id === entityId)?.parent ?? null;
    (window as any).dispatch_command(JSON.stringify({
      command: {
        type: "ReparentEntity",
        entity_id: entityId,
        old_parent: currentParent,
        new_parent: newParent,
      },
      metadata: { authorship: "user", timestamp: Date.now() },
    }));
  };

  if (!scene) {
    return (
      <div className="panel" data-testid="hierarchy-panel">
        <h2>Hierarchy</h2>
        <div className="panel-empty">No scene loaded</div>
      </div>
    );
  }
  if (scene.entities.length === 0) {
    return (
      <div className="panel" data-testid="hierarchy-panel">
        <h2>Hierarchy</h2>
        <div className="panel-empty">No entities</div>
      </div>
    );
  }
  return (
    <div className="panel" data-testid="hierarchy-panel">
      <h2>Hierarchy</h2>
      <div
        className="hierarchy-root-zone"
        onClick={() => onSelect(null)}
        onDragOver={(e) => {
          e.preventDefault();
          e.dataTransfer.dropEffect = "move";
        }}
        onDrop={(e) => {
          e.preventDefault();
          if (draggedId) {
            reparent(draggedId, null);
          }
          setDraggedId(null);
          setDragOverId(null);
        }}
      >
        {scene.entities.map((entity) => {
          const depth = entityDepth(entity, scene.entities);
          const isSelected = selectedId === entity.id;
          return (
            <div
              key={entity.id}
              className={[
                "entity",
                isSelected ? "selected" : "",
                draggedId === entity.id ? "dragging" : "",
                dragOverId === entity.id ? "drag-over" : "",
              ].join(" ").trim()}
              style={{
                paddingLeft: `${depth * 16 + 8}px`,
                cursor: "pointer",
                opacity: draggedId === entity.id ? 0.5 : 1,
              }}
              draggable
              onClick={(e) => {
                e.stopPropagation();
                onSelect(entity.id);
              }}
              onDragStart={(e) => {
                setDraggedId(entity.id);
                e.dataTransfer.effectAllowed = "move";
              }}
              onDragEnd={() => {
                setDraggedId(null);
                setDragOverId(null);
              }}
              onDragOver={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (draggedId !== entity.id) {
                  setDragOverId(entity.id);
                }
              }}
              onDragLeave={(e) => {
                if (!e.currentTarget.contains(e.relatedTarget as Node)) {
                  if (dragOverId === entity.id) setDragOverId(null);
                }
              }}
              onDrop={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (draggedId && draggedId !== entity.id && scene.entities.some((e) => e.id === draggedId)) {
                  reparent(draggedId, entity.id);
                }
                setDraggedId(null);
                setDragOverId(null);
              }}
              data-testid={`hierarchy-entity-${entity.id}`}
            >
              {editingId === entity.id ? (
                <input
                  data-testid="hierarchy-rename-input"
                  className="name-input"
                  autoFocus
                  value={editValue}
                  onChange={(e) => setEditValue(e.target.value)}
                  onBlur={() => commitRename(entity)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      commitRename(entity);
                    } else if (e.key === "Escape") {
                      setEditingId(null);
                    }
                  }}
                  onClick={(e) => e.stopPropagation()}
                />
              ) : (
                <span
                  className="name"
                  onDoubleClick={(e) => {
                    e.stopPropagation();
                    setEditingId(entity.id);
                    setEditValue(entity.name);
                  }}
                >
                  {entity.name}
                </span>
              )}
              {entity.id.startsWith("inst_") && (
                <span
                  className="scene-instance-badge"
                  data-testid={`instance-badge-${entity.id}`}
                  title="Scene Instance child"
                >
                  [I]
                </span>
              )}
              <span className="id">{entity.id.slice(0, 8)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}