/**
 * ModeContextBar — persistent orientation bar for active mode, target, and actions.
 *
 * Phase 2.1 PR1 (ui-workflow-overhaul: Context and Mode Orientation)
 *
 * Displays:
 *   - Mode badge: Scene | Asset Authoring | Logic | Code | Play
 *   - Active target: current scene name, asset logical path, logic graph name, or file
 *   - Dirty indicator (●/○) when the active document has unsaved changes
 *   - Primary mode actions: Play/Stop, Save, Back to Scene
 *
 * Height budget: ~28px. Must NOT increase chrome by more than 32px.
 *
 * CSS tokens: var(--color-*), var(--z-*), var(--space-*)
 * data-testid: mode-context-bar
 */

import type { EditorMode } from "./MenuBar";
import type { LogState } from "../hooks/useLogState";

export interface ModeContextBarProps {
  editorMode: EditorMode;
  /** Scene mode: current scene name */
  currentSceneName?: string | null;
  /** Asset-authoring mode: open asset logical path */
  activeAssetPath?: string | null;
  /** Asset-authoring dirty state */
  assetDirty?: boolean;
  /** Scene mode dirty state */
  sceneDirty?: boolean;
  /** Logic mode: active graph id */
  activeLogicGraphId?: string | null;
  /** Code mode: current file name */
  activeCodeFileName?: string | null;
  /** Play mode: whether we are currently playing */
  isPlaying?: boolean;
  /** Primary mode action: toggle play/stop */
  onTogglePlay?: () => void;
  /** Primary mode action: save (scene or asset) */
  onSave?: () => void;
  /** Asset-authoring: back to scene */
  onBackToScene?: () => void;
  /** Scene mode: can undo (for action availability) */
  canUndo?: boolean;
  /** Scene mode: can redo (for action availability) */
  canRedo?: boolean;
  /** Asset-authoring: can undo */
  assetCanUndo?: boolean;
  /** Asset-authoring: can redo */
  assetCanRedo?: boolean;
}

const MODE_LABELS: Record<EditorMode, string> = {
  scene: "Scene",
  "asset-authoring": "Asset Authoring",
  logic: "Logic",
  code: "Code",
  play: "Play",
  world: "World",
};

const MODE_DOMAINS: Record<EditorMode, string> = {
  scene: "scene",
  "asset-authoring": "asset",
  logic: "logic",
  code: "code",
  play: "runtime",
  world: "world",
};

export default function ModeContextBar({
  editorMode,
  currentSceneName,
  activeAssetPath,
  assetDirty,
  sceneDirty,
  activeLogicGraphId,
  activeCodeFileName,
  isPlaying,
  onTogglePlay,
  onSave,
  onBackToScene,
}: ModeContextBarProps) {
  const isDirty =
    editorMode === "asset-authoring"
      ? assetDirty
      : editorMode === "scene"
        ? sceneDirty
        : false;

  const targetName =
    editorMode === "scene"
      ? (currentSceneName ?? "Untitled")
      : editorMode === "asset-authoring"
        ? (activeAssetPath ?? "No asset open")
        : editorMode === "logic"
          ? (activeLogicGraphId ?? "No graph open")
          : editorMode === "code"
            ? (activeCodeFileName ?? "No file open")
            : null;

  const domainDotClass = `vc-domain-dot vc-domain-dot--${MODE_DOMAINS[editorMode]}`;

  return (
    <div
      className="mode-context-bar"
      data-testid="mode-context-bar"
      style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--space-3)",
        height: "28px",
        padding: "0 var(--space-3)",
        background: "var(--color-surface)",
        borderBottom: "1px solid var(--color-border)",
        fontSize: "var(--fs-sm)",
        color: "var(--color-ink)",
        flexShrink: 0,
        overflow: "hidden",
      }}
    >
      {/* Domain dot */}
      <span
        className={domainDotClass}
        style={{
          width: "7px",
          height: "7px",
          borderRadius: "50%",
          flexShrink: 0,
        }}
        aria-hidden="true"
      />

      {/* Mode badge */}
      <span
        style={{
          fontWeight: 600,
          color: "var(--color-accent-hi)",
          flexShrink: 0,
          textTransform: "uppercase",
          letterSpacing: "0.05em",
          fontSize: "var(--fs-xs)",
        }}
        data-testid="mode-context-bar-mode"
      >
        {MODE_LABELS[editorMode]}
      </span>

      {/* Separator */}
      <span
        aria-hidden="true"
        style={{
          color: "var(--color-border-hi)",
          userSelect: "none",
          flexShrink: 0,
        }}
      >
        /
      </span>

      {/* Active target name */}
      <span
        style={{
          color: "var(--color-ink)",
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
          flex: 1,
          minWidth: 0,
        }}
        data-testid="mode-context-bar-target"
        title={targetName ?? undefined}
      >
        {targetName}
      </span>

      {/* Dirty indicator */}
      {isDirty !== undefined && (
        <span
          aria-label={isDirty ? "Unsaved changes" : "Saved"}
          style={{
            color: isDirty ? "var(--color-warning)" : "var(--color-border-hi)",
            flexShrink: 0,
            fontSize: "var(--fs-sm)",
          }}
          data-testid="mode-context-bar-dirty"
          title={isDirty ? "Unsaved changes" : "Saved"}
        >
          {isDirty ? "●" : "○"}
        </span>
      )}

      {/* Primary mode actions */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--space-1)",
          flexShrink: 0,
        }}
        data-testid="mode-context-bar-actions"
      >
        {/* Play / Stop — scene + play modes */}
        {(editorMode === "scene" || editorMode === "play") && onTogglePlay && (
          <button
            type="button"
            onClick={onTogglePlay}
            data-testid={
              editorMode === "play"
                ? "mode-context-bar-stop-btn"
                : "mode-context-bar-play-btn"
            }
            style={{
              display: "flex",
              alignItems: "center",
              gap: "4px",
              padding: "2px 8px",
              border: "1px solid var(--color-border-hi)",
              borderRadius: "var(--radius-sm)",
              background:
                editorMode === "play"
                  ? "var(--color-danger)"
                  : "var(--color-accent)",
              color:
                editorMode === "play" ? "var(--color-bg)" : "var(--color-bg)",
              fontSize: "var(--fs-xs)",
              fontWeight: 600,
              cursor: "pointer",
              whiteSpace: "nowrap",
            }}
          >
            {editorMode === "play" ? "■ Stop" : "▶ Play"}
          </button>
        )}

        {/* Save — scene + asset-authoring */}
        {(editorMode === "scene" || editorMode === "asset-authoring") &&
          onSave && (
            <button
              type="button"
              onClick={onSave}
              data-testid="mode-context-bar-save-btn"
              disabled={!isDirty}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "4px",
                padding: "2px 8px",
                border: "1px solid var(--color-border-hi)",
                borderRadius: "var(--radius-sm)",
                background: isDirty ? "var(--color-accent)" : "transparent",
                color: isDirty ? "var(--color-bg)" : "var(--color-ink-muted)",
                fontSize: "var(--fs-xs)",
                fontWeight: 600,
                cursor: isDirty ? "pointer" : "not-allowed",
                opacity: isDirty ? 1 : 0.5,
                whiteSpace: "nowrap",
              }}
            >
              Save
            </button>
          )}

        {/* Back to Scene — asset-authoring */}
        {editorMode === "asset-authoring" && onBackToScene && (
          <button
            type="button"
            onClick={onBackToScene}
            data-testid="mode-context-bar-back-btn"
            style={{
              display: "flex",
              alignItems: "center",
              gap: "4px",
              padding: "2px 8px",
              border: "1px solid var(--color-border-hi)",
              borderRadius: "var(--radius-sm)",
              background: "transparent",
              color: "var(--color-ink)",
              fontSize: "var(--fs-xs)",
              fontWeight: 600,
              cursor: "pointer",
              whiteSpace: "nowrap",
            }}
          >
            ← Back
          </button>
        )}
      </div>
    </div>
  );
}
