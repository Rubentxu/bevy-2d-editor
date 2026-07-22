/**
 * LeftDock — Assets browser dock (Phase B, 3-region layout).
 *
 * Renders a DockHeader "Assets" plus a DockBody that hosts AssetNavigator.
 * When the dock is collapsed the body is hidden and a slim icon strip is
 * shown so the toggle target stays reachable (per tasks.md §B.5).
 */

import type { DragEvent } from "react";
import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import AssetNavigator from "../AssetNavigator";
import { DOCK_PANEL_MIME } from "./DockPanel";

interface Props {
  visible: boolean;
  collapsed: boolean;
  onToggleCollapse: () => void;
  onClose: () => void;
}

export default function LeftDock({
  visible,
  collapsed,
  onToggleCollapse,
  onClose,
}: Props) {
  // Tier 1c: tag the header so HTML5 dragstart can stamp the panel id
  // into the dataTransfer without touching existing resize/collapse UX.
  const handleDragStart = (e: DragEvent<HTMLDivElement>) => {
    e.dataTransfer.setData(DOCK_PANEL_MIME, "assets");
    e.dataTransfer.setData("text/plain", "assets");
    e.dataTransfer.effectAllowed = "move";
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
      />
      {!collapsed && (
        <DockBody testId="dock-left-body">
          <AssetNavigator />
        </DockBody>
      )}
    </div>
  );
}
