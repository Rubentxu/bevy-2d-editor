import { useState, useEffect } from "react";
import { SceneDocument } from "../hooks/useSceneState";
import { SceneInstance, parseInstanceChild } from "../services/scene-assets";

interface Props {
  scene: SceneDocument | null;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onRename: (entityId: string, newName: string) => void;
  instances?: Record<string, SceneInstance>;
  onCreateEntity?: () => void;
  /**
   * Phase 3.4 — external rename trigger (F2 shortcut). When this number
   * changes, the panel starts inline-renaming the currently selected
   * entity (if any). The parent owns the counter and bumps it from a
   * keyboard handler.
   */
  renameRequest?: number;
  // v0.82 P2 (ADR-0025): modifier-aware multi-select entry point. When
  // supplied, Shift/Ctrl clicks are routed here so the parent can
  // extend or toggle the selection. Without it, HierarchyPanel falls
  // back to the legacy single-id `onSelect` behaviour.
  onSelectModifier?: (id: string, modifier: "range" | "toggle") => void;
  // Optional set of currently selected ids — when supplied, the
  // selected class is applied to every matching row, not just the
  // primary `selectedId` (which is the most-recently-clicked single
  // id used for the inspector fallback).
  selectedIds?: Set<string>;
}

/**
 * Compute indentation depth for an entity by walking parent chain.
 */
function entityDepth(
  entity: SceneDocument["entities"][number],
  allEntities: SceneDocument["entities"],
): number {
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

/**
 * Compute the component-override status color for a Scene Instance.
 * Returns CSS color string or null if no component overrides exist.
 */
function overrideStatusColor(instance: SceneInstance): string | null {
  const allPatches = [
    ...instance.component_overrides,
    ...instance.orphaned_component_overrides,
  ];
  if (allPatches.length === 0) return null;
  if (
    allPatches.some((p) => p.status === "conflict" || p.status === "orphaned")
  )
    return "#e53e3e"; // red
  if (allPatches.some((p) => p.status === "stale")) return "#d69e2e"; // yellow
  return "#38a169"; // green — all active
}

function entityIcon(entity: SceneDocument["entities"][number]): string {
  const types = entity.components.map((component) => component.type_id);
  if (types.some((typeId) => typeId.endsWith("Sprite2D"))) return "🖼️";
  if (
    types.length > 0 &&
    types.every((typeId) => typeId.endsWith("Transform2D"))
  )
    return "⊕";
  return "📦";
}

export default function HierarchyPanel({
  scene,
  selectedId,
  onSelect,
  onRename,
  instances = {},
  onCreateEntity,
  renameRequest = 0,
  onSelectModifier,
  selectedIds,
}: Props) {
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  // Clear stale editingId if the entity disappears mid-edit
  useEffect(() => {
    if (
      editingId !== null &&
      !scene?.entities.some((e) => e.id === editingId)
    ) {
      setEditingId(null);
    }
  }, [editingId, scene]);

  // Phase 3.4 — external rename trigger (F2 shortcut)
  useEffect(() => {
    if (renameRequest === 0) return;
    if (!selectedId) return;
    const entity = scene?.entities.find((e) => e.id === selectedId);
    if (!entity) return;
    setEditingId(entity.id);
    setEditValue(entity.name);
  }, [renameRequest, selectedId, scene]);

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
    const currentParent =
      scene!.entities.find((e) => e.id === entityId)?.parent ?? null;
    (window as any).dispatch_command(
      JSON.stringify({
        command: {
          type: "ReparentEntity",
          entity_id: entityId,
          old_parent: currentParent,
          new_parent: newParent,
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      }),
    );
  };

  const renderSearch = () => (
    <input
      type="search"
      className="panel-search"
      data-testid="hierarchy-search"
      placeholder="Search entities…"
      aria-label="Search entities"
      value={searchQuery}
      onChange={(event) => setSearchQuery(event.target.value)}
    />
  );

  if (!scene) {
    return (
      <div className="panel" data-testid="hierarchy-panel">
        <div className="panel-header">
          <h2>Hierarchy</h2>
        </div>
        {renderSearch()}
        <div className="panel-empty">No scene loaded</div>
      </div>
    );
  }
  if (scene.entities.length === 0) {
    return (
      <div className="panel" data-testid="hierarchy-panel">
        <div className="panel-header">
          <h2>Hierarchy</h2>
          {onCreateEntity && (
            <button
              className="add-entity-btn"
              data-testid="add-entity-btn"
              onClick={onCreateEntity}
              title="Create new entity (N)"
            >
              + Add Entity <span className="kbd-hint">(N)</span>
            </button>
          )}
        </div>
        {renderSearch()}
        <div
          className="panel-empty panel-empty-cta"
          data-testid="hierarchy-empty-cta"
        >
          <div className="panel-empty-title">No entities yet</div>
          <div className="panel-empty-subtitle">
            Press N or click below to start
          </div>
          {onCreateEntity && (
            <button
              className="add-entity-btn add-entity-btn-cta"
              data-testid="add-entity-btn-empty"
              onClick={onCreateEntity}
              title="Create new entity (N)"
            >
              + Add Entity <span className="kbd-hint">(N)</span>
            </button>
          )}
        </div>
      </div>
    );
  }
  return (
    <div className="panel" data-testid="hierarchy-panel">
      <div className="panel-header">
        <h2>Hierarchy</h2>
        {onCreateEntity && (
          <button
            className="add-entity-btn"
            data-testid="add-entity-btn"
            onClick={onCreateEntity}
            title="Create new entity (N)"
          >
            + Add Entity <span className="kbd-hint">(N)</span>
          </button>
        )}
      </div>
      {renderSearch()}
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
        {scene.entities
          .filter((entity) =>
            entity.name
              .toLowerCase()
              .includes(searchQuery.trim().toLowerCase()),
          )
          .map((entity) => {
            const depth = entityDepth(entity, scene.entities);
            // v0.82 P2 (ADR-0025): row is "selected" if it's the
            // current selection's anchor (single-id primary) OR a
            // member of the multi-select set. The Set is preferred so
            // Shift/Ctrl selections immediately show their highlight.
            const isSelected =
              (selectedIds?.has(entity.id) ?? false) ||
              selectedId === entity.id;
            return (
              <div
                key={entity.id}
                className={[
                  "entity",
                  isSelected ? "selected" : "",
                  draggedId === entity.id ? "dragging" : "",
                  dragOverId === entity.id ? "drag-over" : "",
                ]
                  .join(" ")
                  .trim()}
                style={{
                  paddingLeft: `${depth * 16 + 8}px`,
                  cursor: "pointer",
                  opacity: draggedId === entity.id ? 0.5 : 1,
                }}
                draggable
                onClick={(e) => {
                  e.stopPropagation();
                  // v0.82 P2 (ADR-0025 §F7): Shift+Click extends the
                  // range; Ctrl/Cmd+Click toggles membership. Plain
                  // clicks fall through to the legacy single-id
                  // selector for back-compat. We only dispatch the
                  // modifier variant when the parent wired a handler
                  // so non-upgraded callers still get single-select.
                  if (e.shiftKey && onSelectModifier) {
                    onSelectModifier(entity.id, "range");
                  } else if ((e.ctrlKey || e.metaKey) && onSelectModifier) {
                    onSelectModifier(entity.id, "toggle");
                  } else {
                    onSelect(entity.id);
                  }
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
                  if (
                    draggedId &&
                    draggedId !== entity.id &&
                    scene.entities.some((e) => e.id === draggedId)
                  ) {
                    reparent(draggedId, entity.id);
                  }
                  setDraggedId(null);
                  setDragOverId(null);
                }}
                data-testid={`hierarchy-entity-${entity.id}`}
              >
                <span className="entity-type-icon" aria-hidden="true">
                  {entityIcon(entity)}
                </span>
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
                {(() => {
                  const instId =
                    parseInstanceChild(entity.id)?.instance_id ?? null;
                  if (!instId) return null;
                  const inst = instances[instId];
                  if (!inst) return null;
                  const color = overrideStatusColor(inst);
                  if (!color) return null;
                  return (
                    <span
                      className="override-status-dot"
                      data-testid={`override-dot-${entity.id}`}
                      title="Override status"
                      style={{ backgroundColor: color }}
                    />
                  );
                })()}
                <span className="id">{entity.id.slice(0, 8)}</span>
              </div>
            );
          })}
      </div>
    </div>
  );
}
