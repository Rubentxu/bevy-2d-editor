import { SceneDocument } from "../hooks/useSceneState";

interface Props {
  scene: SceneDocument | null;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
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

export default function HierarchyPanel({ scene, selectedId, onSelect }: Props) {
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
      <div onClick={() => onSelect(null)}>
        {scene.entities.map((entity) => {
          const depth = entityDepth(entity, scene.entities);
          const isSelected = selectedId === entity.id;
          return (
            <div
              key={entity.id}
              className={isSelected ? "entity selected" : "entity"}
              style={{ paddingLeft: `${depth * 16 + 8}px`, cursor: "pointer" }}
              onClick={(e) => {
                e.stopPropagation();
                onSelect(entity.id);
              }}
              data-testid={`hierarchy-entity-${entity.id}`}
            >
              <span className="name">{entity.name}</span>
              <span className="id">{entity.id.slice(0, 8)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}