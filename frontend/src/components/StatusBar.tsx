/**
 * StatusBar — 7-segment status bar (Phase D, Defold-inspired redesign).
 *
 * Segments:
 *   1. position     — cursor world-space coords from `useCanvasViewport`
 *   2. selection    — currently selected entity + count of entities in the
 *                     open scene
 *   3. project      — top-level project name from `useScenes`
 *   4. scene+dirty  — current scene + dirty indicator (●/○) for unsaved ops
 *   5. zoom         — viewport zoom percentage, click opens a zoom dropdown
 *                     (25/50/100/200%/Fit)
 *   6. fps          — frames-per-second from `get_preview_metrics_wasm`
 *   7. build        — build status (Ready / Building / Error) plus a
 *                     dropdown with Rebuild WASM / Export Rust / Open Build
 *                     Output actions
 *
 * Each segment is rendered by the shared `StatusSegment` component and is
 * either a clickable button (when an action/dropdown is available) or a
 * static `<span>` (when the segment is purely informational). All clickable
 * segments carry a `data-testid` so the test suite can introspect them.
 *
 * The dropdowns for zoom / scene / build are inline mini-menus rendered
 * directly by StatusBar — they use the same CSS classes as the menu bar
 * dropdowns so the visual treatment stays consistent.
 */

import { useEffect, useRef, useState } from "react";
import { useCanvasViewport } from "../hooks/useCanvasViewport";
import { useLogState } from "../hooks/useLogState";
import { useSceneState } from "../hooks/useSceneState";
import { useScenes } from "../hooks/useScenes";
import StatusSegment from "./StatusBar/StatusSegment";
import MenuSeparator from "./Menu/MenuSeparator";

interface PreviewMetrics {
  fps?: number;
  frame_time_ms?: number;
  entity_count?: number;
  instance_count?: number;
}

function parseMetrics(value: unknown): PreviewMetrics {
  if (typeof value === "string") {
    try {
      return JSON.parse(value) as PreviewMetrics;
    } catch {
      return {};
    }
  }
  return value && typeof value === "object" ? (value as PreviewMetrics) : {};
}

const ZOOM_PRESETS: { label: string; value: number }[] = [
  { label: "25%", value: 0.25 },
  { label: "50%", value: 0.5 },
  { label: "100%", value: 1 },
  { label: "200%", value: 2 },
];

export interface StatusBarProps {
  selectedEntityId?: string | null;
  /**
   * Optional callbacks invoked when the user picks a value from the zoom,
   * scene, or build dropdowns. App.tsx wires these up so StatusBar stays
   * unaware of higher-level state management (toasts, WASM rebuilds, etc.).
   */
  onZoomSelect?: (zoom: number) => void;
  onZoomFit?: () => void;
  onSaveScene?: (sceneId: string) => void;
  onCloseScene?: (sceneId: string) => void;
  onSaveAllScenes?: () => void;
  onCloseAllScenes?: () => void;
  onRebuildWasm?: () => void;
  onExportRust?: () => void;
  onOpenBuildOutput?: () => void;
}

/**
 * StatusBar reads from `useCanvasViewport()` so any hook consumer that needs
 * the zoom value (e.g. App.tsx calling `setZoom`) will trigger a re-render
 * here automatically.
 */
export default function StatusBar(props: StatusBarProps = {}) {
  const {
    worldPos,
    zoom,
    setZoom,
    fitToContent,
  } = useCanvasViewport();
  const { scene } = useSceneState();
  const logState = useLogState();
  const { scenes, currentId } = useScenes();
  const [metrics, setMetrics] = useState<PreviewMetrics>({});

  useEffect(() => {
    const update = () => {
      try {
        const getter = (window as any).get_preview_metrics_wasm;
        if (typeof getter === "function") {
          setMetrics(parseMetrics(getter()));
        }
      } catch {
        setMetrics({});
      }
    };
    update();
    const interval = window.setInterval(update, 500);
    return () => window.clearInterval(interval);
  }, []);

  const currentScene =
    scenes.find((item) => item.id === currentId) ?? null;
  const sceneName =
    currentScene?.name ?? scene?.name ?? "No scene";
  const entityCount = scene?.entities.length ?? 0;
  const instanceCount =
    metrics.instance_count ??
    scene?.entities.filter((entity) => entity.id.startsWith("inst_")).length ??
    0;
  const fps = Number.isFinite(metrics.fps) ? Math.round(metrics.fps!) : "--";
  const position = worldPos
    ? `(${worldPos.x.toFixed(1)}, ${worldPos.y.toFixed(1)})`
    : "(—, —)";
  const projectName = scenes[0]?.name ?? "Untitled";
  const selectedLabel = props.selectedEntityId
    ? scene?.entities.find((e) => e.id === props.selectedEntityId)?.name ??
      "1 selected"
    : "None";

  // Dropdown state — mutually exclusive: only one segment menu open at a time
  // because the status bar is a narrow horizontal strip and stacking floating
  // menus would be a UX nightmare.
  const [openMenu, setOpenMenu] = useState<null | "zoom" | "scene" | "build">(
    null,
  );
  // Build status — defaults to "Ready". Switch to "Building…" via the build
  // callback; revert to "Ready" after a short delay.
  const [buildStatus, setBuildStatus] = useState<
    "Ready" | "Building…" | "Error"
  >("Ready");
  const rootRef = useRef<HTMLDivElement>(null);

  // Close any open menu on outside-click or Escape.
  useEffect(() => {
    if (!openMenu) return;
    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpenMenu(null);
      }
    };
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setOpenMenu(null);
    };
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [openMenu]);

  const handleZoomPick = (value: number) => {
    setZoom(value);
    props.onZoomSelect?.(value);
    setOpenMenu(null);
  };

  const handleFit = () => {
    fitToContent();
    props.onZoomFit?.();
    setOpenMenu(null);
  };

  const handleRebuild = () => {
    setBuildStatus("Building…");
    props.onRebuildWasm?.();
    // Optimistic revert — a real build pipeline would push status updates
    // through an event bus. Reset to "Ready" after a short delay so the UI
    // doesn't get stuck in the "Building…" state in tests / dev.
    window.setTimeout(() => setBuildStatus("Ready"), 1500);
    setOpenMenu(null);
  };

  const zoomValue = `${Math.round(zoom * 100)}%`;
  const dirtyTitle =
    logState.size > 0 ? `Dirty (${logState.size} ops pending)` : "Saved";

  return (
    <div
      className="status-bar"
      data-testid="status-bar"
      ref={rootRef}
    >
      {/* 1. Position (cursor world coords) */}
      <StatusSegment
        testId="status-segment-position"
        label="Pos"
        value={position}
        color="var(--color-ink-muted)"
        title="Cursor world-space coordinates"
      />

      {/* 2. Selection (current entity + counts) */}
      <StatusSegment
        testId="status-segment-selection"
        label="Sel"
        value={selectedLabel}
        title="Selected entity (None if no selection)"
      />
      <StatusSegment
        testId="status-segment-entities"
        label=""
        value={`${entityCount} ${entityCount === 1 ? "entity" : "entities"}`}
        color="var(--color-ink-muted)"
      />
      <StatusSegment
        testId="status-segment-instances"
        label=""
        value={`${instanceCount} ${instanceCount === 1 ? "instance" : "instances"}`}
        color="var(--color-ink-muted)"
      />

      {/* 3. Project */}
      <StatusSegment
        testId="status-segment-project"
        label="Project"
        value={projectName}
        title="Top-level project (from first scene)"
      />

      {/* 4. Scene + dirty indicator */}
      <StatusSegment
        testId="status-segment-scene"
        label="Scene"
        value={sceneName}
        title={dirtyTitle}
        onClick={() => setOpenMenu(openMenu === "scene" ? null : "scene")}
      >
        <span
          className="status-bar-dirty"
          data-testid="status-bar-dirty"
          data-state={logState.size > 0 ? "dirty" : "saved"}
          title={dirtyTitle}
          aria-label={dirtyTitle}
        >
          {logState.size > 0 ? "●" : "○"}
        </span>
      </StatusSegment>

      {/* 5. Zoom (dropdown) */}
      <StatusSegment
        testId="status-segment-zoom"
        label="Zoom"
        value={zoomValue}
        title="Viewport zoom (click to change)"
        onClick={() => setOpenMenu(openMenu === "zoom" ? null : "zoom")}
      />

      {/* 6. FPS */}
      <StatusSegment
        testId="status-segment-fps"
        label="FPS"
        value={String(fps)}
        title="Frames per second (preview runtime)"
      />

      {/* 7. Build status (dropdown) */}
      <StatusSegment
        testId="status-segment-build"
        label="Build"
        value={buildStatus}
        title="Build status (click for actions)"
        color={buildStatus === "Error" ? "var(--color-danger)" : undefined}
        onClick={() => setOpenMenu(openMenu === "build" ? null : "build")}
      />

      {/* ── Inline dropdowns ─────────────────────────────────────────────── */}
      {openMenu === "zoom" && (
        <div
          className="status-segment-dropdown"
          role="menu"
          data-testid="status-zoom-dropdown"
        >
          {ZOOM_PRESETS.map((preset) => (
            <button
              key={preset.value}
              type="button"
              role="menuitem"
              className="menu-item"
              data-testid={`status-zoom-option-${preset.label.replace("%", "")}`}
              onClick={() => handleZoomPick(preset.value)}
            >
              <span>{preset.label}</span>
              <span className="menu-item-shortcut">
                {Math.abs(zoom - preset.value) < 0.001 ? "●" : ""}
              </span>
            </button>
          ))}
          <MenuSeparator />
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-zoom-fit"
            onClick={handleFit}
          >
            <span>Fit</span>
            <span className="menu-item-shortcut">F</span>
          </button>
        </div>
      )}

      {openMenu === "scene" && (
        <div
          className="status-segment-dropdown"
          role="menu"
          data-testid="status-scene-dropdown"
        >
          {scenes.length === 0 && (
            <div className="status-segment-empty">No open scenes</div>
          )}
          {scenes.map((s) => (
            <button
              key={s.id}
              type="button"
              role="menuitem"
              className="menu-item"
              data-testid={`status-scene-option-${s.id}`}
              onClick={() => {
                props.onSaveScene?.(s.id);
                setOpenMenu(null);
              }}
            >
              <span>
                {s.is_active ? "● " : "○ "}
                {s.name}
                {s.is_dirty ? " •" : ""}
              </span>
              <span className="menu-item-shortcut">Save</span>
            </button>
          ))}
          {scenes.length > 0 && <MenuSeparator />}
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-scene-save-all"
            onClick={() => {
              props.onSaveAllScenes?.();
              setOpenMenu(null);
            }}
          >
            <span>Save all</span>
            <span className="menu-item-shortcut">Ctrl+Shift+S</span>
          </button>
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-scene-close-all"
            onClick={() => {
              props.onCloseAllScenes?.();
              setOpenMenu(null);
            }}
          >
            <span>Close all</span>
          </button>
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-scene-close"
            onClick={() => {
              if (currentId) props.onCloseScene?.(currentId);
              setOpenMenu(null);
            }}
          >
            <span>Close scene</span>
            <span className="menu-item-shortcut">
              {currentScene?.name ?? ""}
            </span>
          </button>
        </div>
      )}

      {openMenu === "build" && (
        <div
          className="status-segment-dropdown"
          role="menu"
          data-testid="status-build-dropdown"
        >
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-build-rebuild"
            onClick={handleRebuild}
          >
            <span>Rebuild WASM</span>
            <span className="menu-item-shortcut">Ctrl+B</span>
          </button>
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-build-export"
            onClick={() => {
              props.onExportRust?.();
              setOpenMenu(null);
            }}
          >
            <span>Export Rust</span>
            <span className="menu-item-shortcut">Ctrl+E</span>
          </button>
          <MenuSeparator />
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            data-testid="status-build-open-output"
            onClick={() => {
              props.onOpenBuildOutput?.();
              setOpenMenu(null);
            }}
          >
            <span>Open Build Output</span>
          </button>
        </div>
      )}
    </div>
  );
}
