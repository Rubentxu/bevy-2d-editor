# G6 — Performance corpus: exploration report

**Cycle**: v0.110.3 candidate (A-lite)
**Generated**: 2026-09-08
**Author**: planning agent against HEAD `721e25e` (G5 archive)

## Why this cycle

G6 is the **last Red gate** of v1.0-stabilization. Per
[`docs/v1.0-stabilization-evidence-map.md`](../v1.0-stabilization-evidence-map.md) §6.1,
no performance corpus exists. The roadmap (`docs/roadmaps/v1.0-stabilization.md` §5.2)
explicitly lists six benchmarks; none are wired.

## Inventory of pre-existing perf-related code (today, HEAD `721e25e`)

### Engine spec — 50-entity roundtrip (already exists)

`frontend/tests/engine.spec.ts:525-?` declares
`save_scene and load_scene roundtrip with 50 entities` in the `@full` cohort.
This is the **seed pattern**: it builds 50 entities programmatically, saves,
reloads, asserts. G6 can scale this to 10k for the new budget.

### Other potentially-relevant specs

- `engine.spec.ts` lines 744: `register schema → save schema → reload → load_project` (no entity count budget)
- `engine.spec.ts` lines 526: 50-entity persistence roundtrip (above)
- `auto-layer.spec.ts`, `tileset.spec.ts` — exist (no size budget)
- `asset-pipeline.spec.ts` — exists (no size budget)
- `world-workspace.spec.ts` — exists (no boundary budget)
- `logic-graph-persistence.spec.ts` — exists (no complex-graph budget)
- `global-search-actions.spec.ts`, `ux-search.spec.ts` — exist (no large-project budget)

### Test cohorts / configs

| Config | Tag | Notes |
|---|---|---|
| `playwright.smoke.config.ts` | `@smoke` | 60 s budget (already breached; engine.spec.ts should be moved out — separate cycle) |
| `playwright.full.config.ts` | `@full` | Superset of all tags — perf cohort should also use this for fallback |
| `playwright.domain.config.ts` | `@domain` | Feature-domain coverage |
| `playwright.persistence.config.ts` | `@persistence` | OPFS flows |
| `playwright.a11y.config.ts` | `@a11y` | axe-core |

**No `playwright.performance.config.ts` exists.** G6 needs to create it.

### Helper / measurement infrastructure

- `frontend/tests/helpers/waitForEditorReady.ts` — single readiness signal (`__bevyEngineStarted === true`).
- `performance.now()` is the only built-in timer available in Playwright specs.
- Playwright `test.setTimeout(120_000)` and `expect.poll()` are available for budget assertions.

No `criterion` in `Cargo.toml`. No native rust benchmark. G6 will be
**Playwright-only** and front-load the full path (browser → WASM bridge → OPFS).

### Canonical content samples

- `examples/platformer-minimal/` — the canonical playable sample game created in G1. Contains:
  - multiple scenes
  - scene assets with components
  - a logic graph (small)
  - one tile-level project (`world.workspace.json` exists)
  - several `.tmx` tilesets
- `examples/` directory also has additional importers (aseprite, ldtk, tiled).

## Constraints observed

1. **WASM rebuild is slow.** A complete `npm run build` followed by `npx vite` startup can
   take 10–20 s. The performance specs MUST NOT measure cold-startup time of the editor.
   They MUST `waitUntilReady` first.

2. **OPFS persistence over `__bevyEngineStarted === true` is acceptable** as long as the
   wasm bridge has hydrated (and `init_project_store` ran). The `__bevyEngineStarted`
   flag is set AFTER `start_engine` returns, which is after hydrate; we used the same
   pattern in `crash-recovery.spec.ts`.

3. **Local runs vs CI**: a 10k entity roundtrip might take 30 s locally and 5 s in CI
   (or vice versa). Budgets MUST include a `performance.now()` + a generous
   `soft budget` (warning) and a `hard budget` (fail). Default to 2× the measured
   value from the first stable run.

## Discoveries

### `__bevyEngineStarted` readiness signal

Looking at `engine-bridge.ts:__bevyEngineStarted`, the flag is set after
`start_engine` returns and Bevy App has finished its first frame. The
existing `waitForEditorReady` helper uses this exactly:
```typescript
await page.waitForFunction(
  () => (window as any).__bevyEngineStarted === true,
  undefined,
  { timeout: timeoutMs },
);
```

### Bridge access patterns

For batch operations (bulk entity creation, bulk asset catalog listing,
bulk project search) the bridge exposes:
- `opfs_save_file(path, contents)` — single-shot write
- `get_scene_snapshot()` — JSON snapshot of the current scene
- `add_entity(name, components)`, etc.

A 10k entity test needs to use the **engine API directly** (e.g.
`bulk_create_entities(5000)` or similar), NOT 10k Playwright clicks.

### Existing 50-entity pattern

The seed pattern at `engine.spec.ts:525` uses `add-entity-btn` clicks. For 10k
that would be 10k clicks → minutes. Better: use `add-entity-btn` in a loop
with `evaluate()` to bypass the UI:
```typescript
for (let i = 0; i < 10000; i++) {
  await page.evaluate(() => (window as any).engine_dispatch_command('add_entity', {...}));
}
```

Or even better: prebuild a 10k-entity scene via direct OPFS write on
`init_project_store`, then measure the load time.

## Required follow-up questions for the user

None blocking. Open question (mentioned in handoff): whether to add a
`criterion` cargo benchmark alongside the Playwright one. **Recommend
Playwright-only for v0.110.3** and revisit criterion only if v1.0 needs
lower-level budgets (most v1.0 budgets involve the full bridge round-trip,
so Playwright is appropriate).

## Next phase outputs expected

The **specification** phase must declare:
- 6 budgets with named soft + hard limits.
- A `playwright.performance.config.ts` that runs the cohort.
- Cross-cohort registration rules (cohort overlap with @full).
- Acceptance: each spec times + asserts against the soft + hard budget.
