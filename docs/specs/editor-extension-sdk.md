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

## Security model (v0.92 vocabulary)

Permissions are structured (`PermissionArea + PermissionScope` pairs):

```text
PermissionArea { Commands, Validators, Recipes, Importers, Inspectors, AssetProcessors, Panels, DiagnosticProviders, Project }
PermissionScope { Read, Write, Propose, Subscribe }
Permission = (PermissionArea, PermissionScope, optional resource glob)
```

Apply-time enforcement: `TransactionKernel::apply_atomic` re-checks declared permissions for every `ChangeOrigin::Plugin` ChangeSet before the preflight loop. The `extension:<id>` actor prefix is the single source of truth for extension-originated ChangeSets.

Extensions do not receive arbitrary project-root filesystem access by default.

## Versioning

Each SDK contract has a semantic protocol version and capability feature flags. Extensions declare minimum/maximum compatibility.

## Rollout acceptance before public ABI

At least three built-in extensions must be implemented using the same SDK surface (ADR-0040 step 2). v0.92 delivers:

1. `builtin.logic-bricks.controllers` — Logic Bricks RustController extension (`Capability::Commands`, `Commands::Propose`);
2. `builtin.logic-recipes` — built-in recipe pack (`Capability::Recipes`, `Recipes::Write`);
3. `builtin.scene-validator` — scene-document validator (`Capability::Validators`, `Project::Read`).

## Architecture (v0.92 implementation)

- **`ExtensionManifest`** in `editor-model::extension`: `id` (opaque string), `version` (SemVer), `capabilities: Vec<CapabilityDescriptor>`, `permissions: Vec<Permission>`. Serde-derivable, JSON-stable.
- **`ExtensionRegistryPort`** trait in `editor-model::ports`: object-safe (`dyn ExtensionRegistryPort`), methods `register`, `unregister`, `list`, `get`. Mirrors `ProjectStore` port-trait pattern.
- **`ExtensionRegistry`** implementation in `editor-application`. Held on `EditorSession` as `Arc<Mutex<dyn ExtensionRegistryPort>>` (8th sub-state).
- **`EditorSession::with_builtins()`** — canonical constructor that registers all three built-in manifests at composition time.
- **`transaction_kernel_check_plugin_permission`** — standalone helper called from `approve_selected_ops_impl`; fires only for `ChangeOrigin::Plugin`; resolves extension via `actor.strip_prefix("extension:")`.
- WASM exports: `register_extension_wasm`, `list_extensions_wasm`, `unregister_extension_wasm`, `submit_plugin_change_set_wasm`.
