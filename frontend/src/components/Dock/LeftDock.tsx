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
 *
 * v0.82 P2 (ADR-0025): accepts `onFloatToggle` + `floating` so the
 * shared `DockHeader` renders a `Float / Dock` button. The parent
 * (`App.tsx`) routes float docks via `useDockResize.setFloatRect` /
 * `removeFloat`. When the panel is floating, this LeftDock instance
 * is *not* rendered at all (the `floatingPanelIds` gate in App.tsx
 * short-circuits to `null`); the FloatingPanel portal renders the
 * AssetNavigator content instead, so we don't need to gate inside the
 * dock itself.
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
  /** v0.82 P2 (ADR-0025): float-toggle wiring. */
  onFloatToggle?: () => void;
  /** v0.82 P2 (ADR-0025): true when this panel is currently in the
   * floating overlay. Always false for the docked instance; the
   * floating instance is rendered by `App.tsx` as a `FloatingPanel`
   * portal and does not use this dock component. */
  floating?: boolean;
}

export default function LeftDock({
  visible,
  collapsed,
  onToggleCollapse,
  onClose,
  onMove,
  onFloatToggle,
  floating,
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
        onFloatToggle={onFloatToggle}
        floating={floating}
      />
      {!collapsed && (
        <DockBody testId="dock-left-body">
          <AssetNavigator />
        </DockBody>
      )}
    </div>
  );
}
