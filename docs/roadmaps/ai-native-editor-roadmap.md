# AI-Native Editor Roadmap

This roadmap reorganizes the next major work for Bevy 2D Editor around one
product goal:

> Build the best browser-based authoring environment for Bevy 2D games, then
> make it agent-native.

It is intentionally broader than a single milestone. The program covers shell
integrity, workflow convergence, agent architecture, runtime diagnostics, and
the product surfaces required for a true "Cursor for Bevy 2D games".

## Executive Summary

The implementation order is:

1. **Complete the workflow-first UI program**.
2. **Adopt Rig as the backend agent runtime**.
3. **Add proposal/review/apply workflows**.
4. **Make agents runtime-aware and background-capable**.

## Program Principles

- **Product integrity before autonomy** — broken menus and placeholder flows must
  be fixed before adding more AI layers.
- **Typed operations over blind edits** — AI work stays auditable.
- **Domain language first** — scene, asset, logic, schema, and preview concepts
  are first-class AI vocabulary.
- **Reviewability over magic** — high-risk actions are staged and inspectable.

## Prerequisite Program — Workflow-First UI Convergence

Before this roadmap becomes the active top priority, the workflow-first program
defined in `docs/roadmaps/ui-workflow-overhaul-roadmap.md` must land through:

1. `editor-shell-integrity`
2. `workflow-surface-convergence`
3. `ui-workflow-overhaul`

This prerequisite is mandated by ADR-0028.

## Phase 0 — Rig Runtime Foundation

### Prerequisite (already complete)

The "Editor Shell Integrity" work that originally lived in the first
half of this Phase 0 section was extracted into its own prerequisite
change `editor-shell-integrity` and completed before v0.85.0. It is
documented in `docs/roadmaps/ui-workflow-overhaul-roadmap.md` Phase 0
(Editor Shell Integrity) and the durable spec
`docs/specs/editor-workflow-convergence.md`. No further work is required
from the AI-native program on shell integrity.

### Goal

Introduce a real agent runtime behind the existing `ai-proxy` API.

### Scope

- Add `crates/agent-runtime`.
- Integrate Rig agent builder/tool abstractions.
- Define manager/worker topology.
- Define tool adapters for:
  - scene commands,
  - scene asset operations,
  - logic graph operations,
  - source file operations,
  - validation queries,
  - preview/runtime diagnostics.
- Preserve current `/v1/propose` compatibility during migration.

### Deliverables

- agent runtime crate,
- tool registry,
- task lifecycle model,
- observability/logging for agent execution,
- provider abstraction and policy seams.

### Success criteria

- At least one request can be delegated to specialists and produce a structured result.
- Existing AI panel can use the new runtime without breaking old flows.

## Phase 1 — Full Semantic Context and Retrieval

### Goal

Move from "prompt with extra fields" to true project-semantic retrieval.

### Scope

- Add retrieval/indexing for docs, ADRs, specs, recipes, validation, and diagnostics.
- Introduce durable project knowledge indexing.
- Add provenance for included context.
- Wire retrieval results into the AI Assistant Panel's context sources
  (chips already rendered by `ui-workflow-overhaul` PR4).

### Success criteria

- The runtime can answer cross-domain questions with clear provenance.
- Prompt size remains bounded while answer quality improves.

## Phase 2 — Agent Workbench and Typed Review Flow

### Goal

Give users a product-grade place to review and approve agent work.

### Scope

- New Agent Workbench UI.
- Proposal staging with grouped operations.
- Diff visualization across scene / asset / logic / code.
- Validation impact preview.
- Approval policies by operation class.
- Retry, reject, partial-apply, rollback affordances.

### Success criteria

- Users can inspect AI work without reading raw implementation details.
- Applying a proposal always triggers validation and clear success/failure reporting.

## Phase 3 — Runtime-Aware Agent Debugging

### Goal

Make agents useful for debugging, not just generation.

### Scope

- preview metrics as agent tools,
- preview mappings and provenance queries,
- rebuild cause tracking,
- hot-reload diagnostics,
- override/resync diagnostics,
- logic evaluation traces and issue summaries.

### Success criteria

- The editor can answer "why is this broken in preview?" with runtime-backed evidence.

## Phase 4 — Background Automation and Project Maintenance

### Goal

Enable long-running, low-risk maintenance tasks that do not require synchronous chat.

### Scope

- indexing jobs,
- stale override audits,
- broken-ref repair suggestions,
- schema normalization suggestions,
- code/file scaffolding jobs,
- recipe/template generation.

### Success criteria

- Users can launch, inspect, cancel, and review background jobs.
- Background jobs never silently apply high-risk changes.

## Phase 5 — Advanced Product Differentiators

### Candidate initiatives

- multi-file/gameplay feature generation from intent,
- richer runtime traces,
- project templates powered by agent recipes,
- plugin-ready agent tool contracts,
- collaborative proposal review,
- test generation for editor workflows and game logic.

These are intentionally deferred until the prior phases are stable.

## Recommended Change Breakdown

| Order | Change name | Why first |
|---|---|---|
| 1 | `rig-agent-runtime-foundation` | creates the backend agent platform after UI convergence |
| 2 | `semantic-project-retrieval` | makes the runtime project-aware |
| 3 | `agent-workbench` | converts backend capability into product UX |
| 4 | `runtime-aware-agent-diagnostics` | differentiates the editor from generic AI IDEs |
| 5 | `background-agent-automation` | compounds productivity safely |

## Out of Scope for the First Program Wave

- CRDT multi-user editing,
- voice interaction,
- fully autonomous game generation without approval,
- open plugin marketplace,
- replacing existing SDDK/change-management discipline.

## References

- `docs/adr/0027-rig-agentic-editor-architecture.md`
- `docs/adr/0028-workflow-first-before-agentic-ai.md`
- `docs/specs/ai-native-editor-capabilities.md`
- `docs/specs/editor-workflow-convergence.md`
- `docs/specs/ui-workflow-overhaul.md`
- `docs/roadmaps/ui-workflow-overhaul-roadmap.md`
- `docs/ROADMAP.md`
