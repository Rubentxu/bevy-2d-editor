# ADR-0043: Agent Runtime Uses Replaceable Orchestration Behind Typed Editor Capabilities

## Status

Accepted — 2026-08-14


## Relationship to ADR-0027 / ADR-0028

**Refines ADR-0027** and **extends ADR-0028 sequencing**. Rig remains the accepted orchestration framework for the planned agent runtime, but it is explicitly an infrastructure adapter rather than the product architecture.

## Decision

`agent-runtime` talks to the editor through `editor-protocol` capability tools. Tools return typed observations/plans and may only request mutation through `ChangeSet`/approved capabilities.

The agent runtime must not:

- import editor storage adapters;
- mutate `EditorSession` directly;
- call raw WASM globals;
- bypass validation/approval policies.

Manager/worker specialization remains useful, but specialist boundaries align with editor capabilities rather than owning duplicate state.

## Sequencing

Rig implementation begins after v0.87 architecture gates and the core capability interfaces exist.

## Consequences

Rig/provider replacement is possible without redesigning the editor. Agent failures remain outside the authoritative state until an approved transaction commits.
