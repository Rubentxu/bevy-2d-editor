# ROADMAP Addendum — v0.86 Candidates

> Generated 2026-07-29 after v0.86.0 shipped the ui-workflow-overhaul cycle.

## Context

After v0.86.0 closes the `ui-workflow-overhaul` cycle (PR1-PR4 merged
across #125 and #126), the workflow-first prerequisite program is
complete. The Hito 8 AI-Native Editor Program is now the active
execution order:

| Order | Change | Status |
|-------|--------|--------|
| 1 | `rig-agent-runtime-foundation` | ⏭️ Next (gate satisfied) |
| 2 | `semantic-project-retrieval` | 🔲 Planned |
| 3 | `agent-workbench` | 🔲 Planned |
| 4 | `runtime-aware-agent-diagnostics` | 🔲 Planned |
| 5 | `background-agent-automation` | 🔲 Planned |

ADR-0027 (Rig-Based Agent Runtime) and ADR-0028 (Workflow-First)
provide the normative framework.

## Candidates

### Tier 1 (must come first — directly from Hito 8 Order 1)

#### 1. Rig agent-runtime crate skeleton
**What**: Add `crates/agent-runtime` with Rig agent builder + tool
trait + Scene tool adapter. Manager/worker topology placeholder.
**Why**: Foundation for every other AI-native feature.
**Effort**: 1-2 weeks. Most of the work is wiring (Rig + tool trait +
Scene command bridge); the actual agent logic comes in subsequent PRs.
**Risks**: Rig API stability — pin a specific version. WASM
compatibility — Rig's HTTP clients may not work in-wasm; isolate
HTTP behind a non-wasm crate.

#### 2. /v1/propose compatibility shim
**What**: Migrate the existing `/v1/propose` endpoint to delegate to
the new agent runtime while preserving the old response shape.
**Why**: Zero-downtime migration. Existing AI panel + tests continue
to work during the transition.
**Effort**: 1 week. Mostly adapter plumbing.

### Tier 2 (depends on Tier 1)

#### 3. Semantic project retrieval
**What**: Index docs, ADRs, specs, recipes, validation, and
diagnostics. Wire results into the AI Assistant Panel context chips
(already rendered by `ui-workflow-overhaul` PR4).
**Why**: Agents become project-aware instead of scene-only.
**Effort**: 2 weeks. WASM-side indexer + Rust-side retriever.

#### 4. Agent workbench UI
**What**: Product-grade review/approval surface. Proposal staging,
typed diff visualisation, validation impact preview, retry/reject/
partial-apply/rollback.
**Why**: Currently the AI panel shows raw proposals. Users need a
typed-review surface to trust agent output.
**Effort**: 2-3 weeks. Reuses `ProposalCard` v2 + adds new panes.

### Tier 3 (depends on Tier 1 + 2)

#### 5. Runtime-aware agent diagnostics
**What**: Expose preview metrics + rebuild cause + logic activation
summaries as agent tools. Agents answer "why is this broken in
preview?" with runtime-backed evidence.
**Why**: Differentiates the editor from generic AI IDEs.
**Effort**: 2 weeks. Reuses the typed `useLogicActivation` hook from
PR4 correction.

#### 6. Background agent automation
**What**: Indexing jobs, stale override audits, broken-ref repair
suggestions, schema normalisation, code scaffolding.
**Why**: Long-running low-risk maintenance compounds productivity.
**Effort**: 3-4 weeks. Requires workbench approval policies + cancel
+ retry affordances.

## Out of scope (defer to post-Hito 8)

- Multi-user collaborative editing (CRDT decision still open)
- Plugin system / marketplace
- Voice / scriptable editor extensions
- Mobile / touch UI

## Recommended next cycle

Start with **#1 (Rig agent-runtime skeleton)** — foundation for everything
else. Then **#2 (propose compatibility shim)** — non-breaking migration.
Then **#3 (semantic retrieval)** — earliest user-visible improvement.

#4 (workbench), #5 (runtime diagnostics), #6 (background automation) can
proceed in parallel once Tier 1+2 are stable.

## Carry-over debt

| Item | Status | Reference |
|------|--------|-----------|
| Bundle +9.68 KB over ADR-0025 budget | Resolved in `debt-backup-ui-workflow-overhaul-pr4` (backup branch, last SHA `c356700`) | Engram obs #5154 |
| 13 weak assertions | Resolved (commit `e07268a`) | Engram obs #5154 |
| 7 testid mismatches | Partially resolved (2/6 fixed; 4 deferred dynamic IDs) | Engram obs #5154 |
| WelcomeOverlay race | Resolved (commit `337bc94`) | Engram obs #5154 |
| WASM env init failures | OPEN — blocks full Playwright suite | Engram obs #5153 |

## Reference

- ADR-0027: Rig-Based Agent Runtime
- ADR-0028: Workflow-First UI Convergence Before Agentic AI
- `docs/roadmaps/ai-native-editor-roadmap.md` (Hito 8 detail)
- `docs/specs/ai-native-editor-capabilities.md`
- Engram obs #4917 (planning baseline)
