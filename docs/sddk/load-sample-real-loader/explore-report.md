# Exploration Report — load-sample-real-loader (cycle 389)

## Carry-forward origin

This cycle closes the **first** of the two v0.110.6 carry-forwards
(marked P3 in `tutorial-walkthrough`'s release-receipt §Open Items /
Follow-ups and v0.110.7's §Carry-forward):

> "Wire `__loadSampleProject` to a backend OPFS loader — the bridge
>  body in `TutorialStepper.tsx` is currently a stub returning
>  `{ok:true}` after 100 ms. Real implementation lands in a follow-up
>  cycle (fetch + OPFS write + engine reload)."

This is also the natural complement to `tour-completed-persistence`
(v0.110.7): the latter persists *that* the user took the tour; this
cycle makes the tour's first action (`__loadSampleProject("platformer-minimal")`)
actually load the sample.

## Why A-lite (not A-min)

The implementation requires:

1. **Network fetch** of JSON files (browser `fetch()` from
   `/examples/platformer-minimal/...`) — VIte serves the sample from
   `examples/` at the dev server. No backend involved.
2. **OPFS writes** for ~9 files at canonical paths
   (`project.json` + scenes + schemas + assets + logic_graphs).
3. **Bridge to engine reload** — `window.load_project()` is already
   exposed by `engine-bridge.ts:150`. No new Rust/WASM work.
4. **Refactor** the duplicated `OPFS_FILES` table from
   `tests/e2e-game-creation.spec.ts` + `tests/git-friendly-roundtrip.spec.ts`
   into a shared `frontend/tests/helpers/sample-loader.ts` (also consumed
   by the production loader).
5. **Tests** verifying the loader end-to-end (1 scenario × 2 projects)
   + verifying the existing tests still pass after the table extraction.

A-min is reserved for UI plumbing that touches ≤ 3 src files. This
cycle touches:

- NEW `frontend/src/services/sampleLoader.ts` (~80 LOC; production loader)
- NEW `frontend/tests/helpers/sample-loader.ts` (~80 LOC; shared test helper
  that mirrors the production loader's `OPFS_FILES` mapping; DEDUPed
  from both spec files)
- MODIFIED `frontend/src/components/TutorialStepper.tsx`
  (replaces stub body with `await mountPlatformerMinimal()`)
- MODIFIED `frontend/tests/e2e-game-creation.spec.ts`
  (delete the local `mountSample`, import the shared one)
- MODIFIED `frontend/tests/git-friendly-roundtrip.spec.ts`
  (same)

5 files. **A-lite** is the right rung.

## Existing scaffolding to reuse

| Pattern | File | Note |
| --- | --- | --- |
| `window.load_project` test bridge | `frontend/src/engine-bridge.ts:150` | Already exposed via `engine-bridge.ts` AFTER `init_project_store` completes. |
| `OPFS_FILES` mapping table | `frontend/tests/e2e-game-creation.spec.ts:40-71` and `frontend/tests/git-friendly-roundtrip.spec.ts:35-66` | Duplicated. The cycle will consolidate. |
| `opfs_save_file` window bridge | `frontend/src/engine-bridge.ts:85` | Already exposed BEFORE `init_project_store`. Reuse directly (don't go through `opfsSaveFile` since the bridge is the production interface). |
| `fetch()` of `examples/platformer-minimal/...` | dev-server | Vite serves `examples/` at `/examples/...` in dev. Production (built) deployment would need a different fallback — but the v1.0 evidence P1 is dev-only anyway. |
| Stale-sample warning UX | none | Out of scope. The loader will overwrite silently; that's the existing UX for `import-asset-btn` and is fine for the tutorial. |

## Architecture delta

None. The cycle is bounded UI plumbing; no architecture-impact boundary
is declared.

## Open Question (resolve in Specify)

**Path A (recommended): the production loader is a thin wrapper around
the shared `OPFS_FILES` mapping + `fetch()`. The shared module lives in
`frontend/src/services/sampleLoader.ts` (production) AND its mapping
table is *exported* so the test helper can import the same const.**

This makes the loader and the test helper a single source of truth: if a
new sample file is added, only one place changes. The test helper
becomes:

```ts
// frontend/tests/helpers/sample-loader.ts
import { OPFS_FILES, mountPlatformerMinimal } from "../../src/services/sampleLoader";
// Helper that runs in Node (test) uses readFile + opfs_save_file instead of fetch.
export async function mountSampleInOpfs(page: Page) {
  for (const { opfsPath, localPath } of OPFS_FILES) {
    const contents = await readFile(
      path.resolve(import.meta.dirname, "../../..", "examples/platformer-minimal", localPath),
      "utf-8",
    );
    await page.evaluate(
      async ({ p, c }) => await (window as any).opfs_save_file(p, c),
      { p: opfsPath, c: contents },
    );
  }
}
```

**Path B (rejected): the loader uses `import.meta.glob` or another
build-time mechanism to bundle the JSON. Rejected because:

- `examples/platformer-minimal/` is at the repo root, not inside `frontend/`.
- Vite refuses to import outside the `frontend/` tree via `import`.
- `fetch()` is the canonical way to read repo-relative assets in Vite dev.

So Path A is the only viable answer.

## Architectural Decision: production vs test source

The production loader must run **in the browser**. The test helper runs
**in Node** (Playwright runner) but writes via `page.evaluate` to OPFS in
the browser. They share the *mapping table* but differ in *how* they get
the file contents:

- production: `fetch('/examples/platformer-minimal/<file>')` then
  `window.opfs_save_file(path, text)`;
- test helper: `readFile(absolute, 'utf-8')` (Node-side) then
  `page.evaluate(window.opfs_save_file, ...)` to write in browser.

The cleanest split:

- `frontend/src/services/sampleLoader.ts` exports `OPFS_FILES` and
  `mountPlatformerMinimal()` (production).
- `frontend/tests/helpers/sample-loader.ts` exports
  `mountSampleInOpfs(page)` that imports `OPFS_FILES` from the production
  module and reads files via Node `fs/promises`.

## Files in scope (predicted)

| Path | Predicted change |
| --- | --- |
| `frontend/src/services/sampleLoader.ts` (NEW) | production loader (~80 LOC); EXPORTS `OPFS_FILES` const + `mountPlatformerMinimal()` async |
| `frontend/src/components/TutorialStepper.tsx` | `__loadSampleProject` body switches from `setTimeout` stub to `await mountPlatformerMinimal()` |
| `frontend/tests/helpers/sample-loader.ts` (NEW) | Node-side helper that mirrors `mountPlatformerMinimal` (uses `readFile` instead of `fetch`) |
| `frontend/tests/e2e-game-creation.spec.ts` | delete local `OPFS_FILES` + `mountSample`; import from `helpers/sample-loader` |
| `frontend/tests/git-friendly-roundtrip.spec.ts` | same |
| `frontend/tests/load-sample-real-loader.spec.ts` (NEW) | 1 scenario × 2 projects = 2 tests for the production loader |

## Cost

- A-lite: ~1 day.
- 6 files: 3 new (1 src service, 1 test helper, 1 spec) + 3 modified
  (TutorialStepper + 2 existing specs deduped).
- 2 new Playwright tests + 2 × existing test sets still pass.

## Risks

- The canonical project layout in the sample uses `scene-assets/...` not
  `assets/...`. If a future contributor adds a new asset file to the
  sample but only writes it to `assets/`, the loader will skip it
  silently. Mitigation: the implementation-receipt should call this out
  explicitly.
- Vite dev-server is required for `fetch()`. If the cycle is exercised
  in production (built) deployment, the path will 404. The cycle's
  acceptance gate must explicitly check that the loader is *dev-only*
  or wires a production fallback. Per the existing scope, dev-only is
  OK.

## Confidence

- Mapping table proven across 2 existing specs.
- `window.load_project` bridge proven (`engine-bridge.ts:150`).
- `opfs_save_file` window bridge proven (`engine-bridge.ts:85`).

Exploration gate ready (`exploration-sufficient`).
