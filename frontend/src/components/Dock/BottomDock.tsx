/**
 * BottomDock — tabbed tools dock (Phase B + tier 2c).
 *
 * Hosts console / search / output / problems tabs. The active tab lives in
 * local component state because in-page tab selection is short-lived and
 * not worth a row in `DockPrefs.panelRegions`. The `panelRegions` model
 * sees the whole bottom dock as one swap unit: drag of the tab strip
 * header publishes the canonical `"bottom"` id, matching the v0.82 P1
 * reducer and the `Move →` menu dispatched by App. The internal tab
 * state is orthogonal — a swap puts a different panel id at the bottom
 * slot, the user's previous tab stickiness still controls what they see
 * when a future swap returns `bottom` to its original slot.
 */

import { useState, type DragEvent } from "react";
import ConsoleTab from "../ConsoleTab";
import OutputTab from "../OutputTab";
import ProblemsTab from "../ProblemsTab";
import SearchTab from "../SearchTab";
import { stampDockPanelDrag } from "./drag-payload";
import type { DockableRegion } from "../../hooks/useDockPrefs";
import type { NavigationTarget } from "../CodeEditor";

interface Props {
  visible: boolean;
  onToggle: () => void;
  onClose: () => void;
  /**
   * v0.82 P1 keyboard-equivalent (ADR-0024). Fires the same `movePanel`
   * setter pointer drops invoke, plus the heading announcer in
   * `DockHeader`. The bottom dock currently does not render a DockHeader
   * (it owns its own `<header class="bottom-dock-header">`) so the menu
   * lives on the dedicated `Move →` button below.
   */
  onMove?: (target: DockableRegion) => void;
  /**
   * Friendly name announced when the user moves the panel via the
   * keyboard menu (defaults to "Tools" — same string used in the
   * existing `aria-label`).
   */
  panelTitle?: string;
  /**
   * v0.82 P2 (ADR-0025): float-toggle wiring. When set, an additional
   * `Float` / `Dock` button is rendered next to the `Move` button on
   * the bottom-dock header. When `floating` is true the dock renders
   * nothing (`App.tsx` gates this via `floatingPanelIds`).
   */
  onFloatToggle?: () => void;
  floating?: boolean;
  /** Wired to SearchTab → CodeEditor navigation. */
  onSourceNavigate?: (target: NavigationTarget) => void;
}

type BottomDockTab = "console" | "search" | "output" | "problems";

const TABS: {
  id: BottomDockTab;
  label: string;
  icon: string;
  count: number;
}[] = [
  { id: "console", label: "Console", icon: "📋", count: 0 },
  { id: "search", label: "Search", icon: "🔍", count: 0 },
  { id: "output", label: "Output", icon: "📤", count: 0 },
  { id: "problems", label: "Problems", icon: "⚠", count: 0 },
];

export default function BottomDock({
  visible,
  onToggle,
  onClose,
  onMove,
  panelTitle = "Tools",
  onFloatToggle,
  floating,
  onSourceNavigate,
}: Props) {
  const [activeTab, setActiveTab] = useState<BottomDockTab>("console");

  // v0.82 P1: stamp the canonical bottom-panel id rather than the legacy
  // `bottom-${activeTab}` shape. The `panelRegions` model treats the
  // bottom dock as one swap unit; per-tab granularity is out of scope
  // (ADR-0024 §Consequences).
  const handleDragStart = (e: DragEvent<HTMLDivElement>) => {
    stampDockPanelDrag(e.dataTransfer, "bottom");
  };

  if (!visible || floating) return null;

  return (
    <aside
      className="dock dock-bottom"
      data-testid="dock-bottom"
      data-panel-id="bottom"
      aria-label="Tools"
    >
      <header
        className="bottom-dock-header"
        draggable
        onDragStart={handleDragStart}
        style={{ cursor: "grab" }}
      >
        <div
          className="bottom-dock-tabs"
          role="tablist"
          aria-label="Bottom dock"
        >
          {TABS.map((tab) => (
            <button
              className={`bottom-dock-tab${activeTab === tab.id ? " active" : ""}`}
              data-testid={`bottom-dock-tab-${tab.id}`}
              key={tab.id}
              type="button"
              role="tab"
              aria-selected={activeTab === tab.id}
              aria-controls={`bottom-dock-panel-${tab.id}`}
              onClick={() => setActiveTab(tab.id)}
            >
              <span aria-hidden="true">{tab.icon}</span>
              {tab.label}
              {tab.count > 0 && (
                <span className="bottom-dock-badge">{tab.count}</span>
              )}
            </button>
          ))}
        </div>
        {onMove && (
          <div className="bottom-dock-move">
            <button
              type="button"
              className="bottom-dock-action bottom-dock-action-move"
              data-testid="dock-bottom-move"
              aria-label={`Move ${panelTitle}`}
              aria-haspopup="menu"
              onClick={(e) => {
                // Tiny inline menu so the bottom dock (which renders its
                // own `<header>` and does not use `DockHeader`) still
                // exposes the same destinations. Avoids a separate
                // popover component for a 3-item list.
                e.stopPropagation();
                const wrap = e.currentTarget.parentElement;
                const menu = wrap?.querySelector<HTMLElement>(
                  ".bottom-dock-move-menu",
                );
                if (!menu) return;
                const isOpen = menu.getAttribute("data-open") === "true";
                menu.setAttribute("data-open", isOpen ? "false" : "true");
              }}
            >
              Move ▾
            </button>
            <div
              className="bottom-dock-move-menu"
              role="menu"
              data-open="false"
              data-testid="dock-bottom-move-menu"
            >
              {(
                [
                  { value: "left" as const, label: "Move to Left" },
                  { value: "right" as const, label: "Move to Right" },
                ] satisfies { value: DockableRegion; label: string }[]
              ).map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  role="menuitem"
                  className="bottom-dock-move-item"
                  data-testid={`dock-bottom-move-${opt.value}`}
                  onClick={() => onMove(opt.value)}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </div>
        )}
        {onFloatToggle && (
          <button
            className="bottom-dock-action"
            type="button"
            data-testid="dock-bottom-float"
            aria-label={`Float ${panelTitle} (Shift+F)`}
            aria-pressed={floating ? "true" : "false"}
            title="Float panel (Shift+F)"
            onClick={onFloatToggle}
          >
            Float
          </button>
        )}
        <button
          className="bottom-dock-action"
          type="button"
          onClick={onToggle}
          title="Minimize bottom dock (F7)"
          aria-label="Minimize bottom dock"
        >
          −
        </button>
        <button
          className="bottom-dock-action"
          type="button"
          onClick={onClose}
          title="Close bottom dock"
          aria-label="Close bottom dock"
        >
          ×
        </button>
      </header>
      <div
        className="bottom-dock-content"
        id={`bottom-dock-panel-${activeTab}`}
        role="tabpanel"
      >
        {activeTab === "console" && <ConsoleTab />}
        {activeTab === "search" && <SearchTab onSourceNavigate={onSourceNavigate} />}
        {activeTab === "output" && <OutputTab />}
        {activeTab === "problems" && <ProblemsTab />}
      </div>
    </aside>
  );
}
