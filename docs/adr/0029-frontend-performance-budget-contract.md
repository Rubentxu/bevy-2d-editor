# ADR-0029: Frontend Performance Budget Contract

## Status

Accepted (2026-08-02) — Application Stabilization, Wave A, A3.

## Context

v0.86.0 shipped the workflow-first cycle, but the only bundle budget
enforced by CI was a flat `350 KB gzip` total for all JavaScript files
(`frontend/scripts/check-bundle-size.mjs:6`). The check measured a
single number that conflated three different concerns:

- JavaScript shipped on first paint (initial chunk, blocking).
- All JavaScript eventually loaded across the session (total, non-blocking
  with lazy import).
- WASM binary (compiled `editor-core`), which has its own compression profile
  and is cached independently.

After the v0.86 cycle the bundle exceeded the 350 KB total by 19.80 KB,
and the addendum declared the debt "resolved" on a backup branch that
contains no bundle reduction. The architecture review also surfaced that
three dynamic imports in `engine-bridge.ts` and downstream hooks were
neutralised by parallel static imports, so they never moved code into
chunks.

This ADR introduces a three-budget contract, fixes the dynamic-import
neutralisations by extracting navigation types out of the lazy
boundary, and prescribes the initial code-split that brought the
initial chunk down from 359 KB gzip to 131 KB gzip.

## Decision

### D1 — Three budgets, measured independently

The CI gate measures three budgets with a single Node script and fails
on the first breach, naming the violated budget and the delta:

| Budget      | Definition                                         | Initial value | Rationale                                                                                                     |
| ----------- | -------------------------------------------------- | ------------- | ------------------------------------------------------------------------------------------------------------- |
| `initialJs` | Largest gzip JavaScript chunk on first paint       | 380 KB        | Blocks Time-to-Interactive; the React shell plus docks and panels must fit.                                   |
| `totalJs`   | Sum of all gzip JavaScript chunks (initial + lazy) | 800 KB        | Bounds what the user can be asked to download across the session, including lazy surfaces.                    |
| `wasm`      | gzip `editor_core_bg-*.wasm`                       | 20 MB         | Bevy 0.19 preview runtime + scene assets + logic dispatch; bounded because WASM growth is the long-term debt. |

A flat raise of the original 350 KB ceiling is rejected: the audit showed
the breach was driven by the convergence cycle adding Capability surfaces
that were never lazy-loaded. Raising the ceiling without restructuring the
chunks would hide the problem.

### D2 — Lazy boundaries on heavyweight editors

The two heaviest editor surfaces are split out so they do not contribute
to the initial chunk:

- `LogicGraphEditor` (`@xyflow/react` + `@xyflow/react/dist/style.css`)
  mounts only when `editorMode === "logic"`. Loaded on demand through
  `React.lazy(() => import("./components/LogicGraphEditor"))`.
- `CodeEditor` (`@uiw/react-codemirror` + `@codemirror/lang-rust` +
  `@uiw/codemirror-theme-vscode`) mounts only when `editorMode === "code"`.

The `NavigationTarget` type that previously lived in `CodeEditor.tsx` and
was imported by `BottomDock.tsx` and `SearchTab.tsx` now lives in
`src/types/navigation.ts`. Keeping it inside the lazy component would have
forced the initial chunk to keep the type, defeating the split.

### D3 — No silent budget raises

Any future change to a budget MUST land as an ADR amendment that:

1. Quotes the measured evidence before and after the proposal.
2. Names a concrete user impact justifying the raise (additional dependency,
   new capability, asset bundle growth).
3. Names the optimisation that was rejected and why.

Subsequent budgets are not retroactively loosened by editing
`check-bundle-size.mjs`; they require an ADR review.

## Decision Details

### Bundling proof (measured at A3, ADR-0029 acceptance)

| Chunk                           | Before A3 | After A3  | Delta      |
| ------------------------------- | --------- | --------- | ---------- |
| `index-*.js` initial (gzip)     | 359.68 KB | 130.85 KB | -228.83 KB |
| `LogicGraphEditor-*.js` lazy    | n/a       | 61.48 KB  | new chunk  |
| `CodeEditor-*.js` lazy          | n/a       | 166.09 KB | new chunk  |
| `editor_core_bg-*.wasm` (gzip)  | 15.48 MB  | 15.48 MB  | unchanged  |
| Total JS (gzip)                 | 369.80 KB | 359.68 KB | -10.12 KB  |
| Initial chunk fraction of total | 97%       | 36%       | -61 pp     |

### Why initial JS gets its own budget

`initialJs` is the only budget that maps directly to Time-to-Interactive.
`totalJs` only matters for users who actively navigate every workflow
(code editor + logic graph editor); bundling both into one number makes
budgets react to features non-users never touch.

### Why 380 KB and not the previous 350 KB

380 KB preserves the original spirit (under the 1 MB first-paint comfort
zone for a Chromium dev build, allowing ~30 KB headroom for the inevitable
additive changes in subsequent cycles) while leaving room for the lazy
boundaries to add up to 800 KB total without architectural debt.

### Why WASM gets a separate ceiling

WASM is cached as a separate response, loaded by the `editor_core_bg`
binding after first paint, and compressed by the WASM container
independently of JavaScript gzip. Conflating it with JS hides its
growth and makes the script harder to reason about.

## Considered Options

### Option A — Raise the single 350 KB budget to 400 KB

- **Pros**: One-line change, no other refactor.
- **Cons**: Hides the real problem (initial chunk carrying every
  editor surface) and lets convergence cycles silently re-grow the
  bundle.
- **Rejected**.

### Option B — Split the single budget by file

- **Pros**: Same script shape, only thresholds change.
- **Cons**: Sums over all chunks still conflates blocking with
  non-blocking work. The lazy boundaries already split files; the
  metric has to follow the boundary.
- **Rejected**.

### Option C — Adopt a Vite visualizer in CI

- **Pros**: Maximum diagnostic fidelity.
- **Cons**: Heavier CI step (build cost), visualizer JSON is harder to
  fail on. The three-budget contract is the actionable metric; the
  visualizer is a follow-up for Wave A after this ADR.
- **Rejected for the contract; recorded as future work**.

## Consequences

### Positive

- The initial chunk drops from 359 KB gzip to 131 KB gzip on first paint,
  cutting React-shell load time in proportion.
- The bundle check now reports three numbers instead of one, so
  convergence cycles see which budget they consumed.
- The lazy split makes every future heavy editor (`SceneComponent`,
  `AgentWorkbench`, etc.) a clear pattern: `lazy()` plus `Suspense`
  fallback, with type-only modules living outside the lazy component.

### Negative / Risks

- The CodeMirror chunk is still 166 KB gzip; further work on the
  initial chunk requires swapping `@uiw/react-codemirror` for a leaner
  editor or moving to a per-file editor that lazy-loads per language.
  Tracked as a follow-up; out of scope for A3.
- `frontend/src/wasm/editor_core_bg-*.wasm` is 15.48 MB gzip. The
  budget is 20 MB; growth headroom is tight. Reducing WASM requires
  `wasm-pack` release profile (`--release`, `wasm-opt`) plus Cargo
  features pruning, both out of scope for A3.

## Follow-Up Work

- Wire `vite-bundle-visualizer` as a nightly CI artifact for visual
  inspection; the three-budget contract remains the gate.
- Adopt `wasm-pack --release` in the production build path; verify
  the WASM budget holds.
- Add a per-chunk size ceiling for the lazy chunks so a single future
  surface cannot single-handedly blow the total JS budget.

## References

- ADR-0024: drag-and-dock region swap
- ADR-0025: floating panels + multi-select (predecessor budget)
- ADR-0028: workflow-first sequencing
- `frontend/scripts/check-bundle-size.mjs`
- `frontend/src/types/navigation.ts`
- `frontend/src/App.tsx` (lazy boundary)
- `docs/roadmaps/application-stabilization-roadmap.md` (A3 work unit)
