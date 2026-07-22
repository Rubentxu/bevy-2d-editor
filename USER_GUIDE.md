# Bevy 2D Editor User Guide

## Place a sprite in 60 seconds

1. Complete the [README quickstart](README.md#quickstart) and open <http://localhost:5173>.
2. Press `N` to create an entity and select it in the Hierarchy.
3. In the Inspector, choose **Add Component** and add `editor.Transform2D` and `editor.Sprite2D`.
4. Set the sprite asset path and adjust position, scale, or anchor values.
5. Press the Play button to run the scene. Stop play mode to return to authoring.
6. Save the scene from the toolbar or command palette (`Ctrl/Cmd+K`).

For reusable content, create a Scene Asset in the Project Asset Browser and use **Place Instance** to add it to the current level.

## Keyboard shortcuts

Shortcuts are disabled while focus is in an input, text area, or editable code field.

| Shortcut | Action |
| --- | --- |
| `Ctrl/Cmd+K` | Open the command palette |
| `?` | Open the keyboard shortcut cheatsheet |
| `N` | Create a new entity |
| `F2` | Rename the selected entity |
| `F` | Fit the viewport |
| `Delete` / `Backspace` | Delete the selected entity |
| `Ctrl/Cmd+Z` | Undo |
| `Ctrl+Y` / `Cmd+Shift+Z` | Redo |
| `Escape` | Close the active modal or palette |

Press `?` in the editor for the current in-app list.

## Core concepts

### Scene

A Scene is the editable level document. It owns entities, component instances, placed Scene Instances, and the operation history used by undo and redo.

### Scene Asset

A Scene Asset is reusable authored content stored in the project asset catalog. It can represent a prop, character, level, layer, or another reusable hierarchy. Editing the asset does not directly rewrite every placement.

### Scene Instance

A Scene Instance places a Scene Asset into a Scene. It retains its source asset identity and can carry instance-specific component overrides while still participating in resync and validation.

### Component

A Component is typed data attached to an entity, such as transform, sprite, collider, or a project-specific gameplay value. Components are edited in the Inspector.

### Schema

A Schema describes the fields and types accepted by a component. Built-in and project schemas drive validation and Inspector controls while preserving unknown fields for forward compatibility.

### Layer

A Layer organizes level content and draw or editing structure. Level layers can contain authored entities, tile or IntGrid data, auto-generated content, or placed Scene Instances.

## Common workflows

### Create a level

1. Create or open a Scene.
2. Add root entities and organize them in the Hierarchy.
3. Add components through the Inspector.
4. Use layers for level structure and tile or IntGrid content.
5. Review the Validation Center before saving.

### Place content from the catalog

1. Open the Project Asset Browser.
2. Select a Scene Asset.
3. Choose **Place Instance**.
4. Position the new instance and set overrides in the Inspector when needed.

### Run the scene

1. Save authoring changes.
2. Press Play in the top bar.
3. Inspect behavior in the Bevy preview canvas.
4. Press Stop to restore the authoring snapshot.

### Save

Use the save action in the top bar or command palette. Project metadata and assets are written to Origin Private File System (OPFS), which is local to the current browser profile and origin.

### Export

Use the relevant export action for the current document. The editor can produce Bevy-oriented Rust/BSN representations; resolve Validation Center errors before integrating exported output into a game project.

## Troubleshooting

### OPFS data is missing

OPFS is scoped to the exact browser origin and profile. `http://localhost:5173` and another port or hostname have different storage. Private browsing, clearing site data, or using a different browser profile can remove or hide the project. Keep exports or source-control copies for important work.

If catalog entries appear inconsistent, wait for the save operation to finish before reloading. ADR-0019 defines the required metadata ordering; that ADR is currently present in the working tree but not yet merged into the repository history.

### Hot reload did not trigger

Hot reload is data-only. Save the source file, Logic Graph, or supported scene data and confirm the status indicator reports the change. Texture hot reload and Rust/WASM recompilation are not covered; run `just wasm` or keep `just watch` running, then refresh the browser when Rust code changes.

### WASM build errors

Verify the toolchain and target:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
just wasm
```

On Linux, native Rust tests may also require system packages such as `libudev` and ALSA development headers.

### Frontend build errors

Reinstall locked dependencies and rebuild generated WASM:

```sh
cd frontend
npm ci
npm run build:wasm
npm run build
```

If generated bindings look stale, remove `frontend/src/wasm` and rerun `npm run build:wasm`.
