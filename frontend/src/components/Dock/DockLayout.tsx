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
 *
 * v0.82 P1 (drag-and-dock region swap, ADR-0024): the left/right/bottom
 * region containers are now drop targets for the v0.81 Tier 1c
 * `application/x-dock-panel` MIME. Each region accepts a single
 * `onMovePanel(panelId, region)` callback that funnels into the
 * `useDockResize.movePanel` reducer — pointer drop and the keyboard
 * `Move →` menu share that exact setter. The center container is
 * explicitly protected (`data-drop-allowed="false"` + no `onDragOver` /
 * `onDrop` handlers) so accidental drops on the scene viewport cannot
 * invoke the swap.
 *
 * Compact mode (viewport < 1280 px): the layout switches to a single
 * column with a tab bar for switching between panels. The center viewport
 * remains the primary tab; other panels are accessible via tabs.
 */

import { useState, type DragEvent } from "react";
import type { ReactNode } from "react";
import DockDivider from "./DockDivider";
import { DOCK_PANEL_MIME, isDockPanelDrag } from "./drag-payload";
import type { DockableRegion, PanelId } from "../../hooks/useDockPrefs";
import { useViewportMode } from "../../hooks/useViewportMode";

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
  statusBarHeight: number;
  onResizeLeft: (deltaPx: number) => void;
  onResizeRight: (deltaPx: number) => void;
  onResizeBottom: (deltaPx: number) => void;
  onResizeStatusBar: (deltaPx: number) => void;
  onResetLeft: () => void;
  onResetRight: () => void;
  onResetBottom: () => void;
  onResetStatusBar: () => void;
  leftVisible: boolean;
  bottomVisible: boolean;
  /**
   * Pointer-drop + keyboard-equivalent setter (ADR-0024). Defaults to a
   * no-op so isolated component tests can render without wiring the full
   * state owner.
   */
  onMovePanel?: (panelId: PanelId, region: DockableRegion) => void;
}

/**
 * Returns true when the dataTransfer looks like a v0.81 Tier 1c
 * dock-panel drag (i.e. carries our custom MIME). Used to gate
 * `preventDefault()` so foreign drags (file drops, text selections, etc.)
 * still bubble to the editor's other drop targets like the canvas
 * asset-drop handler.
 */
function hasDockPanelDrag(e: DragEvent<HTMLElement>): boolean {
  return isDockPanelDrag(e.dataTransfer?.types);
}

/** Compact-mode panel tabs. */
type CompactTab = "assets" | "scene" | "outline" | "properties" | "tools";

const COMPACT_TABS: { id: CompactTab; label: string }[] = [
  { id: "assets", label: "Assets" },
  { id: "scene", label: "Scene" },
  { id: "outline", label: "Outline" },
  { id: "properties", label: "Properties" },
  { id: "tools", label: "Tools" },
];

/** Compact mode layout: single column with tabs for panels. */
function CompactLayout({
  menu,
  status,
  left,
  center,
  right,
  bottom,
  bottomVisible,
}: {
  menu: ReactNode;
  status: ReactNode;
  left: ReactNode;
  center: ReactNode;
  right: ReactNode;
  bottom?: ReactNode;
  bottomVisible: boolean;
}) {
  const [activeTab, setActiveTab] = useState<CompactTab>("scene");

  const renderActivePanel = () => {
    switch (activeTab) {
      case "assets":
        return left;
      case "scene":
        return center;
      case "outline":
      case "properties":
        // RightDock renders both outline and properties in a split view.
        // In compact mode we render the whole RightDock for either tab
        // (the user can still use both sub-panels inside it).
        return right;
      case "tools":
        return bottom ?? null;
    }
  };

  return (
    <div
      className="dock-layout dock-layout--compact"
      data-testid="dock-layout"
      data-compact="true"
      style={{ display: "flex", flexDirection: "column", height: "100%" }}
    >
      {/* Menu always spans full width */}
      <div
        className="dock-layout-region dock-layout-menu"
        style={{ flexShrink: 0 }}
        data-testid="dock-region-menu"
      >
        {menu}
      </div>

      {/* Tab bar */}
      <div
        className="dock-layout-compact-tabs"
        role="tablist"
        aria-label="Dock panels"
        data-testid="dock-compact-tabs"
        style={{
          display: "flex",
          flexShrink: 0,
          borderBottom: "1px solid var(--color-border)",
          background: "var(--color-surface)",
          overflowX: "auto",
          gap: "2px",
          padding: "0 4px",
        }}
      >
        {COMPACT_TABS.map((tab) => {
          // Hide Tools tab if bottom dock is not visible
          if (tab.id === "tools" && !bottomVisible) return null;
          return (
            <button
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={activeTab === tab.id}
              data-testid={`dock-compact-tab-${tab.id}`}
              onClick={() => setActiveTab(tab.id)}
              style={{
                padding: "6px 12px",
                border: "none",
                borderBottom:
                  activeTab === tab.id
                    ? "2px solid var(--color-accent)"
                    : "2px solid transparent",
                background: "transparent",
                color:
                  activeTab === tab.id
                    ? "var(--color-ink)"
                    : "var(--color-ink-muted)",
                cursor: "pointer",
                fontSize: "var(--fs-sm)",
                whiteSpace: "nowrap",
              }}
            >
              {tab.label}
            </button>
          );
        })}
      </div>

      {/* Active panel content — fills remaining space */}
      <div
        style={{ flex: 1, overflow: "hidden", minHeight: 0 }}
        role="tabpanel"
        data-testid={`dock-compact-panel-${activeTab}`}
      >
        {renderActivePanel()}
      </div>

      {/* Status bar */}
      <div
        className="dock-layout-region dock-layout-status"
        style={{ flexShrink: 0, position: "relative" }}
        data-testid="dock-region-status"
      >
        {status}
      </div>
    </div>
  );
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
  statusBarHeight,
  onResizeLeft,
  onResizeRight,
  onResizeBottom,
  onResizeStatusBar,
  onResetLeft,
  onResetRight,
  onResetBottom,
  onResetStatusBar,
  leftVisible,
  bottomVisible,
  onMovePanel = () => undefined,
}: Props) {
  const { mode } = useViewportMode();

  // Track the region currently under the pointer so the visual indicator
  // (`data-drop-active="true"`) follows the cursor. Cleared on dragleave
  // and on every successful drop. We keep this as a small piece of local
  // UI state — the source of truth for the actual move lives in
  // `useDockResize`, not here.
  const [activeRegion, setActiveRegion] = useState<DockableRegion | null>(null);

  const handleRegionDragOver =
    (region: DockableRegion) => (e: DragEvent<HTMLDivElement>) => {
      if (!hasDockPanelDrag(e)) return;
      e.preventDefault();
      e.dataTransfer.dropEffect = "move";
      if (activeRegion !== region) setActiveRegion(region);
    };

  const handleRegionDragLeave =
    (region: DockableRegion) => (e: DragEvent<HTMLDivElement>) => {
      // `relatedTarget` may be null on dragleave when leaving the window;
      // clear unconditionally when the active region matches and we're
      // leaving the container.
      if (activeRegion === region) setActiveRegion(null);
    };

  const handleRegionDrop =
    (region: DockableRegion) => (e: DragEvent<HTMLDivElement>) => {
      if (!hasDockPanelDrag(e)) return;
      e.preventDefault();
      setActiveRegion(null);
      const panelId = e.dataTransfer.getData(DOCK_PANEL_MIME) as PanelId;
      if (!panelId) return;
      onMovePanel(panelId, region);
    };

  // ── Compact mode: single-column tabbed layout ──────────────────────────
  if (mode === "compact") {
    return (
      <CompactLayout
        menu={menu}
        status={status}
        left={left}
        center={center}
        right={right}
        bottom={bottom}
        bottomVisible={bottomVisible}
      />
    );
  }

  // ── Desktop mode: 3-region CSS Grid ───────────────────────────────────

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
      data-compact="false"
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
        className={`dock-layout-region dock-layout-left${activeRegion === "left" ? " dock-layout-region--drop-active" : ""}`}
        style={{ gridArea: "left", position: "relative", minWidth: 0 }}
        data-testid="dock-region-left"
        data-region="left"
        data-drop-allowed="true"
        onDragOver={handleRegionDragOver("left")}
        onDragLeave={handleRegionDragLeave("left")}
        onDrop={handleRegionDrop("left")}
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
        data-region="center"
        data-drop-allowed="false"
      >
        {center}
      </div>
      {bottomVisible && bottom && (
        <div
          className={`dock-layout-bottom${activeRegion === "bottom" ? " dock-layout-region--drop-active" : ""}`}
          style={{ position: "relative", gridArea: "bottom" }}
          data-testid="dock-region-bottom"
          data-region="bottom"
          data-drop-allowed="true"
          onDragOver={handleRegionDragOver("bottom")}
          onDragLeave={handleRegionDragLeave("bottom")}
          onDrop={handleRegionDrop("bottom")}
        >
          {bottom}
          <DockDivider
            orientation="horizontal"
            testId="dock-divider-bottom"
            onResize={onResizeBottom}
            onReset={onResetBottom}
          />
          {/* Status-bar drag handle sits on the bottom edge of the bottom
             dock so it doesn't compete with the bottom-dock divider above
             (which is at top: 0). This avoids the two handles overlapping
             in the same pixel row. */}
          <DockDivider
            orientation="horizontal"
            testId="dock-divider-status"
            onResize={onResizeStatusBar}
            onReset={onResetStatusBar}
          />
        </div>
      )}
      <div
        className={`dock-layout-region dock-layout-right${activeRegion === "right" ? " dock-layout-region--drop-active" : ""}`}
        style={{ gridArea: "right", position: "relative", minWidth: 0 }}
        data-testid="dock-region-right"
        data-region="right"
        data-drop-allowed="true"
        onDragOver={handleRegionDragOver("right")}
        onDragLeave={handleRegionDragLeave("right")}
        onDrop={handleRegionDrop("right")}
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
        style={{ gridArea: "status", position: "relative" }}
        data-testid="dock-region-status"
        data-region="status"
      >
        {status}
      </div>
    </div>
  );
}
