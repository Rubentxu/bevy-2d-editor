/**
 * WorldWorkspace — the main canvas for the World Workspace (ADR-0037 §ww-ui).
 *
 * Renders:
 *   - One square per WorldLevelRef at its position
 *   - SVG directional arrows between linked levels
 *   - Layout-policy toolbar (Free / Grid / Horizontal / Vertical)
 *   - Topology issue badges on levels with broken references
 *   - Minimap showing full world bounds + viewport rectangle
 *
 * Reuses useCanvasViewport pan/zoom scaffolding.
 * Double-click on a level square calls openLevel(levelId) and switches
 * the editor to "scene" mode.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useWorldWorkspace, type WorldWorkspaceState } from "../hooks/useWorldWorkspace";
import { type LayoutPolicy, type LinkDirection, type WorldLevelRef, type WorldLink } from "../services/EditorGateway";
import "./WorldWorkspace.css";

const LEVEL_SQUARE_SIZE = 80;
const MINIMAP_WIDTH = 160;
const MINIMAP_HEIGHT = 120;
const MINIMAP_SCALE = 0.05;

interface Props {
  /** Called when user double-clicks a level to open it in scene mode. */
  onOpenLevel?: (levelId: string, assetRef: string) => void;
  /** Called when user wants to switch back to scene mode. */
  onBackToScene?: () => void;
}

export default function WorldWorkspace({ onOpenLevel, onBackToScene }: Props) {
  const {
    worldDoc,
    topologyIssues,
    selectedLevelId,
    dragState,
    viewport,
    selectLevel,
    placeLevel,
    connectLevels,
    setLayoutPolicy,
    openLevel,
    loadWorld,
    refreshTopology,
  } = useWorldWorkspace();

  const canvasRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStartPos, setDragStartPos] = useState<{ x: number; y: number } | null>(null);

  // Load the first available world on mount
  useEffect(() => {
    if (!worldDoc) {
      // Try to load the first world if any exist
      void (async () => {
        const { getEditorGateway } = await import("../services/EditorGateway");
        const gateway = getEditorGateway();
        const listResult = await gateway.world.listWorlds();
        if (listResult.ok && listResult.value.length > 0) {
          await loadWorld(listResult.value[0].logical_path);
        }
      })();
    }
  }, [worldDoc, loadWorld]);

  // Compute world bounds for minimap
  const worldBounds = useMemo(() => {
    if (!worldDoc?.levels.length) {
      return { minX: 0, minY: 0, maxX: 800, maxY: 600 };
    }
    let minX = Infinity,
      minY = Infinity,
      maxX = -Infinity,
      maxY = -Infinity;
    for (const level of worldDoc.levels) {
      const x = level.position[0];
      const y = level.position[1];
      if (x < minX) minX = x;
      if (y < minY) minY = y;
      if (x > maxX) maxX = x;
      if (y > maxY) maxY = y;
    }
    // Add padding
    const pad = LEVEL_SQUARE_SIZE;
    return {
      minX: minX - pad,
      minY: minY - pad,
      maxX: maxX + LEVEL_SQUARE_SIZE + pad,
      maxY: maxY + LEVEL_SQUARE_SIZE + pad,
    };
  }, [worldDoc?.levels]);

  // Convert world coords to minimap coords
  const worldToMinimap = useCallback(
    (wx: number, wy: number) => {
      const bounds = worldBounds;
      const w = bounds.maxX - bounds.minX;
      const h = bounds.maxY - bounds.minY;
      const scaleX = MINIMAP_WIDTH / Math.max(w, 1);
      const scaleY = MINIMAP_HEIGHT / Math.max(h, 1);
      const scale = Math.min(scaleX, scaleY);
      return {
        x: (wx - bounds.minX) * scale,
        y: (wy - bounds.minY) * scale,
      };
    },
    [worldBounds],
  );

  // Compute minimap scale to fit world in view
  const minimapWorldScale = useMemo(() => {
    const w = worldBounds.maxX - worldBounds.minX;
    const h = worldBounds.maxY - worldBounds.minY;
    return Math.min(MINIMAP_WIDTH / Math.max(w, 1), MINIMAP_HEIGHT / Math.max(h, 1));
  }, [worldBounds]);

  // Get topology issues for a specific level
  const getIssuesForLevel = useCallback(
    (levelId: string) => {
      return topologyIssues.filter((issue) => issue.level_id === levelId);
    },
    [topologyIssues],
  );

  // Handle mouse down on a level square (start drag)
  const handleLevelMouseDown = useCallback(
    (e: React.MouseEvent, level: WorldLevelRef) => {
      e.stopPropagation();
      selectLevel(level.level_id);
      setIsDragging(true);
      setDragStartPos({ x: e.clientX, y: e.clientY });
    },
    [selectLevel],
  );

  // Handle mouse move on canvas (drag)
  const handleCanvasMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isDragging || !selectedLevelId || !canvasRef.current) return;

      const rect = canvasRef.current.getBoundingClientRect();
      // World coords from screen position + viewport pan/zoom
      const worldX = (e.clientX - rect.left - viewport.pan.x) / viewport.zoom;
      const worldY = (e.clientY - rect.top - viewport.pan.y) / viewport.zoom;

      // Update drag state for visual feedback
      void placeLevel(selectedLevelId, worldX, worldY);
    },
    [isDragging, selectedLevelId, viewport, placeLevel],
  );

  // Handle mouse up (end drag)
  const handleCanvasMouseUp = useCallback(() => {
    setIsDragging(false);
    setDragStartPos(null);
  }, []);

  // Handle double-click on level (open in scene mode)
  const handleLevelDoubleClick = useCallback(
    async (level: WorldLevelRef) => {
      try {
        const assetRef = await openLevel(level.level_id);
        onOpenLevel?.(level.level_id, assetRef);
      } catch (err) {
        console.error("[WorldWorkspace] openLevel failed:", err);
      }
    },
    [openLevel, onOpenLevel],
  );

  // Handle canvas double-click (deselect)
  const handleCanvasDoubleClick = useCallback(() => {
    selectLevel(null);
  }, [selectLevel]);

  // Compute arrow path between two levels
  const computeArrowPath = useCallback(
    (from: WorldLevelRef, to: WorldLevelRef, direction: LinkDirection) => {
      const fromCenterX = from.position[0] + LEVEL_SQUARE_SIZE / 2;
      const fromCenterY = from.position[1] + LEVEL_SQUARE_SIZE / 2;
      const toCenterX = to.position[0] + LEVEL_SQUARE_SIZE / 2;
      const toCenterY = to.position[1] + LEVEL_SQUARE_SIZE / 2;

      // For directional links, draw an arrow from center of source to center of target
      // The direction tells us which side of the square the link exits/enters
      let startX = fromCenterX;
      let startY = fromCenterY;
      let endX = toCenterX;
      let endY = toCenterY;

      // Offset based on direction for visual clarity
      const offset = LEVEL_SQUARE_SIZE / 2;
      switch (direction) {
        case "north":
          startY = from.position[1];
          endY = to.position[1] + LEVEL_SQUARE_SIZE;
          break;
        case "south":
          startY = from.position[1] + LEVEL_SQUARE_SIZE;
          endY = to.position[1];
          break;
        case "east":
          startX = from.position[0] + LEVEL_SQUARE_SIZE;
          endX = to.position[0];
          break;
        case "west":
          startX = from.position[0];
          endX = to.position[0] + LEVEL_SQUARE_SIZE;
          break;
        case "undirected":
        default:
          // Direct line between centers
          break;
      }

      return `M ${startX} ${startY} L ${endX} ${endY}`;
    },
    [],
  );

  // Render arrow marker definition
  const renderArrowMarker = useCallback(
    (id: string, color: string) => (
      <defs>
        <marker
          id={id}
          viewBox="0 0 10 10"
          refX="9"
          refY="5"
          markerWidth="6"
          markerHeight="6"
          orient="auto-start-reverse"
        >
          <path d="M 0 0 L 10 5 L 0 10 z" fill={color} />
        </marker>
      </defs>
    ),
    [],
  );

  // Handle layout policy change
  const handleLayoutPolicyChange = useCallback(
    (policy: LayoutPolicy) => {
      void setLayoutPolicy(policy);
    },
    [setLayoutPolicy],
  );

  // If no world is loaded, show empty state
  if (!worldDoc) {
    return (
      <div className="world-workspace world-workspace--empty">
        <div className="world-workspace__empty-state">
          <p>No world loaded.</p>
          <p>Create or open a world from the menu to get started.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="world-workspace">
      {/* Toolbar */}
      <div className="world-workspace__toolbar">
        <span className="world-workspace__toolbar-title">{worldDoc.name}</span>
        <div className="world-workspace__toolbar-actions">
          <button
            type="button"
            className={`world-workspace__layout-btn ${worldDoc.layout_policy.kind === "Free" ? "active" : ""}`}
            onClick={() => handleLayoutPolicyChange({ kind: "Free" })}
            title="Free layout"
          >
            Free
          </button>
          <button
            type="button"
            className={`world-workspace__layout-btn ${worldDoc.layout_policy.kind === "Grid" ? "active" : ""}`}
            onClick={() => handleLayoutPolicyChange({ kind: "Grid", cell_size: 100 })}
            title="Grid layout"
          >
            Grid
          </button>
          <button
            type="button"
            className={`world-workspace__layout-btn ${worldDoc.layout_policy.kind === "Horizontal" ? "active" : ""}`}
            onClick={() => handleLayoutPolicyChange({ kind: "Horizontal" })}
            title="Horizontal layout"
          >
            H
          </button>
          <button
            type="button"
            className={`world-workspace__layout-btn ${worldDoc.layout_policy.kind === "Vertical" ? "active" : ""}`}
            onClick={() => handleLayoutPolicyChange({ kind: "Vertical" })}
            title="Vertical layout"
          >
            V
          </button>
        </div>
        <button
          type="button"
          className="world-workspace__back-btn"
          onClick={onBackToScene}
          title="Back to Scene"
        >
          ← Back
        </button>
      </div>

      {/* Main canvas area */}
      <div className="world-workspace__canvas-wrapper">
        <div
          ref={canvasRef}
          className="world-workspace__canvas"
          onMouseMove={handleCanvasMouseMove}
          onMouseUp={handleCanvasMouseUp}
          onMouseLeave={handleCanvasMouseUp}
          onDoubleClick={handleCanvasDoubleClick}
        >
          {/* SVG layer for arrows */}
          <svg
            className="world-workspace__arrows"
            style={{
              position: "absolute",
              top: 0,
              left: 0,
              width: "100%",
              height: "100%",
              pointerEvents: "none",
              overflow: "visible",
            }}
          >
            {renderArrowMarker("arrowhead-warning", "#f59e0b")}
            {renderArrowMarker("arrowhead-error", "#ef4444")}
            {renderArrowMarker("arrowhead-normal", "#60a5fa")}
            {worldDoc.links.map((link) => {
              const fromLevel = worldDoc.levels.find((l) => l.level_id === link.from);
              const toLevel = worldDoc.levels.find((l) => l.level_id === link.to);
              if (!fromLevel || !toLevel) return null;

              const path = computeArrowPath(fromLevel, toLevel, link.direction);
              const hasIssue = topologyIssues.some(
                (issue) => issue.link_id === link.id && issue.severity === "Warning",
              );
              const hasError = topologyIssues.some(
                (issue) => issue.link_id === link.id && issue.severity === "Error",
              );
              const markerId = hasError
                ? "arrowhead-error"
                : hasIssue
                  ? "arrowhead-warning"
                  : "arrowhead-normal";

              return (
                <path
                  key={link.id}
                  d={path}
                  stroke={hasError ? "#ef4444" : hasIssue ? "#f59e0b" : "#60a5fa"}
                  strokeWidth={2}
                  fill="none"
                  markerEnd={`url(#${markerId})`}
                  opacity={0.8}
                />
              );
            })}
          </svg>

          {/* Level squares */}
          {worldDoc.levels.map((level) => {
            const issues = getIssuesForLevel(level.level_id);
            const hasWarning = issues.some((i) => i.severity === "Warning");
            const hasError = issues.some((i) => i.severity === "Error");
            const isSelected = selectedLevelId === level.level_id;

            return (
              <div
                key={level.level_id}
                className={`world-workspace__level ${isSelected ? "world-workspace__level--selected" : ""} ${hasError ? "world-workspace__level--error" : ""} ${hasWarning ? "world-workspace__level--warning" : ""}`}
                style={{
                  left: level.position[0],
                  top: level.position[1],
                  width: LEVEL_SQUARE_SIZE,
                  height: LEVEL_SQUARE_SIZE,
                }}
                onMouseDown={(e) => handleLevelMouseDown(e, level)}
                onDoubleClick={() => handleLevelDoubleClick(level)}
                title={level.asset_ref}
              >
                <span className="world-workspace__level-name">
                  {level.asset_ref.split("/").pop()}
                </span>
                {hasError && (
                  <span className="world-workspace__badge world-workspace__badge--error" title="Missing asset reference">
                    ⚠
                  </span>
                )}
                {hasWarning && !hasError && (
                  <span className="world-workspace__badge world-workspace__badge--warning" title="Topology issue">
                    ⚠
                  </span>
                )}
              </div>
            );
          })}
        </div>

        {/* Minimap */}
        <div className="world-workspace__minimap">
          <div className="world-workspace__minimap-title">Minimap</div>
          <svg
            width={MINIMAP_WIDTH}
            height={MINIMAP_HEIGHT}
            className="world-workspace__minimap-svg"
          >
            {/* World bounds background */}
            <rect
              x={0}
              y={0}
              width={MINIMAP_WIDTH}
              height={MINIMAP_HEIGHT}
              fill="#1e1e1e"
              stroke="#3f3f46"
              strokeWidth={1}
            />
            {/* Level dots */}
            {worldDoc.levels.map((level) => {
              const pos = worldToMinimap(level.position[0], level.position[1]);
              const issues = getIssuesForLevel(level.level_id);
              const hasError = issues.some((i) => i.severity === "Error");
              return (
                <circle
                  key={level.level_id}
                  cx={pos.x + (LEVEL_SQUARE_SIZE * minimapWorldScale) / 2}
                  cy={pos.y + (LEVEL_SQUARE_SIZE * minimapWorldScale) / 2}
                  r={3}
                  fill={hasError ? "#ef4444" : "#60a5fa"}
                />
              );
            })}
            {/* Viewport rectangle */}
            <rect
              x={(-viewport.pan.x / viewport.zoom - worldBounds.minX) * minimapWorldScale}
              y={(-viewport.pan.y / viewport.zoom - worldBounds.minY) * minimapWorldScale}
              width={(800 / viewport.zoom) * minimapWorldScale}
              height={(600 / viewport.zoom) * minimapWorldScale}
              fill="none"
              stroke="#a855f7"
              strokeWidth={1}
              opacity={0.8}
            />
          </svg>
        </div>
      </div>

      {/* Status bar */}
      <div className="world-workspace__status">
        <span>
          {worldDoc.levels.length} level{worldDoc.levels.length !== 1 ? "s" : ""}
        </span>
        <span>{worldDoc.links.length} link{worldDoc.links.length !== 1 ? "s" : ""}</span>
        {topologyIssues.length > 0 && (
          <span className="world-workspace__status--warning">
            {topologyIssues.length} issue{topologyIssues.length !== 1 ? "s" : ""}
          </span>
        )}
      </div>
    </div>
  );
}
