# Design: v0.82 P2 — Floating Panels + Inspector Multi-Select

> **Cycle**: `v0.82-p2-floating-multi-select`
> **Status**: Designed (Phase 0)
> **Author**: orchestrator (2026-07-23)

This document captures the *how*. For *what* and *why* see
`proposal.md` and `spec/spec.md`.

## D1. Architecture map

### Logical pieces (frontend)

```
                      ┌──────────────────────────────────────┐
                      │ App.tsx                              │
                      │  state:                              │
                      │   - selectedIds: Set<StableId>       │
                      │   - lastClickedId                    │
                      │   - floatingPanels: Map<PanelId, …>  │
                      └────┬─────────────┬───────────────┬───┘
                           │             │               │
                ┌──────────▼─────┐  ┌────▼────────┐  ┌──▼───────────────┐
                │ HierarchyPanel │  │ Inspector…  │  │ DockLayout       │
                │ Shift/Ctrl/A   │  │ mixed-value │  │ + FloatingPanel  │
                └────────────────┘  └─────────────┘  │   (Portal)       │
                                                     └──┬───────────────┘
                                                        │
                                                  ┌─────▼──────────────┐
                                                  │ useDockPrefs       │
                                                  │  schemaVersion 3   │
                                                  └────────────────────┘
```

### Logical pieces (backend)

```
                    crates/editor-core/src/command.rs
                                │
            Command::SetComponentFieldOnMultiple { … }
                                │
                                ▼
                    crates/editor-core/src/processor.rs
                                │
                                │  builds Batch (one entry per entity)
                                ▼
                       existing apply_batch (no new logic)
```

## D2. DockPrefs schema v3

### Current shape (v2)

```ts
type DockPrefsV2 = {
  schemaVersion: 2;
  panelRegions: {
    hierarchy: "left" | "right" | "bottom";
    inspector: "left" | "right" | "bottom";
    assets:    "left" | "right" | "bottom";
    history:   "left" | "right" | "bottom";
  };
  panelWidths?: Partial<Record<PanelId, number>>;
  // …possibly other keys added in v0.82 P1
};
```

### New shape (v3)

```ts
type FloatingPanelState = {
  x: number;
  y: number;
  width: number;
  height: number;
  last_floated_at: number;   // epoch millis
};

type DockPrefsV3 = DockPrefsV2 & {
  schemaVersion: 3;
  floats: Partial<Record<PanelId, FloatingPanelState>>;
};
```

Migration `v2 → v3`:

```ts
function migratePrefs(raw: DockPrefsV1 | DockPrefsV2 | DockPrefsV3): DockPrefsV3 {
  if (raw.schemaVersion === 3) return raw;
  if (raw.schemaVersion === 2) {
    return { ...raw, schemaVersion: 3, floats: {} };
  }
  // …existing v1→v2 path; same as before
}
```

The loader (`useDockPrefs.ts → load()`) reads, migrates, and saves
back as v3 (only on first load).

### PanelId enum

```ts
type PanelId = "hierarchy" | "inspector" | "assets" | "history";
```

These are the panels that can be docked *and* floated. Each can be
moved between regions (v0.82 P1) and toggled to float.

## D3. FloatingPanel component

### Component shape

```tsx
interface FloatingPanelProps {
  panelId: PanelId;
  title: string;
  initialRect: { x: number; y: number; width: number; height: number };
  onDock: () => void;
  onFocus: () => void;
  focused: boolean;
  children: React.ReactNode;
}

export function FloatingPanel(props: FloatingPanelProps): React.ReactPortal { … }
```

### Render strategy

```tsx
return createPortal(
  <div className={`floating-panel ${focused ? "focused" : ""}`}
       style={{ left, top, width, height }}>
    <div className="floating-panel__header"
         onPointerDown={startDrag}
         onClick={onFocus}
         ref={headerRef}>
      <span className="floating-panel__title">{title}</span>
      <button onClick={onDock} aria-label="Dock panel">×</button>
    </div>
    <div className="floating-panel__content">{children}</div>
  </div>,
  document.body,
);
```

### Drag implementation

```ts
function startDrag(e: React.PointerEvent) {
  setDragging(true);
  const startX = e.clientX, startY = e.clientY;
  const origX = leftRef.current, origY = topRef.current;
  const move = (ev: PointerEvent) => {
    const dx = ev.clientX - startX, dy = ev.clientY - startY;
    const nextX = clamp(origX + dx, 0, window.innerWidth - 100);
    const nextY = clamp(origY + dy, 0, window.innerHeight - 40);
    pendingRef.current = { x: nextX, y: nextY };
    if (!rafScheduled) {
      rafScheduled = true;
      requestAnimationFrame(() => {
        setLeft(pendingRef.current!.x);
        setTop(pendingRef.current!.y);
        rafScheduled = false;
      });
    }
  };
  const up = () => {
    setDragging(false);
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", up);
    saveFloatsToPrefs({ x: left, y: top });
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", up);
}
```

### Visibility model in App.tsx

```tsx
const [floatingPanelIds, setFloatingPanelIds] = useState<Set<PanelId>>(new Set());
const [focusedFloatingPanel, setFocusedFloatingPanel] = useState<PanelId | null>(null);
const [panelFloatRects, setPanelFloatRects] = useState<Map<PanelId, FloatingPanelState>>(new Map());

// Render docked panels (in CSS Grid) EXCEPT those whose ids are in floatingPanelIds.
// Render floating panels via <FloatingPanel> for each id in floatingPanelIds.
```

When a panel id is in `floatingPanelIds`, the dock container skips
it (renders nothing for that slot). The FloatingPanel takes over and
renders the *same content component* (`<HierarchyPanel />`,
`<InspectorPanel />`) inside its portal.

### Z-index scale (CSS)

```css
:root {
  --z-floating-panel: 100;
  --z-floating-panel-focused: 101;
  --z-modal: 1000;
}

.floating-panel {
  position: fixed;
  z-index: var(--z-floating-panel);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
}
.floating-panel.focused { z-index: var(--z-floating-panel-focused); }
.floating-panel__header {
  cursor: grab;
  user-select: none;
  padding: 6px 10px;
  border-bottom: 1px solid var(--color-border);
}
.floating-panel__header:active { cursor: grabbing; }
```

## D4. Selection shape & Inspector multi-view

### App.tsx state

```tsx
const [selectedIds, setSelectedIds] = useState<Set<StableId>>(new Set());
const [lastClickedId, setLastClickedId] = useState<StableId | null>(null);

const primaryId = selectedIds.size === 1 ? [...selectedIds][0] : null;
const isMultiSelect = selectedIds.size > 1;

const onSelect = useCallback((id: StableId, modifier: ClickModifier) => {
  setSelectedIds(prev => {
    const next = new Set(prev);
    if (modifier === "range") {
      // include every id between lastClickedId and id in scene.entities order
      const scene = useSceneStore.getState().scene;
      const ids = scene.entities.map(e => e.id);
      const fromIdx = ids.indexOf(lastClickedId!);
      const toIdx = ids.indexOf(id);
      const [lo, hi] = fromIdx < toIdx ? [fromIdx, toIdx] : [toIdx, fromIdx];
      for (let i = lo; i <= hi; i++) next.add(ids[i]);
    } else if (modifier === "toggle") {
      if (next.has(id)) next.delete(id); else next.add(id);
    } else {
      next.clear();
      next.add(id);
    }
    return next;
  });
  setLastClickedId(id);
}, [lastClickedId]);

const clearSelection = useCallback(() => {
  setSelectedIds(new Set());
  setLastClickedId(null);
}, []);
```

Global keyboard handler:

```tsx
useEffect(() => {
  const onKey = (e: KeyboardEvent) => {
    if (isTextInput(e.target)) return;
    if (e.key === "Escape") { clearSelection(); e.preventDefault(); return; }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a") {
      const ids = useSceneStore.getState().scene.entities.map(x => x.id);
      setSelectedIds(new Set(ids));
      e.preventDefault();
    }
  };
  window.addEventListener("keydown", onKey);
  return () => window.removeEventListener("keydown", onKey);
}, [clearSelection]);
```

### Inspector multi-select view

```tsx
function InspectorBody({ selectedIds, scene, dispatch, schemas }: Props) {
  if (selectedIds.size === 0) return <EmptyState />;
  if (selectedIds.size === 1) return <SingleInspector id={[...selectedIds][0]} … />;

  const entities = scene.entities.filter(e => selectedIds.has(e.id));
  const componentTypes = uniqueBy(entities.flatMap(e => e.components.map(c => c.type_id)));
  return (
    <section className="inspector-multi">
      <header>{selectedIds.size} entities selected · {componentTypes.length} components in common</header>
      {componentTypes.map(typeId => (
        <ComponentSection
          key={typeId}
          typeId={typeId}
          entities={entities}
          dispatch={dispatch}
        />
      ))}
    </section>
  );
}
```

### ComponentSection — value aggregation

```tsx
function ComponentSection({ typeId, entities, dispatch }: Props) {
  const owningEntities = entities.filter(e => e.components.some(c => c.type_id === typeId));
  const schema = schemas[typeId];
  const values = collectValues(typeId, owningEntities);
  // values: { [fieldPath]: { homogeneous: boolean, value?, divergentCount: number } }

  return (
    <section>
      <h4>{typeId}
        <span className="badge">{owningEntities.length}/{entities.length} entities have this component</span>
      </h4>
      {schema.fields.map(field => {
        const agg = values[field.path];
        if (!agg.homogeneous) {
          return <MixedField key={field.path} path={field.path} name={field.label} schema={field} />;
        }
        return <EditableField
          key={field.path}
          name={field.label}
          value={agg.value}
          onCommit={(v) => dispatch({
            type: "SetComponentFieldOnMultiple",
            entity_ids: owningEntities.map(e => e.id),
            type_id: typeId,
            field_path: field.path,
            value: v,
          })}
        />;
      })}
    </section>
  );
}
```

## D5. Command: SetComponentFieldOnMultiple

### Variant

```rust
/// Update one field on the same component of multiple entities at once.
/// Inverse is generated via `apply_batch` of N per-entity
/// `SetComponentField` commands; each capture its own pre-state.
SetComponentFieldOnMultiple {
    /// Sorted, de-duplicated target entities.
    entity_ids: Vec<StableId>,
    type_id: String,
    field_path: String,
    value: serde_json::Value,
},
```

### Processor

```rust
fn apply_set_component_field_on_multiple(
    doc: &mut Document,
    entity_ids: &[StableId],
    type_id: &str,
    field_path: &str,
    value: &serde_json::Value,
) -> Result<Command, CommandError> {
    // Build a Batch of SetComponentField; reuse existing machinery.
    let inner: Vec<Command> = entity_ids
        .iter()
        .map(|&id| Command::SetComponentField {
            entity_id: id,
            type_id: type_id.to_string(),
            field_path: field_path.to_string(),
            value: value.clone(),
        })
        .collect();
    let batch = Command::Batch {
        label: format!("Multi-set field {}.{}", type_id, field_path),
        commands: inner,
    };
    apply_batch(doc, &batch)
}
```

`apply_batch` already handles partial-failure rollback, so
`SetComponentFieldOnMultiple` gets that behavior for free.

### Edge cases

- **Empty `entity_ids`**: return `CommandError::InvalidArgument("empty
  entity_ids")`.
- **Duplicate ids in `entity_ids`**: de-duplicate at apply time (so
  the inverse is also de-duplicated and we don't double-write).
- **Entity doesn't have `type_id` component**: skip (counted as
  "1/N has it" in the UI). Sub-apply returns
  `CommandError::ComponentNotFound` for that inner command, batch
  rolls back. **Decided**: we skip at the frontend side (don't
  dispatch for entities that don't own the component). Inner
  Rust call receives only owning entities.

## D6. Persistence wiring (PR1)

`useDockPrefs.ts` additions:

```ts
export const DOCK_PREFS_SCHEMA_VERSION = 3;

type DockPrefs = {
  schemaVersion: 3;
  panelRegions: PanelRegions;
  panelWidths?: Partial<Record<PanelId, number>>;
  floats: Partial<Record<PanelId, FloatingPanelState>>;
};

// load() branches:
//   v1 → migrated → v2 → migrated → v3 (output)
//   v2 → migrated → v3
//   v3 → return as-is
//
// save() always writes v3.
```

`useDockPrefs` exposes:

```ts
const [prefs, setPrefs] = useState<DockPrefs | null>(null);
const setFloatRect = useCallback((panelId: PanelId, rect: FloatingPanelState) => {
  setPrefs(p => p ? { ...p, floats: { ...p.floats, [panelId]: rect } } : p);
  // …persists via the existing `save` from v0.82 P1
}, []);
```

## D7. Edge-case matrix

| Case | Behavior |
|------|----------|
| Float a panel that has unsaved scene changes | Still persists float state via localStorage write-through (carry over from v0.82 P1) |
| Drag floating panel fully off-screen | `clamp` ensures left/top stay ≥0 and within `viewport - header` |
| Multi-edit with one entity missing the field | Frontend filter ensures all dispatched entities own the component; if any inner SetComponentField fails, the Batch rolls back (covered by test) |
| Select-all on empty scene | No-op (selectedIds stays empty) |
| Esc while in text input | Passes through to input |
| Two float panels overlap | The header click promotes the clicked one; the user can still reach the lower one by clicking its visible part |
| Floating panel + DnD reparent in Hierarchy | The floating panel still has working DnD in its content (Hierarchy rows); only the panel-level drag conflicts (mitigated by `draggable={!isFloating}` on the header) |

## D8. Verification artifacts

- `crates/editor-core/tests/multi_select.rs` — Rust tests
- `frontend/tests/ux-floating-panel.spec.ts` — Playwright suite
- `frontend/tests/ux-multi-select.spec.ts` — Playwright suite
- `frontend/tests/ux-dock-prefs-v3.spec.ts` — schema migration test

## D9. Why this design over the alternatives

- **Portal vs CSS-overrides**: Portal is simpler — no need to teach
  the CSS Grid about floating. We just stop rendering the panel in
  the grid when it's floating.
- **Custom drag vs react-draggable**: Custom is 20 LOC vs +15 KB;
  the bundle-budget overage from v0.82 P1 already needs attention.
- **`Set<StableId>` vs `string[]`**: Set gives O(1) toggle. We
  accept the React re-render cost on each change (one extra memo
  per consumer).
- **`SetComponentFieldOnMultiple` vs `Batch` only**: The variant
  expresses intent explicitly; the frontend doesn't have to know
  about `Batch` semantics (it can dispatch one command). The
  inverse/partial-failure behavior comes from `apply_batch`.
- **Schema v3 bump vs backward-compatible field**: The new field
  lives at the root (per ADR-0017 conventions); bumps are first
  class.

## D10. Open architectural questions

None at the end of Phase 0. All decisions resolved.
