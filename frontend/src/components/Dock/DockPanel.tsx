/**
 * DockPanel — Header + Body wrapper with HTML5 drag-and-drop support.
 *
 * Phase C (v0.81 Tier 1c, drag-and-dock): wraps a DockHeader + DockBody
 * inside a `<section class="dock-panel">` that exposes the standard drag
 * events used by the dock system:
 *
 *   - `draggable={true}` on the header (visual cue: cursor: grab)
 *   - `dragstart` writes the panel id into the dataTransfer under the
 *     MIME `application/x-dock-panel`
 *   - `dragover` on the panel reveals the dashed accent outline so users
 *     see where the panel will land
 *   - `drop` reads the source panel id and calls `onRegionChange(target)`
 *     so the parent can update which region owns the panel
 *
 * Region-swap behaviour is left to v0.82 — Tier 1c wires the dataflow
 * (panel id in transfer, drop handler callable) without actually moving
 * the panel DOM. Existing tests therefore keep passing unchanged.
 *
 * v0.82 P1 (ADR-0024): the MIME constant is now re-exported for legacy
 * callers, but new code should import from `./drag-payload` (single
 * source of truth). Kept here so the Tier 1c infrastructure doesn't
 * break existing test imports.
 */

import { useState, type DragEventHandler, type ReactNode } from "react";
import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import { DOCK_PANEL_MIME, isDockPanelDrag } from "./drag-payload";

export { DOCK_PANEL_MIME } from "./drag-payload";

export type DockRegion = "left" | "center" | "right" | "bottom";

interface DockPanelProps {
  title: string;
  testId: string;
  region: DockRegion;
  panelId: string;
  children: ReactNode;
  collapsed?: boolean;
  onRegionChange?: (panelId: string, target: DockRegion) => void;
  onClose?: () => void;
  onCollapse?: () => void;
  /** When false, the header is not draggable (default true for Tier 1c). */
  draggable?: boolean;
}

export default function DockPanel({
  title,
  testId,
  region,
  panelId,
  children,
  collapsed = false,
  onRegionChange,
  onClose,
  onCollapse,
  draggable = true,
}: DockPanelProps) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleDragStart: DragEventHandler<HTMLDivElement> = (e) => {
    e.dataTransfer.setData(DOCK_PANEL_MIME, panelId);
    e.dataTransfer.setData("text/plain", panelId);
    e.dataTransfer.effectAllowed = "move";
  };

  const handleDragOver: DragEventHandler<HTMLElement> = (e) => {
    if (!isDockPanelDrag(e.dataTransfer.types)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    if (!isDragOver) setIsDragOver(true);
  };

  const handleDragLeave: DragEventHandler<HTMLElement> = () => {
    setIsDragOver(false);
  };

  const handleDrop: DragEventHandler<HTMLElement> = (e) => {
    e.preventDefault();
    setIsDragOver(false);
    const sourcePanelId =
      e.dataTransfer.getData(DOCK_PANEL_MIME) ||
      e.dataTransfer.getData("text/plain");
    if (!sourcePanelId || sourcePanelId === panelId) return;
    onRegionChange?.(sourcePanelId, region);
  };

  return (
    <section
      className={`dock-panel dock-panel--${region}${
        isDragOver ? " dock-panel--drag-over" : ""
      }`}
      data-testid={testId}
      data-panel-id={panelId}
      data-region={region}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <DockHeader
        title={title}
        testId={testId ? `${testId}-header` : undefined}
        collapsed={collapsed}
        draggable={draggable}
        onDragStart={draggable ? handleDragStart : undefined}
        onToggleCollapse={onCollapse ?? (() => {})}
        onClose={onClose}
      />
      {!collapsed && (
        <DockBody testId={testId ? `${testId}-body` : undefined}>
          {children}
        </DockBody>
      )}
    </section>
  );
}
