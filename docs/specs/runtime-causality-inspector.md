# Specification — Runtime Causality Inspector

## Goal

Explain *why* a runtime entity behaves/looks as it does by connecting runtime state to authoring definitions, overrides, logic and source.

## Causality view

For a selected runtime/editor entity, surface:

```text
World / Level
  ↓
Scene Asset definition
  ↓
Scene Instance
  ↓
Instance components + overrides
  ↓
LogicGraph bindings and recent activations
  ↓
Runtime Bevy entity/components
  ↓
Rust source locations / system diagnostics
  ↓
Recent ChangeSets affecting it
```

## Runtime observations

- FPS/frame timings;
- preview rebuild count + last rebuild cause;
- entity mapping/provenance;
- selected component values;
- Logic Bricks node activation trace (bounded ring buffer);
- collisions/events relevant to registered diagnostics;
- hot reload events;
- compiler/runtime errors where available.

## Navigation

Each causality edge is actionable: jump to Scene Asset, override field, logic node, source line or ChangeSet.

## Performance/privacy

Tracing is opt-in by diagnostic channel and bounded. Do not collect unbounded frame-by-frame histories by default.
