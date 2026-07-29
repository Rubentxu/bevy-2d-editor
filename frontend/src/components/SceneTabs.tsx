import { useState, useRef, useEffect } from "react";
import { SceneInfo } from "../hooks/useScenes";
import PromptDialog from "./PromptDialog";

interface Props {
  scenes: SceneInfo[];
  currentId: string | null;
  onTabClick: (id: string) => void;
  onNewScene: (name: string) => void;
  onDeleteScene: (id: string) => void;
  onRenameScene: (id: string, newName: string) => void;
}

export default function SceneTabs({
  scenes,
  currentId,
  onTabClick,
  onNewScene,
  onDeleteScene,
  onRenameScene,
}: Props) {
  const [contextMenu, setContextMenu] = useState<{
    id: string;
    x: number;
    y: number;
  } | null>(null);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const renameInputRef = useRef<HTMLInputElement>(null);

  // T3.2 — new scene dialog state (replaces window.prompt)
  const [newSceneDialogOpen, setNewSceneDialogOpen] = useState(false);

  // Close context menu on outside click
  useEffect(() => {
    if (contextMenu === null) return;
    const handler = () => setContextMenu(null);
    document.addEventListener("click", handler);
    return () => document.removeEventListener("click", handler);
  }, [contextMenu]);

  // Focus rename input when entering rename mode
  useEffect(() => {
    if (renamingId !== null && renameInputRef.current) {
      renameInputRef.current.focus();
      renameInputRef.current.select();
    }
  }, [renamingId]);

  const commitRename = () => {
    if (renamingId === null) return;
    const trimmed = renameValue.trim();
    if (trimmed) {
      onRenameScene(renamingId, trimmed);
    }
    setRenamingId(null);
  };

  const handleNewScene = () => {
    setNewSceneDialogOpen(true);
  };

  const handleNewSceneSubmit = (name: string) => {
    setNewSceneDialogOpen(false);
    onNewScene(name);
  };

  return (
    <div className="scene-tabs" data-testid="scene-tabs">
      {scenes.map((scene) => (
        <div
          key={scene.id}
          className={`scene-tab${scene.id === currentId ? " active" : ""}`}
          data-testid={`scene-tab-${scene.id}`}
          onClick={() => onTabClick(scene.id)}
          onContextMenu={(e) => {
            e.preventDefault();
            setContextMenu({ id: scene.id, x: e.clientX, y: e.clientY });
          }}
          onDoubleClick={() => {
            setRenamingId(scene.id);
            setRenameValue(scene.name);
          }}
        >
          {renamingId === scene.id ? (
            <input
              ref={renameInputRef}
              className="scene-tab-rename-input"
              value={renameValue}
              onChange={(e) => setRenameValue(e.target.value)}
              onBlur={commitRename}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitRename();
                else if (e.key === "Escape") setRenamingId(null);
              }}
              onClick={(e) => e.stopPropagation()}
            />
          ) : (
            <>
              <span className="scene-tab-name">{scene.name}</span>
              {scene.is_dirty && (
                <span
                  className="scene-tab-dirty-dot"
                  data-testid={`scene-tab-${scene.id}-dot`}
                />
              )}
            </>
          )}
        </div>
      ))}

      {/* "+" new scene button */}
      <button
        className="scene-tab-new-btn"
        data-testid="scene-tab-new-btn"
        onClick={handleNewScene}
        title="Create new scene"
      >
        +
      </button>

      {/* Context menu */}
      {contextMenu !== null && (
        <div
          className="scene-tab-context-menu"
          style={{ top: contextMenu.y, left: contextMenu.x }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            onClick={() => {
              const scene = scenes.find((s) => s.id === contextMenu.id);
              if (scene) {
                setRenamingId(scene.id);
                setRenameValue(scene.name);
              }
              setContextMenu(null);
            }}
          >
            Rename
          </button>
          {scenes.length > 1 && (
            <button
              className="danger"
              onClick={() => {
                onDeleteScene(contextMenu.id);
                setContextMenu(null);
              }}
            >
              Delete
            </button>
          )}
        </div>
      )}

      {/* T3.2 — in-app dialog replacing window.prompt */}
      {newSceneDialogOpen && (
        <PromptDialog
          title="New Scene"
          label="Scene name"
          placeholder="New Scene"
          defaultValue="New Scene"
          onConfirm={handleNewSceneSubmit}
          onCancel={() => setNewSceneDialogOpen(false)}
        />
      )}
    </div>
  );
}
