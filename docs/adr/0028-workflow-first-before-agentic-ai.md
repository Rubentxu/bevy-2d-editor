# ADR-0028: Workflow-First UI Convergence Before Agentic AI

## Status

Accepted (2026-07-27) — prerequisite sequencing decision for the AI-native editor program; extended by the v0.87 Architecture Foundation gate and the master roadmap in [MASTER_ROADMAP.md](../roadmaps/MASTER_ROADMAP.md) (2026-08-14).

## Context

The Bevy 2D Editor already contains substantial backend and domain capability:

- `SceneDocument`, Scene Assets, Scene Instances, Component Overrides,
  Level Layers, Logic Bricks, source files, play mode, hot reload, Validation Center,
  and code-aware AI context.

The audit of the current product surfaces found a different bottleneck:

1. **Shell integrity debt** — menu visibility defects, layout breakage on narrow
   viewports, floating-panel placeholders, console error floods.
2. **Workflow exposure gaps** — features implemented in `editor-core` or `ai-proxy`
   are only partially surfaced in the UI.
3. **Mode and navigation drift** — scene / asset / logic / code / play modes do not
   always communicate active context clearly.
4. **Docs/shortcut drift** — menus, keyboard shortcuts, and user docs diverge from
   shipped behavior.

At the same time, ADR-0027 establishes a Rig-based agent runtime for the
AI-native roadmap. That decision is valid, but it does **not** mean the next best
implementation step is to build more AI immediately.

If the editor shell remains unreliable or semantically inconsistent, a stronger AI
layer will amplify confusion instead of compounding value.

## Decision

Treat **workflow convergence and graphical editor integrity as the mandatory
prerequisite** to the AI-native editor program.

Concretely:

1. The next planning and implementation work MUST prioritize shell/workflow
   convergence before advanced agentic surfaces.
2. The UI/workflow convergence program becomes its own documented roadmap and
   capability spec.
3. The AI-native roadmap remains valid, but its first executable phases depend on
   the workflow-first program landing first.

## Decision Details

### D1 — Product integrity beats feature multiplication

The editor must first become a trustworthy product surface:

- menus visible and operable,
- dock system coherent,
- mode context obvious,
- primary workflows discoverable,
- docs and shortcuts aligned.

AI is not exempt from this rule. A proposal panel layered onto a confusing shell
is not a better product.

### D2 — UI/workflow convergence is a capability program, not a polish sprint

This is not a cosmetic pass. It is a capability program spanning:

- shell integrity,
- Project Asset Browser maturity,
- Hierarchy/Inspector clarity,
- Logic Bricks continuity,
- Validation Center unification,
- Search/command convergence,
- mode coherence,
- documentation and shortcut coherence.

### D3 — AI-native work must consume converged product surfaces

Future agent workflows must be built on top of stabilized surfaces:

- Validation Center as the health surface,
- search/palette as the navigation surface,
- Runtime Preview Inspector as the runtime-debug surface,
- Agent Workbench as the review/apply surface.

If those surfaces are weak or contradictory, the AI layer inherits the weakness.

### D4 — The sequencing is strict

Recommended sequence:

1. `editor-shell-integrity`
2. `workflow-surface-convergence`
3. `ui-workflow-overhaul`
4. `rig-agent-runtime-foundation`
5. later AI-native phases

This ADR does not cancel ADR-0027. It **reorders execution priority**.

## Considered Options

### Option A — Continue directly into Rig runtime and agent workbench

- **Pros**: faster visible AI progress.
- **Cons**: piles advanced flows onto unstable product surfaces.
- **Rejected**: wrong leverage order.

### Option B — Do a lightweight bugfix round, then resume AI work

- **Pros**: cheaper in the short term.
- **Cons**: preserves structural workflow incoherence; problems return immediately.
- **Rejected**: too shallow.

### Option C — Establish workflow-first convergence as a formal prerequisite

- **Pros**: strongest compounding path; aligns shell, workflows, docs, and future AI.
- **Cons**: delays advanced agent features slightly.
- **Accepted**.

## Consequences

### Positive

- Prevents the AI layer from becoming a thin demo on top of a fragmented editor.
- Gives the team a clean implementation order.
- Makes every later AI feature easier to explain, verify, and adopt.

### Negative / Risks

- Some AI-native milestones move later in calendar order.
- Requires discipline to treat UX/workflow debt as architecture, not polish.

## Follow-Up Artifacts

- `docs/specs/ui-workflow-overhaul.md`
- `docs/roadmaps/ui-workflow-overhaul-roadmap.md`
- updates to `docs/roadmaps/ai-native-editor-roadmap.md`

## References

- ADR-0027 — Rig-Based Agent Runtime for the AI-Native Bevy 2D Editor
- `docs/specs/editor-workflow-convergence.md`
- `docs/ROADMAP.md`
