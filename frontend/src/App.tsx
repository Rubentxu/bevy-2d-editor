import { useEffect, useState, useCallback } from "react";
import "./styles.css";
import { initEngine, isEngineReady } from "./engine-bridge";
import { useSceneState, SceneDocument } from "./hooks/useSceneState";
import { useLogState } from "./hooks/useLogState";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import TopBar from "./components/TopBar";
import HierarchyPanel from "./components/HierarchyPanel";
import InspectorPanel from "./components/InspectorPanel";

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

  useKeyboardShortcuts({
    onUndo: handleUndo,
    onRedo: handleRedo,
    logState,
    selectedEntityId,
    onDeleteEntity: handleDeleteEntity,
  });

  return (
    <div className="app">
      <TopBar
        logState={logState}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onSave={handleSave}
        onLoad={handleLoad}
        error={error || initError}
        onDismissError={() => setError(null)}
      />
      <div className="main">
        <HierarchyPanel
          scene={scene}
          selectedId={selectedEntityId}
          onSelect={setSelectedEntityId}
        />
        <div className="canvas-container">
          {!ready && (
            <div style={{ padding: 16, color: "#888" }}>
              {initError ? `Error: ${initError}` : "Loading WASM..."}
            </div>
          )}
          <canvas id="bevy-canvas" />
        </div>
        <InspectorPanel
          scene={scene}
          selectedId={selectedEntityId}
          onRename={handleRename}
          onSetField={handleSetField}
          onRemoveComponent={handleRemoveComponent}
          onAddComponent={handleAddComponent}
        />
      </div>
    </div>
  );
}