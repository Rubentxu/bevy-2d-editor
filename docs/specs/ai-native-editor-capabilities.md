# AI-Native Bevy 2D Editor — Durable Capability Spec

> Status: Draft durable spec.
> Planning source of truth for the AI-native editor program.
> Authoritative references: [ADR-0015](../adr/0015-code-aware-ai-context-model.md) · [ADR-0016](../adr/0016-scene-component-authoring.md) · [ADR-0027](../adr/0027-rig-agentic-editor-architecture.md).

This spec defines the **observable capabilities** required to evolve Bevy 2D
Editor from a browser-based scene editor with AI assistance into a
**Cursor-like, agent-native authoring environment for Bevy 2D games**.

It describes what the product must do, not how each slice is implemented.

## Purpose

Give the editor a durable contract for agentic AI that is:

- grounded in editor-owned domain concepts,
- safe to review and approve,
- capable of multi-step autonomous work,
- useful across scene, asset, logic, code, and runtime workflows.

## Quick Path

1. Stabilize the editor shell and workflow coherence first.
2. Expose all already-implemented project context to the agent runtime.
3. Introduce Rig-based orchestration behind the existing proxy boundary.
4. Add proposal/review/apply flows for typed editor and code changes.
5. Add runtime-aware diagnostics and background automation.

## Non-Goals (v1 of the agentic program)

- Unbounded auto-apply to any project artifact.
- Arbitrary plugin execution authored by the model.
- Multi-user collaborative AI editing.
- Free-form code execution inside logic graphs.
- Replacing the editor's command model with direct LLM state mutation.

## Capability Map

| Capability | Direction | Summary |
|---|---|---|
| `agent-runtime` | NEW | Rig-based manager/worker runtime behind `ai-proxy` |
| `semantic-project-context` | MODIFIED | Full multi-source context for scenes, assets, logic, code, diagnostics |
| `agent-workbench` | NEW | Proposal/review/apply UI with staged execution |
| `cross-domain-operations` | NEW | Typed plans spanning scene, asset, logic, and code changes |
| `runtime-aware-diagnostics` | NEW | Agents can inspect preview/runtime/editor mismatches |
| `background-automation` | NEW | Agents can run maintenance, indexing, repair, and scaffolding tasks |
| `validation-first-ai` | MODIFIED | Validation Center and policy gates participate in every AI workflow |
| `knowledge-retrieval` | NEW | Semantic retrieval over docs, schemas, code, assets, and proposals |

## Invariants (non-negotiable)

1. **Editor-owned state remains authoritative.** Agents may read, propose, and
   execute, but `SceneDocument`, `Scene Asset`, `LogicGraphAsset`, command logs,
   and validated file writes remain the durable truth.
2. **No blind mutation path.** Every agent action must travel through a typed,
   auditable adapter.
3. **Project semantics over file semantics.** The primary AI language is editor
   domain language (`Scene Asset`, `Scene Instance`, `Component Override`, etc.),
   not raw path manipulation.
4. **Validation is first-class.** AI output that would create invalid state must
   be blocked, downgraded to proposal-only, or explicitly escalated.
5. **Human trust beats autonomy theater.** High-risk changes require staged review.

## Detailed Requirements

## 1. `agent-runtime`

### Outcome

The backend exposes a genuine agent runtime, not just a single prompt endpoint.

### Required behavior

- The runtime MUST support manager/worker composition.
- The runtime MUST support tool calling.
- The runtime MUST support provider abstraction so the editor can target OpenAI,
  Anthropic-compatible, Ollama-compatible, or future providers without rewriting
  orchestration logic.
- The runtime MUST preserve the existing HTTP/API seam of `ai-proxy`.
- The runtime MUST emit structured statuses for task lifecycle:
  `queued | planning | waiting_for_approval | executing | validating | done | failed`.

### Acceptance criteria

- A single user request can delegate sub-work to at least two specialist agents.
- The runtime reports task lifecycle and sub-task status back to the UI.
- Provider change does not require changing the editor-facing API.

## 2. `semantic-project-context`

### Outcome

Agents receive all relevant context for a task without brute-force dumping the
whole project into the prompt.

### Required behavior

- Context assembly MUST support these source classes:
  - scene snapshot,
  - selected entity,
  - combined schemas,
  - source files,
  - logic graphs,
  - scene asset catalog,
  - selected scene asset body,
  - validation issues,
  - preview/runtime diagnostics,
  - project docs/ADRs/specs.
- Retrieval MUST be selective and explainable.
- The runtime MUST be able to answer "why did you include this context?" with
  source provenance.

### Acceptance criteria

- The agent can answer questions that require both scene data and source data.
- The agent can answer questions that require both overrides and runtime preview evidence.
- Context provenance is surfaced to the UI or logs for debugging.

### PR1 Implementation Note (Phase A — `workflow-surface-convergence`)

PR1 of the `workflow-surface-convergence` change (Phase A) implements S1 and S2:

- **S1**: `selected_entity` is derived in `App.tsx:328-340` from `scene.selectedEntityId` and
  `scene.entities`, passed as an option to `useAIAssistant`, and assembled into
  `extraContext.selected_entity` via `assembleMultiSourceContext` (`useAIAssistant.ts:184-214`).
  The four context sources (`logic_graphs`, `scene_assets.catalog`, `scene_assets.selected_body`,
  `selected_entity`) are all sent to the AI proxy on each request.
- **S2**: `list_logic_graph_assets` seeds 3 built-in recipes (`lga_recipe_jump`,
  `lga_recipe_health`, `lga_recipe_proximity`) on first call and returns the full catalog.
  Created graphs are registered in `LogicGraphCatalog` and persisted to OPFS via
  `save_logic_graph_body` / `load_logic_graph_body`.

See `docs/specs/editor-workflow-convergence.md` § "PR1 Scenarios (Phase A — S1 + S2)" for
full behavioral specification and test evidence.

## 3. `agent-workbench`

### Outcome

The product has a first-class UI for planning, reviewing, approving, applying,
and re-validating AI work.

### Required behavior

- Users MUST be able to inspect a proposal before apply.
- A proposal MUST show:
  - intent summary,
  - affected surfaces,
  - scene/asset/logic/code diffs as applicable,
  - validation impact,
  - runtime risk summary,
  - rollback path.
- The workbench MUST support partial approval when the runtime emits multiple
  operations grouped by domain.
- Rejected proposals MUST be editable, retryable, or convertible into a follow-up prompt.

### Acceptance criteria

- A user can compare a proposal against current project state without opening raw JSON manually.
- A user can apply only the safe subset of a mixed proposal.
- Validation reruns automatically after apply.

## 4. `cross-domain-operations`

### Outcome

Agents can perform tasks that span multiple editor domains coherently.

### Required behavior

- The runtime MUST support plans that touch more than one domain, for example:
  - scene + asset,
  - scene + code,
  - logic + runtime,
  - schema + scene component + code.
- A cross-domain plan MUST stay decomposed into typed sub-operations.
- Every sub-operation MUST declare dependency ordering.

### Example tasks the system must support

- "Create a patrol enemy with health and a pickup drop."
- "Turn these entities into a reusable Scene Asset and place three Scene Instances."
- "Fix stale overrides after this asset changed."
- "Create a SceneComponent schema bound to this Scene Asset and wire the Rust side."
- "Add a jump recipe and bind it to the selected actor."

## 5. `runtime-aware-diagnostics`

### Outcome

Agents can debug mismatches between authored data and preview/runtime behavior.

### Required behavior

- The runtime MUST expose tools for:
  - preview metrics,
  - preview mappings,
  - preview rebuild count and cause,
  - hot-reload events,
  - validation issues,
  - graph validation,
  - override status,
  - selected context.
- Agents MUST be able to explain failures using these diagnostics, not only by
  re-reading source files.

### Acceptance criteria

- The system can diagnose a "why is this actor invisible?" case using preview + component context.
- The system can diagnose a "why did this logic graph not trigger?" case using graph + runtime context.

## 6. `background-automation`

### Outcome

The editor can run durable, non-interactive maintenance tasks in the background.

### Required behavior

- Users MUST be able to launch background jobs for:
  - broken-reference repair suggestions,
  - stale override audits,
  - project indexing,
  - schema cleanup suggestions,
  - source-file scaffolding,
  - content generation from recipes/templates.
- Background jobs MUST be cancellable and inspectable.
- Background jobs MUST not silently auto-apply high-risk mutations.

## 7. `validation-first-ai`

### Outcome

AI work is part of the same project health model as manual work.

### Required behavior

- Validation Center MUST display AI-originated proposal/apply failures.
- AI-originated changes MUST carry provenance so users know what came from an agent.
- The runtime MUST classify issues as:
  - invalid proposal,
  - blocked by policy,
  - blocked by validation,
  - runtime verification failed,
  - user approval required.

## 8. `knowledge-retrieval`

### Outcome

The editor has a durable knowledge layer that makes agents project-aware.

### Required behavior

- Retrieval MUST index:
  - ADRs,
  - specs,
  - roadmap notes,
  - component schemas,
  - source files,
  - scene/asset metadata,
  - built-in recipes,
  - accepted proposal artifacts.
- The system MUST support semantic retrieval plus typed rendering.
- Retrieval MUST be resilient to partial indexing; missing sources degrade gracefully.

## Product Surfaces Required by the Capability Set

The following visible surfaces are required for the AI-native program to count
as product-complete:

1. **AI Assistant Panel** — short-form interaction, current-context tasks.
2. **Agent Workbench** — staged proposal review and execution.
3. **Validation Center** — AI and non-AI health issues in one place.
4. **Global Search / Command Surface** — jump to entities/assets/files/issues/tasks.
5. **Runtime Preview Inspector** — runtime-aware agent debugging companion.

## Dependencies

This spec assumes the following prerequisites are available or completed:

- shell integrity and workflow coherence,
- full multi-source context wiring,
- actionable Validation Center,
- stable source file and logic graph surfaces,
- reliable save/load/hot-reload observability.

## Sequencing

Recommended implementation order:

1. Shell/workflow stabilization and gap remediation.
2. Context exposure parity (logic graphs, scene assets, selected entity, diagnostics).
3. Rig runtime foundation inside backend.
4. Proposal/review/apply workbench.
5. Runtime-aware diagnostics.
6. Background automation and project knowledge retrieval.

## References

- `docs/specs/editor-workflow-convergence.md`
- `docs/roadmaps/ai-native-editor-roadmap.md`
- `docs/adr/0027-rig-agentic-editor-architecture.md`
