# UX Information Architecture Direction

## Mental model

The user should understand four layers:

1. **Project** — files/assets/worlds/scenes.
2. **Document** — what is currently being authored.
3. **Context** — selection and related references.
4. **Runtime** — edit/play diagnostics, separate from durable document identity.

## Suggested shell

```text
Menu / global commands
Context/document bar
-------------------------------------------------
Project/Outline | Active document | Inspector
                |                 |
-------------------------------------------------
Problems / Changes / Runtime / Console
Status bar
```

Panels may dock/float, but their semantic role remains stable.

## Context bar

Should answer at a glance:

- active project;
- active document type/name;
- dirty state;
- runtime state;
- related definition/instance when relevant;
- validation blocker count.

## Bottom workbench convergence

Problems, Changes, Runtime and Console are related evidence/workflow surfaces. They can share common row/navigation primitives even if they remain separate tabs.

## Action hierarchy

### Persistent

- state/status;
- active context;
- critical errors;
- primary tool selection.

### Contextual

- Open Logic;
- jump to definition;
- reimport;
- replace asset;
- override actions;
- less frequent entity operations.

Use context menus, hover actions, command palette and inspector actions rather than permanent text buttons on every row.

## Error UX

Error messages should include:

- what failed;
- affected resource;
- whether data was changed;
- recovery/retry action;
- link/navigation to details when available.

Avoid raw Rust/JS bridge errors as the only user-facing message.

