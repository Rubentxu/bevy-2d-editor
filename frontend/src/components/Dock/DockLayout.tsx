/**
 * DockLayout — CSS Grid host for the 3-region dock layout.
 *
 * Phase B (Defold-inspired redesign): wraps MenuBar / StatusBar / 3 dock
 * regions in a CSS Grid that consumes the dock-dimension custom properties
 * (`--dock-left-w`, `--dock-right-w`, `--dock-bottom-h`) maintained by
 * `useDockResize`. The grid template areas are:
 *
 *   ┌──────────────────────────────────────────┐
 *   │              menu                        │
 *   ├────────┬───────────────────┬─────────────┤
 *   │        │                   │   outline   │
 *   │  left  │      center       │             │
 *   │        │                   ├─────────────┤
 *   │        │                   │ properties  │
 *   ├────────┴───────────────────┴─────────────┤
 *   │              status                      │
 *   └──────────────────────────────────────────┘
 *
 * The dividers (between left/center, center/right, and inside the right
 * dock) live on top of the grid edges and report drag deltas back to
 * `useDockResize`.
 */

import type { ReactNode } from "react";
import DockDivider from "./DockDivider";

interface Props {
  menu: ReactNode;
  status: ReactNode;
  left: ReactNode;
  center: ReactNode;
  right: ReactNode;
  bottom?: ReactNode;
  leftWidth: number;
  rightWidth: number;
  bottomHeight: number;
  onResizeLeft: (deltaPx: number) => void;
  onResizeRight: (deltaPx: number) => void;
  onResizeBottom: (deltaPx: number) => void;
  onResetLeft: () => void;
  onResetRight: () => void;
  onResetBottom: () => void;
  leftVisible: boolean;
  bottomVisible: boolean;
}

export default function DockLayout({
  menu,
  status,
  left,
  center,
  right,
  bottom,
  leftWidth,
  rightWidth,
  bottomHeight,
  onResizeLeft,
  onResizeRight,
  onResizeBottom,
  onResetLeft,
  onResetRight,
  onResetBottom,
  leftVisible,
  bottomVisible,
}: Props) {
  // CSS Grid template rows. When the bottom dock is hidden the bottom row is
  // a no-op (`auto`) so the center region expands into the freed space.
  const rows = bottomVisible
    ? "var(--menu-h, 40px) 1fr var(--dock-bottom-h, 240px) var(--status-h, 24px)"
    : "var(--menu-h, 40px) 1fr var(--status-h, 24px)";
  // CSS Grid template areas match the row template above.
  const areas = bottomVisible
    ? `"menu menu menu" "left center right" "left bottom right" "status status status"`
    : `"menu menu menu" "left center right" "status status status"`;

  return (
    <div
      className="dock-layout"
      data-testid="dock-layout"
      style={{
        display: "grid",
        gridTemplateColumns: `${leftVisible ? `${leftWidth}px ` : ""}1fr ${rightWidth}px`,
        gridTemplateRows: rows,
        gridTemplateAreas: areas,
      }}
    >
      <div
        className="dock-layout-region dock-layout-menu"
        style={{ gridArea: "menu" }}
      >
        {menu}
      </div>
      <div
        className="dock-layout-region dock-layout-left"
        style={{ gridArea: "left", position: "relative", minWidth: 0 }}
        data-testid="dock-region-left"
      >
        {left}
        {leftVisible && (
          <DockDivider
            orientation="vertical"
            testId="dock-divider-left"
            onResize={onResizeLeft}
            onReset={onResetLeft}
          />
        )}
      </div>
      <div
        className="dock-layout-region dock-layout-center"
        style={{ gridArea: "center", minWidth: 0, minHeight: 0 }}
        data-testid="dock-region-center"
      >
        {center}
      </div>
      {bottomVisible && bottom && (
        <div
          className="dock-layout-bottom"
          style={{ position: "relative", gridArea: "bottom" }}
          data-testid="dock-region-bottom"
        >
          {bottom}
          <DockDivider
            orientation="horizontal"
            testId="dock-divider-bottom"
            onResize={onResizeBottom}
            onReset={onResetBottom}
          />
        </div>
      )}
      <div
        className="dock-layout-region dock-layout-right"
        style={{ gridArea: "right", position: "relative", minWidth: 0 }}
        data-testid="dock-region-right"
      >
        {right}
        <DockDivider
          orientation="vertical"
          testId="dock-divider-right"
          onResize={onResizeRight}
          onReset={onResetRight}
        />
      </div>
      <div
        className="dock-layout-region dock-layout-status"
        style={{ gridArea: "status" }}
      >
        {status}
      </div>
    </div>
  );
}
