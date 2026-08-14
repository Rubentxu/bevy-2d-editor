# Application Stabilization and Roadmap Convergence

This program restores a trustworthy release baseline after v0.86.0, converges
project documentation, and creates the minimum architectural seams required
before Hito 8 implementation begins.

## Current Status

| Field             | State                                                             |
| ----------------- | ----------------------------------------------------------------- |
| Program           | In progress                                                       |
| Baseline          | `main` at `91203b6` after v0.86.0                                 |
| Current work unit | A1 + A2 + A3 verified locally; CI run and release remain          |
| Hito 8            | Blocked until the release-health gate passes                      |
| Durable contract  | `docs/specs/application-stabilization-and-roadmap-convergence.md` |
| Prior debt branch | `debt-backup-ui-workflow-overhaul-pr4` (selective replay only)    |

## Verified Baseline

The 2026-08-02 audit established the starting point:

- Rust integration tests do not compile because several `Entity` fixtures omit
  the required `local_id` field.
- Frontend lint passes, while format checking fails across 33 files.
- Production build emits 369.80 KB gzip of JavaScript against the current
  350 KB total-JavaScript budget.
- Playwright discovers 472 tests but the full run is not deterministic; failures
  include editor readiness, WASM bridge, OPFS, selectors, and accessibility.
- The repository advertises CI, but no `.github/workflows/ci.yml` exists.
- ROADMAP, addendum, README, CHANGELOG, and CONTEXT contain conflicting status.

These facts supersede historical pass counts until new evidence is recorded.

## Execution Order

### Wave A - Release Health

| PR  | Work unit                                                     | Status       | Required evidence                                                          |
| --- | ------------------------------------------------------------- | ------------ | -------------------------------------------------------------------------- |
| A1  | Restore Rust fixtures and introduce the initial CI workflow   | In progress  | Workspace tests and wasm32 check pass locally; CI run remains              |
| A2  | Normalize Rust/frontend formatting and frontend static checks | Done locally | Rust format, frontend format, lint, and typecheck pass locally; CI added   |
| A3  | Define and enforce the performance budget contract            | Done locally | Initial JS 127.79 KB, total JS 368.57 KB, WASM 14.76 MB measured; CI added |

Rules:

- A1 MUST repair fixtures without redesigning `Entity`.
- A2 MUST remain behavior-neutral so formatting does not hide product changes.
- A3 MUST optimize or redefine a metric through an ADR; it MUST NOT silently
  increase the existing threshold.

### Wave B - Test Reliability

| PR  | Work unit                                                                         | Required evidence                                   |
| --- | --------------------------------------------------------------------------------- | --------------------------------------------------- |
| B1  | Replace fragmented readiness polling with one editor-ready contract               | Smoke tests cannot call a half-initialized bridge   |
| B2  | Split Playwright into smoke, domain, persistence, accessibility, and full cohorts | Three consecutive deterministic smoke runs          |
| B3  | Replay valid PR4 debt fixes onto current `main`                                   | Real clicks/assertions pass; no null-event bypasses |

Diagnostic order for B2:

1. Run affected tests with one worker.
2. Classify readiness, shared storage, stale test, and product failures.
3. Isolate state before restoring parallel workers.
4. Keep retries disabled while diagnosing.
5. A skip requires explicit rationale and a tracked unblock condition.

### Wave C - Documentation Convergence

| PR  | Work unit                                                           | Required evidence                                                |
| --- | ------------------------------------------------------------------- | ---------------------------------------------------------------- |
| C1  | Reconcile ROADMAP, README, CHANGELOG, CONTEXT, specs, and ADR index | All surfaces report the same current version and next change     |
| C2  | Define documentation ownership and drift checks                     | `docs-check` validates tags, ADR index, active cycles, and links |

The source-of-truth hierarchy is:

| Artifact                        | Owns                               |
| ------------------------------- | ---------------------------------- |
| `CONTEXT.md`                    | Domain language                    |
| `docs/adr/`                     | Decisions and trade-offs           |
| `docs/specs/`                   | Durable behavior                   |
| `docs/ROADMAP.md`               | Current status and execution order |
| `docs/roadmaps/`                | Forward delivery programs          |
| `CHANGELOG.md`                  | Published release history          |
| `README.md` and `USER_GUIDE.md` | Public entry and user workflows    |
| `sddk/`                         | Local cycle provenance             |
| `docs/sddk/`                    | Frozen pre-policy history          |

### Wave D - Architecture Seams

| PR  | Work unit                                                                    | Boundary created                                                |
| --- | ---------------------------------------------------------------------------- | --------------------------------------------------------------- |
| D1  | Introduce a typed, injectable `EditorGateway`                                | Frontend no longer has three competing WASM access paths        |
| D2  | Extract `useEditorWorkspaceController` and command catalog                   | `App.tsx` returns to composition-root responsibilities          |
| D3  | Encapsulate active document, Operation Log, dirty state, and scene switching | One Scene Session invariant boundary                            |
| D4  | Move cohesive scene-facing WASM adapters behind a stable facade              | `editor-core/lib.rs` stops owning scene workflow implementation |

Wave D constraints:

- Do not introduce a new frontend state-management framework.
- Do not change persisted JSON, OPFS layout, or exported WASM names.
- Do not merge Scene, Scene Asset, and LogicGraphAsset operation logs.
- Every extraction starts with characterization tests and preserves behavior.

### Wave E - Hito 8 Readiness

Before implementation, refresh `rig-agent-runtime-foundation` so that:

- `agent-runtime` is transport-neutral and does not depend on `axum`,
  `ai-proxy`, or `editor-core`;
- `ai-proxy` retains HTTP DTOs, policy boundary, and response mapping;
- agents emit staged proposals rather than mutating browser-owned state;
- approved operations return through `EditorGateway` and typed editor commands;
- the foundation scope is manager, lifecycle, registry, policy contracts, and the
  minimum specialists needed to prove delegation and compatibility.

## Dependency Graph

```text
A1 -> A2 -> A3
 |           |
 +----> B1 -> B2 -> B3
               |
               +----> C1 -> C2
                         |
                         +----> D1 -> D2
                         +----> D3 -> D4
                                      |
                                      +----> Wave E / Hito 8
```

Documentation work may be prepared earlier, but C1 records measured outcomes
from Waves A and B and therefore closes after those waves.

## Release-Health Gate

Hito 8 becomes executable only when all mandatory checks pass on the same commit:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-targets --release --locked
cargo check -p editor-core --target wasm32-unknown-unknown
cd frontend && npm run format:check
cd frontend && npm run lint
cd frontend && npx tsc --noEmit
cd frontend && npm run build:check
cd frontend && npx playwright test <smoke-cohort>
cd frontend && npx playwright test <required-domain-and-persistence-cohorts>
```

The full suite and accessibility cohort also run before a stabilization release.

## Traceability

| Program wave | Durable requirement                               | Existing decision or evidence               |
| ------------ | ------------------------------------------------- | ------------------------------------------- |
| A            | Release-Health Gates; Performance Budget Contract | ADR-0025 and current build evidence         |
| B            | Editor Readiness and E2E Cohorts                  | ADR-0017 and PR4 debt artifacts             |
| C            | Documentation Convergence                         | ADR-0028 sequencing and PR #127 remediation |
| D1-D2        | Editor Gateway; Workspace Controller              | Workflow convergence specs                  |
| D3-D4        | Scene Session                                     | ADR-0001, command system, Operation Log     |
| E            | Hito 8 Readiness                                  | ADR-0027 and ADR-0028                       |

## Out of Scope

- New AI-native product features before the gate passes.
- Collaborative editing, plugin marketplace, voice, or mobile UI.
- Visual redesign unrelated to reliability.
- Replacing OPFS, React, Bevy, CodeMirror, or Logic Bricks.
- Broad renames or speculative abstractions without a failing test or boundary need.

## Completion

The program completes when Waves A-D pass their evidence gates, documentation
matches the measured release state, and the corrected Hito 8 design is ready for
implementation. Completion unblocks `rig-agent-runtime-foundation`; it does not
silently include that feature work.
