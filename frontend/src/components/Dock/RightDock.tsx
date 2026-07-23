/**
 * RightDock — Outline (top) + Properties (bottom) split (Phase B).
 *
 * Each side gets its own DockHeader + DockBody and a DockDivider between them.
 * The vertical split ratio lives in `topHeightPct` (default 60%) so the user
 * can drag the divider to resize the two sections independently from the
 * outer right-dock width.
 *
 * v0.82 P1 (ADR-0024): the right region is treated as one swap unit. The
 * outline and properties sub-panels share a single drag source: dragging
 * EITHER header moves the whole right-region pair to the destination.
 * Because the v0.82 P1 spec scopes the swap to "active panels" and `data-
 * panel-id` is unchanged for the E2E selectors, we publish the canonical
 * id `"outline"` on the outline header and `"properties"` on the
 * properties header. The reducer treats both as a swap of the whole
 * right slot for the purposes of `panelRegions`, and the keyboard `Move
 * →` menu on either half dispatches the same `onMove(target)` setter.
 */

import type { DragEvent, ReactNode } from "react";
import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import DockDivider from "./DockDivider";
import { stampDockPanelDrag } from "./drag-payload";
import type { DockableRegion } from "../../hooks/useDockPrefs";

interface Props {
  visible: boolean;
  outlineVisible: boolean;
  propertiesVisible: boolean;
  outlineCollapsed: boolean;
  propertiesCollapsed: boolean;
  topHeightPct: number;
  outline: ReactNode;
  properties: ReactNode;
  onToggleCollapseOutline: () => void;
  onToggleCollapseProperties: () => void;
  onCloseOutline: () => void;
  onCloseProperties: () => void;
  onResizeSplit: (deltaPx: number) => void;
  onResetSplit: () => void;
  onOpen: () => void;
  /**
   * Optional keyboard-equivalent for the v0.82 P1 `Move →` menu. Either
   * header accepts the same target value — both `outline` and
   * `properties` live in the right region from the `panelRegions`
   * perspective, so a single shared setter applies to the whole pair.
   */
  onMove?: (target: DockableRegion) => void;
}

export default function RightDock({
  visible,
  outlineVisible,
  propertiesVisible,
  outlineCollapsed,
  propertiesCollapsed,
  topHeightPct,
  outline,
  properties,
  onToggleCollapseOutline,
  onToggleCollapseProperties,
  onCloseOutline,
  onCloseProperties,
  onResizeSplit,
  onResetSplit,
  onOpen,
  onMove,
}: Props) {
  // Tier 1c + v0.82 P1: stamp canonical panel ids. The DOM-rooted
  // `data-panel-id` selectors (`right-outline`, `right-properties`) stay
  // for legacy tests; the dataTransfer payload is the canonical id
  // matching the `panelRegions` key.
  const handleOutlineDragStart = (e: DragEvent<HTMLDivElement>) => {
    stampDockPanelDrag(e.dataTransfer, "outline");
  };
  const handlePropertiesDragStart = (e: DragEvent<HTMLDivElement>) => {
    stampDockPanelDrag(e.dataTransfer, "properties");
  };

  if (!visible) {
    return (
      <div
        className="dock dock-right dock-collapsed-strip"
        data-testid="dock-right-strip"
        data-panel-id="right-outline"
      >
        <button
          type="button"
          className="dock-collapsed-button"
          aria-label="Open Outline + Properties dock"
          onClick={onOpen}
        >
          ▦
        </button>
      </div>
    );
  }

  return (
    <div
      className="dock dock-right"
      data-testid="dock-right"
      data-panel-id="right"
    >
      {outlineVisible && (
        <>
          <div
            className="dock dock-right-top"
            style={{ flexBasis: `${topHeightPct}%` }}
            data-testid="dock-right-outline"
            data-panel-id="right-outline"
          >
            <DockHeader
              title="Outline"
              testId="dock-right-outline-header"
              collapsed={outlineCollapsed}
              draggable
              onDragStart={handleOutlineDragStart}
              onToggleCollapse={onToggleCollapseOutline}
              onClose={onCloseOutline}
              onMove={onMove}
            />
            {!outlineCollapsed && (
              <DockBody testId="dock-right-outline-body">{outline}</DockBody>
            )}
          </div>
          {propertiesVisible && (
            <DockDivider
              orientation="horizontal"
              testId="dock-right-divider"
              onResize={onResizeSplit}
              onReset={onResetSplit}
            />
          )}
        </>
      )}
      {propertiesVisible && (
        <div
          className="dock dock-right-bottom"
          style={{ flexBasis: `${100 - topHeightPct}%` }}
          data-testid="dock-right-properties"
          data-panel-id="right-properties"
        >
          <DockHeader
            title="Properties"
            testId="dock-right-properties-header"
            collapsed={propertiesCollapsed}
            draggable
            onDragStart={handlePropertiesDragStart}
            onToggleCollapse={onToggleCollapseProperties}
            onClose={onCloseProperties}
            onMove={onMove}
          />
          {!propertiesCollapsed && (
            <DockBody testId="dock-right-properties-body">
              {properties}
            </DockBody>
          )}
        </div>
      )}
    </div>
  );
}
