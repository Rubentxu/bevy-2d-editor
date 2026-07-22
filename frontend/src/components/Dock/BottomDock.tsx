import { useState, type DragEvent } from "react";
import ConsoleTab from "../ConsoleTab";
import OutputTab from "../OutputTab";
import ProblemsTab from "../ProblemsTab";
import SearchTab from "../SearchTab";
import { DOCK_PANEL_MIME } from "./DockPanel";

interface Props {
  visible: boolean;
  onToggle: () => void;
  onClose: () => void;
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

export default function BottomDock({ visible, onToggle, onClose }: Props) {
  const [activeTab, setActiveTab] = useState<BottomDockTab>("console");

  // Tier 1c: when the user grabs the tab strip, stamp the active tab id so
  // the region-swap hook (v0.82) can decide which bottom panel to relocate.
  const handleDragStart = (e: DragEvent<HTMLDivElement>) => {
    e.dataTransfer.setData(DOCK_PANEL_MIME, `bottom-${activeTab}`);
    e.dataTransfer.setData("text/plain", `bottom-${activeTab}`);
    e.dataTransfer.effectAllowed = "move";
  };

  if (!visible) return null;

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
        {activeTab === "search" && <SearchTab />}
        {activeTab === "output" && <OutputTab />}
        {activeTab === "problems" && <ProblemsTab />}
      </div>
    </aside>
  );
}
