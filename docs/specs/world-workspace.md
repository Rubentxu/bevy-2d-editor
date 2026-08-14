# Specification — World Workspace

## Goal

Provide a spatial/topological overview of multiple Level Scene Assets and their transitions.

## Model

```rust
WorldDocument
  id
  name
  layout_policy
  levels: Vec<WorldLevelRef>
  links: Vec<WorldLink>
```

`WorldLevelRef` contains a Level Scene Asset reference plus world placement metadata. Level content remains inside the Level Scene Asset.

## Features

- drag levels in world canvas;
- snap to optional world grid;
- resize/visualise level bounds;
- create directional/typed links;
- edit entrances/exits and spawn mappings;
- show broken/missing neighbour references;
- open level on double-click;
- minimap/overview;
- filters by tags/region/chapter;
- validate unreachable levels and invalid reciprocal links;
- optional streaming/load-zone metadata.

## Layout policies

- Free;
- Grid;
- Horizontal;
- Vertical.

`GridVania` can emerge as a workflow preset over `Grid`, not necessarily a separate storage primitive.

## Agent/tool capabilities

- `create_world`;
- `add_level_to_world`;
- `connect_levels`;
- `find_unreachable_levels`;
- `layout_world` proposal;
- `create_room_chain` recipe.
