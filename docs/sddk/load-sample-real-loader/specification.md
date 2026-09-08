# Specification — load-sample-real-loader (cycle 389)

**Cycle:** load-sample-real-loader
**Path:** A-lite
**Sequence:** 389 (cycle start)
**Carries forward from:** v0.110.6 (tutorial-walkthrough) carry-forward #1
**Base:** v0.110.7 = `47117dfcfbc23965efffdefe3ed8403588908546` (HEAD, archive of tour-completed-persistence)
**Status at cycle start:** 9 ✅ / 0 🟡 / 0 🔴 (v1.0 product gates all green)

## Goal

Replace the `__loadSampleProject` stub in `TutorialStepper.tsx`
(resolving to `services/tour.ts`'s placeholder) with a real loader that
fetches `examples/platformer-minimal/*.json` from the Vite dev server,
writes each file to OPFS at the canonical project path, then invokes
`window.load_project()` to hydrate the engine. Extract the duplicated
`OPFS_FILES` mapping table from two test specs into a shared helper so
production and tests share one source of truth.

## Scope

### REQ-1: New production service module
- **Where:** `frontend/src/services/sampleLoader.ts` (NEW)
- **Exports:**
  - `OPFS_FILES`: `ReadonlyArray<{ opfsPath: string; localPath: string }>`
    — the canonical mapping table
  - `mountPlatformerMinimal(): Promise<{ ok: boolean; written: number; errors: string[] }>`
- **Behaviour:** for each entry in `OPFS_FILES`, fetch
  `/examples/platformer-minimal/<localPath>` (Vite dev-server), read
  text from response, call `window.opfs_save_file(opfsPath, text)`.
  Aggregate errors; return `{ok: true|false, written, errors}`.  The
  Vite path mapping is: `localPath` is a relative path inside the
  sample; the production loader pre-pends `/examples/platformer-minimal/`.

### REQ-2: New shared test helper
- **Where:** `frontend/tests/helpers/sample-loader.ts` (NEW)
- **Exports:**
  - `mountSampleInOpfs(page: Page): Promise<void>` — same mapping, but
    uses Node `fs/promises.readFile` to read files at the
    `frontend/tests/helpers/../../examples/platformer-minimal/`
    absolute path, then `page.evaluate(window.opfs_save_file, ...)` to
    write them.
  - imports `OPFS_FILES` from `../../src/services/sampleLoader` so the
    mapping table is single-source.

### REQ-3: Wire `__loadSampleProject` to the production loader
- **Where:** `frontend/src/components/TutorialStepper.tsx`
- **Change:** replace the stub body (lines 98–105 today) with:
  ```ts
  w.__loadSampleProject = async (id: string) => {
    if (id !== "platformer-minimal") {
      return { ok: false, error: `Unknown sample id: ${id}` };
    }
    const result = await mountPlatformerMinimal();
    // After OPFS writes complete, hydrate the engine.
    try {
      await (window as unknown as {
        load_project?: () => Promise<void>;
      }).load_project?.();
    } catch (e) {
      return { ok: false, error: `engine reload failed: ${String(e)}` };
    }
    return result.ok ? { ok: true } : { ok: false, error: result.errors.join("; ") };
  };
  ```
- The bridge contract stays the same (returns `Promise<{ok, error?}>`).
- The effect that calls the bridge at step 1 (`useEffect(() => { if (open) { loader("platformer-minimal") } }, [open])`)
  is unchanged.

### REQ-4: Dedup the two existing specs
- **Where:** `frontend/tests/e2e-game-creation.spec.ts` and
  `frontend/tests/git-friendly-roundtrip.spec.ts`
- **Change:** delete their local `OPFS_FILES` arrays and
  `mountSample(page)` helpers, replace with `import { mountSampleInOpfs } from "./helpers/sample-loader"`
  and call `mountSampleInOpfs(page)` in the same places.
- **No behavioural change**: same files, same OPFS paths, same
  in-browser `opfs_save_file` calls.

### REQ-5: New behaviour test
- **Where:** `frontend/tests/load-sample-real-loader.spec.ts` (NEW)
- **Coverage:** 1 scenario × 2 projects = 2 tests
- **Scenario S1 (end-to-end loader):**
  1. `page.goto("/")`; wait for engine ready.
  2. `page.evaluate(() => window.load_project && load_project.toString().includes("[native code]"))` —
     confirm the bridge is registered.
  3. `page.evaluate` on the production loader:
     `await window.__loadSampleProject("platformer-minimal")` (the
     bridge stub currently installed by `TutorialStepper` won't be
     present unless the stepper is mounted; we'll install a test
     bridge instead).
  4. Assert the result is `{ok:true}`.
  5. Assert that after the loader, `window.opfs_load_file("project.json").ok === true`
     (proves the project metadata was written).
  6. Assert that the editor topbar shows the active scene name
     `"main"` (proves `load_project` ran successfully — this is the
     same observable used by `e2e-game-creation.spec.ts`).

### REQ-6: Regression guards
- `tests/tutorial-walkthrough.spec.ts` (4 scenarios × 2 projects = 8
  tests) continue to pass. The bridge contract is unchanged from the
  stepper's perspective (it still calls
  `window.__loadSampleProject("platformer-minimal")` and gets back
  `{ok|error}`); only the body of the bridge in `TutorialStepper.tsx`
  differs.
- `tests/tour-completed-persistence.spec.ts` (1 scenario × 2 projects
  = 2 tests) continues to pass.
- `tests/e2e-game-creation.spec.ts` continues to pass after the
  refactor (deduplication).
- `tests/git-friendly-roundtrip.spec.ts` continues to pass after the
  refactor.

### REQ-7: Static checks clean
- `npx tsc --noEmit -p .` clean.
- `npx eslint --max-warnings=0` on touched files clean.

### REQ-8: Error handling
- The loader is **best-effort**: if any single file fails to fetch or
  write, the loader logs the error, continues with the remaining files,
  and returns `{ok: false, errors}` once the loop completes.  The
  TutorialStepper's bridge then surfaces a concise error message
  (`result.errors.join("; ")`).
- The `load_project()` engine call is wrapped in `try/catch`; its
  failure does NOT mask the OPFS-write result.

### REQ-9: Dev-only (documented limitation)
- The loader assumes Vite's dev-server is serving `examples/` at
  `/examples/...`. In production builds, the path may 404; the loader
  will report that to the user. Documented in the implementation-receipt
  §Limitations. Out of scope for this cycle: production fallback.

## Out of scope

- A production build-time fallback for the dev-server `fetch()`. (Out of
  v1.0-stabilization scope; the IDE P1 evidence is dev-only and that
  pattern is established.)
- A loader for sample ids other than `"platformer-minimal"`. The bridge
  is currently called only with that id, and adding more samples would
  require a registry — separate cycle.
- A loader for *selected* files (e.g., just the scene assets the user is
  looking at). The cycle loads the full canonical project.

## Implementation guidance

### File 1: `frontend/src/services/sampleLoader.ts` (NEW, ~80 LOC)

```ts
/**
 * Production loader for the canonical `examples/platformer-minimal/`
 * sample project. Exposed via `window.__loadSampleProject` from
 * TutorialStepper. Expects Vite dev-server to serve the sample at
 * `/examples/platformer-minimal/<localPath>`.
 */
export const OPFS_FILES = [
  { opfsPath: "project.json", localPath: "project.json" },
  { opfsPath: "schemas/game.PlayerController.schema.json", localPath: "schemas/game.PlayerController.schema.json" },
  { opfsPath: "schemas/game.EnemyPatrol.schema.json", localPath: "schemas/game.EnemyPatrol.schema.json" },
  { opfsPath: "scenes/main.scene.json", localPath: "scenes/main.scene.json" },
  { opfsPath: "assets/characters/player.asset.json", localPath: "scene-assets/characters/player.actor.json" },
  { opfsPath: "assets/characters/enemy.asset.json", localPath: "scene-assets/characters/enemy.actor.json" },
  { opfsPath: "assets/environment/ground.asset.json", localPath: "scene-assets/environment/ground.fragment.json" },
  { opfsPath: "assets/effects/pickup.asset.json", localPath: "scene-assets/effects/pickup.actor.json" },
  { opfsPath: "logic_graphs/contact-death.logic.json", localPath: "logic-graphs/contact-death.logic.json" },
] as const;

export async function mountPlatformerMinimal() {
  const errors: string[] = [];
  let written = 0;
  for (const { opfsPath, localPath } of OPFS_FILES) {
    try {
      const resp = await fetch(`/examples/platformer-minimal/${localPath}`);
      if (!resp.ok) {
        errors.push(`${localPath}: HTTP ${resp.status}`);
        continue;
      }
      const text = await resp.text();
      const result = await (window as unknown as {
        opfs_save_file?: (p: string, c: string) => Promise<{ok: boolean; error?: string}>;
      }).opfs_save_file?.(opfsPath, text);
      if (!result?.ok) {
        errors.push(`${opfsPath}: ${result?.error ?? "unknown"}`);
      } else {
        written += 1;
      }
    } catch (e) {
      errors.push(`${localPath}: ${String(e)}`);
    }
  }
  return { ok: errors.length === 0, written, errors };
}
```

### File 2: `frontend/tests/helpers/sample-loader.ts` (NEW, ~30 LOC)

```ts
import { readFile } from "node:fs/promises";
import path from "node:path";
import { OPFS_FILES } from "../../src/services/sampleLoader";
import type { Page } from "@playwright/test";

export async function mountSampleInOpfs(page: Page): Promise<void> {
  const sampleDir = path.resolve(import.meta.dirname, "..", "..", "examples", "platformer-minimal");
  for (const { opfsPath, localPath } of OPFS_FILES) {
    const contents = await readFile(path.join(sampleDir, localPath), "utf-8");
    const r = await page.evaluate(
      async ({ p, c }) => await (window as any).opfs_save_file(p, c),
      { p: opfsPath, c: contents },
    );
    if (!r?.ok) {
      throw new Error(`opfs_save_file failed for ${opfsPath}: ${r?.error ?? "unknown"}`);
    }
  }
}
```

### File 3: `frontend/src/components/TutorialStepper.tsx`

`__loadSampleProject` body change (see REQ-3 above).

### Files 4 + 5: `tests/e2e-game-creation.spec.ts` and
`tests/git-friendly-roundtrip.spec.ts`

Delete the local `OPFS_FILES`, replace `mountSample` with
`mountSampleInOpfs(page)` from the helper.

### File 6: `frontend/tests/load-sample-real-loader.spec.ts` (NEW)

## Acceptance Gate

| Gate | Evidence |
| --- | --- |
| `requirements-testable` | All 9 REQs observable + test-assertable |
| `implementation-feasible` | `window.load_project` is exposed; `OPFS_FILES` mapping proven across 2 specs |
| `scope-bounded` | 6 files (3 src, 2 test refactors, 1 new spec), A-lite path |
| `carry-forward-closure` | Closes v0.110.6 + v0.110.7 carry-forward #1 (of 3, then 1 of 1) |
