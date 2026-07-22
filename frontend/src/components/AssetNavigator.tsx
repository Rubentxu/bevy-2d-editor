/**
 * AssetNavigator — collapsible tree of asset buckets in the left dock.
 *
 * Phase B (Defold-inspired redesign): renders two top-level folders
 * (Project / Built-in) with child sections whose counts come from the existing
 * hooks (useScenes, useSceneAssets, useCodeFiles, useAssetFiles). For v0.80
 * the tree shows structure + counts only — drag-and-drop wiring lands in
 * v0.81 per tasks.md §B.6.
 */

import { useMemo, useState } from "react";
import { useScenes } from "../hooks/useScenes";
import { useSceneAssets } from "../hooks/useSceneAssets";
import { useCodeFiles } from "../hooks/useCodeFiles";
import { useAssetFiles } from "../hooks/useAssetFiles";

interface Section {
  key: string;
  label: string;
  icon: string;
  count: number;
}

function CountBadge({ count }: { count: number }) {
  return <span className="asset-navigator-count">{count}</span>;
}

interface FolderProps {
  label: string;
  icon: string;
  defaultOpen: boolean;
  sections: Section[];
  testId: string;
}

function Folder({ label, icon, defaultOpen, sections, testId }: FolderProps) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="asset-navigator-folder" data-testid={testId}>
      <button
        type="button"
        className="asset-navigator-folder-header"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        <span className="asset-navigator-caret">{open ? "▾" : "▸"}</span>
        <span className="asset-navigator-icon">{icon}</span>
        <span className="asset-navigator-label">{label}</span>
      </button>
      {open && (
        <ul className="asset-navigator-section-list">
          {sections.map((s) => (
            <li
              key={s.key}
              className="asset-navigator-section"
              data-testid={`asset-navigator-section-${s.key}`}
            >
              <span className="asset-navigator-icon">{s.icon}</span>
              <span className="asset-navigator-label">{s.label}</span>
              <CountBadge count={s.count} />
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default function AssetNavigator() {
  const { scenes } = useScenes();
  const { entries } = useSceneAssets();
  const { files } = useCodeFiles();
  const { files: assetFiles } = useAssetFiles();

  const projectSections = useMemo<Section[]>(
    () => [
      { key: "scenes", label: "Scenes", icon: "🎬", count: scenes.length },
      {
        key: "scene-assets",
        label: "Scene Assets",
        icon: "🎨",
        count: entries.length,
      },
      { key: "code", label: "Code", icon: "📝", count: files.length },
      { key: "tilesets", label: "Tilesets", icon: "🗺️", count: 0 },
      { key: "files", label: "Files", icon: "🖼️", count: assetFiles.length },
    ],
    [scenes.length, entries.length, files.length, assetFiles.length],
  );

  const builtinSections = useMemo<Section[]>(
    () => [
      { key: "primitives", label: "Primitives", icon: "⭐", count: 0 },
      { key: "samples", label: "Samples", icon: "📦", count: 0 },
    ],
    [],
  );

  return (
    <div className="asset-navigator" data-testid="asset-navigator">
      <Folder
        label="Project"
        icon="📁"
        defaultOpen
        sections={projectSections}
        testId="asset-navigator-folder-project"
      />
      <Folder
        label="Built-in"
        icon="📁"
        defaultOpen={false}
        sections={builtinSections}
        testId="asset-navigator-folder-builtin"
      />
    </div>
  );
}
