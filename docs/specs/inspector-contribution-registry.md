# SPEC-UX-002 — Inspector and Contribution Registry

**Status:** Proposed  
**ADR:** 0062

## Contribution categories

```text
CommandContribution
PanelContribution
InspectorContribution
WorkspaceToolContribution
AssetEditorContribution
GraphNodeViewContribution
MenuContribution
StatusContribution
```

## Inspector contract

Conceptual fields:

```text
id
version
group
priority
supported_subjects
required_capabilities
render contribution
commands
```

Ordering: group -> priority -> stable ID. Conflicts produce diagnostics, never nondeterministic UI.

## Built-in groups

Identity, Transform, Rendering, Physics, Components, Logic, Overrides, Source, Validation, Runtime, Causality.

## Extension security

Extensions receive capability-scoped data/commands, not unrestricted ProjectStore, raw World, browser globals or TransactionKernel internals.

## Migration order

1. Source/Validation;
2. Runtime/Causality;
3. Overrides;
4. Logic;
5. Components;
6. Transform/Rendering.

Preserve behaviour before visual redesign.
