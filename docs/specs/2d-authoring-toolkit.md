# Specification — 2D Direct Manipulation Toolkit

## Objective

Make common level/scene editing faster than editing numeric fields in the Inspector.

## Core tools

### Selection

- click select;
- Ctrl/Cmd toggle;
- Shift range in hierarchy;
- box/lasso selection in viewport;
- select by type/component/layer;
- lock/unlock selection targets.

### Transform gizmos

- translate X/Y/free;
- rotate;
- scale;
- pivot/anchor visualisation;
- local/world axes where meaningful.

### Snapping and guides

- grid snap;
- pixel snap;
- vertex/edge/anchor snap where supported;
- temporary guides;
- configurable increments.

### Bulk layout

- align left/right/top/bottom/centres;
- distribute horizontal/vertical;
- equal spacing;
- match width/height/scale where compatible.

### Layers

- visibility;
- lock;
- order;
- parallax metadata;
- isolate/solo.

## Transaction behavior

A pointer drag emits many preview updates but commits **one semantic command/batch** at gesture end. Escape cancels the gesture and restores pre-state.

## Performance

Viewport manipulation must not require serializing the complete project for each pointer move. Use transient gesture state and commit the final semantic mutation.

## Accessibility

Every critical manipulation has keyboard/inspector alternatives and visible numeric values.
