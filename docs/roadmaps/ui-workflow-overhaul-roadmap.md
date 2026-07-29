# UI Workflow Overhaul Roadmap

This roadmap defines the graphical editor and workflow convergence program that
must land before the advanced AI-native roadmap becomes the main execution focus.

## Executive Summary

The implementation order is:

1. **Editor shell integrity**
2. **Workflow surface convergence**
3. **UI workflow overhaul by primary surfaces**
4. **Then** Rig runtime and agentic workbench phases

## Phase 0 — Editor Shell Integrity

### Goal

Fix every issue that makes the shell unreliable or misleading.

### Scope

- menu stacking/visibility,
- viewport-width strategy,
- code-files runtime error flood,
- floating panel placeholder debt,
- onboarding duplication,
- status bar width/layout correctness,
- mode-header truthfulness.

### Deliverables

- stable menu layer,
- explicit minimum-width or compact mode behavior,
- clean startup console,
- real floating panel content,
- shell consistency tests.

## Phase 1 — Workflow Surface Convergence

### Goal

Expose existing backend/editor capabilities coherently.

### Scope

- AI context parity,
- logic graph browsing/opening continuity,
- Validation Center unification,
- actionable search,
- docs/shortcuts/menu drift removal.

### Deliverables

- selected entity / scene assets / logic graphs wired into AI context,
- logic listing/opening flows,
- unified validation composition,
- actionable search results,
- updated docs and shortcuts.

## Phase 2 — UI Overhaul by Surface

### Goal

Improve the editor as a product surface, not as a collection of widgets.

### Workstreams (planned)

The Phase 2 workstreams are the conceptual scope. Actual execution
consolidated them into 4 PRs for review economics and bundle budget
(ADR-0025).

| # | Workstream | PR | Status |
|---|---|---|---|
| 2.1 | Context and Mode Orientation | PR1 | Archived (v0.86.0, PR #126) |
| 2.2 | Project Asset Browser v2 | — | Absorbed by `workflow-surface-convergence` PR1/PR2/PR3 (v0.85.0) |
| 2.3 | Hierarchy + Inspector v2 | PR2 | Archived (v0.86.0, PR #126) |
| 2.4 | Validation Center v2 | PR3 (consolidated with 2.5) | Archived (v0.86.0, PR #126) |
| 2.5 | Search / Command Surface | PR3 (consolidated with 2.4) | Archived (v0.86.0, PR #126) |
| 2.6 | Logic Workflow v2 | PR4 (consolidated with 2.7 + 2.8) | Archived (v0.86.0, PR #126) |
| 2.7 | Runtime Preview Inspector v2 | PR4 (consolidated with 2.6 + 2.8) | Archived (v0.86.0, PR #126) |
| 2.8 | AI Panel v2 | PR4 (consolidated with 2.6 + 2.7) | Archived (v0.86.0, PR #126) |

**Notes on consolidation**:
- **PR3** merged workstreams 2.4 (Validation Center v2) and 2.5 (Search/Command
  Surface) into a single coherent inbox + command surface for shared
  result-row presentation (`SearchResultRow.tsx`).
- **PR4** merged workstreams 2.6 (Logic Workflow v2), 2.7 (Runtime Preview
  Inspector v2), and 2.8 (AI Panel v2) into a single Logic/Runtime/AI surface
  cycle. Required App.tsx wiring for 11 new prop families across three
  components plus a typed `useLogicActivation` hook.
- **2.2** was removed from this roadmap because its scope
  ("denser browsing + optional visual mode, role/version/relationship
  metadata, inline actions and fewer browser prompts") was fully
  delivered by `workflow-surface-convergence` PR1/PR2/PR3 — see
  `docs/specs/editor-workflow-convergence.md` for the durable spec.

#### Workstream details (legacy reference)

The original 8 workstreams are kept below for historical reference only;
they no longer represent the execution plan.

<details>
<summary>Legacy 8-workstream layout (collapsed)</summary>

#### 2.1 Context and Mode Orientation
- mode context bar
- active-target breadcrumbs
- dirty state prominence

#### 2.2 Project Asset Browser v2
- denser browsing + optional visual mode
- role/version/relationship metadata
- inline actions and fewer browser prompts

#### 2.3 Hierarchy + Inspector v2
- richer row semantics
- grouped inspector sections
- stronger multi-select UX

#### 2.4 Validation Center v2
- inbox layout
- domain grouping
- detail + action pane

#### 2.5 Search / Command Surface
- action-oriented results
- consistent shortcut model

#### 2.6 Logic Workflow v2
- recipe-first entry
- stronger scene/inspector integration

#### 2.7 Runtime Preview Inspector v2
- diagnostics-first runtime surface

#### 2.8 AI Panel v2
- explicit context controls
- ask/propose/fix/review task framing

</details>

## Phase 3 — Bridge into the AI-Native Program

### Goal

Hand off from the UI/workflow program into the Rig-based agentic architecture.

### Gate

Do not promote the AI-native roadmap to the top of execution order until:

- shell blockers are closed,
- major workflow surfaces are coherent,
- validation/search/runtime/AI surfaces are strong enough to host agent workflows.

## Recommended Change Breakdown

| Order | Change name | Why first |
|---|---|---|
| 1 | `editor-shell-integrity` | removes hard blockers |
| 2 | `workflow-surface-convergence` | exposes already-built capability honestly |
| 3 | `ui-workflow-overhaul` | upgrades the main product surfaces |
| 4 | `rig-agent-runtime-foundation` | now lands on top of stronger UI surfaces |

## Relationship to the AI-Native Roadmap

This roadmap precedes:

- `docs/roadmaps/ai-native-editor-roadmap.md`

It does not replace it. It makes it executable in the right order.

## References

- `docs/adr/0028-workflow-first-before-agentic-ai.md`
- `docs/specs/ui-workflow-overhaul.md`
- `docs/specs/editor-workflow-convergence.md`
- `docs/roadmaps/ai-native-editor-roadmap.md`
