/**
 * useSceneHandlers — extracts the ~30+ useCallback handlers from App.tsx.
 *
 * Per the App.tsx decomposition plan, this hook owns every `handleX`
 * closure that was previously declared inline inside `AppInner`. The
 * hook is intentionally headless (no JSX, no DOM effects): the
 * composition root in App.tsx wires the context, calls the hook, and
 * passes the returned object down to AppShell and other consumers.
 *
 * The hook does not own the *state* it operates on — it only wires the
 * setters. State ownership stays where it always was: workspace
 * selection / mode live in `useEditorWorkspaceController`, dialog open
 * flags live in AppInner, and panel-local toggles live in `useDockResize`.
 * The wiring is one-pass: pass a `SceneHandlersContext`, get a flat
 * bag of memoised handlers back.
 */
import { useCallback } from "react";
import type { EditorMode } from "../components/MenuBar";
import type { DispatchResult } from "./useSceneState";
import type { TilesetMetadata } from "../services/tilesets";
import type {
  PanelId,
  DockableRegion,
  FloatingPanelState,
} from "./useDockPrefs";
import {
  sceneCreate,
  sceneSwitch,
  sceneSwitchCommit,
  sceneDelete,
  sceneRename,
} from "../services/scenes";
import { findSourceLocation } from "../services/code-files";
import type { NavigationTarget } from "../types/navigation";
import type { ToastSeverity } from "./useToasts";

/** Toast shorthand used everywhere inside App.tsx. */
export type AddToastFn = (message: string, severity?: ToastSeverity) => string;

/**
 * Imperative scene state interface — only the operations App.tsx needs.
 */
export interface SceneStateApi {
  refresh: () => Promise<void>;
  dispatch: (envelope: object) => Promise<DispatchResult>;
}

/**
 * Asset authoring state interface — only the operations App.tsx needs.
 */
export interface SceneAssetsApi {
  open: (assetId: string) => Promise<void>;
  close: () => void;
  dispatch: (command: object) => Promise<string | void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  save: () => Promise<void>;
  create: (name: string, role: string) => Promise<void>;
  rename: (assetId: string, newPath: string) => Promise<void>;
  duplicate: (assetId: string) => Promise<void>;
  deleteAsset: (assetId: string) => Promise<void>;
  placeInstance: (
    assetId: string,
    translation: { x: number; y: number },
  ) => Promise<void>;
  dirty: boolean;
  logState: { can_undo: boolean; can_redo: boolean; size: number };
}

/**
 * Multi-scene list API — only the calls App.tsx makes.
 */
export interface ScenesApi {
  scenes: Array<{ id: string; name: string }>;
  currentId: string | null;
  refresh: () => Promise<void>;
}

/**
 * Workspace selection + mode API — used to keep the controller as the
 * source of truth for editorMode/selection.
 */
export interface WorkspaceApi {
  editorMode: EditorMode;
  setEditorMode: (mode: EditorMode) => void;
  selectedIds: Set<string>;
  selectedEntityId: string | null;
  setSelectedEntityId: (id: string | null) => void;
  selectEntity: (id: string, modifier: "plain" | "range" | "toggle") => void;
  clearSelection: () => void;
  setSelectedIds: (ids: Set<string>) => void;
  setPendingNavigation: (target: NavigationTarget | null) => void;
  setPendingBackToScene: (pending: boolean) => void;
}

/**
 * Dialog open-flag setters — every `setXOpen` App.tsx used to own.
 *
 * Boolean setters accept a value or an updater function (the `useState`
 * shape) so the dialogs API is uniform. All concrete `useState`
 * setters in React already accept either form.
 */
export interface DialogFlagsApi {
  setExportRustOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  setSaveModalOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  setSaveWorkspacePresetOpen: (
    open: boolean | ((prev: boolean) => boolean),
  ) => void;
  setAboutOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  setAiPanelOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  setEnabledSources: (updater: (prev: Set<string>) => Set<string>) => void;
  setValidationCenterOpen: (
    open: boolean | ((prev: boolean) => boolean),
  ) => void;
  setTilesetPanelOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  setSelectedTilesetId: (id: string | null) => void;
  setAutoLayerPanelOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  setIsDragOverCanvas: (over: boolean) => void;
  setActiveAssetLogicalPath: (path: string | null) => void;
  setRenameRequestTick: (updater: (prev: number) => number) => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setCheatSheetOpen: (open: boolean) => void;
  setPendingSwitchId: (id: string | null) => void;
  setPendingSwitchSource: (source: string | null) => void;
  setLeftCollapsed: (collapsed: boolean | ((prev: boolean) => boolean)) => void;
  setFocusedFloatingPanel: (id: PanelId | null) => void;
  setApplyingIds: (updater: (prev: Set<string>) => Set<string>) => void;
}

/**
 * Dock + fullscreen API — docked panels, floating panels, fullscreen.
 * Only the surface used by handlers is declared.
 */
export interface DockApi {
  prefs: {
    left: { width: number; visible: boolean };
    right: { width: number; visible: boolean; topHeight: number };
    bottom: { height: number; visible: boolean };
    statusBar: { height: number };
    floats: Partial<Record<PanelId, FloatingPanelState>>;
  };
  toggleLeft: () => void;
  toggleBottom: () => void;
  toggleOutline: () => void;
  toggleProperties: () => void;
  movePanel: (panelId: PanelId, target: DockableRegion) => void;
  removeFloat: (panelId: PanelId) => void;
  setFloatRect: (panelId: PanelId, rect: FloatingPanelState) => void;
  saveCurrentAsPreset: (name: string) => void;
  setLeftWidth: (w: number) => void;
  setRightWidth: (w: number) => void;
  setBottomHeight: (h: number) => void;
  setStatusBarHeight: (h: number) => void;
  setRightTopHeight: (h: number) => void;
}

export interface FullscreenApi {
  toggle: () => void;
}

export interface ViewportApi {
  zoom: number;
  pan: { x: number; y: number };
  fitToContent: () => void;
  reset: () => void;
}

/**
 * Scene asset catalogue entries — needed for `handleOpenAsset` without
 * forcing the hook to know the full `useSceneAssets` shape.
 */
export interface AssetCatalogEntry {
  asset_id: string;
  logical_path: string;
  role: string;
}

/**
 * AI assistant operations — `submit` / `applyProposal` come from
 * `useAIAssistant`. App.tsx keeps the hook result and threads both
 * closures through this context so the handlers can call them.
 */
export interface AiApi {
  submit: (
    dispatch: (envelope: object) => Promise<DispatchResult>,
  ) => Promise<void>;
  applyProposal: (
    proposalId: string,
    dispatch: (envelope: object) => Promise<DispatchResult>,
  ) => Promise<void>;
}

/**
 * Entity with components — used by `handleSetFieldOnMultiple` to filter
 * owning ids. App.tsx passes the live `scene.entities` list.
 */
export interface EntityWithComponents {
  id: string;
  name: string;
  components: ReadonlyArray<{ type_id: string }>;
}

export interface PendingSwitch {
  id: string | null;
  source: string | null;
}

export interface SceneHandlersContext {
  scene: SceneStateApi;
  workspace: WorkspaceApi;
  assets: SceneAssetsApi;
  scenes: ScenesApi;
  dialogs: DialogFlagsApi;
  dock: DockApi;
  fullscreen: FullscreenApi;
  viewport: ViewportApi;
  assetEntries: AssetCatalogEntry[];
  addToast: AddToastFn;
  ai: AiApi;
  /** Currently focused floating panel id (read-only here; AppInner clears it on dock). */
  focusedFloatingPanel: PanelId | null;
  /** Currently active entities (with components) — needed for set-field-on-multiple + create-entity suffix. */
  entities: ReadonlyArray<EntityWithComponents>;
}

/**
 * Return shape — flat object so consumers can destructure once.
 *
 * Each member is wrapped in `useCallback` so the reference is stable
 * across renders when its dependencies don't change. This keeps the
 * memoised panel-content trees in App.tsx from churning.
 */
export interface SceneHandlers {
  // ── Scene / entity editing ────────────────────────────────────────────
  handleDeleteEntity: (id: string) => Promise<void>;
  handleDeleteEntities: (ids: Iterable<string>) => Promise<void>;
  handleCreateEntity: () => Promise<void>;
  handleRename: (entityId: string, newName: string) => Promise<void>;
  handleSetField: (
    entityId: string,
    typeId: string,
    fieldPath: string,
    value: unknown,
  ) => Promise<void>;
  handleSetFieldOnMultiple: (
    entityIds: string[],
    typeId: string,
    fieldPath: string,
    value: unknown,
  ) => Promise<void>;
  handleAddComponent: (entityId: string, typeId: string) => Promise<void>;
  handleRemoveComponent: (entityId: string, typeId: string) => Promise<void>;
  handleAttachLogic: (instanceId: string) => Promise<void>;
  handleOpenBoundLogic: (entityId: string) => Promise<void>;
  handleCreateFromRecipe: () => void;
  handleInspectRuntimeLogic: () => void;
  handleSwitchToLogicMode: () => void;

  // ── Undo / Redo / Save / Load ─────────────────────────────────────────
  handleUndo: () => Promise<void>;
  handleRedo: () => Promise<void>;
  handleSave: () => void;
  handleSaveConfirm: (name: string) => Promise<void>;
  handleSaveWorkspacePresetSubmit: (name: string) => void;
  handleAbout: () => void;
  handleLoad: () => Promise<void>;

  // ── Multi-scene ───────────────────────────────────────────────────────
  handleTabClick: (id: string) => Promise<void>;
  handleNewScene: (name: string) => Promise<void>;
  handleDeleteScene: (id: string) => Promise<void>;
  handleRenameScene: (id: string, newName: string) => Promise<void>;
  handleSaveAndSwitch: () => Promise<void>;
  handleDiscardAndSwitch: () => Promise<void>;
  handleCancelSwitch: () => void;
  pendingSwitchId: string | null;
  pendingSwitchSource: string | null;

  // ── Asset authoring ───────────────────────────────────────────────────
  handleOpenAsset: (assetId: string) => Promise<void>;
  handleAssetCreate: (name: string, role: string) => Promise<void>;
  handleAssetRename: (assetId: string, newPath: string) => Promise<void>;
  handleAssetDuplicate: (assetId: string) => Promise<void>;
  handleAssetDelete: (assetId: string) => Promise<void>;
  handleBackToScene: () => void;
  handleAssetSaveAndLeave: () => Promise<void>;
  handleAssetDiscardAndLeave: () => void;
  handleAssetCancelBack: () => void;
  assetDirty: boolean;

  // ── Asset command dispatch (C-2 adapter) ───────────────────────────────
  handleAssetCommit: (
    localId: string,
    typeId: string,
    fieldPath: string,
    value: unknown,
  ) => Promise<void>;
  handleAssetAddComponent: (localId: string, typeId: string) => Promise<void>;
  handleAssetRemoveComponent: (
    localId: string,
    typeId: string,
  ) => Promise<void>;
  handleAssetUndo: () => Promise<void>;
  handleAssetRedo: () => Promise<void>;
  handleAssetSave: () => Promise<void>;

  // ── Mode switches ─────────────────────────────────────────────────────
  handleOpenLogic: () => void;
  handleOpenCode: () => void;
  handleOpenWorldWorkspace: () => void;
  handleJumpToSource: (typeId: string) => Promise<void>;
  handleTogglePlay: () => void;

  // ── Canvas drag-drop ──────────────────────────────────────────────────
  handleCanvasDragOver: (e: React.DragEvent) => void;
  handleCanvasDragLeave: (e: React.DragEvent) => void;
  handleCanvasDrop: (e: React.DragEvent) => Promise<void>;

  // ── Floating panel management ─────────────────────────────────────────
  handleFloatPanel: (panelId: PanelId) => void;
  handleDockFloatingPanel: (panelId: PanelId) => void;
  handleMovePanel: (panelId: PanelId, target: DockableRegion) => void;

  // ── Dock resize handlers (drag deltas) ────────────────────────────────
  handleResizeLeft: (delta: number) => void;
  handleResizeRight: (delta: number) => void;
  handleResizeBottom: (delta: number) => void;
  handleResizeStatusBar: (delta: number) => void;
  handleResizeRightSplit: (deltaPx: number) => void;

  // ── AI panel toggles ──────────────────────────────────────────────────
  handleToggleAI: () => void;
  handleContextToggle: (sourceName: string, enabled: boolean) => void;
  handleToggleValidationCenter: () => void;
  handleValidationCenterNavigate: (
    issue: import("../services/validation-center").ValidationIssue,
  ) => Promise<void>;
  handleToggleTileset: () => void;
  handleSelectTileset: (tileset: TilesetMetadata) => void;
  handleToggleAutoLayer: () => void;
  handleSubmitAI: () => Promise<void>;
  handleApplyProposal: (proposalId: string) => Promise<void>;

  // ── Viewport / camera ─────────────────────────────────────────────────
  handleResetViewport: () => void;
  handleFitViewport: () => void;

  // ── Dock toggles exposed by useDockResize (re-exposed for callers) ────
  handleToggleLeftDock: () => void;
  handleToggleOutlineDock: () => void;
  handleTogglePropertiesDock: () => void;
  handleToggleBottomDock: () => void;
  handleToggleFullscreen: () => void;
}

/**
 * `useSceneHandlers` — given the workspace / scene / dock / toast /
 * viewport / dialog-flag context, return a flat bag of memoised
 * handlers that App.tsx previously declared inline.
 */
export function useSceneHandlers(
  ctx: SceneHandlersContext,
  pendingSwitch: PendingSwitch,
): SceneHandlers {
  const {
    scene,
    workspace,
    assets,
    scenes,
    dialogs,
    dock,
    fullscreen,
    viewport,
    assetEntries,
    addToast,
    ai,
    entities,
    focusedFloatingPanel,
  } = ctx;

  // ── Undo / Redo / Save / Load ───────────────────────────────────────────
  const handleUndo = useCallback(async () => {
    try {
      const snap = await (window as any).undo();
      JSON.parse(snap);
      await scene.refresh();
      workspace.setSelectedEntityId(null);
    } catch (e) {
      addToast(`Undo failed: ${e}`, "error");
    }
  }, [scene, workspace, addToast]);

  const handleRedo = useCallback(async () => {
    try {
      const snap = await (window as any).redo();
      await scene.refresh();
      workspace.setSelectedEntityId(null);
    } catch (e) {
      addToast(`Redo failed: ${e}`, "error");
    }
  }, [scene, workspace, addToast]);

  const handleSave = useCallback(() => {
    dialogs.setSaveModalOpen(true);
  }, [dialogs]);

  const handleSaveConfirm = useCallback(
    async (name: string) => {
      dialogs.setSaveModalOpen(false);
      try {
        const path = await (window as any).save_scene(name);
        console.log(`Saved to ${path}`);
      } catch (e) {
        addToast(`Save failed: ${e}`, "error");
      }
    },
    [dialogs, addToast],
  );

  const handleSaveWorkspacePresetSubmit = useCallback(
    (name: string) => {
      dialogs.setSaveWorkspacePresetOpen(false);
      dock.saveCurrentAsPreset(name);
    },
    [dialogs, dock],
  );

  const handleAbout = useCallback(() => {
    dialogs.setAboutOpen(true);
  }, [dialogs]);

  const handleLoad = useCallback(async () => {
    try {
      await (window as any).load_project();
      await scene.refresh();
      workspace.setSelectedEntityId(null);
    } catch (e) {
      addToast(`Load project failed: ${e}`, "error");
    }
  }, [scene, workspace, addToast]);

  // ── Entity field rename / set-field / add-remove component ──────────────
  const handleRename = useCallback(
    async (entityId: string, newName: string) => {
      const result = await scene.dispatch({
        command: {
          type: "RenameEntity",
          entity_id: entityId,
          new_name: newName,
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      });
      if (result.error) addToast(`Rename failed: ${result.error}`, "error");
    },
    [scene, addToast],
  );

  const handleSetField = useCallback(
    async (entityId: string, typeId: string, fieldPath: string, value: any) => {
      const result = await scene.dispatch({
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
    },
    [scene, addToast],
  );

  const handleRemoveComponent = useCallback(
    async (entityId: string, typeId: string) => {
      const result = await scene.dispatch({
        command: {
          type: "RemoveComponent",
          entity_id: entityId,
          type_id: typeId,
        },
        metadata: { authorship: "user", timestamp: Date.now() },
      });
      if (result.error)
        addToast(`Remove component failed: ${result.error}`, "error");
    },
    [scene, addToast],
  );

  const handleSetFieldOnMultiple = useCallback(
    async (
      entityIds: string[],
      typeId: string,
      fieldPath: string,
      value: unknown,
    ) => {
      if (entityIds.length === 0) return;
      const owningIds = entities
        .filter((e) => entityIds.includes(e.id))
        .filter((e) => e.components.some((c) => c.type_id === typeId))
        .map((e) => e.id);
      if (owningIds.length === 0) {
        addToast("No selected entities own that component.", "error");
        return;
      }
      const result = await scene.dispatch({
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
    },
    [entities, scene, addToast],
  );

  const handleAddComponent = useCallback(
    async (entityId: string, typeId: string) => {
      const result = await scene.dispatch({
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
    },
    [scene, addToast],
  );

  // ── Logic graph / recipe workflows ─────────────────────────────────────
  const handleAttachLogic = useCallback(
    async (_instanceId: string) => {
      workspace.setEditorMode("logic");
    },
    [workspace],
  );

  const handleOpenBoundLogic = useCallback(
    async (_entityId: string) => {
      workspace.setEditorMode("logic");
    },
    [workspace],
  );

  const handleCreateFromRecipe = useCallback(() => {
    workspace.setEditorMode("logic");
  }, [workspace]);

  const handleInspectRuntimeLogic = useCallback(() => {
    workspace.setEditorMode("logic");
  }, [workspace]);

  const handleSwitchToLogicMode = useCallback(() => {
    workspace.setEditorMode("logic");
  }, [workspace]);

  // ── Entity CRUD ────────────────────────────────────────────────────────
  const handleDeleteEntity = useCallback(
    async (id: string) => {
      if (!id) return;
      await scene.dispatch({
        command: { type: "DeleteEntity", id },
        metadata: { authorship: "keyboard", timestamp: Date.now() },
      });
      workspace.setSelectedEntityId(null);
    },
    [scene, workspace],
  );

  const handleDeleteEntities = useCallback(
    async (ids: Iterable<string>) => {
      const arr = Array.from(ids);
      if (arr.length === 0) return;
      await scene.dispatch({
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
      workspace.clearSelection();
    },
    [scene, workspace],
  );

  const handleCreateEntity = useCallback(async () => {
    let maxSuffix = 0;
    const re = /^Entity (\d+)$/;
    for (const e of entities) {
      const m = re.exec(e.name);
      if (m) {
        const n = parseInt(m[1], 10);
        if (!Number.isNaN(n) && n > maxSuffix) maxSuffix = n;
      }
    }
    const newName = `Entity ${maxSuffix + 1}`;
    const newId = `ent_${Date.now()}_${Math.floor(Math.random() * 1e6).toString(36)}`;
    await scene.dispatch({
      command: {
        type: "CreateEntity",
        id: newId,
        name: newName,
        components: [],
      },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    workspace.setSelectedEntityId(newId);
  }, [entities, scene, workspace]);

  // ── Multi-scene ────────────────────────────────────────────────────────
  const handleTabClick = useCallback(
    async (id: string) => {
      if (id === scenes.currentId) return;
      const result = await sceneSwitch(id);
      if (result.dirtyPromptRequired) {
        dialogs.setPendingSwitchId(id);
        dialogs.setPendingSwitchSource(result.sourceName);
      }
      await scene.refresh();
    },
    [scenes, scene, dialogs],
  );

  const handleNewScene = useCallback(
    async (name: string) => {
      await sceneCreate(name);
      await scenes.refresh();
    },
    [scenes],
  );

  const handleDeleteScene = useCallback(
    async (id: string) => {
      await sceneDelete(id);
      await scenes.refresh();
    },
    [scenes],
  );

  const handleRenameScene = useCallback(
    async (id: string, newName: string) => {
      await sceneRename(id, newName);
      await scenes.refresh();
    },
    [scenes],
  );

  const handleSaveAndSwitch = useCallback(async () => {
    if (!pendingSwitch.id) return;
    const currentScene = scenes.scenes.find((s) => s.id === scenes.currentId);
    if (currentScene) {
      await (window as any).save_scene(currentScene.name);
    }
    await sceneSwitchCommit(pendingSwitch.id);
    dialogs.setPendingSwitchId(null);
    dialogs.setPendingSwitchSource(null);
    await scene.refresh();
    await scenes.refresh();
  }, [pendingSwitch, scenes, scene, dialogs]);

  const handleDiscardAndSwitch = useCallback(async () => {
    if (!pendingSwitch.id) return;
    await sceneSwitchCommit(pendingSwitch.id);
    dialogs.setPendingSwitchId(null);
    dialogs.setPendingSwitchSource(null);
    await scene.refresh();
    await scenes.refresh();
  }, [pendingSwitch, scenes, scene, dialogs]);

  const handleCancelSwitch = useCallback(() => {
    dialogs.setPendingSwitchId(null);
    dialogs.setPendingSwitchSource(null);
  }, [dialogs]);

  // ── Asset authoring ────────────────────────────────────────────────────
  const handleOpenAsset = useCallback(
    async (assetId: string) => {
      const entry = assetEntries.find((e) => e.asset_id === assetId);
      if (!entry) return;
      await assets.open(assetId);
      dialogs.setActiveAssetLogicalPath(entry.logical_path);
      workspace.setEditorMode("asset-authoring");
    },
    [assetEntries, assets, dialogs, workspace],
  );

  const handleAssetCreate = useCallback(
    async (name: string, role: string) => {
      await assets.create(name, role);
    },
    [assets],
  );

  const handleAssetRename = useCallback(
    async (assetId: string, newPath: string) => {
      await assets.rename(assetId, newPath);
    },
    [assets],
  );

  const handleAssetDuplicate = useCallback(
    async (assetId: string) => {
      await assets.duplicate(assetId);
    },
    [assets],
  );

  const handleAssetDelete = useCallback(
    async (assetId: string) => {
      await assets.deleteAsset(assetId);
    },
    [assets],
  );

  const handleBackToScene = useCallback(() => {
    if (assets.dirty) {
      workspace.setPendingBackToScene(true);
    } else {
      assets.close();
      dialogs.setActiveAssetLogicalPath(null);
      workspace.setEditorMode("scene");
    }
  }, [assets, dialogs, workspace]);

  const handleAssetSaveAndLeave = useCallback(async () => {
    await assets.save();
    workspace.setPendingBackToScene(false);
    assets.close();
    dialogs.setActiveAssetLogicalPath(null);
    workspace.setEditorMode("scene");
  }, [assets, dialogs, workspace]);

  const handleAssetDiscardAndLeave = useCallback(() => {
    assets.close();
    workspace.setPendingBackToScene(false);
    dialogs.setActiveAssetLogicalPath(null);
    workspace.setEditorMode("scene");
  }, [assets, dialogs, workspace]);

  const handleAssetCancelBack = useCallback(() => {
    workspace.setPendingBackToScene(false);
  }, [workspace]);

  // ── Mode switches ─────────────────────────────────────────────────────
  const handleOpenLogic = useCallback(() => {
    workspace.setEditorMode("logic");
  }, [workspace]);

  const handleOpenCode = useCallback(() => {
    workspace.setEditorMode("code");
  }, [workspace]);

  const handleOpenWorldWorkspace = useCallback(() => {
    workspace.setEditorMode("world");
  }, [workspace]);

  const handleJumpToSource = useCallback(
    async (typeId: string) => {
      const loc = await findSourceLocation(typeId);
      if (loc) {
        workspace.setPendingNavigation({ fileId: loc.file_id, line: loc.line });
        workspace.setEditorMode("code");
      }
    },
    [workspace],
  );

  const handleTogglePlay = useCallback(() => {
    if (workspace.editorMode === "play") {
      (window as any).exit_play_mode();
      workspace.setEditorMode("scene");
    } else {
      (window as any).enter_play_mode();
      workspace.setEditorMode("play");
    }
  }, [workspace]);

  // ── Asset command dispatch (C-2 adapter wraps fieldPath as [fieldPath]) ─
  const handleAssetCommit = useCallback(
    async (localId: string, typeId: string, fieldPath: string, value: any) => {
      const command = {
        type: "SetComponentValue",
        local_id: localId,
        type_id: typeId,
        field_path: [fieldPath],
        value,
      };
      await assets.dispatch(command);
    },
    [assets],
  );

  const handleAssetAddComponent = useCallback(
    async (localId: string, typeId: string) => {
      const command = {
        type: "AddComponent",
        local_id: localId,
        type_id: typeId,
        values: {},
      };
      await assets.dispatch(command);
    },
    [assets],
  );

  const handleAssetRemoveComponent = useCallback(
    async (localId: string, typeId: string) => {
      const command = {
        type: "RemoveComponent",
        local_id: localId,
        type_id: typeId,
      };
      await assets.dispatch(command);
    },
    [assets],
  );

  const handleAssetUndo = useCallback(async () => {
    await assets.undo();
  }, [assets]);

  const handleAssetRedo = useCallback(async () => {
    await assets.redo();
  }, [assets]);

  const handleAssetSave = useCallback(async () => {
    await assets.save();
  }, [assets]);

  // ── Canvas drag-drop ──────────────────────────────────────────────────
  const handleCanvasDragOver = useCallback(
    (e: React.DragEvent) => {
      if (
        e.dataTransfer.types.includes("application/x-bevy-asset-id") ||
        e.dataTransfer.types.includes("Files")
      ) {
        e.preventDefault();
        e.dataTransfer.dropEffect = "copy";
        dialogs.setIsDragOverCanvas(true);
      }
    },
    [dialogs],
  );

  const handleCanvasDragLeave = useCallback(
    (e: React.DragEvent) => {
      if (
        e.relatedTarget instanceof Node &&
        e.currentTarget.contains(e.relatedTarget)
      ) {
        return;
      }
      dialogs.setIsDragOverCanvas(false);
    },
    [dialogs],
  );

  const handleCanvasDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      dialogs.setIsDragOverCanvas(false);
      const assetId = e.dataTransfer.getData("application/x-bevy-asset-id");
      if (!assetId) return;
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const translation = {
        x: (e.clientX - rect.left - viewport.pan.x) / viewport.zoom,
        y: (e.clientY - rect.top - viewport.pan.y) / viewport.zoom,
      };
      try {
        await assets.placeInstance(assetId, translation);
      } catch (err) {
        addToast(`Drop failed: ${err}`, "error");
      }
    },
    [assets, viewport, addToast, dialogs],
  );

  // ── Floating panel management ─────────────────────────────────────────
  const handleFloatPanel = useCallback(
    (panelId: PanelId) => {
      const existing = dock.prefs.floats[panelId];
      if (existing) {
        dock.removeFloat(panelId);
        return;
      }
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
      dialogs.setFocusedFloatingPanel(panelId);
    },
    [dock, dialogs],
  );

  const handleDockFloatingPanel = useCallback(
    (panelId: PanelId) => {
      dock.removeFloat(panelId);
      if (focusedFloatingPanel === panelId) {
        dialogs.setFocusedFloatingPanel(null);
      }
    },
    [dock, focusedFloatingPanel, dialogs],
  );

  const handleMovePanel = useCallback(
    (panelId: PanelId, target: DockableRegion) =>
      dock.movePanel(panelId, target),
    [dock],
  );

  // ── Dock resize ───────────────────────────────────────────────────────
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
    (delta: number) =>
      dock.setStatusBarHeight(dock.prefs.statusBar.height - delta),
    [dock],
  );

  const handleResizeRightSplit = useCallback(
    (deltaPx: number) => {
      const pctDelta = (deltaPx / Math.max(dock.prefs.right.width, 200)) * 50;
      dock.setRightTopHeight(dock.prefs.right.topHeight + pctDelta);
    },
    [dock],
  );

  // ── AI panel toggles ──────────────────────────────────────────────────
  const handleToggleAI = useCallback(() => {
    dialogs.setAiPanelOpen((prev: boolean) => !prev);
  }, [dialogs]);

  const handleContextToggle = useCallback(
    (sourceName: string, enabled: boolean) => {
      dialogs.setEnabledSources((prev) => {
        const next = new Set(prev);
        if (enabled) {
          next.add(sourceName);
        } else {
          next.delete(sourceName);
        }
        return next;
      });
    },
    [dialogs],
  );

  const handleToggleValidationCenter = useCallback(() => {
    dialogs.setValidationCenterOpen((prev: boolean) => !prev);
  }, [dialogs]);

  const handleValidationCenterNavigate = useCallback(
    async (issue: import("../services/validation-center").ValidationIssue) => {
      if (issue.affected_entity_id) {
        workspace.setSelectedEntityId(issue.affected_entity_id);
      } else if (issue.affected_asset_id) {
        workspace.setEditorMode("asset-authoring");
        try {
          await (window as any).__openSceneAssetFromSearch?.(
            issue.affected_asset_id,
          );
        } catch (e) {
          console.warn("[App] open asset failed:", e);
        }
      } else if (issue.affected_scene_id) {
        workspace.setEditorMode("scene");
        try {
          await sceneSwitch(issue.affected_scene_id);
        } catch (e) {
          console.warn("[App] scene switch failed:", e);
        }
      } else {
        workspace.setEditorMode("code");
      }
    },
    [workspace],
  );

  const handleToggleTileset = useCallback(() => {
    dialogs.setTilesetPanelOpen((prev: boolean) => !prev);
  }, [dialogs]);

  const handleSelectTileset = useCallback(
    (tileset: TilesetMetadata) => {
      dialogs.setSelectedTilesetId(tileset.id);
    },
    [dialogs],
  );

  const handleToggleAutoLayer = useCallback(() => {
    dialogs.setAutoLayerPanelOpen((prev: boolean) => !prev);
  }, [dialogs]);

  const handleSubmitAI = useCallback(async () => {
    await ai.submit(scene.dispatch);
  }, [ai, scene]);

  const handleApplyProposal = useCallback(
    async (proposalId: string) => {
      dialogs.setApplyingIds((prev) => new Set([...prev, proposalId]));
      try {
        await ai.applyProposal(proposalId, scene.dispatch);
      } finally {
        dialogs.setApplyingIds((prev) => {
          const next = new Set(prev);
          next.delete(proposalId);
          return next;
        });
      }
    },
    [ai, scene, dialogs],
  );

  // ── Viewport ──────────────────────────────────────────────────────────
  const handleResetViewport = useCallback(() => {
    viewport.reset();
  }, [viewport]);

  const handleFitViewport = useCallback(() => {
    viewport.fitToContent();
  }, [viewport]);

  // ── Dock toggles ──────────────────────────────────────────────────────
  const handleToggleLeftDock = useCallback(() => {
    dock.toggleLeft();
  }, [dock]);

  const handleToggleOutlineDock = useCallback(() => {
    dock.toggleOutline();
  }, [dock]);

  const handleTogglePropertiesDock = useCallback(() => {
    dock.toggleProperties();
  }, [dock]);

  const handleToggleBottomDock = useCallback(() => {
    dock.toggleBottom();
  }, [dock]);

  const handleToggleFullscreen = useCallback(() => {
    fullscreen.toggle();
  }, [fullscreen]);

  return {
    handleDeleteEntity,
    handleDeleteEntities,
    handleCreateEntity,
    handleRename,
    handleSetField,
    handleSetFieldOnMultiple,
    handleAddComponent,
    handleRemoveComponent,
    handleAttachLogic,
    handleOpenBoundLogic,
    handleCreateFromRecipe,
    handleInspectRuntimeLogic,
    handleSwitchToLogicMode,
    handleUndo,
    handleRedo,
    handleSave,
    handleSaveConfirm,
    handleSaveWorkspacePresetSubmit,
    handleAbout,
    handleLoad,
    handleTabClick,
    handleNewScene,
    handleDeleteScene,
    handleRenameScene,
    handleSaveAndSwitch,
    handleDiscardAndSwitch,
    handleCancelSwitch,
    pendingSwitchId: pendingSwitch.id,
    pendingSwitchSource: pendingSwitch.source,
    handleOpenAsset,
    handleAssetCreate,
    handleAssetRename,
    handleAssetDuplicate,
    handleAssetDelete,
    handleBackToScene,
    handleAssetSaveAndLeave,
    handleAssetDiscardAndLeave,
    handleAssetCancelBack,
    assetDirty: assets.dirty,
    handleAssetCommit,
    handleAssetAddComponent,
    handleAssetRemoveComponent,
    handleAssetUndo,
    handleAssetRedo,
    handleAssetSave,
    handleOpenLogic,
    handleOpenCode,
    handleOpenWorldWorkspace,
    handleJumpToSource,
    handleTogglePlay,
    handleCanvasDragOver,
    handleCanvasDragLeave,
    handleCanvasDrop,
    handleFloatPanel,
    handleDockFloatingPanel,
    handleMovePanel,
    handleResizeLeft,
    handleResizeRight,
    handleResizeBottom,
    handleResizeStatusBar,
    handleResizeRightSplit,
    handleToggleAI,
    handleContextToggle,
    handleToggleValidationCenter,
    handleValidationCenterNavigate,
    handleToggleTileset,
    handleSelectTileset,
    handleToggleAutoLayer,
    handleSubmitAI,
    handleApplyProposal,
    handleResetViewport,
    handleFitViewport,
    handleToggleLeftDock,
    handleToggleOutlineDock,
    handleTogglePropertiesDock,
    handleToggleBottomDock,
    handleToggleFullscreen,
  };
}
