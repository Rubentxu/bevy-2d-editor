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
 *
 * v0.82 P2 (ADR-0025): adds `onFloatToggleOutline` /
 * `onFloatToggleProperties` + `outlineFloating` / `propertiesFloating`.
 * Each sub-panel can be lifted into its own FloatingPanel portal —
 * independently. The parent (`App.tsx`) gates render: when a sub-panel
 * is floating, RightDock simply omits that section from the dock grid
 * and the FloatingPanel portal takes over.
 */

import type { DragEvent, ReactNode } from "react";
import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import DockDivider from "./DockDivider";
import { stampDockPanelDrag } from "./drag-payload";
import type { DockableRegion } from "../../hooks/useDockPrefs";

type EditorMode = "scene" | "asset-authoring" | "logic" | "code" | "play" | "world";

interface Props {
  visible: boolean;
  outlineVisible: boolean;
  propertiesVisible: boolean;
  outlineCollapsed: boolean;
  propertiesCollapsed: boolean;
  topHeightPct: number;
  outline: ReactNode;
  properties: ReactNode;
  /** v0.82 P2 (ADR-0025) Phase C T3.1: drives mode-aware header titles. */
  editorMode?: EditorMode;
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
  /** v0.82 P2 (ADR-0025): float-toggle per sub-panel. */
  onFloatToggleOutline?: () => void;
  onFloatToggleProperties?: () => void;
  outlineFloating?: boolean;
  propertiesFloating?: boolean;
}

/** Derive mode-aware header titles for the right dock panels (Phase C T3.1).
 *
 * The outline body (outlinePanelContent in App.tsx) is non-empty only in
 * scene and asset-authoring modes. In logic/code/play the outline body is
 * empty, so we use "Outline" — not a mode label that falsely promises content.
 */
function getOutlineTitle(editorMode: EditorMode = "scene"): string {
  switch (editorMode) {
    case "asset-authoring":
      return "Project Assets";
    case "scene":
      return "Outline";
    // logic/code/play/world: outline body is empty — use generic "Outline"
    case "logic":
    case "code":
    case "play":
    case "world":
      return "Outline";
  }
}

/** Derive mode-aware header titles for the right dock properties panel.
 *
 * The properties body (propertiesPanelContent in App.tsx) is non-empty only in
 * scene and asset-authoring modes. In logic/code/play it is empty, so we use
 * the generic "Properties" label rather than a mode label that promises content.
 */
function getPropertiesTitle(editorMode: EditorMode = "scene"): string {
  switch (editorMode) {
    case "asset-authoring":
      return "Authoring";
    case "scene":
      return "Properties";
    // logic/code/play/world: properties body is empty — use generic "Properties"
    case "logic":
    case "code":
    case "play":
    case "world":
      return "Properties";
  }
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
  editorMode,
  onToggleCollapseOutline,
  onToggleCollapseProperties,
  onCloseOutline,
  onCloseProperties,
  onResizeSplit,
  onResetSplit,
  onOpen,
  onMove,
  onFloatToggleOutline,
  onFloatToggleProperties,
  outlineFloating,
  propertiesFloating,
}: Props) {
  const outlineTitle = getOutlineTitle(editorMode);
  const propertiesTitle = getPropertiesTitle(editorMode);
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
      {outlineVisible && !outlineFloating && (
        <>
          <div
            className="dock dock-right-top"
            style={{ flexBasis: `${topHeightPct}%` }}
            data-testid="dock-right-outline"
            data-panel-id="right-outline"
          >
            <DockHeader
              title={outlineTitle}
              testId="dock-right-outline-header"
              collapsed={outlineCollapsed}
              draggable
              onDragStart={handleOutlineDragStart}
              onToggleCollapse={onToggleCollapseOutline}
              onClose={onCloseOutline}
              onMove={onMove}
              onFloatToggle={onFloatToggleOutline}
              floating={outlineFloating}
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
      {propertiesVisible && !propertiesFloating && (
        <div
          className="dock dock-right-bottom"
          style={{ flexBasis: `${100 - topHeightPct}%` }}
          data-testid="dock-right-properties"
          data-panel-id="right-properties"
        >
          <DockHeader
            title={propertiesTitle}
            testId="dock-right-properties-header"
            collapsed={propertiesCollapsed}
            draggable
            onDragStart={handlePropertiesDragStart}
            onToggleCollapse={onToggleCollapseProperties}
            onClose={onCloseProperties}
            onMove={onMove}
            onFloatToggle={onFloatToggleProperties}
            floating={propertiesFloating}
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
