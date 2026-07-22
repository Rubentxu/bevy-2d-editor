/**
 * RightDock — Outline (top) + Properties (bottom) split (Phase B).
 *
 * Each side gets its own DockHeader + DockBody and a DockDivider between them.
 * The vertical split ratio lives in `topHeightPct` (default 60%) so the user
 * can drag the divider to resize the two sections independently from the
 * outer right-dock width.
 */

import type { DragEvent, ReactNode } from "react";
import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import DockDivider from "./DockDivider";
import { DOCK_PANEL_MIME } from "./DockPanel";

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
}: Props) {
  // Tier 1c: stamp each half-panel id into the dataTransfer so the future
  // region-swap hook (v0.82) knows which panel the user picked up.
  const handleOutlineDragStart = (e: DragEvent<HTMLDivElement>) => {
    e.dataTransfer.setData(DOCK_PANEL_MIME, "outline");
    e.dataTransfer.setData("text/plain", "outline");
    e.dataTransfer.effectAllowed = "move";
  };
  const handlePropertiesDragStart = (e: DragEvent<HTMLDivElement>) => {
    e.dataTransfer.setData(DOCK_PANEL_MIME, "properties");
    e.dataTransfer.setData("text/plain", "properties");
    e.dataTransfer.effectAllowed = "move";
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
    <div className="dock dock-right" data-testid="dock-right" data-panel-id="right">
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
          />
          {!propertiesCollapsed && (
            <DockBody testId="dock-right-properties-body">{properties}</DockBody>
          )}
        </div>
      )}
    </div>
  );
}
