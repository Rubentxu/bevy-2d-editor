/**
 * LeftDock — Assets browser dock (Phase B, 3-region layout).
 *
 * Renders a DockHeader "Assets" plus a DockBody that hosts AssetNavigator.
 * When the dock is collapsed the body is hidden and a slim icon strip is
 * shown so the toggle target stays reachable (per tasks.md §B.5).
 */

import DockHeader from "./DockHeader";
import DockBody from "./DockBody";
import AssetNavigator from "../AssetNavigator";

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
  if (!visible) {
    return (
      <div
        className="dock dock-left dock-collapsed-strip"
        data-testid="dock-left-strip"
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
    <div className="dock dock-left" data-testid="dock-left">
      <DockHeader
        title="Assets"
        testId="dock-left-header"
        collapsed={collapsed}
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
