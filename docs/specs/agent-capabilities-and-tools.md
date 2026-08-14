# Specification — Agent Capabilities, Tools and Approval

## Goal

Make agents reliable editor operators rather than prompt-driven direct writers.

## Architecture

```text
agent-runtime (Rig)
  ↓
Typed Tool Registry
  ↓
Editor Capability APIs
  ↓
Plan / ChangeSet
  ↓
Validation + Change Workbench
  ↓
Transaction Kernel
```

## Specialist roles

- Manager/Planner;
- Scene specialist;
- Asset/override specialist;
- World/level specialist;
- Logic specialist;
- Code specialist;
- Validation specialist;
- Runtime diagnostics specialist.

These are prompt/orchestration roles, not separate state owners.

## Tool classes

### Read tools

- query project resources;
- search semantic index;
- inspect schema;
- inspect selected entity/asset/world;
- inspect validation;
- inspect runtime causality;
- read source fragments.

### Planning tools

- simulate capability;
- build ChangeSet;
- estimate affected resources;
- run validation without applying.

### Mutating tools

Mutating tools only submit an approved or auto-approved `ChangeSet` through application capabilities.

## Approval matrix

| Operation | Default |
|---|---|
| read/search/diagnostics | auto |
| create non-destructive scene content | proposal-first |
| bulk change > threshold | human required |
| source file write | human required initially |
| delete/rename authored source/assets | human mandatory |
| migration | human mandatory |
| low-risk formatter/index maintenance | policy-defined auto |

## Observability

Record task, model/provider, tool calls, retrieved resource IDs, proposed changes, validation outcome and user decision. Do not persist secrets or unnecessary prompt content.
