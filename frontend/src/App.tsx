import {
  Suspense,
  lazy,
  useEffect,
  useState,
  useCallback,
  useMemo,
} from "react";
import "./styles.css";
import { initEngine, isEngineReady } from "./engine-bridge";
import { useSceneState, SceneDocument } from "./hooks/useSceneState";
import { useLogState } from "./hooks/useLogState";
import { useAppShortcuts } from "./hooks/useAppShortcuts";
import { useAIAssistant } from "./hooks/useAIAssistant";
import MenuBar from "./components/MenuBar";
import AppHeader from "./components/AppHeader";
import HierarchyPanel from "./components/HierarchyPanel";
import InspectorPanel from "./components/InspectorPanel";
import AIAssistantPanel, { type TaskMode } from "./components/AIAssistantPanel";
import ExportRustModal from "./components/ExportRustModal";
import ValidationCenter from "./components/ValidationCenter";
import SaveSceneModal from "./components/SaveSceneModal";
import PromptDialog from "./components/PromptDialog";
import ConfirmDialog from "./components/ConfirmDialog";
import SceneTabs from "./components/SceneTabs";
import UnsavedChangesDialog from "./components/UnsavedChangesDialog";
import ProjectAssetBrowser from "./components/ProjectAssetBrowser";
import AssetAuthoringView from "./components/AssetAuthoringView";
import AssetUnsavedChangesDialog from "./components/AssetUnsavedChangesDialog";
import { TilesetPanel } from "./components/TilesetPanel";
import { AutoLayerPanel } from "./components/AutoLayerPanel";
import GameOverlay from "./components/GameOverlay";
import ConsoleTab from "./components/ConsoleTab";
import StatusBar from "./components/StatusBar";
import ViewportControls from "./components/ViewportControls";
import CommandPalette from "./components/CommandPalette";
import CheatSheet from "./components/CheatSheet";
import OnboardingBanner from "./components/OnboardingBanner";
import type { NavigationTarget } from "./types/navigation";

const LogicGraphEditor = lazy(() => import("./components/LogicGraphEditor"));
const CodeEditor = lazy(() => import("./components/CodeEditor"));
import { useCanvasViewport } from "./hooks/useCanvasViewport";
import { useDockResize } from "./hooks/useDockResize";
import { useEditorWorkspaceController } from "./hooks/useEditorWorkspaceController";
import { useSceneHandlers } from "./hooks/useSceneHandlers";
import { useAppModeController } from "./hooks/useAppModeController";
import { useAppCommandPalette } from "./hooks/useAppCommandPalette";
import { useSearchBridges } from "./hooks/useSearchBridges";
import { useFullscreenBody } from "./hooks/useFullscreenBody";
import { AppShell } from "./components/AppShell";
import type {
  DockableRegion,
  FloatingPanelState,
  PanelId,
} from "./hooks/useDockPrefs";
import DockLayout from "./components/Dock/DockLayout";
import LeftDock from "./components/Dock/LeftDock";
import CenterDock from "./components/Dock/CenterDock";
import { FloatingPanel } from "./components/FloatingPanel/FloatingPanel";
import RightDock from "./components/Dock/RightDock";
import BottomDock from "./components/Dock/BottomDock";
import AssetNavigator from "./components/AssetNavigator";
import { useScenes } from "./hooks/useScenes";
import { useSceneAssets } from "./hooks/useSceneAssets";
import { useLogicGraph } from "./hooks/useLogicGraph";
import {
  listLogicGraphAssets,
  openLogicGraphAsset,
  type LogicGraphCatalogEntry,
} from "./services/logic-graphs";
import { ToastProvider, useToasts } from "./hooks/useToasts";
import Toasts from "./components/Toasts";
import { useFullscreen } from "./hooks/useFullscreen";
import { WelcomeDismissalProvider } from "./components/WelcomeDismissalContext";
import WelcomeOverlay from "./components/WelcomeOverlay";
import {
  sceneCreate,
  sceneSwitch,
  sceneSwitchCommit,
  sceneDelete,
  sceneRename,
} from "./services/scenes";
import { type TilesetMetadata } from "./services/tilesets";
import {
  type AutoLayerPayload,
  type LevelLayerPayload,
  placeSceneInstance,
} from "./services/scene-assets";
import { findSourceLocation } from "./services/code-files";
import WorldWorkspace from "./components/WorldWorkspace";

type EditorMode =
  "scene" | "asset-authoring" | "logic" | "code" | "play" | "world";

export default function App() {
  return (
    <ToastProvider>
      <AppInner />
    </ToastProvider>
  );
}

function AppInner() {
  const [ready, setReady] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);

  const { addToast } = useToasts();
  const initGuard = (() => {
    let guard = false;
    return {
      current: guard,
      set: (v: boolean) => (guard = v),
      get: () => guard,
    };
  })();

  const { scene, refresh, dispatch } = useSceneState();

  // Workspace-level composition state (mode, multi-select, dirty
  // guards, test bridge) lives in the controller hook so App.tsx can
  // stay a pure composition root. The scene-order option is filled in
  // after `scene` is hydrated, so the range-select helper is wired
  // through a layout effect to avoid a TDZ.
  const workspace = useEditorWorkspaceController({
    sceneOrderForRangeSelect: scene?.entities.map((e) => e.id),
  });
  const {
    editorMode,
    setEditorMode,
    selectedIds,
    lastClickedId,
    selectedEntityId,
    selectEntity,
    setSelectedEntityId,
    setSelectedIds,
    clearSelection,
    pendingNavigation,
    setPendingNavigation,
    pendingBackToScene,
    setPendingBackToScene,
    bindTestHooks,
  } = workspace;

  // v0.82 P2 (ADR-0025 §F8 + §F7): Esc clears selection; Ctrl/Cmd+A
  // selects every entity. Both pass through when a text input or
  // context menu is focused so we don't fight the user's editing.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement | null;
      if (t) {
        const tag = (t.tagName ?? "").toLowerCase();
        if (
          tag === "input" ||
          tag === "textarea" ||
          t.isContentEditable ||
          tag === "select"
        ) {
          return;
        }
      }
      if (e.key === "Escape") {
        if (selectedIds.size > 0 || lastClickedId) {
          clearSelection();
          e.preventDefault();
        }
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") {
        if (!scene?.entities?.length) return;
        // Wave D2: workspace exposes `selectAll` through `setSelectedIds`
        // path — go through the controller's domain by setting
        // `lastClickedId` is not enough; the controller's `setSelectedIds`
        // is internal. For now, replicate the same call as before by
        // toggling each id off/on (preserves the contract).
        setSelectedIds(new Set(scene.entities.map((e) => e.id)));
        e.preventDefault();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [clearSelection, selectedIds.size, lastClickedId, scene, setSelectedIds]);

  const logState = useLogState();
  const { zoom, pan, reset: resetViewport, fitToContent } = useCanvasViewport();
  const [isDragOverCanvas, setIsDragOverCanvas] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [cheatSheetOpen, setCheatSheetOpen] = useState(false);
  const [renameRequestTick, setRenameRequestTick] = useState(0);
  const [aiPanelOpen, setAiPanelOpen] = useState(false);
  // v2 AI task mode + context source toggles (PR4 correction)
  const [taskMode, setTaskMode] = useState<TaskMode>("ask");
  const [enabledSources, setEnabledSources] = useState<Set<string>>(new Set());
  const [validationCenterOpen, setValidationCenterOpen] = useState(false);
  const [tilesetPanelOpen, setTilesetPanelOpen] = useState(false);
  const [selectedTilesetId, setSelectedTilesetId] = useState<string | null>(
    null,
  );
  const [leftCollapsed, setLeftCollapsed] = useState(false);
  const [autoLayerPanelOpen, setAutoLayerPanelOpen] = useState(false);
  const [selectedAutoLayerId, setSelectedAutoLayerId] = useState<string | null>(
    null,
  );
  const [exportRustOpen, setExportRustOpen] = useState(false);
  const [saveModalOpen, setSaveModalOpen] = useState(false);
  const [saveWorkspacePresetOpen, setSaveWorkspacePresetOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [applyingIds, setApplyingIds] = useState<Set<string>>(new Set());
  const { scenes, currentId, refresh: refreshScenes } = useScenes();
  const [pendingSwitchId, setPendingSwitchId] = useState<string | null>(null);
  const [pendingSwitchSource, setPendingSwitchSource] = useState<string | null>(
    null,
  );

  // ── Asset Authoring Mode ─────────────────────────────────────────────────
  // (editorMode + setEditorMode now live in useEditorWorkspaceController.)

  // Phase C T3.4 test hook: expose workspace hooks for Playwright tests
  // that need to switch mode/selection/open AI panel. The controller
  // owns the editorMode + selection setters; the AI panel setter still
  // lives here because it controls a top-level dialog whose state
  // (setAiPanelOpen) is owned by this component.
  bindTestHooks();
  useEffect(() => {
    if (typeof window === "undefined") return;
    (window as any).__openAIPanel = () => setAiPanelOpen(true);
  }, []);
  const [activeAssetLogicalPath, setActiveAssetLogicalPath] = useState<
    string | null
  >(null);

  // ── Cross-mode Navigation (rust-source-integration) ─────────────────────
  // (pendingNavigation/setPendingNavigation live in the controller.)

  const {
    entries: assetEntries,
    assetDoc,
    logState: assetLogState,
    dirty: assetDirty,
    open: openAsset,
    close: closeAsset,
    dispatch: dispatchAssetCommand,
    undo: undoAsset,
    redo: redoAsset,
    save: saveAsset,
    create: createAsset,
    rename: renameAsset,
    duplicate: duplicateAsset,
    deleteAsset: deleteAssetFn,
    placeInstance,
    instances,
    removeInstance,
    replaceInstanceAsset,
  } = useSceneAssets();

  // ── Auto Layer State ────────────────────────────────────────────────────
  // Derive the first auto layer from assetDoc as a fallback for AutoLayerPanel
  const autoLayers: AutoLayerPayload[] =
    (assetDoc?.layers?.filter(
      (l: LevelLayerPayload) => l.kind === "auto",
    ) as AutoLayerPayload[]) ?? [];

  const selectedAutoLayer: AutoLayerPayload | null = selectedAutoLayerId
    ? (autoLayers.find((l) => l.id === selectedAutoLayerId) ?? null)
    : (autoLayers[0] ?? null);

  // ── AI Assistant ─────────────────────────────────────────────────────────
  // Hito 4 Order 6: logic graph state for multi-source context
  const { graph: activeLogicGraph } = useLogicGraph();

  // Hito 4 Order 6: logic graph catalog entries for ProjectAssetBrowser
  const [logicGraphEntries, setLogicGraphEntries] = useState<
    LogicGraphCatalogEntry[]
  >([]);
  const refreshLogicGraphEntries = useCallback(async () => {
    try {
      const entries = await listLogicGraphAssets();
      setLogicGraphEntries(entries);
    } catch (e) {
      console.warn("[App] refreshLogicGraphEntries failed:", e);
    }
  }, []);

  // Poll for logic graph entries every 2s (slower cadence than scene assets)
  useEffect(() => {
    refreshLogicGraphEntries();
    const interval = setInterval(refreshLogicGraphEntries, 2000);
    return () => clearInterval(interval);
  }, [refreshLogicGraphEntries]);

  // Hito 4 Order 6: derive scene asset context from useSceneAssets
  const sceneAssetContext = useMemo(
    () => ({
      catalog: assetEntries.map((e) => ({
        id: e.asset_id,
        name: e.logical_path,
        role: e.role,
      })),
      selected_body: assetDoc ? JSON.stringify(assetDoc) : null,
    }),
    [assetEntries, assetDoc],
  );

  // Hito 4 Order 6: derive selected entity from scene hierarchy + selectedEntityId
  const selectedEntity = useMemo(() => {
    if (!selectedEntityId || !scene?.entities) return null;
    const entity = scene.entities.find((e) => e.id === selectedEntityId);
    if (!entity) return null;
    return {
      stable_id: entity.id,
      components: entity.components.map((c) => ({
        type_id: c.type_id,
        values: c.values,
      })),
    };
  }, [selectedEntityId, scene]);

  const {
    prompt,
    setPrompt,
    loading: aiLoading,
    proposals,
    error: aiError,
    contextStats,
    contextUsedChars,
    submit,
    applyProposal,
    discardProposal,
  } = useAIAssistant({
    onApplied: refresh,
    logicGraph: activeLogicGraph,
    sceneAssetContext,
    selectedEntity,
  });

  useEffect(() => {
    if (initGuard.get()) return;
    initGuard.set(true);

    initEngine("bevy-canvas", () => {
      // FPS and sprite position events handled silently (legacy)
    })
      .then(() => setReady(isEngineReady()))
      .catch((e) => {
        const msg = String(e);
        setInitError(msg);
        addToast(`Engine init failed: ${msg}`, "error");
      });
  }, [addToast]);

  // ── Handler extraction (useSceneHandlers) ─────────────────────────────────
  // All ~30+ useCallback handlers that previously lived inline in AppInner
  // were lifted into `useSceneHandlers`. The hook takes a context object
  // (scene + workspace + assets + dock + viewport + ai + dialog flags +
  // addToast) plus the pending-scene-switch tuple, and returns a memoised
  // bag of handlers AppInner + AppShell can destructure.

  // ── Dock layout (Phase B) ──────────────────────────────────────────────────
  const dock = useDockResize();

  // v0.82 P2 (ADR-0025): floating panels — the set of panel ids currently
  // lifted out of the CSS-Grid layout into a `createPortal(…)` overlay.
  // Derived from `dock.prefs.floats` keys so persistence and runtime
  // state are aligned. A `focusedFloatingPanel` (single id) drives the
  // `--z-floating-panel-focused` z-index bump on the most recently
  // clicked float header.
  const floatingPanelIds = useMemo<Set<PanelId>>(
    () => new Set(Object.keys(dock.prefs.floats) as PanelId[]),
    [dock.prefs.floats],
  );
  const [focusedFloatingPanel, setFocusedFloatingPanel] =
    useState<PanelId | null>(null);

  // ── Fullscreen viewport (Phase E) ─────────────────────────────────────────
  const fullscreen = useFullscreen();

  const handlers = useSceneHandlers(
    {
      scene: { refresh, dispatch },
      workspace: {
        editorMode,
        setEditorMode,
        selectedIds,
        selectedEntityId,
        setSelectedEntityId,
        selectEntity,
        clearSelection,
        setSelectedIds,
        setPendingNavigation,
        setPendingBackToScene,
      },
      assets: {
        open: openAsset,
        close: closeAsset,
        dispatch: dispatchAssetCommand,
        undo: undoAsset,
        redo: redoAsset,
        save: saveAsset,
        create: createAsset,
        rename: renameAsset,
        duplicate: duplicateAsset,
        deleteAsset: deleteAssetFn,
        placeInstance,
        dirty: assetDirty,
        logState: assetLogState,
      },
      scenes: {
        scenes: scenes as Array<{ id: string; name: string }>,
        currentId,
        refresh: refreshScenes,
      },
      dialogs: {
        setExportRustOpen,
        setSaveModalOpen,
        setSaveWorkspacePresetOpen,
        setAboutOpen,
        setAiPanelOpen,
        setEnabledSources,
        setValidationCenterOpen,
        setTilesetPanelOpen,
        setSelectedTilesetId,
        setAutoLayerPanelOpen,
        setIsDragOverCanvas,
        setActiveAssetLogicalPath,
        setRenameRequestTick,
        setCommandPaletteOpen,
        setCheatSheetOpen,
        setPendingSwitchId,
        setPendingSwitchSource,
        setLeftCollapsed,
        setFocusedFloatingPanel,
        setApplyingIds,
      },
      dock,
      fullscreen,
      viewport: { zoom, pan, fitToContent, reset: resetViewport },
      assetEntries,
      addToast,
      ai: { submit, applyProposal },
      focusedFloatingPanel,
      entities: scene?.entities ?? [],
    },
    { id: pendingSwitchId, source: pendingSwitchSource },
  );

  // Handlers are now provided by `handlers` (useSceneHandlers) below.

  // Apply the data-fullscreen attribute to body — useFullscreen already
  // mirrors this, but make sure any mount-time flip is reflected in the
  // hook state for tests/components querying it.
  useFullscreenBody(fullscreen.enabled);

  useAppShortcuts({
    editorMode,
    handlers,
    selectedIds,
    selectedEntityId,
    logState,
    assetLogState,
    setCommandPaletteOpen,
    setCheatSheetOpen,
    setRenameRequestTick,
    fitToContent,
    dock,
    fullscreen,
  });
  // ── Command palette catalog & cheat sheet (commit 4) ───────────────────────────
  // Both lists, plus the cross-window executor used by Global Search
  // (`window.__executeCommand`), are built here. The hook preserves the
  // exact action wiring the original AppInner had — including the
  // editor-mode-dependent dispatch in Undo/Redo and the entity-id
  // gated Delete action.
  const {
    paletteCommands,
    serializablePaletteItems,
    executeCommandById,
    cheatSheetGroups,
  } = useAppCommandPalette({
    editorMode,
    selectedEntityId,
    handlers,
    setExportRustOpen,
    setCheatSheetOpen,
    setRenameRequestTick,
    setEditorMode,
    fitToContent,
    resetViewport,
  });

  useSearchBridges({
    serializablePaletteItems,
    executeCommandById,
    openAsset: async (assetId: string) => {
      await openAsset(assetId);
    },
    openLogicGraphAsset,
    setEditorMode,
    setValidationCenterOpen,
  });

  // Bridges installed via `useSearchBridges` above; no App-level
  // imperative wiring remains in this file.

  // ── Mode-routed panel content (commit 3) ───────────────────────────────────────
  // The three useMemo blocks that materialised the outline / properties /
  // bottom panel bodies were lifted into `useAppModeController`. The
  // hook reads the same state and returns ReactNodes that AppShell
  // shares between docked and floating portals.
  const { outlineContent, propertiesContent, bottomContent } =
    useAppModeController({
      editorMode,
      selectedEntityId,
      selectedIds,
      selectEntity,
      setSelectedEntityId,
      setPendingNavigation,
      pendingNavigation,
      scene,
      instances,
      prompt,
      setPrompt,
      aiLoading,
      proposals,
      aiError,
      contextStats,
      contextUsedChars,
      discardProposal,
      taskMode,
      setTaskMode,
      enabledSources,
      applyingIds,
      aiPanelOpen,
      validationCenterOpen,
      tilesetPanelOpen,
      autoLayerPanelOpen,
      selectedTilesetId,
      selectedAutoLayerId,
      assetDoc,
      assetEntries,
      assetDirty,
      assetLogState,
      activeAssetLogicalPath,
      removeInstance,
      replaceInstanceAsset,
      placeInstance,
      refresh,
      renameRequestTick,
      logicGraphEntries,
      handlers,
    });

  return (
    <AppShell
      handlers={handlers}
      editorMode={editorMode}
      selectedEntityId={selectedEntityId}
      pendingNavigation={pendingNavigation}
      activeAssetLogicalPath={activeAssetLogicalPath}
      assetDirty={assetDirty}
      logState={logState}
      assetLogState={assetLogState}
      activeLogicGraph={activeLogicGraph}
      scenes={scenes}
      currentId={currentId}
      setEditorMode={setEditorMode}
      setSaveModalOpen={setSaveModalOpen}
      setExportRustOpen={setExportRustOpen}
      setSaveWorkspacePresetOpen={setSaveWorkspacePresetOpen}
      setAboutOpen={setAboutOpen}
      setCommandPaletteOpen={setCommandPaletteOpen}
      setCheatSheetOpen={setCheatSheetOpen}
      setFocusedFloatingPanel={setFocusedFloatingPanel}
      setLeftCollapsed={setLeftCollapsed}
      setPendingNavigation={setPendingNavigation}
      dock={dock}
      fullscreen={fullscreen}
      floatingPanelIds={floatingPanelIds}
      focusedFloatingPanel={focusedFloatingPanel}
      leftCollapsed={leftCollapsed}
      aiPanelOpen={aiPanelOpen}
      validationCenterOpen={validationCenterOpen}
      tilesetPanelOpen={tilesetPanelOpen}
      autoLayerPanelOpen={autoLayerPanelOpen}
      exportRustOpen={exportRustOpen}
      saveModalOpen={saveModalOpen}
      saveWorkspacePresetOpen={saveWorkspacePresetOpen}
      aboutOpen={aboutOpen}
      commandPaletteOpen={commandPaletteOpen}
      cheatSheetOpen={cheatSheetOpen}
      pendingSwitchId={pendingSwitchId}
      pendingSwitchSource={pendingSwitchSource}
      pendingBackToScene={pendingBackToScene}
      ready={ready}
      initError={initError}
      pan={pan}
      zoom={zoom}
      isDragOverCanvas={isDragOverCanvas}
      outlinePanelContent={outlineContent}
      propertiesPanelContent={propertiesContent}
      bottomPanelContent={bottomContent}
      paletteCommands={paletteCommands}
      cheatSheetGroups={cheatSheetGroups}
    />
  );
}
