/**
 * AppShell — the rendered shell of the editor workspace.
 *
 * Per the App.tsx decomposition plan, AppInner kept the imperative
 * state and the handler bag, but the JSX tree grew past 1300 lines and
 * was the largest single block of `<div>`/`<DockLayout>` plumbing left
 * in the file. AppShell lifts that JSX into a real React component so
 * AppInner can shrink back to a composition root.
 *
 * The component takes a single `AppShellProps` bag — every dependency
 * is explicit, no context, no store. AppInner keeps owning all state
 * and just passes the props down.
 */
import type { EditorMode } from "./MenuBar";
import type { PaletteCommand } from "./CommandPalette";
import type { ShortcutGroup as CheatSheetGroup } from "./CheatSheet";
import type {
  DockableRegion,
  FloatingPanelState,
  PanelId,
} from "../hooks/useDockPrefs";
import type { LogState } from "../hooks/useLogState";
import type { NavigationTarget } from "../types/navigation";
import AppHeader from "./AppHeader";
import GameOverlay from "./GameOverlay";
import StatusBar from "./StatusBar";
import ViewportControls from "./ViewportControls";
import DockLayout from "./Dock/DockLayout";
import LeftDock from "./Dock/LeftDock";
import CenterDock from "./Dock/CenterDock";
import RightDock from "./Dock/RightDock";
import BottomDock from "./Dock/BottomDock";
import { FloatingPanel } from "./FloatingPanel/FloatingPanel";
import AssetNavigator from "./AssetNavigator";
import WorldWorkspace from "./WorldWorkspace";
import ExportRustModal from "./ExportRustModal";
import SaveSceneModal from "./SaveSceneModal";
import PromptDialog from "./PromptDialog";
import ConfirmDialog from "./ConfirmDialog";
import UnsavedChangesDialog from "./UnsavedChangesDialog";
import AssetUnsavedChangesDialog from "./AssetUnsavedChangesDialog";
import CommandPalette from "./CommandPalette";
import CheatSheet from "./CheatSheet";
import OnboardingBanner from "./OnboardingBanner";
import WelcomeOverlay from "./WelcomeOverlay";
import { WelcomeDismissalProvider } from "./WelcomeDismissalContext";
import Toasts from "./Toasts";
import type { SceneHandlers } from "../hooks/useSceneHandlers";
import type { SceneInfo } from "../hooks/useScenes";
import type { ReactNode } from "react";

/**
 * Imperative surface required by AppShell — every setter and getter
 * the JSX depends on. Keeping this flat (rather than passing the
 * whole `useSceneHandlers` return) makes the contract obvious.
 */
export interface AppShellProps {
  handlers: SceneHandlers;

  // Workspace / mode state
  editorMode: EditorMode;
  selectedEntityId: string | null;
  pendingNavigation: NavigationTarget | null;
  activeAssetLogicalPath: string | null;
  assetDirty: boolean;
  logState: LogState;
  assetLogState: LogState;
  activeLogicGraph: { logical_path: string } | null;
  scenes: SceneInfo[];
  currentId: string | null;

  // Workspace setters (the ones the JSX calls directly, not via handlers)
  setEditorMode: (mode: EditorMode) => void;
  setSaveModalOpen: (open: boolean) => void;
  setExportRustOpen: (open: boolean) => void;
  setSaveWorkspacePresetOpen: (open: boolean) => void;
  setAboutOpen: (open: boolean) => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setCheatSheetOpen: (open: boolean) => void;
  setFocusedFloatingPanel: (id: PanelId | null) => void;
  setLeftCollapsed: (collapsed: boolean | ((prev: boolean) => boolean)) => void;
  setPendingNavigation: (target: NavigationTarget | null) => void;

  // Dock + fullscreen API
  dock: {
    prefs: {
      left: { width: number; visible: boolean };
      right: {
        width: number;
        visible: boolean;
        topHeight: number;
        outlineVisible: boolean;
        propertiesVisible: boolean;
        outlineCollapsed: boolean;
        propertiesCollapsed: boolean;
      };
      bottom: { height: number; visible: boolean };
      statusBar: { height: number };
      floats: Partial<Record<PanelId, FloatingPanelState>>;
    };
    toggleLeft: () => void;
    toggleRight: () => void;
    toggleBottom: () => void;
    toggleOutline: () => void;
    toggleOutlineCollapsed: () => void;
    toggleProperties: () => void;
    togglePropertiesCollapsed: () => void;
    movePanel: (panelId: PanelId, target: DockableRegion) => void;
    setFloatRect: (panelId: PanelId, rect: FloatingPanelState) => void;
    setLeftWidth: (w: number) => void;
    setRightWidth: (w: number) => void;
    setBottomHeight: (h: number) => void;
    setStatusBarHeight: (h: number) => void;
    setRightTopHeight: (h: number) => void;
    reset: () => void;
    applyPreset: (presetId: string) => void;
  };
  fullscreen: { toggle: () => void };

  // Floating panels
  floatingPanelIds: Set<PanelId>;
  focusedFloatingPanel: PanelId | null;
  leftCollapsed: boolean;

  // Dialog open flags
  aiPanelOpen: boolean;
  validationCenterOpen: boolean;
  tilesetPanelOpen: boolean;
  autoLayerPanelOpen: boolean;
  exportRustOpen: boolean;
  saveModalOpen: boolean;
  saveWorkspacePresetOpen: boolean;
  aboutOpen: boolean;
  commandPaletteOpen: boolean;
  cheatSheetOpen: boolean;

  // Pending-scene-switch dialog state
  pendingSwitchId: string | null;
  pendingSwitchSource: string | null;
  pendingBackToScene: boolean;

  // Canvas state
  ready: boolean;
  initError: string | null;
  pan: { x: number; y: number };
  zoom: number;
  isDragOverCanvas: boolean;

  // Memoised panel content (passed through from AppInner so the same
  // JSX instance is shared between docked and floating renders).
  outlinePanelContent: ReactNode;
  propertiesPanelContent: ReactNode;
  bottomPanelContent: ReactNode;

  // Command palette + cheat sheet data
  paletteCommands: PaletteCommand[];
  cheatSheetGroups: CheatSheetGroup[];
}

export function AppShell(props: AppShellProps) {
  const {
    handlers,
    editorMode,
    selectedEntityId,
    pendingNavigation,
    activeAssetLogicalPath,
    assetDirty,
    logState,
    assetLogState,
    activeLogicGraph,
    scenes,
    currentId,
    setEditorMode,
    setSaveModalOpen,
    setExportRustOpen,
    setSaveWorkspacePresetOpen,
    setAboutOpen,
    setCommandPaletteOpen,
    setCheatSheetOpen,
    setFocusedFloatingPanel,
    setLeftCollapsed,
    setPendingNavigation,
    dock,
    fullscreen,
    floatingPanelIds,
    focusedFloatingPanel,
    leftCollapsed,
    aiPanelOpen,
    validationCenterOpen,
    tilesetPanelOpen,
    autoLayerPanelOpen,
    exportRustOpen,
    saveModalOpen,
    saveWorkspacePresetOpen,
    aboutOpen,
    commandPaletteOpen,
    cheatSheetOpen,
    pendingSwitchId,
    pendingSwitchSource,
    pendingBackToScene,
    ready,
    initError,
    pan,
    zoom,
    isDragOverCanvas,
    outlinePanelContent,
    propertiesPanelContent,
    bottomPanelContent,
    paletteCommands,
    cheatSheetGroups,
  } = props;

  return (
    <div className="app">
      <DockLayout
        onMovePanel={handlers.handleMovePanel}
        menu={
          <>
            <AppHeader
              editorMode={editorMode}
              onOpenAssets={() => {}}
              onBackToScene={
                editorMode === "asset-authoring"
                  ? handlers.handleBackToScene
                  : undefined
              }
              onOpenLogic={
                editorMode === "scene" ? handlers.handleOpenLogic : undefined
              }
              onOpenCode={
                editorMode === "scene" ? handlers.handleOpenCode : undefined
              }
              onOpenWorldWorkspace={handlers.handleOpenWorldWorkspace}
              logState={editorMode === "scene" ? logState : assetLogState}
              onUndo={
                editorMode === "scene"
                  ? handlers.handleUndo
                  : handlers.handleAssetUndo
              }
              onRedo={
                editorMode === "scene"
                  ? handlers.handleRedo
                  : handlers.handleAssetRedo
              }
              onSave={
                editorMode === "scene"
                  ? handlers.handleSave
                  : handlers.handleAssetSave
              }
              onSaveAs={() => setSaveModalOpen(true)}
              onLoad={handlers.handleLoad}
              onExportRust={() => setExportRustOpen(true)}
              onNewScene={() =>
                handlers.handleNewScene(`scene_${Date.now()}`)
              }
              onDeleteEntity={() => {
                if (selectedEntityId)
                  void handlers.handleDeleteEntity(selectedEntityId);
              }}
              selectedEntityId={selectedEntityId}
              onToggleAI={handlers.handleToggleAI}
              aiPanelOpen={aiPanelOpen}
              onToggleValidationCenter={handlers.handleToggleValidationCenter}
              validationCenterOpen={validationCenterOpen}
              onToggleTileset={handlers.handleToggleTileset}
              tilesetPanelOpen={tilesetPanelOpen}
              onToggleAutoLayer={handlers.handleToggleAutoLayer}
              autoLayerPanelOpen={autoLayerPanelOpen}
              onTogglePlay={handlers.handleTogglePlay}
              onOpenSearch={() => setCommandPaletteOpen(true)}
              onOpenCheatSheet={() => setCheatSheetOpen(true)}
              onWelcomeTour={() =>
                console.warn("[menu] TODO: wire Welcome Tour")
              }
              onAbout={handlers.handleAbout}
              onToggleLeftDock={dock.toggleLeft}
              onToggleOutlineDock={dock.toggleOutline}
              onTogglePropertiesDock={dock.toggleProperties}
              onToggleFullscreen={fullscreen.toggle}
              onResetLayout={dock.reset}
              onApplyPreset={dock.applyPreset}
              onSaveWorkspacePreset={() => {
                setSaveWorkspacePresetOpen(true);
              }}
              // ModeContextBar props
              currentSceneName={
                scenes.find((s) => s.id === currentId)?.name ?? null
              }
              activeAssetPath={activeAssetLogicalPath}
              assetDirty={assetDirty}
              sceneDirty={logState.size > 0}
              activeLogicGraphId={activeLogicGraph?.logical_path ?? null}
              activeCodeFileName={pendingNavigation?.fileId ?? null}
              isPlaying={editorMode === "play"}
              canUndo={logState.can_undo}
              canRedo={logState.can_redo}
              assetCanUndo={assetLogState.can_undo}
              assetCanRedo={assetLogState.can_redo}
            />
            {editorMode === "play" && (
              <GameOverlay onStop={handlers.handleTogglePlay} />
            )}
          </>
        }
        status={
          <StatusBar
            selectedEntityId={selectedEntityId}
            onExportRust={() => setExportRustOpen(true)}
          />
        }
        leftWidth={dock.prefs.left.width}
        rightWidth={dock.prefs.right.width}
        bottomHeight={dock.prefs.bottom.height}
        statusBarHeight={dock.prefs.statusBar.height}
        onResizeLeft={handlers.handleResizeLeft}
        onResizeRight={handlers.handleResizeRight}
        onResizeBottom={handlers.handleResizeBottom}
        onResizeStatusBar={handlers.handleResizeStatusBar}
        onResetLeft={() => dock.setLeftWidth(280)}
        onResetRight={() => dock.setRightWidth(320)}
        onResetBottom={() => dock.setBottomHeight(240)}
        onResetStatusBar={() => dock.setStatusBarHeight(24)}
        leftVisible={dock.prefs.left.visible}
        bottomVisible={dock.prefs.bottom.visible && editorMode === "scene"}
        left={
          floatingPanelIds.has("assets") ? null : (
            <LeftDock
              visible={dock.prefs.left.visible}
              collapsed={leftCollapsed}
              onToggleCollapse={() => setLeftCollapsed((v) => !v)}
              onClose={dock.toggleLeft}
              onMove={(target) => dock.movePanel("assets", target)}
              onFloatToggle={() => handlers.handleFloatPanel("assets")}
              floating={false}
            />
          )
        }
        center={
          <CenterDock
            scenes={scenes}
            currentId={currentId}
            onTabClick={handlers.handleTabClick}
            onNewScene={handlers.handleNewScene}
            onDeleteScene={handlers.handleDeleteScene}
            onRenameScene={handlers.handleRenameScene}
            canvas={
              editorMode === "world" ? (
                <WorldWorkspace
                  onOpenLevel={(levelId, _assetRef) => {
                    // Open level from world workspace switches to scene mode
                    setEditorMode("scene");
                  }}
                  onBackToScene={() => setEditorMode("scene")}
                />
              ) : (
                <div
                  className={`canvas-container${isDragOverCanvas ? " canvas-drop-active" : ""}`}
                  data-testid="canvas-drop-target"
                  onDragOver={handlers.handleCanvasDragOver}
                  onDragLeave={handlers.handleCanvasDragLeave}
                  onDrop={handlers.handleCanvasDrop}
                >
                  {!ready && (
                    <div style={{ padding: 16, color: "#888" }}>
                      {initError ? `Error: ${initError}` : "Loading WASM..."}
                    </div>
                  )}
                  <div
                    className="canvas-transform"
                    style={{
                      transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
                    }}
                  >
                    <canvas id="bevy-canvas" />
                  </div>
                  {isDragOverCanvas && (
                    <div
                      className="canvas-drop-outline"
                      data-testid="canvas-drop-outline"
                      aria-hidden="true"
                    />
                  )}
                  <ViewportControls />
                </div>
              )
            }
          />
        }
        right={
          <RightDock
            visible={dock.prefs.right.visible}
            outlineVisible={dock.prefs.right.outlineVisible}
            propertiesVisible={dock.prefs.right.propertiesVisible}
            outlineCollapsed={dock.prefs.right.outlineCollapsed}
            propertiesCollapsed={dock.prefs.right.propertiesCollapsed}
            topHeightPct={dock.prefs.right.topHeight}
            editorMode={editorMode}
            outlineFloating={floatingPanelIds.has("outline")}
            propertiesFloating={floatingPanelIds.has("properties")}
            onFloatToggleOutline={() => handlers.handleFloatPanel("outline")}
            onFloatToggleProperties={() =>
              handlers.handleFloatPanel("properties")
            }
            outline={outlinePanelContent}
            properties={propertiesPanelContent}
            onToggleCollapseOutline={dock.toggleOutlineCollapsed}
            onToggleCollapseProperties={dock.togglePropertiesCollapsed}
            onCloseOutline={dock.toggleOutline}
            onCloseProperties={dock.toggleProperties}
            onResizeSplit={handlers.handleResizeRightSplit}
            onResetSplit={() => dock.setRightTopHeight(60)}
            onOpen={dock.toggleRight}
            onMove={(target) => dock.movePanel("outline", target)}
          />
        }
        bottom={
          floatingPanelIds.has("bottom") ? null : (
            <BottomDock
              visible={dock.prefs.bottom.visible && editorMode === "scene"}
              onToggle={dock.toggleBottom}
              onClose={dock.toggleBottom}
              onMove={(target) => dock.movePanel("bottom", target)}
              onFloatToggle={() => handlers.handleFloatPanel("bottom")}
              floating={false}
              onSourceNavigate={setPendingNavigation}
            />
          )
        }
      />
      {/* v0.82 P2 (ADR-0025) floating panels — render portals for any
       * panel id whose entry lives in `dock.prefs.floats`. Each portal
       * hosts a lightweight body that points the user back at the dock
       * region it lifted from; filling the floating portal with the
       * full docked content (Inspector / Hierarchy / AssetNavigator
       * / BottomDock tabs) is the next iteration. */}
      {Array.from(floatingPanelIds).map((panelId) => {
        const rect = dock.prefs.floats[panelId];
        if (!rect) return null;
        // Mode-aware floating panel titles — mirrors RightDock.getOutlineTitle/getPropertiesTitle
        // so floating portals show the same labels as their docked counterparts.
        const outlineFloatingTitle =
          editorMode === "asset-authoring"
            ? "Project Assets"
            : editorMode === "scene"
              ? "Outline"
              : "Outline"; // logic/code/play: outline body is empty
        const propertiesFloatingTitle =
          editorMode === "asset-authoring"
            ? "Authoring"
            : editorMode === "scene"
              ? "Properties"
              : "Properties"; // logic/code/play: properties body is empty
        const floatingTitles: Record<PanelId, string> = {
          assets: "Assets",
          outline: outlineFloatingTitle,
          properties: propertiesFloatingTitle,
          bottom: "Tools",
          "change-workbench": "Workbench",
        };
        return (
          <FloatingPanel
            key={panelId}
            panelId={panelId}
            title={floatingTitles[panelId]}
            initialRect={rect}
            focused={focusedFloatingPanel === panelId}
            onFocus={() => setFocusedFloatingPanel(panelId)}
            onDock={() => handlers.handleDockFloatingPanel(panelId)}
            onPersistRect={(next) => dock.setFloatRect(panelId, next)}
          >
            {/* Phase B T2.1: render the actual dock body content, not a placeholder.
                The panel body matches what the docked version renders. */}
            <div data-testid={`floating-panel-${panelId}-body`}>
              {panelId === "assets" && <AssetNavigator />}
              {panelId === "outline" && outlinePanelContent}
              {panelId === "properties" && propertiesPanelContent}
              {panelId === "bottom" && bottomPanelContent}
            </div>
          </FloatingPanel>
        );
      })}
      {exportRustOpen && (
        <ExportRustModal onClose={() => setExportRustOpen(false)} />
      )}
      {saveModalOpen && (
        <SaveSceneModal
          defaultName={
            scenes.find((s) => s.id === currentId)?.name ?? "level_01"
          }
          onSave={handlers.handleSaveConfirm}
          onCancel={() => setSaveModalOpen(false)}
        />
      )}
      {saveWorkspacePresetOpen && (
        <PromptDialog
          title="Save Workspace Preset"
          label="Preset name"
          placeholder="e.g. level-design"
          defaultValue=""
          onConfirm={handlers.handleSaveWorkspacePresetSubmit}
          onCancel={() => setSaveWorkspacePresetOpen(false)}
        />
      )}
      {aboutOpen && (
        <ConfirmDialog
          title="About"
          message="Bevy 2D Editor v0.80.0"
          confirmLabel="OK"
          onConfirm={() => setAboutOpen(false)}
          onCancel={() => setAboutOpen(false)}
        />
      )}
      {pendingSwitchId !== null && pendingSwitchSource !== null && (
        <UnsavedChangesDialog
          sourceName={pendingSwitchSource}
          onSave={handlers.handleSaveAndSwitch}
          onDiscard={handlers.handleDiscardAndSwitch}
          onCancel={handlers.handleCancelSwitch}
        />
      )}
      {pendingBackToScene && activeAssetLogicalPath && (
        <AssetUnsavedChangesDialog
          logicalPath={activeAssetLogicalPath}
          unsavedCount={assetLogState.size}
          onSave={handlers.handleAssetSaveAndLeave}
          onDiscard={handlers.handleAssetDiscardAndLeave}
          onCancel={handlers.handleAssetCancelBack}
        />
      )}
      {commandPaletteOpen && (
        <CommandPalette
          commands={paletteCommands}
          onClose={() => setCommandPaletteOpen(false)}
        />
      )}
      {cheatSheetOpen && (
        <CheatSheet
          groups={cheatSheetGroups}
          onClose={() => setCheatSheetOpen(false)}
        />
      )}
      <WelcomeDismissalProvider>
        <OnboardingBanner
          onCreateBlankScene={() =>
            handlers.handleNewScene(`scene_${Date.now()}`)
          }
          onOpenLogicEditor={handlers.handleOpenLogic}
        />
        <WelcomeOverlay
          onTakeTour={() => setEditorMode("asset-authoring")}
          onSkip={() => undefined}
        />
      </WelcomeDismissalProvider>
      <Toasts />
    </div>
  );
}
