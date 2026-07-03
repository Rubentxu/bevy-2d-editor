import { useEffect, useState, useCallback } from "react";
import "./styles.css";
import { initEngine, isEngineReady } from "./engine-bridge";
import { useSceneState, SceneDocument } from "./hooks/useSceneState";
import { useLogState } from "./hooks/useLogState";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useAIAssistant } from "./hooks/useAIAssistant";
import TopBar from "./components/TopBar";
import HierarchyPanel from "./components/HierarchyPanel";
import InspectorPanel from "./components/InspectorPanel";
import AIAssistantPanel from "./components/AIAssistantPanel";
import ExportRustModal from "./components/ExportRustModal";
import ValidationCenter from "./components/ValidationCenter";
import SceneTabs from "./components/SceneTabs";
import UnsavedChangesDialog from "./components/UnsavedChangesDialog";
import ProjectAssetBrowser from "./components/ProjectAssetBrowser";
import AssetAuthoringView from "./components/AssetAuthoringView";
import AssetUnsavedChangesDialog from "./components/AssetUnsavedChangesDialog";
import { TilesetPanel } from "./components/TilesetPanel";
import { AutoLayerPanel } from "./components/AutoLayerPanel";
import LogicGraphEditor from "./components/LogicGraphEditor";
import CodeEditor, { type NavigationTarget } from "./components/CodeEditor";
import { useScenes } from "./hooks/useScenes";
import { useSceneAssets } from "./hooks/useSceneAssets";
import { sceneCreate, sceneSwitch, sceneSwitchCommit, sceneDelete, sceneRename } from "./services/scenes";
import { type TilesetMetadata } from "./services/tilesets";
import { type AutoLayerPayload, type LevelLayerPayload } from "./services/scene-assets";
import { findSourceLocation } from "./services/code-files";

type EditorMode = "scene" | "asset-authoring" | "logic" | "code" | "play";

export default function App() {
  const [ready, setReady] = useState(false);
  const [initError, setInitError] = useState<string | null>(null);
  const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const initGuard = (() => {
    let guard = false;
    return { current: guard, set: (v: boolean) => (guard = v), get: () => guard };
  })();

  const { scene, refresh, dispatch } = useSceneState();
  const logState = useLogState();
  const [aiPanelOpen, setAiPanelOpen] = useState(false);
  const [validationCenterOpen, setValidationCenterOpen] = useState(false);
  const [tilesetPanelOpen, setTilesetPanelOpen] = useState(false);
  const [selectedTilesetId, setSelectedTilesetId] = useState<string | null>(null);
  const [autoLayerPanelOpen, setAutoLayerPanelOpen] = useState(false);
  const [selectedAutoLayerId, setSelectedAutoLayerId] = useState<string | null>(null);
  const [exportRustOpen, setExportRustOpen] = useState(false);
  const [applyingIds, setApplyingIds] = useState<Set<string>>(new Set());
  const { scenes, currentId, refresh: refreshScenes } = useScenes();
  const [pendingSwitchId, setPendingSwitchId] = useState<string | null>(null);
  const [pendingSwitchSource, setPendingSwitchSource] = useState<string | null>(null);

  // ── Asset Authoring Mode ─────────────────────────────────────────────────
  const [editorMode, setEditorMode] = useState<EditorMode>("scene");
  const [activeAssetLogicalPath, setActiveAssetLogicalPath] = useState<string | null>(null);
  const [pendingBackToScene, setPendingBackToScene] = useState(false);

  // ── Cross-mode Navigation (rust-source-integration) ─────────────────────
  // Holds the target file + line for scene → code jump-to-source navigation.
  const [pendingNavigation, setPendingNavigation] = useState<NavigationTarget | null>(null);

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
      (l: LevelLayerPayload) => l.kind === "auto"
    ) as AutoLayerPayload[]) ?? [];

  const selectedAutoLayer: AutoLayerPayload | null =
    selectedAutoLayerId
      ? autoLayers.find((l) => l.id === selectedAutoLayerId) ?? null
      : autoLayers[0] ?? null;

  // ── AI Assistant ─────────────────────────────────────────────────────────
  const {
    prompt,
    setPrompt,
    loading: aiLoading,
    proposals,
    error: aiError,
    submit,
    applyProposal,
    discardProposal,
  } = useAIAssistant({
    onApplied: refresh,
  });

  const handleToggleAI = useCallback(() => {
    setAiPanelOpen((prev) => !prev);
  }, []);

  const handleToggleValidationCenter = useCallback(() => {
    setValidationCenterOpen((prev) => !prev);
  }, []);

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
      .catch((e) => setInitError(String(e)));
  }, []);

  const handleUndo = async () => {
    try {
      const snap = await (window as any).undo();
      const parsed = JSON.parse(snap);
      await refresh();
      setSelectedEntityId(null);
    } catch (e) {
      setError(`Undo failed: ${e}`);
    }
  };

  const handleRedo = async () => {
    try {
      const snap = await (window as any).redo();
      await refresh();
      setSelectedEntityId(null);
    } catch (e) {
      setError(`Redo failed: ${e}`);
    }
  };

  const handleSave = async () => {
    const name = window.prompt("Scene name:", "level_01");
    if (!name) return;
    try {
      const path = await (window as any).save_scene(name);
      setError(null);
      // Briefly show success — use topbar status
      console.log(`Saved to ${path}`);
    } catch (e) {
      setError(`Save failed: ${e}`);
    }
  };

  const handleLoad = async () => {
    try {
      await (window as any).load_project();
      await refresh();
      setSelectedEntityId(null);
    } catch (e) {
      setError(`Load project failed: ${e}`);
    }
  };

  const handleRename = async (entityId: string, newName: string) => {
    const result = await dispatch({
      command: { type: "RenameEntity", entity_id: entityId, new_name: newName },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    if (result.error) setError(`Rename failed: ${result.error}`);
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
    if (result.error) setError(`Set field failed: ${result.error}`);
  };

  const handleRemoveComponent = async (entityId: string, typeId: string) => {
    const result = await dispatch({
      command: { type: "RemoveComponent", entity_id: entityId, type_id: typeId },
      metadata: { authorship: "user", timestamp: Date.now() },
    });
    if (result.error) setError(`Remove component failed: ${result.error}`);
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
    if (result.error) setError(`Add component failed: ${result.error}`);
  };

  const handleDeleteEntity = useCallback(async (id: string) => {
    if (!id) return;
    await dispatch({
      command: { type: "DeleteEntity", id },
      metadata: { authorship: "keyboard", timestamp: Date.now() },
    });
    setSelectedEntityId(null);
  }, [dispatch]);

  // ── Multi-scene handlers ─────────────────────────────────────────────────

  const handleTabClick = useCallback(async (id: string) => {
    if (id === currentId) return;
    const result = await sceneSwitch(id);
    if (result.dirty_prompt_required) {
      setPendingSwitchId(id);
      setPendingSwitchSource(result.source_name);
    }
    // If no dirty prompt, the switch already happened server-side
    await refresh();
  }, [currentId, refresh]);

  const handleNewScene = useCallback(async (name: string) => {
    await sceneCreate(name);
    await refreshScenes();
  }, [refreshScenes]);

  const handleDeleteScene = useCallback(async (id: string) => {
    await sceneDelete(id);
    await refreshScenes();
  }, [refreshScenes]);

  const handleRenameScene = useCallback(async (id: string, newName: string) => {
    await sceneRename(id, newName);
    await refreshScenes();
  }, [refreshScenes]);

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
    [assetEntries, openAsset]
  );

  const handleAssetCreate = useCallback(
    async (name: string, role: string) => {
      await createAsset(name, role);
    },
    [createAsset]
  );

  const handleAssetRename = useCallback(
    async (assetId: string, newPath: string) => {
      await renameAsset(assetId, newPath);
    },
    [renameAsset]
  );

  const handleAssetDuplicate = useCallback(
    async (assetId: string) => {
      await duplicateAsset(assetId);
    },
    [duplicateAsset]
  );

  const handleAssetDelete = useCallback(
    async (assetId: string) => {
      await deleteAssetFn(assetId);
    },
    [deleteAssetFn]
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
    [dispatchAssetCommand]
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
    [dispatchAssetCommand]
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
    [dispatchAssetCommand]
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

  useKeyboardShortcuts({
    onUndo: editorMode === "scene" ? handleUndo : handleAssetUndo,
    onRedo: editorMode === "scene" ? handleRedo : handleAssetRedo,
    logState: editorMode === "scene" ? logState : assetLogState,
    selectedEntityId,
    onDeleteEntity: handleDeleteEntity,
  });

  return (
    <div className="app">
      <TopBar
        editorMode={editorMode}
        onOpenAssets={() => {}}
        onBackToScene={editorMode === "asset-authoring" ? handleBackToScene : undefined}
        onOpenLogic={editorMode === "scene" ? handleOpenLogic : undefined}
        onOpenCode={editorMode === "scene" ? handleOpenCode : undefined}
        logState={editorMode === "scene" ? logState : assetLogState}
        onUndo={editorMode === "scene" ? handleUndo : handleAssetUndo}
        onRedo={editorMode === "scene" ? handleRedo : handleAssetRedo}
        onSave={editorMode === "scene" ? handleSave : handleAssetSave}
        onLoad={handleLoad}
        onExportRust={() => setExportRustOpen(true)}
        onToggleAI={handleToggleAI}
        aiPanelOpen={aiPanelOpen}
        onToggleValidationCenter={handleToggleValidationCenter}
        validationCenterOpen={validationCenterOpen}
        onToggleTileset={handleToggleTileset}
        tilesetPanelOpen={tilesetPanelOpen}
        onToggleAutoLayer={handleToggleAutoLayer}
        autoLayerPanelOpen={autoLayerPanelOpen}
        error={error || initError}
        onDismissError={() => setError(null)}
      />
      {editorMode === "scene" && (
        <SceneTabs
          scenes={scenes}
          currentId={currentId}
          onTabClick={handleTabClick}
          onNewScene={handleNewScene}
          onDeleteScene={handleDeleteScene}
          onRenameScene={handleRenameScene}
        />
      )}
      {/* Canvas always mounted — AssetAuthoringView overlays .main content (C-4) */}
      <div className="canvas-container">
        {!ready && (
          <div style={{ padding: 16, color: "#888" }}>
            {initError ? `Error: ${initError}` : "Loading WASM..."}
          </div>
        )}
        <canvas id="bevy-canvas" />
      </div>
      <div className="main">
        {editorMode === "logic" ? (
          <>
            <LogicGraphEditor editorMode={editorMode} />
          </>
        ) : editorMode === "code" ? (
          <>
            <CodeEditor
            navigationTarget={pendingNavigation}
            onEditorReady={() => setPendingNavigation(null)}
          />
          </>
        ) : editorMode === "scene" ? (
          <>
            {aiPanelOpen && (
              <AIAssistantPanel
                aiState={{ prompt, loading: aiLoading, proposals, error: aiError }}
                onToggle={handleToggleAI}
                onPromptChange={setPrompt}
                onSubmit={handleSubmitAI}
                onApply={handleApplyProposal}
                onDiscard={discardProposal}
                applyingIds={applyingIds}
              />
            )}
            {validationCenterOpen && (
              <ValidationCenter onClose={handleToggleValidationCenter} />
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
            />
            <InspectorPanel
              scene={scene}
              selectedId={selectedEntityId}
              onRename={handleRename}
              onSetField={handleSetField}
              onRemoveComponent={handleRemoveComponent}
              onAddComponent={handleAddComponent}
              instances={instances}
              onRemoveInstance={removeInstance}
              onReplaceInstanceAsset={replaceInstanceAsset}
              assetEntries={assetEntries}
              onJumpToSource={handleJumpToSource}
            />
          </>
        ) : (
          /* Asset Authoring Mode — overlays canvas via .main (C-4) */
          <>
            <ProjectAssetBrowser
              entries={assetEntries}
              onCreate={handleAssetCreate}
              onRename={handleAssetRename}
              onDuplicate={handleAssetDuplicate}
              onDelete={handleAssetDelete}
              onOpen={handleOpenAsset}
              onPlaceInstance={placeInstance}
            />
            {autoLayerPanelOpen && (
              selectedAutoLayer ? (
              <AutoLayerPanel
                layer={selectedAutoLayer}
                assetRef={activeAssetLogicalPath ?? ""}
                onRegenerate={refresh}
              />
            ) : (
              <div className="tileset-panel">
                <h3>Auto Layer</h3>
                <p style={{ fontSize: 12, color: '#666' }}>No auto layers in this asset. Open a level scene asset to edit auto layers.</p>
              </div>
            )
            )}
            {assetDoc && (
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
          </>
        )}
      </div>
      {exportRustOpen && <ExportRustModal onClose={() => setExportRustOpen(false)} />}
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
    </div>
  );
}
