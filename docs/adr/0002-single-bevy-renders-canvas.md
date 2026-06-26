# Single Bevy instance renders the entire canvas — React never touches it

## Status

Accepted

## Context

The Bevy 2D Editor renders two categories of visuals inside the same canvas: **scene entities** (sprites, transforms, hierarchy) and **editor overlays** (grid lines, gizmo handles, selection outlines, box select rectangles). The editor UI around the canvas (hierarchy panel, inspector, toolbar) is React/DOM.

The question is whether editor overlays should be rendered by the same Bevy instance that renders the scene, or by a separate rendering layer (HTML/CSS/Canvas2D overlay positioned on top of the Bevy canvas).

## Decision

A **single Bevy WASM instance** renders everything inside the canvas. React never touches the canvas element. Editor overlays (grid, gizmos, selection, box select) are Bevy entities, components, or gizmo draw calls within the same World.

## Considered Options

### Option A — Single Bevy instance (chosen)

One Bevy app. Scene entities and editor visuals coexist in the same World. Editor-only entities (camera, grid, gizmo handles) are tagged so they don't export to `DynamicScene`. Editor configuration (selection state, grid spacing, snap toggle) lives as Bevy Resources.

**Pros:**

- Gizmos, selection outlines, and entities share the same camera transform and coordinate space automatically — zero sync code.
- Picking/raycast operates naturally over entities in the same World.
- During interactive gestures (drag), gizmo and entity move together in the same frame — no cross-system lag.
- `bevy_gizmos` already provides line/rect/circle drawing ideal for grid, box select, and move handles.
- Fewer moving parts: one render loop, one input loop, one coordinate system.

**Cons:**

- All editor visual logic lives in Rust/Bevy, not JS. Teams strong in React but weak in Bevy may find gizmo/grid iteration slower.
- Debugging editor visuals requires Bevy tooling, not browser DevTools.
- Editor entity cleanup and tagging discipline is required to prevent editor-only entities from leaking into exports.

### Option B — Bevy for scene + HTML/CSS overlays for editor visuals

Bevy renders only scene entities. Grid, gizmos, selection, and box select are HTML/CSS/Canvas2D elements positioned over the Bevy canvas.

**Rejected because:** Coordinate synchronization between Bevy world space and DOM screen space is painful and fragile. Every camera pan/zoom, canvas resize, DPI change, and scroll requires recomputing DOM overlay positions. During entity drag, the gizmo (DOM) and entity (Bevy) would need per-frame position sync across the WASM↔JS boundary, introducing latency and complexity. This effectively rebuilds a rendering pipeline in JS that Bevy already provides.

### Option C — Two Bevy instances

One Bevy instance for scene preview, another for editor overlays, composited together.

**Rejected because:** Two Bevy instances double the WASM footprint, require inter-instance communication for coordinate sharing, and solve no problem that a single instance with entity tagging doesn't solve better.

## Consequences

- The canvas is exclusively Bevy-owned. React's `<canvas>` ref is passed to the WASM core once at startup; React never writes to it.
- Editor visual systems (grid rendering, gizmo drawing, selection outline, box select) are Bevy systems/plugins, not React components.
- Editor-only entities must be tagged (e.g., `EditorOnly` marker component) so the `DynamicScene Export` adapter skips them.
- Editor runtime configuration (`SelectionState`, `GridConfig`, `SnapConfig`) lives as Bevy Resources, queryable by editor systems and snapshot-serializable for React.
- The `SceneDocument`, operation log, and schema registry remain outside Bevy's World as Rust structs — Bevy renders derived state, it does not own document truth.
- Future HTML-based editor features (e.g., an SVG minimap, or a DOM-based timeline) can still exist as separate React components outside the canvas — the decision applies only to what renders *inside* the canvas.
