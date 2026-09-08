# G6 — Performance corpus: specification

**Cycle**: v0.110.3 candidate (A-lite)
**Generated**: 2026-09-08
**Status**: Draft for Specify phase

## 1. Goal

Define a deterministic Playwright performance corpus that proves v1.0 is
shippable at the canonical content sizes documented in
`docs/roadmaps/v1.0-stabilization.md` §5.2.

## 2. Scope

**In scope:**

- Six Playwright specs, one per benchmark listed in `v1.0-stabilization.md` §5.2.
- One Playwright config file: `frontend/playwright.performance.config.ts`.
- Documented soft + hard budgets per spec.
- One helper file: `frontend/tests/helpers/perfBudget.ts` to wrap the
  measurement + assertion logic.

**Out of scope (deferred to a future cycle):**

- Native `criterion` benchmarks for the Rust internals. See handoff for rationale.
- Cold-startup time of the editor (separate from the perf corpus).
- Smoke-cohort 60 s budget fix (carry-forward from pre-baseline).

## 3. The six benchmarks

Each benchmark is declared with three numbers:

- **Setup overhead (informational)**: time spent *before* measurement
  (e.g., mounting a 10k entity project). Not asserted.
- **Soft budget (warning)**: above this the spec logs a warning.
- **Hard budget (fail)**: above this `expect()` fails.

Defaults follow the existing 50-entity roundtrip at `engine.spec.ts:526`,
which today runs in roughly 1.2 s wall clock on local Chromium. For new
budgets we use a `2× linear` extrapolation plus 50% headroom for CI flakiness.

### 3.1 P1 — 10k entities roundtrip

- **Operation**: mount a prebuilt 10k-entity scene via OPFS-init → load → snapshot → save → reload → snapshot assert.
- **Soft budget**: 8 s
- **Hard budget**: 20 s
- **Cohort**: `@performance`
- **Why matters**: the v1.0-stabilization roadmap explicitly names "10k entities".

### 3.2 P2 — Large tile level paint/erase

- **Operation**: paint 200 tiles on a 100×100 grid, then erase 200 tiles.
- **Soft budget**: 12 s paint, 12 s erase
- **Hard budget**: 25 s paint, 25 s erase
- **Cohort**: `@performance`
- **Why matters**: paint/erase is the dominant user interaction in tile-level authoring.
- **Empirical baseline**: paint 200 cells currently measures ~13 s on local Chromium (65 ms/cell through the bridge). The 25 s hard budget leaves 2× headroom for CI; the 12 s soft budget flags regressions past the local measurement.

### 3.3 P3 — Multi-level world navigation

- **Operation**: 50-scene world. Open the world, switch to scene 25, back to scene 1, to scene 49. Measure average switch latency.
- **Soft budget**: 0.5 s average switch
- **Hard budget**: 1.5 s average switch
- **Cohort**: `@performance`
- **Why matters**: per the `world-workspace.spec.ts` file, multi-level worlds are a v1.0 first-class artifact.

### 3.4 P4 — 500-asset catalog listing

- **Operation**: project with 500 asset files (scripts + tilesets + scenes). Trigger the asset navigator panel → list all 500.
- **Soft budget**: 1 s render after list completes
- **Hard budget**: 3 s render after list completes
- **Cohort**: `@performance`
- **Why matters**: assets drive the editor's largest content type.

### 3.5 P5 — 200-node logic graph dispatch

- **Operation**: build a 200-node logic graph, dispatch a BeginPlay event, measure mean command latency over 50 dispatches.
- **Soft budget**: 50 ms mean dispatch
- **Hard budget**: 200 ms mean dispatch
- **Cohort**: `@performance`
- **Why matters**: per `logic-graph-persistence.spec.ts`, large graphs are realistic.

### 3.6 P6 — 1000-file project source listing (proxy for "project search")

- **Operation**: 1000 source files in the project. Trigger `list_source_files()` (the closest bridge analog to "index walk"; `global_search` does not exist as a bridge today) and measure its wall time.
- **Soft budget**: 1 s
- **Hard budget**: 3 s
- **Cohort**: `@performance`
- **Why matters**: per `engine-bridge.ts` only `find_source_location(typeId)` exists for source queries; we choose the closest available bridge (`list_source_files`) so the spec runs against real code today. Documented limitation; once a `global_search` bridge is exposed, this spec should be re-scoped to call it.

## 4. Cohort + tagging rules

| Tag | Meaning | Where it appears |
|---|---|---|
| `@performance` | Runs only in the performance cohort (`playwright.performance.config.ts`) | `tests/perf-*.spec.ts` |
| `@full` | Runs in the full cohort (superset tag) — `playwright.full.config.ts` | `tests/perf-*.spec.ts` |

This means: perf specs DO run when an operator selects the full cohort
(they get included as `@full`-tagged specs). They are optional in normal CI
flows (the default `@smoke` + `@domain` + `@persistence` + `@a11y`).
Operators may opt in to the perf cohort via `npm run test:perf`.

### 4.1 `package.json` script

Add a script:
```json
"test:perf": "playwright test --config=playwright.performance.config.ts"
```

### 4.2 CI workflow

The `ci.yml` workflow should run `@performance` on a nightly schedule
(separately from PR-time checks), because:
- The cohort takes 2–5 minutes.
- Failure is not blocking for merge (it's a budget regression, not a bug).

This is a follow-up CI change and is documented but not implemented in this cycle.

## 5. Measurement helper

`frontend/tests/helpers/perfBudget.ts` exports:

```typescript
export interface PerfBudgetSpec {
  /** Display name for the assertion message. */
  name: string;
  /** Soft budget in ms — exceeded logs a warning but does not fail. */
  softMs: number;
  /** Hard budget in ms — exceeded fails the test. */
  hardMs: number;
}

export async function measureMs(fn: () => Promise<void>): Promise<number>;

export async function assertWithinBudget(
  spec: PerfBudgetSpec,
  fn: () => Promise<void>,
): Promise<number>;
```

`measureMs(fn)` returns the wall-clock time `fn` took in ms.
`assertWithinBudget(spec, fn)` runs `fn`, compares wall time to soft + hard,
and `expect().toBeLessThan(hardMs)` for hard budget (asserting against soft is
advisory only — soft is logged via `console.warn`).

## 6. Coverage acceptance

The cycle's verify phase will:
1. Start `npm run build` (assumed already-built for development; `npx vite build` for tests).
2. Run `npm run test:perf`.
3. Capture wall times per spec into a perf report (committed alongside `verify-report.md`).
4. If any spec exceeds its hard budget on the canonical runner, the cycle FAILS until the perf regression is fixed.

## 7. Risks + mitigations

| Risk | Mitigation |
|---|---|
| WASM rebuild inflates the cold-start timer | Hard budgets explicitly exclude cold-start. Each spec calls `waitUntilReady` first. |
| CI runner is slower than local Chromium | Hard budgets use 2× local measurement + 50% headroom. |
| Test data (10k entities, 200-tile grids) requires deterministic generation | Use `page.evaluate()` to call engine APIs deterministically, not UI clicks. |
| Specs may timeout on the 30-entity browser because of WSL2 / container limits | Use `expect.poll()` for the budget assertion rather than `setTimeout(timeoutMs)`. |

## 8. What is NOT in this cycle

- No `examples/platformer-minimal/` mutation (the canonical content is frozen).
- No `criterion` Rust benchmark (different layer; future cycle).
- No CI wiring (the budget regressions are not merge-blocking; the cohort runs nightly).
- No cold-start budget (separate spec, separate cycle).
- No smoke-cohort 60 s fix (pre-existing carry-forward).
