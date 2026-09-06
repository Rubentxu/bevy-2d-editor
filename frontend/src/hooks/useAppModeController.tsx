/**
 * useAppModeController — caps the per-mode panel composition.
 *
 * The three `useMemo` blocks in AppInner that materialise the outline,
 * properties, and bottom panel bodies (separately so the same JSX
 * instance can be shared between docked and floating portals) carry a
 * lot of conditional rendering keyed on `editorMode`. This hook
 * encapsulates that mode-routing logic: pass it the live state bag and
 * it returns memoised ReactNode props ready to drop into AppShell.
 *
 * The hook is intentionally pure: no DOM effects, no global writes.
 * Per the App.tsx decomposition plan, this is commit 3 — only the
 * panel-routing composition moves out. Handler routing lives in
 * `useSceneHandlers` and the JSX shell lives in `AppShell`.
 */
import { Suspense, lazy, useMemo } from "react";
import type { ReactNode } from "react";
import type { EditorMode } from "../components/MenuBar";
import type { LogicGraphCatalogEntry } from "../services/logic-graphs";
import type { SceneHandlers } from "./useSceneHandlers";
import type { SceneDocument } from "./useSceneState";
import type { Proposal } from "./useAIAssistant";
import { openLogicGraphAsset } from "../services/logic-graphs";
import type { NavigationTarget } from "../types/navigation";
import type {
  SceneAssetCatalogEntry,
  SceneAssetDocument,
  SceneInstance,
} from "../services/scene-assets";
import type {
  AutoLayerPayload,
  LevelLayerPayload,
} from "../services/scene-assets";
import ConsoleTab from "../components/ConsoleTab";
import HierarchyPanel from "../components/HierarchyPanel";
import InspectorPanel from "../components/InspectorPanel";
import ProjectAssetBrowser from "../components/ProjectAssetBrowser";
import AssetAuthoringView from "../components/AssetAuthoringView";
import { AutoLayerPanel } from "../components/AutoLayerPanel";
import { TilesetPanel } from "../components/TilesetPanel";
import ValidationCenter from "../components/ValidationCenter";
import AIAssistantPanel from "../components/AIAssistantPanel";
import type { TaskMode } from "../components/AIAssistantPanel";
import type { PerSourceStats } from "../types/ai";

const LogicGraphEditor = lazy(() => import("../components/LogicGraphEditor"));
const CodeEditor = lazy(() => import("../components/CodeEditor"));

export interface AppModeControllerContext {
  editorMode: EditorMode;
  selectedEntityId: string | null;
  selectedIds: Set<string>;
  selectEntity: (id: string, modifier: "plain" | "range" | "toggle") => void;
  setSelectedEntityId: (id: string | null) => void;
  setPendingNavigation: (target: NavigationTarget | null) => void;
  pendingNavigation: NavigationTarget | null;

  // Scene document + instances
  scene: SceneDocument | null;
  instances: Record<string, SceneInstance>;

  // AI panel state
  prompt: string;
  setPrompt: (s: string) => void;
  aiLoading: boolean;
  proposals: Proposal[];
  aiError: string | null;
  contextStats: PerSourceStats[];
  contextUsedChars: number;
  discardProposal: (id: string) => void;
  taskMode: TaskMode;
  setTaskMode: (m: TaskMode) => void;
  enabledSources: Set<string>;
  applyingIds: Set<string>;

  // Dialog flags
  aiPanelOpen: boolean;
  validationCenterOpen: boolean;
  tilesetPanelOpen: boolean;
  autoLayerPanelOpen: boolean;
  selectedTilesetId: string | null;
  selectedAutoLayerId: string | null;

  // Asset authoring state
  assetDoc: SceneAssetDocument | null;
  assetEntries: SceneAssetCatalogEntry[];
  assetDirty: boolean;
  assetLogState: { can_undo: boolean; can_redo: boolean; size: number };
  activeAssetLogicalPath: string | null;
  removeInstance: (id: string) => Promise<void>;
  replaceInstanceAsset: (
    instanceId: string,
    newAssetId: string,
  ) => Promise<void>;
  placeInstance: (
    assetId: string,
    translation?: { x: number; y: number },
  ) => Promise<void>;
  refresh: () => Promise<void>;
  renameRequestTick: number;

  // Logic graphs
  logicGraphEntries: LogicGraphCatalogEntry[];

  // Handlers (for editor-mode panel wiring)
  handlers: SceneHandlers;
}

/**
 * Three memoised ReactNodes for the outline / properties / bottom panel
 * bodies. Each block uses `useMemo` keyed on the same dependency
 * arrays that the inline version in App.tsx used, so swapping the
 * hook into AppInner does not change referential identity for
 * consumers.
 */
export interface AppModePanels {
  outlineContent: ReactNode;
  propertiesContent: ReactNode;
  bottomContent: ReactNode;
}

/**
 * Materialise the per-mode panel bodies as ReactNodes. The bodies are
 * keyed by editorMode and dialog flags; when none of those change,
 * the same ReactNode instance is reused, which keeps the dock and
 * floating portals from remounting the same content twice.
 */
export function useAppModeController(
  ctx: AppModeControllerContext,
): AppModePanels {
  const {
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
  } = ctx;

  // Auto-layer derived state — kept here so the panel mount route and
  // the regeneration callback see the same fallback path.
  const autoLayers: AutoLayerPayload[] =
    (assetDoc?.layers?.filter(
      (l: LevelLayerPayload) => l.kind === "auto",
    ) as AutoLayerPayload[]) ?? [];
  const selectedAutoLayer: AutoLayerPayload | null = selectedAutoLayerId
    ? (autoLayers.find((l) => l.id === selectedAutoLayerId) ?? null)
    : (autoLayers[0] ?? null);

  const outlineContent = useMemo(
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
                onToggle={handlers.handleToggleAI}
                onPromptChange={setPrompt}
                onSubmit={handlers.handleSubmitAI}
                onApply={handlers.handleApplyProposal}
                onDiscard={discardProposal}
                applyingIds={applyingIds}
                contextStats={contextStats}
                contextUsedChars={contextUsedChars}
                taskMode={taskMode}
                onTaskModeChange={setTaskMode}
                enabledSources={enabledSources}
                onContextToggle={handlers.handleContextToggle}
              />
            )}
            {validationCenterOpen && (
              <ValidationCenter
                onClose={handlers.handleToggleValidationCenter}
                onNavigate={handlers.handleValidationCenterNavigate}
              />
            )}
            {tilesetPanelOpen && (
              <TilesetPanel
                selectedTilesetId={selectedTilesetId}
                onSelectTileset={handlers.handleSelectTileset}
                assetDoc={assetDoc}
                activeAssetLogicalPath={activeAssetLogicalPath}
              />
            )}
            <HierarchyPanel
              scene={scene}
              selectedId={selectedEntityId}
              onSelect={setSelectedEntityId}
              onRename={handlers.handleRename}
              instances={instances}
              onCreateEntity={
                editorMode === "scene" ? handlers.handleCreateEntity : undefined
              }
              renameRequest={renameRequestTick}
              onSelectModifier={
                editorMode === "scene"
                  ? (id, mod) => selectEntity(id, mod)
                  : undefined
              }
              selectedIds={selectedIds}
              onAttachLogic={handlers.handleAttachLogic}
              onOpenBoundLogic={handlers.handleOpenBoundLogic}
              onCreateFromRecipe={handlers.handleCreateFromRecipe}
              onInspectRuntimeLogic={handlers.handleInspectRuntimeLogic}
            />
          </>
        )}
        {editorMode === "asset-authoring" && (
          <ProjectAssetBrowser
            entries={assetEntries}
            logicGraphEntries={logicGraphEntries}
            onCreate={handlers.handleAssetCreate}
            onRename={handlers.handleAssetRename}
            onDuplicate={handlers.handleAssetDuplicate}
            onDelete={handlers.handleAssetDelete}
            onOpen={handlers.handleOpenAsset}
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
    // The dependency arrays match the original App.tsx useMemo deps
    // exactly. We disable the exhaustive-deps rule because the
    // `handlers.handle*` and setter references are stable across renders.
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
      handlers.handleToggleAI,
      setPrompt,
      handlers.handleSubmitAI,
      handlers.handleApplyProposal,
      discardProposal,
      applyingIds,
      contextStats,
      contextUsedChars,
      handlers.handleToggleValidationCenter,
      handlers.handleSelectTileset,
      setSelectedEntityId,
      handlers.handleRename,
      handlers.handleCreateEntity,
      renameRequestTick,
      selectEntity,
      handlers.handleAssetCreate,
      handlers.handleAssetRename,
      handlers.handleAssetDuplicate,
      handlers.handleAssetDelete,
      handlers.handleOpenAsset,
      placeInstance,
    ],
  );

  const propertiesContent = useMemo(
    () => (
      <div className="dock-content dock-content-properties">
        {editorMode === "scene" && (
          <InspectorPanel
            scene={scene}
            selectedId={selectedEntityId}
            selectedIds={selectedIds}
            onRename={handlers.handleRename}
            onSetField={handlers.handleSetField}
            onSetFieldOnMultiple={handlers.handleSetFieldOnMultiple}
            onRemoveComponent={handlers.handleRemoveComponent}
            onAddComponent={handlers.handleAddComponent}
            instances={instances}
            onRemoveInstance={removeInstance}
            onReplaceInstanceAsset={replaceInstanceAsset}
            assetEntries={assetEntries}
            onJumpToSource={handlers.handleJumpToSource}
            onAttachLogic={handlers.handleAttachLogic}
            onOpenBoundLogic={handlers.handleOpenBoundLogic}
            onCreateFromRecipe={handlers.handleCreateFromRecipe}
            onInspectRuntimeLogic={handlers.handleInspectRuntimeLogic}
            onSwitchToLogicMode={handlers.handleSwitchToLogicMode}
          />
        )}
        {editorMode === "asset-authoring" && assetDoc && (
          <AssetAuthoringView
            document={assetDoc}
            activeEntityId={null}
            onSelectEntity={() => {}}
            onCommit={handlers.handleAssetCommit}
            onAddComponent={handlers.handleAssetAddComponent}
            onRemoveComponent={handlers.handleAssetRemoveComponent}
            onUndo={handlers.handleAssetUndo}
            onRedo={handlers.handleAssetRedo}
            onSave={handlers.handleAssetSave}
            onBackToScene={handlers.handleBackToScene}
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
      handlers.handleRename,
      handlers.handleSetField,
      handlers.handleSetFieldOnMultiple,
      handlers.handleRemoveComponent,
      handlers.handleAddComponent,
      removeInstance,
      replaceInstanceAsset,
      handlers.handleJumpToSource,
      handlers.handleAssetCommit,
      handlers.handleAssetAddComponent,
      handlers.handleAssetRemoveComponent,
      handlers.handleAssetUndo,
      handlers.handleAssetRedo,
      handlers.handleAssetSave,
      handlers.handleBackToScene,
      refresh,
    ],
  );

  const bottomContent = useMemo(
    () => <ConsoleTab />,
    // ConsoleTab has no props — safe to omit from deps
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );

  return { outlineContent, propertiesContent, bottomContent };
}
