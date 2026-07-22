/**
 * CenterDock — Scene viewport area (Phase B, 3-region layout).
 *
 * Stacks SceneTabs on top and the canvas (the existing `.canvas-container`
 * markup from App.tsx) in the middle. Phase C will add BottomDock at the
 * bottom; for now we only render tabs + canvas + a simple bottom-dock toggle
 * placeholder.
 */

import type { ReactNode } from "react";
import SceneTabs from "../SceneTabs";
import type { SceneInfo } from "../../hooks/useScenes";

interface SceneTabsProps {
  scenes: SceneInfo[];
  currentId: string | null;
  onTabClick: (id: string) => void;
  onNewScene: (name: string) => void;
  onDeleteScene: (id: string) => void;
  onRenameScene: (id: string, newName: string) => void;
}

interface Props {
  canvas: ReactNode;
  scenes: SceneInfo[];
  currentId: string | null;
  onTabClick: (id: string) => void;
  onNewScene: (name: string) => void;
  onDeleteScene: (id: string) => void;
  onRenameScene: (id: string, newName: string) => void;
}

export default function CenterDock({
  canvas,
  scenes,
  currentId,
  onTabClick,
  onNewScene,
  onDeleteScene,
  onRenameScene,
}: Props) {
  const tabsProps: SceneTabsProps = {
    scenes,
    currentId,
    onTabClick,
    onNewScene,
    onDeleteScene,
    onRenameScene,
  };
  return (
    <div
      className="dock dock-center"
      data-testid="dock-center"
      data-panel-id="center"
    >
      <SceneTabs {...tabsProps} />
      <div className="dock-center-canvas">{canvas}</div>
    </div>
  );
}
