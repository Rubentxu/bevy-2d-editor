# Frontend and UX Architecture

## Stable workspaces

```text
World
Design
Logic
Animate
Debug
Code
```

`Play`, `Pause` and `Step` are runtime state, not workspaces. Tile/Sprite/SceneAsset/AutoLayer are tools/editors within a workspace.

## Persistent shell

```text
+--------------------------------------------------------------+
| Project | Breadcrumb | Search | Play/Pause/Step | AI/Command |
+-----------+--------------------------------------+-----------+
| Navigator |                Workspace             | Inspector |
|           |                                      |           |
+-----------+--------------------------------------+-----------+
| Assets | Problems | Changes | Console | Trace                |
+--------------------------------------------------------------+
```

## Lenses

Transversal views of selected subjects:

- Hierarchy;
- Spatial;
- Dependencies;
- Changes;
- Causality;
- Runtime.

## Inspector registry

Replace monolithic inspector ownership with deterministic contributions such as Transform, Rendering, Physics, Components, Logic, Overrides, Source, Runtime, Causality and Validation.

## Contribution registry

Feature modules register commands, panels, inspectors, workspace tools, asset editors, graph node renderers, menus and status items.

## UX rules

- progressive disclosure;
- stable tool placement;
- command palette parity;
- keyboard-first paths for frequent workflows;
- provenance near edited values;
- impact before destructive actions;
- AI proposals appear as normal reviewable changes.
