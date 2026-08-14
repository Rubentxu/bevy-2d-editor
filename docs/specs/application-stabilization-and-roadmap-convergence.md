# Application Stabilization & Roadmap Convergence

## Purpose

Establish stabilization as the mandatory path after v0.86.0 and before Hito 8 implementation. Define the gates and invariants that prevent skipped work, documentation drift, and agent-runtime concerns leaking into the application before the foundation is proven green.

## Non-Goals

- Starting Hito 8 implementation (Rig, agent runtime, or later AI-native phases) before release-health gates pass.
- Renaming or rearranging existing domain terms; the glossary stays fixed.
- Substituting `skipped` or `flaky-retried` for a failing test.
- New aesthetics, UI flows, or product copy outside stabilization scope.

## Invariant

No feature work or Rig scaffolding begins until release-health gates report green for the current commit. Correcting the existing Hito 8 design is allowed, but implementation remains blocked by this gate.

## Requirements

### Requirement: Release-Health Gates

The project MUST publish a single release-health signal that aggregates Rust test status, frontend static gates (typecheck, lint, format, build), and CI pipeline state. The signal MUST be the only authority for "ready to start Hito 8".

#### Scenario: All gates green

- GIVEN a commit on the stabilization branch
- WHEN the CI pipeline finishes
- THEN release-health reports `green` and the Hito 8 roadmap step is unblocked

#### Scenario: Any gate fails

- GIVEN at least one of Rust tests, frontend static gates, or CI fails
- WHEN the pipeline finishes
- THEN release-health reports `red`, the failing gate is named, and Hito 8 stays blocked

### Requirement: Performance Budget Contract

The project MUST track three budgets independently: initial JS bundle, total JS bundle, and WASM. The budget file MUST fail CI on regression and MUST NOT allow silent increases via undocumented overrides.

#### Scenario: A measured artifact exceeds its configured threshold

- GIVEN initial JS, total JS, or WASM exceeds its documented limit
- WHEN CI runs the budget check
- THEN CI fails naming the violated budget and the delta

### Requirement: Editor Readiness & E2E Cohorts

The project MUST expose a single readiness signal gating the editor's main flow. E2E suites MUST be split into smoke, domain, persistence, accessibility, and full cohorts. Each run MUST be deterministic from a seeded state; skipped and retried tests MUST NOT mask a root cause.

#### Scenario: Repeated failure surfaces

- GIVEN an E2E test fails repeatedly from the same seeded state
- WHEN the run completes
- THEN the suite is marked `red`, the failure is classified, and no skip or retry hides it

### Requirement: Documentation Convergence

The project MUST keep four surfaces aligned: CONTEXT.md (language), docs/adr/ (decisions), docs/specs/ (behavior), and ROADMAP (current sequence). Each CHANGELOG release MUST reference the shipped change, pull request, or durable provenance available in the repository.

#### Scenario: Glossary drift

- GIVEN a PR introducing a synonym for an existing term
- WHEN the docs lint runs
- THEN CI fails pointing to the canonical term in CONTEXT.md

### Requirement: Editor Gateway

The editor's frontend boundary MUST be a typed, injectable interface. Compatibility shims for the legacy `window` global MUST be temporary and marked for removal.

#### Scenario: Direct window access

- GIVEN a component reaching into `window` outside the gateway
- WHEN the lint rule runs
- THEN the lint fails citing the gateway contract

### Requirement: Workspace Controller

`App.tsx` MUST return to being a composition root. It MUST NOT contain business logic beyond wiring providers, the gateway, the workspace controller, and layout composition.

#### Scenario: Workspace transition remains in App.tsx

- GIVEN a PR adds mode, selection, or dirty-transition behavior directly to App.tsx
- WHEN the architecture review runs
- THEN the review fails until that behavior is covered and moved behind the workspace controller

### Requirement: Scene Session

The SceneDocument, OperationLog, and the dirty/switch invariants MUST be encapsulated in a single `scene-session` module. Outside callers MUST access state through the module's public surface, not raw mutable references.

#### Scenario: External mutation

- GIVEN code that mutates SceneDocument outside the session module
- WHEN the typing check runs
- THEN the check fails because the field is private to the module

### Requirement: Hito 8 Readiness

The agent runtime MUST be transport-neutral and proposal-first. It MUST NOT depend on `axum`, `ai-proxy`, or `editor-core`; HTTP mapping remains in `ai-proxy`, while approved operations remain owned by editor adapters.

#### Scenario: Forbidden dependency

- GIVEN a PR adds `axum`, `ai-proxy`, or `editor-core` to the agent runtime crate
- WHEN CI runs the dependency policy check
- THEN CI fails citing the proposal-first rule

## References

- [ADR-0027](../adr/0027-rig-agentic-editor-architecture.md): agent runtime boundaries
- [ADR-0028](../adr/0028-workflow-first-before-agentic-ai.md): prerequisite sequencing
- `../roadmaps/application-stabilization-roadmap.md` — current sequence
- `../ROADMAP.md` — milestone map
- `../../CONTEXT.md` — canonical glossary
