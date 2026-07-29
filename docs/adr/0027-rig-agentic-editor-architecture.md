# ADR-0027: Rig-Based Agent Runtime for the AI-Native Bevy 2D Editor

## Status

Accepted (2026-07-27) — planning baseline for the AI-native editor program

## Context

The Bevy 2D Editor already has three strong foundations for an AI-native
product:

1. **Editor-owned domain model** — `SceneDocument`, `Scene Asset`, `Scene Instance`,
   `Component Override`, `Level Layer`, `LogicGraphAsset`, `SourceFile`.
2. **Typed mutation seam** — reversible commands and operation logs in `editor-core`.
3. **Multi-source AI context** — ADR-0015 extended the AI proxy so the model can
   receive scene snapshot, schemas, source files, logic graphs, scene assets, and
   selected entity.

The current AI surface is still a proposal endpoint plus a frontend panel. It is
useful, but it is not yet an **agent runtime**. The gap is architectural:

- There is no first-class orchestration model for multi-step tasks.
- There is no manager/worker structure for specialized agents.
- There is no durable tool abstraction above raw prompt assembly.
- There is no retrieval strategy for selectively loading project knowledge.
- There is no review/apply pipeline that treats editor changes, code changes, and
  validation impact as one operation.

The next milestone needs an agent runtime that stays **Rust-native**, composes
with the existing `ai-proxy`, and supports tool calling, retrieval, memory, and
multi-agent delegation without pushing orchestration into ad-hoc prompt code.

Rig is a strong fit because it provides:

- provider-agnostic agent builders,
- tool calling,
- dynamic context / retrieval patterns,
- vector-store integrations,
- manager-worker agent composition,
- Rust-first ergonomics that fit the current backend stack.

## Decision

Adopt **Rig** as the orchestration framework for the next-generation agentic AI
layer, while keeping the Bevy 2D Editor's domain model and command model as the
source of truth.

## Architecture

```text
Frontend UI
  ├─ AI Assistant Panel
  ├─ Agent Workbench
  ├─ Validation Center
  └─ Runtime Preview Inspector
           │
           ▼
crates/ai-proxy
  ├─ HTTP/API boundary
  ├─ auth / policy / rate limits
  ├─ request normalization
  └─ delegates to agent runtime
           │
           ▼
crates/agent-runtime   (NEW)
  ├─ Rig agent builders
  ├─ manager / worker orchestration
  ├─ tool registry
  ├─ retrieval / memory
  ├─ proposal planner
  ├─ execution reviewer
  └─ diagnostics synthesizer
           │
           ├──────── reads project knowledge
           │          (scene, schemas, assets, logic graphs, source files,
           │           validation, preview diagnostics, docs)
           │
           └──────── emits typed plans + approved operations
                               │
                               ▼
                        editor-core / frontend bridges
                          command dispatch + save + preview + validation
```

## Decision Details

### D1 — Rig orchestrates agents; editor commands remain the mutation boundary

Rig manages agents, tools, retrieval, and delegation. It does **not** become the
source of truth for editor state. Durable mutations still happen through:

- `Command` / `AssetCommand` / future agent-specific command surfaces,
- validated frontend services,
- operation logs in `editor-core`.

**Why**: the editor already has a strong typed mutation seam. Replacing it with
free-form LLM actions would destroy undo/redo, validation, and reviewability.

### D2 — Introduce a dedicated `crates/agent-runtime` crate

The current `ai-proxy` remains the HTTP/API boundary. Rig-based orchestration,
tools, retrieval, and manager/worker composition move into a new internal crate:

- `crates/agent-runtime`

`ai-proxy` depends on `agent-runtime`; `editor-core` does not.

**Why**: keep transport concerns separate from agent concerns, and avoid turning
`ai-proxy` into a god-module.

### D3 — Use manager/worker composition for specialized editor agents

The runtime is organized as a manager-worker system. A top-level manager agent
may call specialist agents as tools, for example:

- **Scene Agent** — scene structure, entities, component edits
- **Asset Agent** — Scene Assets, Scene Instances, overrides, resync
- **Logic Agent** — Logic Bricks, recipes, bindings, graph validation
- **Code Agent** — source files, Rust-aware changes, file generation
- **Validation Agent** — project issues, warnings, rollout risk
- **Runtime Agent** — preview metrics, mappings, provenance, hot-reload diagnostics

**Why**: this matches the editor's existing bounded domains and lets prompts stay
small and testable.

### D4 — Retrieval is project-semantic, not file-dump-first

Rig retrieval is used to load only the most relevant context for a task.
Retrieval inputs may include:

- ADRs and specs,
- component schemas,
- source files,
- scene/asset metadata,
- logic graph descriptors and recipes,
- validation issues,
- runtime diagnostics,
- prior accepted AI proposals.

The retrieval model is **semantic + typed**:

- semantic ranking narrows candidate context,
- typed adapters format the final context for each sub-agent.

**Why**: the editor already exceeds the context size where brute-force prompt
assembly remains cheap or reliable.

### D5 — The agent runtime emits staged artifacts, not instant blind writes

Agent work is split into stages:

1. **Intent analysis**
2. **Context retrieval**
3. **Plan / proposal generation**
4. **Validation and risk review**
5. **Human approval when needed**
6. **Execution through typed commands or file writes**
7. **Post-apply validation + runtime verification**

The default product surface is a **proposal-first** workflow, not silent
auto-apply.

**Why**: this preserves trust and makes the editor usable for production work.

### D6 — Memory is split into three layers

1. **Request memory** — per interaction context.
2. **Session memory** — current editor session, pending tasks, active proposals.
3. **Project memory** — durable artifacts such as accepted proposals, review notes,
   reusable recipes, and indexed docs.

Rig manages agent-side memory composition; the editor still owns domain state.

### D7 — Runtime diagnostics become first-class tools

Agents must be able to query:

- preview metrics,
- preview mappings,
- last rebuild cause,
- hot-reload events,
- validation issues,
- graph validation,
- override status,
- selected entity / selected asset / selected graph context.

**Why**: a Bevy game editor becomes AI-native only when the agent can reason
about runtime/editor mismatches, not just static files.

## Considered Options

### Option A — Keep the custom `ai-proxy` orchestration and grow it by hand

- **Pros**: zero new dependency, full control.
- **Cons**: prompt assembly, tools, retrieval, and delegation all become custom
  infrastructure; higher maintenance; slower experimentation.
- **Rejected**: too much framework work for a team building a product, not an AI SDK.

### Option B — Push orchestration into the frontend

- **Pros**: faster local iteration, simpler backend.
- **Cons**: leaks secrets/provider config, weakens policy enforcement, duplicates
  logic across browser/runtime boundaries.
- **Rejected**: conflicts with the existing Rust proxy architecture.

### Option C — Use Rig as orchestration, keep domain validation and commands custom

- **Pros**: keeps existing editor strengths, adds strong agent primitives,
  preserves Rust-first posture.
- **Cons**: adds one architectural layer and new concepts for maintainers.
- **Accepted**.

## Consequences

### Positive

- Clear path from "AI panel" to real agent runtime.
- Keeps the editor's typed command model and reviewability intact.
- Enables specialized sub-agents without scattering prompt logic everywhere.
- Makes retrieval, memory, and provider abstraction explicit instead of ad hoc.
- Gives a foundation for background agents, proposal review, and runtime-aware debugging.

### Negative / Risks

- Adds architectural surface area (`agent-runtime`, tool registry, retrieval index).
- Requires careful boundaries so Rig does not bypass command validation.
- Introduces dependency risk if Rig APIs evolve quickly.
- Needs clear observability so agent failures are diagnosable by maintainers.

## Rollout Constraints

1. Rig adoption must start behind the existing `ai-proxy` interface.
2. Existing `/v1/propose` flows must keep working during migration.
3. Agent execution may only mutate through approved command/file adapters.
4. Delete/rename file operations remain policy-gated; the agent runtime does not
   weaken ADR-0015 restrictions by default.
5. Every new agent surface must define approval semantics: auto-apply,
   proposal-first, or human-mandatory.

## Follow-Up Work

- Durable capability spec: `docs/specs/ai-native-editor-capabilities.md`
- Workflow and product-gap spec: `docs/specs/editor-workflow-convergence.md`
- Delivery plan: `docs/roadmaps/ai-native-editor-roadmap.md`

## References

- ADR-0011 — Logic Bricks compiled-Rust direction
- ADR-0015 — Code-Aware AI Context Model
- ADR-0016 — Scene-Component Authoring
- ROADMAP — Hito 4/Hito 6/Hito 7 status and current shell
- Rig Playbook — multi-agent systems, dynamic context, dynamic tools
- Rig docs — provider abstraction, tool calling, embeddings/vector stores
