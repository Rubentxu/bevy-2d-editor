/**
 * AppHeader — hosts the MenuBar and ModeContextBar in a vertical stack.
 *
 * Phase 2.1 PR1 (ui-workflow-overhaul: Context and Mode Orientation)
 *
 * AppHeader replaces MenuBar in the DockLayout `menu` prop, stacking:
 *   1. MenuBar (existing — the application menu + toolbar)
 *   2. ModeContextBar (~28px — active mode, target, dirty state, primary actions)
 *
 * This component is pass-through — it receives all MenuBar props plus
 * ModeContextBar props and renders both. No additional state lives here.
 */

import MenuBar, { type EditorMode } from "./MenuBar";
import ModeContextBar from "./ModeContextBar";
import type { LogState } from "../hooks/useLogState";

export interface AppHeaderProps {
  // ── MenuBar props (forwarded directly) ──────────────────────────────────
  editorMode?: EditorMode;
  onOpenAssets?: () => void;
  onBackToScene?: () => void;
  onOpenLogic?: () => void;
  onOpenCode?: () => void;
  onOpenWorldWorkspace?: () => void;
  logState: LogState;
  onUndo: () => void;
  onRedo: () => void;
  onSave: () => void;
  onSaveAs?: () => void;
  onLoad: () => void;
  onExportRust: () => void;
  onNewScene?: () => void;
  onDeleteEntity?: () => void;
  selectedEntityId?: string | null;
  onToggleAI: () => void;
  aiPanelOpen: boolean;
  onToggleValidationCenter: () => void;
  validationCenterOpen: boolean;
  onToggleTileset: () => void;
  tilesetPanelOpen: boolean;
  onToggleAutoLayer: () => void;
  autoLayerPanelOpen: boolean;
  onTogglePlay?: () => void;
  onOpenSearch?: () => void;
  onOpenCheatSheet?: () => void;
  onWelcomeTour?: () => void;
  onAbout?: () => void;
  onToggleLeftDock?: () => void;
  onToggleOutlineDock?: () => void;
  onTogglePropertiesDock?: () => void;
  onToggleFullscreen?: () => void;
  onResetLayout?: () => void;
  onApplyPreset?: (presetId: string) => void;
  onSaveWorkspacePreset?: () => void;

  // ── ModeContextBar props ─────────────────────────────────────────────────
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
  /** Scene mode: can undo */
  canUndo?: boolean;
  /** Scene mode: can redo */
  canRedo?: boolean;
  /** Asset-authoring: can undo */
  assetCanUndo?: boolean;
  /** Asset-authoring: can redo */
  assetCanRedo?: boolean;
}

export default function AppHeader(props: AppHeaderProps) {
  const {
    // MenuBar
    editorMode,
    onOpenAssets,
    onBackToScene,
    onOpenLogic,
    onOpenCode,
    logState,
    onUndo,
    onRedo,
    onSave,
    onSaveAs,
    onLoad,
    onExportRust,
    onNewScene,
    onDeleteEntity,
    selectedEntityId,
    onToggleAI,
    aiPanelOpen,
    onToggleValidationCenter,
    validationCenterOpen,
    onToggleTileset,
    tilesetPanelOpen,
    onToggleAutoLayer,
    autoLayerPanelOpen,
    onTogglePlay,
    onOpenSearch,
    onOpenCheatSheet,
    onWelcomeTour,
    onAbout,
    onToggleLeftDock,
    onToggleOutlineDock,
    onTogglePropertiesDock,
    onToggleFullscreen,
    onResetLayout,
    onApplyPreset,
    onSaveWorkspacePreset,
    // ModeContextBar
    currentSceneName,
    activeAssetPath,
    assetDirty,
    sceneDirty,
    activeLogicGraphId,
    activeCodeFileName,
    isPlaying,
    canUndo,
    canRedo,
    assetCanUndo,
    assetCanRedo,
  } = props;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
      }}
    >
      <MenuBar
        editorMode={editorMode}
        onOpenAssets={onOpenAssets}
        onBackToScene={onBackToScene}
        onOpenLogic={onOpenLogic}
        onOpenCode={onOpenCode}
        logState={logState}
        onUndo={onUndo}
        onRedo={onRedo}
        onSave={onSave}
        onSaveAs={onSaveAs}
        onLoad={onLoad}
        onExportRust={onExportRust}
        onNewScene={onNewScene}
        onDeleteEntity={onDeleteEntity}
        selectedEntityId={selectedEntityId}
        onToggleAI={onToggleAI}
        aiPanelOpen={aiPanelOpen}
        onToggleValidationCenter={onToggleValidationCenter}
        validationCenterOpen={validationCenterOpen}
        onToggleTileset={onToggleTileset}
        tilesetPanelOpen={tilesetPanelOpen}
        onToggleAutoLayer={onToggleAutoLayer}
        autoLayerPanelOpen={autoLayerPanelOpen}
        onTogglePlay={onTogglePlay}
        onOpenSearch={onOpenSearch}
        onOpenCheatSheet={onOpenCheatSheet}
        onWelcomeTour={onWelcomeTour}
        onAbout={onAbout}
        onToggleLeftDock={onToggleLeftDock}
        onToggleOutlineDock={onToggleOutlineDock}
        onTogglePropertiesDock={onTogglePropertiesDock}
        onToggleFullscreen={onToggleFullscreen}
        onResetLayout={onResetLayout}
        onApplyPreset={onApplyPreset}
        onSaveWorkspacePreset={onSaveWorkspacePreset}
      />
      <ModeContextBar
        editorMode={editorMode ?? "scene"}
        currentSceneName={currentSceneName}
        activeAssetPath={activeAssetPath}
        assetDirty={assetDirty}
        sceneDirty={sceneDirty}
        activeLogicGraphId={activeLogicGraphId}
        activeCodeFileName={activeCodeFileName}
        isPlaying={isPlaying}
        onTogglePlay={onTogglePlay}
        onSave={onSave}
        onBackToScene={onBackToScene}
        canUndo={canUndo}
        canRedo={canRedo}
        assetCanUndo={assetCanUndo}
        assetCanRedo={assetCanRedo}
      />
    </div>
  );
}
