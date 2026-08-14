# Specification — Editor Extension SDK

## Goal

Enable extensibility without exposing mutable internals or committing prematurely to a binary ABI.

## Capability interfaces

Extensions can register:

### Actions
Command palette/menu/shortcut actions that invoke capabilities.

### Validators
Read semantic snapshots and emit typed `ValidationIssue`s.

### Recipes
Plan ChangeSets from user parameters/context.

### Importers
Parse external sources and generate semantic import plans.

### Inspectors
Contribute field/component editors via typed schema descriptors.

### Panels/tools
Render UI and call typed backend capabilities.

### Runtime diagnostics
Subscribe to bounded runtime observation streams.

## Security model

Permissions are capability-based:

```text
project.read
scene.write
asset.write
source.write
runtime.observe
filesystem.import
```

Extensions do not receive arbitrary project-root filesystem access by default.

## Versioning

Each SDK contract has a semantic protocol version and capability feature flags. Extensions declare minimum/maximum compatibility.

## Rollout acceptance before public ABI

At least three built-in extensions must be implemented using the same SDK surface, e.g.:

1. Aseprite importer;
2. platformer recipe pack;
3. additional project validator.
