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
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
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
import CommandPalette, {
  type PaletteCommand,
} from "./components/CommandPalette";
import CheatSheet, {
  type ShortcutGroup as CheatSheetGroup,
} from "./components/CheatSheet";
import OnboardingBanner from "./components/OnboardingBanner";
import type { NavigationTarget } from "./types/navigation";

const LogicGraphEditor = lazy(() => import("./components/LogicGraphEditor"));
const CodeEditor = lazy(() => import("./components/CodeEditor"));
import { useCanvasViewport } from "./hooks/useCanvasViewport";
import { useDockResize } from "./hooks/useDockResize";
import { useEditorWorkspaceController } from "./hooks/useEditorWorkspaceController";
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

type EditorMode = "scene" | "asset-authoring" | "logic" | "code" | "play";

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

  const handleToggleAI = useCallback(() => {
    setAiPanelOpen((prev) => !prev);
  }, []);

  // v2 context source toggle — updates the enabledSources Set.
  const handleContextToggle = useCallback(
    (sourceName: string, enabled: boolean) => {
      setEnabledSources((prev) => {
        const next = new Set(prev);
        if (enabled) {
          next.add(sourceName);
        } else {
          next.delete(sourceName);
        }
        return next;
      });
    },
    [],
  );

  const handleToggleValidationCenter = useCallback(() => {
    setValidationCenterOpen((prev) => !prev);
  }, []);

  // Phase B T2.5: Validation Center issue navigation.
  // Navigate to the surface owning the issue: entity focus, asset open, or scene switch.
  const handleValidationCenterNavigate = useCallback(
    async (issue: import("./services/validation-center").ValidationIssue) => {
      if (issue.affected_entity_id) {
        setSelectedEntityId(issue.affected_entity_id);
      } else if (issue.affected_asset_id) {
        setEditorMode("asset-authoring");
        try {
          // CRITICAL ISSUE 4: use __openSceneAssetFromSearch so asset navigation
          // goes through the App-owned scene asset state (updates assetDoc, activeAssetId).
          await (window as any).__openSceneAssetFromSearch?.(
            issue.affected_asset_id,
          );
        } catch (e) {
          console.warn("[App] open asset failed:", e);
        }
      } else if (issue.affected_scene_id) {
        setEditorMode("scene");
        try {
          await sceneSwitch(issue.affected_scene_id);
        } catch (e) {
          console.warn("[App] scene switch failed:", e);
        }
      } else {
        // No specific target — switch to code mode as fallback.
        setEditorMode("code");
      }
    },
    [sceneSwitch],
  );

  const handleToggleTileset = useCallback(() => {
    setTilesetPanelOpen((prev) => !prev);
  }, []);

  const handleSelectTileset = useCallback((tileset: TilesetMetadata) => {
    setSelectedTilesetId(tileset.id);
  }, []);

  const handleToggleAutoLayer = useCallback(() => {
    setAutoLayerPanelOpen((prev) => !prev);
  }, []);

  const handleSubmitAI = useCallback(async () => {
    await submit(dispatch);
  }, [submit, dispatch]);

  const handleApplyProposal = useCallback(
    async (proposalId: string) => {
      setApplyingIds((prev) => new Set([...prev, proposalId]));
      try {
        await applyProposal(proposalId, dispatch);
      } finally {
        setApplyingIds((prev) => {
          const next = new Set(prev);
          next.delete(proposalId);
          return next;
        });
      }
    },
    [applyProposal, dispatch],
  );

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

  const handleUndo = async () => {
    try {
      const snap = await (window as any).undo();
      const parsed = JSON.parse(snap);
      await refresh();
      setSelectedEntityId(null);
    } catch (e) {
      addToast(`Undo failed: ${e}`, "error");
    }
  };

  const handleRedo = async () => {
    try {
      const snap = await (window as any).redo();
      await refresh();
      setSelectedEntityId(null);
    } catch (e) {
      addToast(`Redo failed: ${e}`, "error");
    }
  };

  const handleSave = async () => {
    // Phase 1.5: replace window.prompt with SaveSceneModal
    setSaveModalOpen(true);
  };

  const handleSaveConfirm = async (name: string) => {
    setSaveModalOpen(false);
    try {
      const path = await (window as any).save_scene(name);
      console.log(`Saved to ${path}`);
    } catch (e) {
      addToast(`Save failed: ${e}`, "error");
    }
  };

  const handleSaveWorkspacePresetSubmit = (name: string) => {
    setSaveWorkspacePresetOpen(false);
    dock.saveCurrentAsPreset(name);
  };

  const handleAbout = () => {
    setAboutOpen(true);
  };

  const handleLoad = async () => {
    try {
      await (window as any).load_project();
      await refresh();
      setSelectedEntityId(null);
    } catch (e) {
      addToast(`Load project failed: ${e}`, "error");
    }
  };

  const handleRename = async (entityId: string, newName: string) => {
    const result = await dispatch({
      command: { type: "RenameEntity", entity_id: entityId, new_name: newName },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    if (result.error) addToast(`Rename failed: ${result.error}`, "error");
  };

  const handleSetField = async (
    entityId: string,
    typeId: string,
    fieldPath: string,
    value: any,
  ) => {
    const result = await dispatch({
      command: {
        type: "SetComponentField",
        entity_id: entityId,
        type_id: typeId,
        field_path: fieldPath,
        value,
      },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    if (result.error) addToast(`Set field failed: ${result.error}`, "error");
  };

  const handleRemoveComponent = async (entityId: string, typeId: string) => {
    const result = await dispatch({
      command: {
        type: "RemoveComponent",
        entity_id: entityId,
        type_id: typeId,
      },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    if (result.error)
      addToast(`Remove component failed: ${result.error}`, "error");
  };

  // v0.82 P2 (ADR-0025): apply the same field to the same component
  // on every supplied entity in one atomic command. Rust fans the
  // command out into a `Batch` of per-entity SetComponentFields, so a
  // partial failure rolls back. The frontend filters out non-owners
  // before dispatch so we never hit a ComponentNotFound at apply.
  const handleSetFieldOnMultiple = async (
    entityIds: string[],
    typeId: string,
    fieldPath: string,
    value: unknown,
  ) => {
    if (entityIds.length === 0) return;
    // Defence in depth: the Rust side also rejects empty arrays with
    // CommandError::InvalidArgument, but a no-op call here keeps the
    // OperationLog clean.
    const owningIds = (scene?.entities ?? [])
      .filter((e) => entityIds.includes(e.id))
      .filter((e) => e.components.some((c) => c.type_id === typeId))
      .map((e) => e.id);
    if (owningIds.length === 0) {
      addToast("No selected entities own that component.", "error");
      return;
    }
    const result = await dispatch({
      command: {
        type: "SetComponentFieldOnMultiple",
        entity_ids: owningIds,
        type_id: typeId,
        field_path: fieldPath,
        value,
      },
      metadata: {
        authorship: "user",
        timestamp: Date.now(),
        rationale: `Multi-edit ${typeId}.${fieldPath} on ${owningIds.length} entities`,
      },
    });
    if (result.error)
      addToast(`Set field on multiple failed: ${result.error}`, "error");
  };

  const handleAddComponent = async (entityId: string, typeId: string) => {
    const result = await dispatch({
      command: {
        type: "AddComponent",
        entity_id: entityId,
        type_id: typeId,
        values: {},
      },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    if (result.error)
      addToast(`Add component failed: ${result.error}`, "error");
  };

  const handleDeleteEntity = useCallback(
    async (id: string) => {
      if (!id) return;
      await dispatch({
        command: { type: "DeleteEntity", id },
        metadata: { authorship: "keyboard", timestamp: Date.now() },
      });
      setSelectedEntityId(null);
    },
    [dispatch],
  );

  // v0.82 P2 (ADR-0025 §F11): multi-entity delete wraps N
  // per-entity DeleteEntity commands into a single Batch so the
  // OperationLog captures one entry with a descriptive label.
  const handleDeleteEntities = useCallback(
    async (ids: Iterable<string>) => {
      const arr = Array.from(ids);
      if (arr.length === 0) return;
      await dispatch({
        command: {
          type: "Batch",
          label: `Delete ${arr.length} entities`,
          commands: arr.map((id) => ({
            type: "DeleteEntity",
            id,
          })),
        },
        metadata: { authorship: "keyboard", timestamp: Date.now() },
      });
      clearSelection();
    },
    [dispatch, clearSelection],
  );

  // ── Logic Workflow v2 handlers (PR4 correction) ───────────────────────────
  // Opens the attach-logic dialog or navigates to logic mode for the entity.
  const handleAttachLogic = useCallback(async (instanceId: string) => {
    console.log("[App] handleAttachLogic called for instance:", instanceId);
    // TODO: wire to the attach-logic dialog / logic authoring workflow
    setEditorMode("logic");
  }, []);

  // Delegates to the existing handleOpenLogic with the bound asset id.
  const handleOpenBoundLogic = useCallback(async (entityId: string) => {
    console.log("[App] handleOpenBoundLogic called for entity:", entityId);
    // TODO: wire to bound asset loader
    setEditorMode("logic");
    // openLogicGraphAsset(assetId) — deferred; needs bound asset id from instance
  }, []);

  // Navigates to the RecipePicker in logic mode (no parameters — uses selected entity).
  const handleCreateFromRecipe = useCallback(() => {
    console.log("[App] handleCreateFromRecipe called");
    // TODO: wire to recipe picker / create-from-recipe workflow
    setEditorMode("logic");
  }, []);

  // Switches to logic mode to inspect the selected entity's runtime logic state.
  const handleInspectRuntimeLogic = useCallback(() => {
    console.log("[App] handleInspectRuntimeLogic called");
    // TODO: wire to runtime logic inspector
    setEditorMode("logic");
  }, []);

  // Sets editorMode to "logic".
  const handleSwitchToLogicMode = useCallback(() => {
    setEditorMode("logic");
  }, []);

  // ── Create entity (Phase 1.4 — UX overhaul) ──────────────────────────────
  // Counter + suffix derivation lives here so button + N shortcut stay in sync.
  // New entity gets name "Entity N" where N is one greater than the highest
  // existing "Entity <n>" suffix to avoid collisions.
  const handleCreateEntity = useCallback(async () => {
    const existing = scene?.entities ?? [];
    let maxSuffix = 0;
    const re = /^Entity (\d+)$/;
    for (const e of existing) {
      const m = re.exec(e.name);
      if (m) {
        const n = parseInt(m[1], 10);
        if (!Number.isNaN(n) && n > maxSuffix) maxSuffix = n;
      }
    }
    const newName = `Entity ${maxSuffix + 1}`;
    // Stable unique id — timestamp + random suffix to avoid collisions across
    // rapid double-clicks before the scene snapshot updates.
    const newId = `ent_${Date.now()}_${Math.floor(Math.random() * 1e6).toString(36)}`;
    await dispatch({
      command: {
        type: "CreateEntity",
        id: newId,
        name: newName,
        components: [],
      },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    setSelectedEntityId(newId);
  }, [scene, dispatch]);

  // ── Multi-scene handlers ─────────────────────────────────────────────────

  const handleTabClick = useCallback(
    async (id: string) => {
      if (id === currentId) return;
      const result = await sceneSwitch(id);
      if (result.dirty_prompt_required) {
        setPendingSwitchId(id);
        setPendingSwitchSource(result.source_name);
      }
      // If no dirty prompt, the switch already happened server-side
      await refresh();
    },
    [currentId, refresh],
  );

  const handleNewScene = useCallback(
    async (name: string) => {
      await sceneCreate(name);
      await refreshScenes();
    },
    [refreshScenes],
  );

  const handleDeleteScene = useCallback(
    async (id: string) => {
      await sceneDelete(id);
      await refreshScenes();
    },
    [refreshScenes],
  );

  const handleRenameScene = useCallback(
    async (id: string, newName: string) => {
      await sceneRename(id, newName);
      await refreshScenes();
    },
    [refreshScenes],
  );

  const handleSaveAndSwitch = useCallback(async () => {
    if (!pendingSwitchId) return;
    // Save current scene (user initiated from dialog)
    const currentScene = scenes.find((s) => s.id === currentId);
    if (currentScene) {
      await (window as any).save_scene(currentScene.name);
    }
    await sceneSwitchCommit(pendingSwitchId);
    setPendingSwitchId(null);
    setPendingSwitchSource(null);
    await refresh();
    await refreshScenes();
  }, [pendingSwitchId, currentId, scenes, refresh, refreshScenes]);

  const handleDiscardAndSwitch = useCallback(async () => {
    if (!pendingSwitchId) return;
    await sceneSwitchCommit(pendingSwitchId);
    setPendingSwitchId(null);
    setPendingSwitchSource(null);
    await refresh();
    await refreshScenes();
  }, [pendingSwitchId, refresh, refreshScenes]);

  const handleCancelSwitch = useCallback(() => {
    setPendingSwitchId(null);
    setPendingSwitchSource(null);
  }, []);

  // ── Asset Authoring handlers ─────────────────────────────────────────────

  const handleOpenAsset = useCallback(
    async (assetId: string) => {
      const entry = assetEntries.find((e) => e.asset_id === assetId);
      if (!entry) return;
      await openAsset(assetId);
      setActiveAssetLogicalPath(entry.logical_path);
      setEditorMode("asset-authoring");
    },
    [assetEntries, openAsset],
  );

  const handleAssetCreate = useCallback(
    async (name: string, role: string) => {
      await createAsset(name, role);
    },
    [createAsset],
  );

  const handleAssetRename = useCallback(
    async (assetId: string, newPath: string) => {
      await renameAsset(assetId, newPath);
    },
    [renameAsset],
  );

  const handleAssetDuplicate = useCallback(
    async (assetId: string) => {
      await duplicateAsset(assetId);
    },
    [duplicateAsset],
  );

  const handleAssetDelete = useCallback(
    async (assetId: string) => {
      await deleteAssetFn(assetId);
    },
    [deleteAssetFn],
  );

  // "Back to Scene" — check dirty BEFORE flipping mode (per D4)
  const handleBackToScene = useCallback(() => {
    if (assetDirty) {
      setPendingBackToScene(true);
    } else {
      // Not dirty — safe to leave immediately
      closeAsset();
      setActiveAssetLogicalPath(null);
      setEditorMode("scene");
    }
  }, [assetDirty, closeAsset]);

  const handleAssetSaveAndLeave = useCallback(async () => {
    await saveAsset();
    setPendingBackToScene(false);
    closeAsset();
    setActiveAssetLogicalPath(null);
    setEditorMode("scene");
  }, [saveAsset, closeAsset]);

  const handleAssetDiscardAndLeave = useCallback(() => {
    // Close without saving — no file write
    closeAsset();
    setPendingBackToScene(false);
    setActiveAssetLogicalPath(null);
    setEditorMode("scene");
  }, [closeAsset]);

  const handleAssetCancelBack = useCallback(() => {
    setPendingBackToScene(false);
  }, []);

  // ── Logic Graph handlers ─────────────────────────────────────────────────
  const handleOpenLogic = useCallback(() => {
    setEditorMode("logic");
  }, []);

  // ── Code editor handlers ────────────────────────────────────────────────
  const handleOpenCode = useCallback(() => {
    setEditorMode("code");
  }, []);

  // Cross-mode jump-to-source handler (scene inspector → code editor).
  // Resolves the type_id → source location and navigates to the file + line.
  const handleJumpToSource = useCallback(async (typeId: string) => {
    const loc = await findSourceLocation(typeId);
    if (loc) {
      setPendingNavigation({ fileId: loc.file_id, line: loc.line });
      setEditorMode("code");
    }
  }, []);

  // Asset command dispatch with C-2 adapter: fieldPath string → [fieldPath]
  const handleAssetCommit = useCallback(
    async (localId: string, typeId: string, fieldPath: string, value: any) => {
      // Wrap fieldPath as [fieldPath] for SetComponentValue.field_path: Vec<String>
      const command = {
        type: "SetComponentValue",
        local_id: localId,
        type_id: typeId,
        field_path: [fieldPath], // C-2: 1-element array wrap
        value,
      };
      await dispatchAssetCommand(command);
    },
    [dispatchAssetCommand],
  );

  const handleAssetAddComponent = useCallback(
    async (localId: string, typeId: string) => {
      const command = {
        type: "AddComponent",
        local_id: localId,
        type_id: typeId,
        values: {},
      };
      await dispatchAssetCommand(command);
    },
    [dispatchAssetCommand],
  );

  const handleAssetRemoveComponent = useCallback(
    async (localId: string, typeId: string) => {
      const command = {
        type: "RemoveComponent",
        local_id: localId,
        type_id: typeId,
      };
      await dispatchAssetCommand(command);
    },
    [dispatchAssetCommand],
  );

  const handleAssetUndo = useCallback(async () => {
    await undoAsset();
  }, [undoAsset]);

  const handleAssetRedo = useCallback(async () => {
    await redoAsset();
  }, [redoAsset]);

  const handleAssetSave = useCallback(async () => {
    await saveAsset();
  }, [saveAsset]);

  const handleTogglePlay = useCallback(() => {
    if (editorMode === "play") {
      (window as any).exit_play_mode();
      setEditorMode("scene");
    } else {
      (window as any).enter_play_mode();
      setEditorMode("play");
    }
  }, [editorMode]);

  // ── Drag-drop from ProjectAssetBrowser to canvas (Phase 3.1) ─────────────
  // Listens for the custom `application/x-bevy-asset-id` MIME produced by
  // ProjectAssetBrowser row dragstart and calls placeSceneInstance with a
  // world-space translation computed from the drop cursor + current
  // viewport zoom/pan.
  const handleCanvasDragOver = useCallback((e: React.DragEvent) => {
    // Only handle drags that carry our custom asset MIME — ignore unrelated
    // drags (text selections, file drops from the OS, etc.).
    if (
      e.dataTransfer.types.includes("application/x-bevy-asset-id") ||
      e.dataTransfer.types.includes("Files")
    ) {
      e.preventDefault();
      e.dataTransfer.dropEffect = "copy";
      setIsDragOverCanvas(true);
    }
  }, []);

  const handleCanvasDragLeave = useCallback((e: React.DragEvent) => {
    // Only clear if leaving the container itself (not a child)
    if (
      e.relatedTarget instanceof Node &&
      e.currentTarget.contains(e.relatedTarget)
    ) {
      return;
    }
    setIsDragOverCanvas(false);
  }, []);

  const handleCanvasDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      setIsDragOverCanvas(false);
      const assetId = e.dataTransfer.getData("application/x-bevy-asset-id");
      if (!assetId) return;
      // Compute world-space cursor position by inverse-transforming the
      // canvas-container's bounding rect (matches useCanvasViewport math).
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const translation = {
        x: (e.clientX - rect.left - pan.x) / zoom,
        y: (e.clientY - rect.top - pan.y) / zoom,
      };
      try {
        await placeSceneInstance(assetId, translation);
      } catch (err) {
        addToast(`Drop failed: ${err}`, "error");
      }
    },
    [pan.x, pan.y, zoom, addToast],
  );

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

  /**
   * Lift a docked panel into the floating state. Computes a sensible
   * default rect anchored near the top-left of the viewport the first
   * time a panel floats; subsequent toggles keep the previous rect.
   */
  const handleFloatPanel = useCallback(
    (panelId: PanelId) => {
      const existing = dock.prefs.floats[panelId];
      if (existing) {
        // Already floating — toggle off: dock it.
        dock.removeFloat(panelId);
        return;
      }
      // Seed rect sized per panel id: left/right regions are narrower
      // (matching the canonical dock widths); outline/properties are
      // mirrored widths; assets + bottom get wider defaults.
      const width =
        panelId === "bottom"
          ? 720
          : Math.max(
              280,
              dock.prefs.left.width || dock.prefs.right.width || 320,
            );
      const height = panelId === "bottom" ? 280 : 420;
      const rect: FloatingPanelState = {
        x:
          typeof window === "undefined"
            ? 64
            : Math.max(0, Math.floor(window.innerWidth * 0.06)),
        y:
          typeof window === "undefined"
            ? 64
            : Math.max(0, Math.floor(window.innerHeight * 0.08)),
        width,
        height,
        last_floated_at: Date.now(),
      };
      dock.setFloatRect(panelId, rect);
      setFocusedFloatingPanel(panelId);
    },
    [dock],
  );

  const handleDockFloatingPanel = useCallback(
    (panelId: PanelId) => {
      dock.removeFloat(panelId);
      if (focusedFloatingPanel === panelId) setFocusedFloatingPanel(null);
    },
    [dock, focusedFloatingPanel],
  );

  // Drag-and-dock region swap setter (v0.82 P1, ADR-0024). Both pointer
  // drops in DockLayout and the keyboard `Move →` menu in DockHeader /
  // BottomDock funnel through this exact setter so the reducer in
  // useDockResize stays the single source of truth.
  const handleMovePanel = useCallback(
    (panelId: PanelId, target: DockableRegion) =>
      dock.movePanel(panelId, target),
    [dock],
  );
  // ── Fullscreen viewport (Phase E) ─────────────────────────────────────────
  const fullscreen = useFullscreen();

  // Apply the data-fullscreen attribute to body — useFullscreen already
  // mirrors this, but make sure any mount-time flip is reflected in the
  // hook state for tests/components querying it.
  useEffect(() => {
    if (fullscreen.enabled) {
      document.body.dataset.fullscreen = "true";
    } else {
      delete document.body.dataset.fullscreen;
    }
  }, [fullscreen.enabled]);

  useKeyboardShortcuts({
    enabled: editorMode !== "play",
    onUndo: editorMode === "scene" ? handleUndo : handleAssetUndo,
    onRedo: editorMode === "scene" ? handleRedo : handleAssetRedo,
    // v0.82 P2 (ADR-0025): route Delete/Backspace through the multi-
    // delete sink when more than one id is selected. The hook keeps
    // a single-id fallback for the legacy single-select flow.
    onDeleteEntities:
      editorMode === "scene" && selectedIds.size > 1
        ? (ids) => void handleDeleteEntities(ids)
        : undefined,
    selectedIds,
    logState: editorMode === "scene" ? logState : assetLogState,
    selectedEntityId,
    onDeleteEntity: handleDeleteEntity,
    onCreateEntity: editorMode === "scene" ? handleCreateEntity : undefined,
    onOpenCommandPalette: () => setCommandPaletteOpen(true),
    onOpenCheatSheet: () => setCheatSheetOpen(true),
    onRenameSelected: () => setRenameRequestTick((t) => t + 1),
    onFitViewport: () => fitToContent(),
    onToggleBottomDock: dock.toggleBottom,
    onToggleLeftDock: dock.toggleLeft,
    onToggleOutlineDock: dock.toggleOutline,
    onTogglePropertiesDock: dock.toggleProperties,
    onToggleFullscreen: fullscreen.toggle,
  });
  // Drag deltas from DockDivider are signed (positive = mouse moves right/down).
  // For the LEFT divider we want the left dock to grow when delta is positive,
  // so we pass delta as-is. For the RIGHT divider the right dock grows when
  // delta is positive, so we pass -delta (drag-left widens the right column).
  const [leftCollapsed, setLeftCollapsed] = useState(false);
  // Note: outlineCollapsed and propertiesCollapsed now live in DockPrefs
  // (persisted to OPFS) instead of local useState so they survive reloads.
  // See useDockPrefs.toggleOutlineCollapsed / togglePropertiesCollapsed.
  const handleResizeLeft = useCallback(
    (delta: number) => dock.setLeftWidth(dock.prefs.left.width + delta),
    [dock],
  );
  const handleResizeRight = useCallback(
    (delta: number) => dock.setRightWidth(dock.prefs.right.width - delta),
    [dock],
  );
  const handleResizeBottom = useCallback(
    (delta: number) => dock.setBottomHeight(dock.prefs.bottom.height - delta),
    [dock],
  );
  const handleResizeStatusBar = useCallback(
    // Dragging the divider UP (negative screen delta) should grow the
    // status bar; same convention as the bottom-dock divider (which is
    // `height - delta` because dragging down shrinks it).
    (delta: number) =>
      dock.setStatusBarHeight(dock.prefs.statusBar.height - delta),
    [dock],
  );
  const handleResizeRightSplit = useCallback(
    (deltaPx: number) => {
      // Dragging the inner divider down (positive delta) should grow the top
      // region; convert pixel delta into a percentage of the right dock height
      // by dividing by the dock's rendered height (we approximate with the
      // current right width which scales proportionally with the layout).
      const pctDelta = (deltaPx / Math.max(dock.prefs.right.width, 200)) * 50;
      dock.setRightTopHeight(dock.prefs.right.topHeight + pctDelta);
    },
    [dock],
  );

  // ── Command palette catalog (Phase 3.2) ───────────────────────────────────
  // Static list of >15 commands wired to existing App.tsx handlers. Built
  // every render but only when one of the dependencies changes (the
  // handlers are useCallback-wrapped so identity is stable).
  const paletteCommands = useMemo<PaletteCommand[]>(
    () => [
      // File
      {
        id: "file.save",
        label: "Save Scene",
        shortcut: "Ctrl+S",
        group: "File",
        action: handleSave,
      },
      {
        id: "file.load",
        label: "Load Project",
        group: "File",
        action: handleLoad,
      },
      {
        id: "file.export",
        label: "Export Rust",
        group: "File",
        action: () => setExportRustOpen(true),
      },
      {
        id: "file.new-scene",
        label: "New Scene",
        group: "File",
        action: () => handleNewScene(`scene_${Date.now()}`),
      },
      // Edit
      {
        id: "edit.undo",
        label: "Undo",
        shortcut: "Ctrl+Z",
        group: "Edit",
        action: () => {
          if (editorMode === "scene") void handleUndo();
          else void handleAssetUndo();
        },
      },
      {
        id: "edit.redo",
        label: "Redo",
        shortcut: "Ctrl+Shift+Z",
        group: "Edit",
        action: () => {
          if (editorMode === "scene") void handleRedo();
          else void handleAssetRedo();
        },
      },
      {
        id: "edit.delete",
        label: "Delete Selection",
        shortcut: "Del",
        group: "Edit",
        action: () => {
          if (selectedEntityId) void handleDeleteEntity(selectedEntityId);
        },
      },
      {
        id: "edit.new-entity",
        label: "New Entity",
        shortcut: "N",
        group: "Edit",
        action: () => {
          if (editorMode === "scene") void handleCreateEntity();
        },
      },
      {
        id: "edit.rename",
        label: "Rename Selected",
        shortcut: "F2",
        group: "Edit",
        action: () => setRenameRequestTick((t) => t + 1),
      },
      // View
      {
        id: "view.toggle-ai",
        label: "Toggle AI Panel",
        group: "View",
        action: handleToggleAI,
      },
      {
        id: "view.toggle-validation",
        label: "Toggle Validation Center",
        group: "View",
        action: handleToggleValidationCenter,
      },
      {
        id: "view.toggle-tileset",
        label: "Toggle Tileset",
        group: "View",
        action: handleToggleTileset,
      },
      {
        id: "view.toggle-autolayer",
        label: "Toggle Auto Layer",
        group: "View",
        action: handleToggleAutoLayer,
      },
      {
        id: "view.reset-viewport",
        label: "Reset Viewport",
        group: "View",
        action: () => resetViewport(),
      },
      {
        id: "view.fit-viewport",
        label: "Fit Viewport",
        shortcut: "F",
        group: "View",
        action: () => fitToContent(),
      },
      {
        id: "view.open-logic",
        label: "Open Logic Editor",
        group: "View",
        action: handleOpenLogic,
      },
      {
        id: "view.open-code",
        label: "Open Code Editor",
        group: "View",
        action: handleOpenCode,
      },
      {
        id: "view.open-browser",
        label: "Open Project Browser",
        group: "View",
        action: () => setEditorMode("asset-authoring"),
      },
      // Assets
      {
        id: "assets.create",
        label: "Create Scene Asset",
        group: "Assets",
        action: () => handleAssetCreate(`asset_${Date.now()}`, "actor"),
      },
      // Play
      {
        id: "play.toggle",
        label: "Play / Stop",
        group: "Play",
        action: handleTogglePlay,
      },
      // Help
      {
        id: "help.cheatsheet",
        label: "Show Cheat Sheet",
        shortcut: "?",
        group: "Help",
        action: () => setCheatSheetOpen(true),
      },
    ],
    [
      editorMode,
      selectedEntityId,
      handleSave,
      handleLoad,
      handleNewScene,
      handleUndo,
      handleAssetUndo,
      handleRedo,
      handleAssetRedo,
      handleDeleteEntity,
      handleCreateEntity,
      handleToggleAI,
      handleToggleValidationCenter,
      handleToggleTileset,
      handleToggleAutoLayer,
      resetViewport,
      fitToContent,
      handleOpenLogic,
      handleOpenCode,
      handleAssetCreate,
      handleTogglePlay,
    ],
  );

  // ── Command palette executor for Global Search (CRITICAL ISSUE 2) ──────────
  // Expose command palette items as search results via window.
  // The serializable command metadata (id, label, shortcut, group) is returned;
  // the actual action is dispatched via __executeCommand(commandId).
  const serializablePaletteItems = useMemo(() => {
    return paletteCommands.map((cmd) => ({
      id: cmd.id,
      label: cmd.label,
      shortcut: cmd.shortcut,
      group: cmd.group,
    }));
  }, [paletteCommands]);

  // Command executor: looks up command by id and invokes its action.
  const executeCommandById = useCallback(
    (commandId: string) => {
      const cmd = paletteCommands.find((c) => c.id === commandId);
      if (cmd) {
        cmd.action();
      }
    },
    [paletteCommands],
  );

  // Expose to window for SearchTab Global Search integration.
  if (typeof window !== "undefined") {
    (window as any).__getCommandPaletteItems = () => serializablePaletteItems;
    (window as any).__executeCommand = (commandId: string) =>
      executeCommandById(commandId);
    // CRITICAL ISSUE 3: scene-asset search must use App-owned useSceneAssets().open()
    // so the React state (assetDoc, activeAssetId) is updated and the authoring
    // UI re-renders with the opened asset. The low-level openSceneAsset() only
    // calls the WASM bridge without updating React state.
    (window as any).__openSceneAssetFromSearch = async (assetId: string) => {
      await openAsset(assetId);
    };
    // PR3: logic-graph search opens the graph in logic mode.
    (window as any).__openLogicGraphFromSearch = async (assetId: string) => {
      await openLogicGraphAsset(assetId);
    };
    // PR3: schema search opens the schema authoring panel filtered to that schema.
    (window as any).__focusSchemaFromSearch = (typeId: string) => {
      setEditorMode("asset-authoring");
      // SchemaAuthoringPanel reads __focusedSchemaId and scrolls/filters to it.
      (window as any).__focusedSchemaId = typeId;
    };
    // PR3: validation-issue search opens the Validation Center.
    (window as any).__openValidationCenter = () => {
      setValidationCenterOpen(true);
    };
    // PR3: navigation to a specific validation issue (highlights the issue).
    (window as any).__navigateToValidationIssue = (issueId: string) => {
      (window as any).__focusedValidationIssueId = issueId;
    };
  }

  // ── Cheat sheet shortcuts (Phase 3.3) ─────────────────────────────────────
  const cheatSheetGroups = useMemo<CheatSheetGroup[]>(
    () => [
      {
        title: "General",
        entries: [
          { keys: ["Ctrl", "K"], label: "Open command palette" },
          { keys: ["?"], label: "Open cheat sheet" },
        ],
      },
      {
        title: "Editing",
        entries: [
          { keys: ["Ctrl", "Z"], label: "Undo" },
          { keys: ["Ctrl", "Y"], label: "Redo" },
          { keys: ["Ctrl", "Shift", "Z"], label: "Redo (alternate)" },
          { keys: ["N"], label: "New entity" },
          { keys: ["F2"], label: "Rename selected entity" },
          { keys: ["Del"], label: "Delete selection" },
          { keys: ["Backspace"], label: "Delete selection" },
        ],
      },
      {
        title: "Viewport",
        entries: [
          { keys: ["F"], label: "Fit viewport to content" },
          { keys: ["Wheel"], label: "Zoom around cursor" },
          { keys: ["Space", "+ Drag"], label: "Pan canvas" },
        ],
      },
      {
        title: "Play",
        entries: [
          {
            keys: ["Space", "+ W/A/S/D"],
            label: "Move (gamepad in play mode)",
          },
        ],
      },
    ],
    [],
  );

  // ── Panel content (shared between dock layout and floating portals) ──────────
  // Phase B T2.1: floating panels render real dock content instead of placeholders.
  // Extracted as useMemo so the same JSX instance is shared between docked and
  // floating renders — avoids creating two separate React trees for the same UI.

  const outlinePanelContent = useMemo(
    () => (
      <div className="dock-content dock-content-outline">
        {editorMode === "scene" && (
          <>
            {aiPanelOpen && (
              <AIAssistantPanel
                aiState={{
                  prompt,
                  loading: aiLoading,
                  proposals,
                  error: aiError,
                  contextStats,
                  contextUsedChars,
                }}
                onToggle={handleToggleAI}
                onPromptChange={setPrompt}
                onSubmit={handleSubmitAI}
                onApply={handleApplyProposal}
                onDiscard={discardProposal}
                applyingIds={applyingIds}
                contextStats={contextStats}
                contextUsedChars={contextUsedChars}
                taskMode={taskMode}
                onTaskModeChange={setTaskMode}
                enabledSources={enabledSources}
                onContextToggle={handleContextToggle}
              />
            )}
            {validationCenterOpen && (
              <ValidationCenter
                onClose={handleToggleValidationCenter}
                onNavigate={handleValidationCenterNavigate}
              />
            )}
            {tilesetPanelOpen && (
              <TilesetPanel
                selectedTilesetId={selectedTilesetId}
                onSelectTileset={handleSelectTileset}
                assetDoc={assetDoc}
                activeAssetLogicalPath={activeAssetLogicalPath}
              />
            )}
            <HierarchyPanel
              scene={scene}
              selectedId={selectedEntityId}
              onSelect={setSelectedEntityId}
              onRename={handleRename}
              instances={instances}
              onCreateEntity={
                editorMode === "scene" ? handleCreateEntity : undefined
              }
              renameRequest={renameRequestTick}
              onSelectModifier={
                editorMode === "scene"
                  ? (id, mod) => selectEntity(id, mod)
                  : undefined
              }
              selectedIds={selectedIds}
              onAttachLogic={handleAttachLogic}
              onOpenBoundLogic={handleOpenBoundLogic}
              onCreateFromRecipe={handleCreateFromRecipe}
              onInspectRuntimeLogic={handleInspectRuntimeLogic}
            />
          </>
        )}
        {editorMode === "asset-authoring" && (
          <ProjectAssetBrowser
            entries={assetEntries}
            logicGraphEntries={logicGraphEntries}
            onCreate={handleAssetCreate}
            onRename={handleAssetRename}
            onDuplicate={handleAssetDuplicate}
            onDelete={handleAssetDelete}
            onOpen={handleOpenAsset}
            onOpenLogicGraph={async (assetId) => {
              await openLogicGraphAsset(assetId);
            }}
            onPlaceInstance={placeInstance}
          />
        )}
        {editorMode === "logic" && (
          <Suspense
            fallback={
              <div className="surface-loading">Loading logic graph...</div>
            }
          >
            <LogicGraphEditor editorMode={editorMode} />
          </Suspense>
        )}
        {editorMode === "code" && (
          <Suspense
            fallback={
              <div className="surface-loading">Loading source editor...</div>
            }
          >
            <CodeEditor
              navigationTarget={pendingNavigation}
              onEditorReady={() => setPendingNavigation(null)}
            />
          </Suspense>
        )}
      </div>
    ),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [
      editorMode,
      scene,
      selectedEntityId,
      selectedIds,
      instances,
      aiPanelOpen,
      validationCenterOpen,
      tilesetPanelOpen,
      assetDoc,
      activeAssetLogicalPath,
      assetEntries,
      pendingNavigation,
      prompt,
      aiLoading,
      proposals,
      aiError,
      handleToggleAI,
      setPrompt,
      handleSubmitAI,
      handleApplyProposal,
      discardProposal,
      applyingIds,
      contextStats,
      contextUsedChars,
      handleToggleValidationCenter,
      handleSelectTileset,
      setSelectedEntityId,
      handleRename,
      handleCreateEntity,
      renameRequestTick,
      selectEntity,
      handleAssetCreate,
      handleAssetRename,
      handleAssetDuplicate,
      handleAssetDelete,
      handleOpenAsset,
      placeInstance,
    ],
  );

  const propertiesPanelContent = useMemo(
    () => (
      <div className="dock-content dock-content-properties">
        {editorMode === "scene" && (
          <InspectorPanel
            scene={scene}
            selectedId={selectedEntityId}
            selectedIds={selectedIds}
            onRename={handleRename}
            onSetField={handleSetField}
            onSetFieldOnMultiple={handleSetFieldOnMultiple}
            onRemoveComponent={handleRemoveComponent}
            onAddComponent={handleAddComponent}
            instances={instances}
            onRemoveInstance={removeInstance}
            onReplaceInstanceAsset={replaceInstanceAsset}
            assetEntries={assetEntries}
            onJumpToSource={handleJumpToSource}
            onAttachLogic={handleAttachLogic}
            onOpenBoundLogic={handleOpenBoundLogic}
            onCreateFromRecipe={handleCreateFromRecipe}
            onInspectRuntimeLogic={handleInspectRuntimeLogic}
            onSwitchToLogicMode={handleSwitchToLogicMode}
          />
        )}
        {editorMode === "asset-authoring" && assetDoc && (
          <AssetAuthoringView
            document={assetDoc}
            activeEntityId={null}
            onSelectEntity={() => {}}
            onCommit={handleAssetCommit}
            onAddComponent={handleAssetAddComponent}
            onRemoveComponent={handleAssetRemoveComponent}
            onUndo={handleAssetUndo}
            onRedo={handleAssetRedo}
            onSave={handleAssetSave}
            onBackToScene={handleBackToScene}
            canUndo={assetLogState.can_undo}
            canRedo={assetLogState.can_redo}
            dirty={assetDirty}
          />
        )}
        {editorMode === "asset-authoring" &&
          autoLayerPanelOpen &&
          (selectedAutoLayer ? (
            <AutoLayerPanel
              layer={selectedAutoLayer}
              assetRef={activeAssetLogicalPath ?? ""}
              onRegenerate={refresh}
            />
          ) : (
            <div className="tileset-panel">
              <h3>Auto Layer</h3>
              <p style={{ fontSize: 12, color: "#666" }}>
                No auto layers in this asset. Open a level scene asset to edit
                auto layers.
              </p>
            </div>
          ))}
      </div>
    ),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [
      editorMode,
      scene,
      selectedEntityId,
      selectedIds,
      instances,
      assetDoc,
      assetEntries,
      assetLogState,
      assetDirty,
      autoLayerPanelOpen,
      selectedAutoLayer,
      activeAssetLogicalPath,
      handleRename,
      handleSetField,
      handleSetFieldOnMultiple,
      handleRemoveComponent,
      handleAddComponent,
      removeInstance,
      replaceInstanceAsset,
      handleJumpToSource,
      handleAssetCommit,
      handleAssetAddComponent,
      handleAssetRemoveComponent,
      handleAssetUndo,
      handleAssetRedo,
      handleAssetSave,
      handleBackToScene,
      refresh,
    ],
  );

  const bottomPanelContent = useMemo(
    () => <ConsoleTab />,
    // ConsoleTab has no props — safe to omit from deps
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  return (
    <div className="app">
      <DockLayout
        onMovePanel={handleMovePanel}
        menu={
          <>
            <AppHeader
              editorMode={editorMode}
              onOpenAssets={() => {}}
              onBackToScene={
                editorMode === "asset-authoring" ? handleBackToScene : undefined
              }
              onOpenLogic={editorMode === "scene" ? handleOpenLogic : undefined}
              onOpenCode={editorMode === "scene" ? handleOpenCode : undefined}
              logState={editorMode === "scene" ? logState : assetLogState}
              onUndo={editorMode === "scene" ? handleUndo : handleAssetUndo}
              onRedo={editorMode === "scene" ? handleRedo : handleAssetRedo}
              onSave={editorMode === "scene" ? handleSave : handleAssetSave}
              onSaveAs={() => setSaveModalOpen(true)}
              onLoad={handleLoad}
              onExportRust={() => setExportRustOpen(true)}
              onNewScene={() => handleNewScene(`scene_${Date.now()}`)}
              onDeleteEntity={() => {
                if (selectedEntityId) void handleDeleteEntity(selectedEntityId);
              }}
              selectedEntityId={selectedEntityId}
              onToggleAI={handleToggleAI}
              aiPanelOpen={aiPanelOpen}
              onToggleValidationCenter={handleToggleValidationCenter}
              validationCenterOpen={validationCenterOpen}
              onToggleTileset={handleToggleTileset}
              tilesetPanelOpen={tilesetPanelOpen}
              onToggleAutoLayer={handleToggleAutoLayer}
              autoLayerPanelOpen={autoLayerPanelOpen}
              onTogglePlay={handleTogglePlay}
              onOpenSearch={() => setCommandPaletteOpen(true)}
              onOpenCheatSheet={() => setCheatSheetOpen(true)}
              onWelcomeTour={() =>
                console.warn("[menu] TODO: wire Welcome Tour")
              }
              onAbout={handleAbout}
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
            {editorMode === "play" && <GameOverlay onStop={handleTogglePlay} />}
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
        onResizeLeft={handleResizeLeft}
        onResizeRight={handleResizeRight}
        onResizeBottom={handleResizeBottom}
        onResizeStatusBar={handleResizeStatusBar}
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
              onFloatToggle={() => handleFloatPanel("assets")}
              floating={false}
            />
          )
        }
        center={
          <CenterDock
            scenes={scenes}
            currentId={currentId}
            onTabClick={handleTabClick}
            onNewScene={handleNewScene}
            onDeleteScene={handleDeleteScene}
            onRenameScene={handleRenameScene}
            canvas={
              <div
                className={`canvas-container${isDragOverCanvas ? " canvas-drop-active" : ""}`}
                data-testid="canvas-drop-target"
                onDragOver={handleCanvasDragOver}
                onDragLeave={handleCanvasDragLeave}
                onDrop={handleCanvasDrop}
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
            onFloatToggleOutline={() => handleFloatPanel("outline")}
            onFloatToggleProperties={() => handleFloatPanel("properties")}
            outline={outlinePanelContent}
            properties={propertiesPanelContent}
            onToggleCollapseOutline={dock.toggleOutlineCollapsed}
            onToggleCollapseProperties={dock.togglePropertiesCollapsed}
            onCloseOutline={dock.toggleOutline}
            onCloseProperties={dock.toggleProperties}
            onResizeSplit={handleResizeRightSplit}
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
              onFloatToggle={() => handleFloatPanel("bottom")}
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
            onDock={() => handleDockFloatingPanel(panelId)}
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
          onSave={handleSaveConfirm}
          onCancel={() => setSaveModalOpen(false)}
        />
      )}
      {saveWorkspacePresetOpen && (
        <PromptDialog
          title="Save Workspace Preset"
          label="Preset name"
          placeholder="e.g. level-design"
          defaultValue=""
          onConfirm={handleSaveWorkspacePresetSubmit}
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
          onSave={handleSaveAndSwitch}
          onDiscard={handleDiscardAndSwitch}
          onCancel={handleCancelSwitch}
        />
      )}
      {pendingBackToScene && activeAssetLogicalPath && (
        <AssetUnsavedChangesDialog
          logicalPath={activeAssetLogicalPath}
          unsavedCount={assetLogState.size}
          onSave={handleAssetSaveAndLeave}
          onDiscard={handleAssetDiscardAndLeave}
          onCancel={handleAssetCancelBack}
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
          onCreateBlankScene={() => handleNewScene(`scene_${Date.now()}`)}
          onOpenLogicEditor={handleOpenLogic}
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
