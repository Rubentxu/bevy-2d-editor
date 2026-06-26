# Design: UI Panels

> Change: `ui-panels` · Phase: sddk-design · Path: A-lite
> Model: MiniMax-M3 (orchestrator)

---

## §1. Module Layout

```
frontend/src/
├── App.tsx                              (modified) — 3-column layout, top-level state
├── main.tsx                             (unchanged)
├── engine-bridge.ts                     (modified) — get_scene_snapshot, useSceneState helpers
├── hooks/
│   ├── useSceneState.ts                 (new) — scene + dispatch + refresh
│   └── useLogState.ts                   (new) — can_undo, can_redo polling
├── components/
│   ├── TopBar.tsx                       (new) — status + undo/redo + save/load
│   ├── HierarchyPanel.tsx               (new) — entity tree + selection
│   ├── InspectorPanel.tsx               (new) — components + field editors
│   └── ComponentEditor.tsx              (new) — per-FieldType widget
```

## §2. Rust: get_scene_snapshot

```rust
/// Get the current SceneDocument as JSON. Returns null if no scene loaded.
/// Read-only — does not mutate state.
#[wasm_bindgen]
pub fn get_scene_snapshot() -> JsValue {
    SCENE_DOC.with(|s| {
        match s.borrow().as_ref() {
            Some(doc) => serde_wasm_bindgen::to_value(doc).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    })
}
```

Simple, no new dependencies. Thread-local read.

## §3. React State Management

Single source of truth at `App.tsx` level:

```tsx
function App() {
  const [scene, setScene] = useState<SceneDocument | null>(null);
  const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);
  const [logState, setLogState] = useState({ size: 0, can_undo: false, can_redo: false });
  const [error, setError] = useState<string | null>(null);
  
  // Initial load + polling
  useEffect(() => {
    refresh();
    const interval = setInterval(() => {
      getLogState().then(setLogState);
    }, 500);
    return () => clearInterval(interval);
  }, []);
  
  const refresh = async () => {
    const snap = await getSceneSnapshot();
    setScene(snap);
  };
  
  const dispatch = async (envelope: object) => {
    const result = await dispatchCommand(envelope);
    const parsed = JSON.parse(result);
    if (parsed.snapshot) setScene(parsed.snapshot);
    if (parsed.error) setError(parsed.error);
    getLogState().then(setLogState);
  };
  
  // ...
}
```

## §4. Hooks

### useSceneState

```typescript
// frontend/src/hooks/useSceneState.ts
export function useSceneState() {
  const [scene, setScene] = useState<SceneDocument | null>(null);
  
  const refresh = useCallback(async () => {
    const snap = await getSceneSnapshot();
    setScene(snap);
  }, []);
  
  const dispatch = useCallback(async (envelope: object) => {
    try {
      const result = await dispatchCommand(envelope);
      const parsed = JSON.parse(result);
      if (parsed.snapshot) setScene(parsed.snapshot);
      return parsed;
    } catch (e) {
      return { error: String(e) };
    }
  }, []);
  
  useEffect(() => { refresh(); }, [refresh]);
  
  return { scene, refresh, dispatch };
}
```

### useLogState

```typescript
export function useLogState() {
  const [state, setState] = useState({ size: 0, can_undo: false, can_redo: false });
  
  useEffect(() => {
    const update = async () => {
      const s = await getLogState();
      setState(s);
    };
    update();
    const interval = setInterval(update, 500);
    return () => clearInterval(interval);
  }, []);
  
  return state;
}
```

## §5. Component Design

### TopBar

```tsx
function TopBar({ logState, onUndo, onRedo, onSave, onLoad, error }) {
  return (
    <div className="topbar">
      <span className="status">Bevy 2D Editor</span>
      <button onClick={onUndo} disabled={!logState.can_undo}>Undo</button>
      <button onClick={onRedo} disabled={!logState.can_redo}>Redo</button>
      <button onClick={onSave}>Save</button>
      <button onClick={onLoad}>Load Project</button>
      {error && <span className="error">{error}</span>}
    </div>
  );
}
```

### HierarchyPanel

```tsx
function HierarchyPanel({ scene, selectedId, onSelect }) {
  if (!scene) return <div className="panel">No scene</div>;
  if (scene.entities.length === 0) return <div className="panel">No entities</div>;
  
  return (
    <div className="panel">
      {scene.entities.map(e => (
        <div
          key={e.id}
          className={selectedId === e.id ? 'entity selected' : 'entity'}
          onClick={() => onSelect(e.id)}
          style={{ paddingLeft: indentLevel(e, scene.entities) * 16 + 8 }}
        >
          {e.name} <span className="id">{e.id}</span>
        </div>
      ))}
    </div>
  );
}
```

`indentLevel` walks parent chain to compute depth.

### InspectorPanel

```tsx
function InspectorPanel({ scene, selectedId, onDispatch }) {
  if (!scene || !selectedId) return <div>Select an entity</div>;
  const entity = scene.entities.find(e => e.id === selectedId);
  if (!entity) return <div>Entity not found</div>;
  
  return (
    <div className="panel">
      <input
        type="text"
        defaultValue={entity.name}
        onBlur={(e) => onDispatch({
          command: { type: 'RenameEntity', entity_id: selectedId, new_name: e.target.value },
          metadata: { authorship: 'user', timestamp: Date.now() }
        })}
      />
      {entity.components.map(c => (
        <ComponentCard
          key={c.type_id}
          component={c}
          onDispatch={onDispatch}
          onRemove={() => onDispatch({
            command: { type: 'RemoveComponent', entity_id: selectedId, type_id: c.type_id },
            metadata: { authorship: 'user', timestamp: Date.now() }
          })}
        />
      ))}
      <AddComponentButton entityId={selectedId} onDispatch={onDispatch} />
    </div>
  );
}
```

### ComponentCard + ComponentEditor

```tsx
function ComponentCard({ component, onDispatch, entityId, onRemove }) {
  return (
    <div className="component-card">
      <header>
        <span>{component.type_id}</span>
        <button onClick={onRemove}>×</button>
      </header>
      {Object.entries(component.values).map(([field, value]) => (
        <ComponentEditor
          key={field}
          fieldPath={field}
          value={value}
          typeId={component.type_id}
          entityId={entityId}
          onDispatch={onDispatch}
        />
      ))}
    </div>
  );
}

function ComponentEditor({ fieldPath, value, typeId, entityId, onDispatch }) {
  // Choose widget based on field type inferred from value shape
  if (typeof value === 'object' && value !== null && 'x' in value && 'y' in value) {
    // Vec2-like: 2 inputs
    return <Vec2Editor {...} />;
  }
  if (typeof value === 'object' && value !== null && 'r' in value && 'g' in value && 'b' in value && 'a' in value) {
    // Color-like: 4 inputs
    return <ColorEditor {...} />;
  }
  if (typeof value === 'string' && ANCHOR_VALUES.includes(value)) {
    // Anchor: dropdown
    return <AnchorEditor {...} />;
  }
  if (typeof value === 'number') {
    return <NumberEditor {...} />;
  }
  if (typeof value === 'boolean') {
    return <CheckboxEditor {...} />;
  }
  // Default: text
  return <TextEditor {...} />;
}
```

### AddComponentButton

```tsx
function AddComponentButton({ entityId, onDispatch }) {
  const [open, setOpen] = useState(false);
  const schemas = useSchemas(); // returns combined_registry type_ids
  
  return (
    <div>
      <button onClick={() => setOpen(!open)}>+ Add Component</button>
      {open && (
        <div className="dropdown">
          {schemas.map(s => (
            <div key={s} onClick={() => {
              onDispatch({
                command: {
                  type: 'AddComponent',
                  entity_id: entityId,
                  type_id: s,
                  values: {}  // Use defaults from schema
                },
                metadata: { authorship: 'user', timestamp: Date.now() }
              });
              setOpen(false);
            }}>{s}</div>
          ))}
        </div>
      )}
    </div>
  );
}
```

## §6. CSS Layout

```css
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: system-ui, sans-serif;
  background: #1a1a2e;
  color: #eee;
}

.topbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-bottom: 1px solid #333;
  background: #16213e;
  height: 48px;
}

.main {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.canvas-container {
  flex: 1;
  position: relative;
  background: #0a0a14;
}

.panel {
  width: 280px;
  padding: 12px;
  border-right: 1px solid #333;
  background: #16213e;
  overflow-y: auto;
}

.panel.inspector {
  border-right: none;
  border-left: 1px solid #333;
}

.entity {
  padding: 4px 8px;
  cursor: pointer;
  border-radius: 4px;
}

.entity.selected {
  background: #0f3460;
}

.component-card {
  background: #0f3460;
  border-radius: 4px;
  padding: 8px;
  margin-top: 8px;
}

button {
  background: #1a1a2e;
  color: #eee;
  border: 1px solid #444;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

input[type="text"], input[type="number"] {
  background: #1a1a2e;
  color: #eee;
  border: 1px solid #444;
  padding: 4px;
  border-radius: 4px;
  width: 100%;
}
```

## §7. Backward Compatibility

- All 23 existing Playwright tests pass unchanged (they test wasm directly, not UI)
- `get_scene_snapshot()` is additive
- UI panels replace spike UI but don't break wasm API
- Existing JS-exposed functions all still work

## §8. Test Strategy

### Rust unit test for get_scene_snapshot
- `test_get_scene_snapshot_with_scene`: scene loaded, returns Some
- `test_get_scene_snapshot_no_scene`: no scene, returns None
- `test_get_scene_snapshot_does_not_mutate`: read doesn't change op log size

### Playwright E2E
- `UI hierarchy shows entities and selects one` — load scene with 3 entities, verify 3 in hierarchy, click one, verify selection
- `UI inspector shows components and edits Vec2 field` — select entity, edit Transform2D.translation.x, verify change
- `UI undo button works` — dispatch via Add button, click Undo, verify state reverted

## §9. Performance Notes

- `useLogState` polls every 500ms (acceptable for UI responsiveness)
- `dispatch` is sync (no async race)
- `get_scene_snapshot` is sync, copies doc → JSON
- No virtualization in Hierarchy (linear scan O(n)); n < 100 for MVP

## §10. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| UI refresh too frequent | Polling 500ms for log state, immediate refresh on dispatch |
| Component value parsing | Type inference via value shape (Vec2 has x/y, Color has r/g/b/a) |
| Schema dropdown empty | Fallback: read combined_registry via window.list_schemas |
| Selected entity deleted by undo | Inspector checks entity exists; shows "not found" |
| Long names overflow | CSS ellipsis |
| Async save/load race | Disable buttons during async |
| Infinite render | useEffect deps carefully |

## §11. Open Questions

1. **Live preview while editing Vec2** — On every keystroke or on blur? MVP: on blur. Future: debounce.
2. **Json textarea fallback** — For complex values not fitting simple widgets. MVP: render as JSON string. Future: tree editor.
3. **Reparent UI** — Out of scope for MVP. Defer.
4. **Multi-select** — Out of scope. Defer.
5. **Template instantiation from UI** — Out of scope. Defer.