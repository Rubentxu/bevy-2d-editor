/**
 * LeftDock — Assets browser dock (Phase B, 3-region layout).
 *
 * Renders a DockHeader "Assets" plus a DockBody that hosts AssetNavigator.
 * When the dock is collapsed the body is hidden and a slim icon strip is
 * shown so the toggle target stays reachable (per tasks.md §B.5).
 *
 * v0.82 P1 (ADR-0024): accepts an `onMove(target)` prop that the parent
 * (`App`) wires into the keyboard-equivalent `Move →` menu. The
 * dataTransfer payload published on HTML5 drag is the bare canonical
 * panel id (`"assets"`) — not the regionalised `data-panel-id`
 * (`"left-assets"`) — so the swap reducer and the keyboard menu stay
 * aligned on a single identifier across every layer of the dock system.
 */

import type { DragEvent } from "react";
import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import AssetNavigator from "../AssetNavigator";
import { stampDockPanelDrag } from "./drag-payload";
import type { DockableRegion } from "../../hooks/useDockPrefs";

interface Props {
  visible: boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
  onClose: () => void;
  onMove?: (target: DockableRegion) => void;
}

export default function LeftDock({
  visible,
  collapsed,
  onToggleCollapse,
  onClose,
  onMove,
}: Props) {
  // Tier 1c + v0.82 P1: stamp the canonical panel id into the dataTransfer.
  // The bare id (`"assets"`) is the `panelRegions` key — not the regional
  // `data-panel-id` rendered on the DOM root (`"left-assets"`).
  const handleDragStart = (e: DragEvent<HTMLDivElement>) => {
    stampDockPanelDrag(e.dataTransfer, "assets");
  };

  if (!visible) {
    return (
      <div
        className="dock dock-left dock-collapsed-strip"
        data-testid="dock-left-strip"
        data-panel-id="left-assets"
      >
        <button
          type="button"
          className="dock-collapsed-button"
          aria-label="Open Assets dock"
          onClick={onClose}
        >
          📁
        </button>
      </div>
    );
  }

  return (
    <div
      className="dock dock-left"
      data-testid="dock-left"
      data-panel-id="left-assets"
    >
      <DockHeader
        title="Assets"
        testId="dock-left-header"
        collapsed={collapsed}
        draggable
        onDragStart={handleDragStart}
        onToggleCollapse={onToggleCollapse}
        onClose={onClose}
        onMove={onMove}
      />
      {!collapsed && (
        <DockBody testId="dock-left-body">
          <AssetNavigator />
        </DockBody>
      )}
    </div>
  );
}
