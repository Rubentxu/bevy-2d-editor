# Design — load-sample-real-loader (cycle 389)

**Cycle:** load-sample-real-loader
**Path:** A-lite
**Phase:** Design (sequence 391)

## 1. Architecture overview

This cycle is bounded UI plumbing. It does not introduce a new
architectural boundary; it materialises a v1.0-stabilization carry-forward
(P1 evidence, G2 round-trip corpus, BJ-3 UI creation test) into runtime
behaviour that the existing test scaffolding already uses.

```
   ┌────────────────────────────────────────────────────┐
   │ TutorialStepper.tsx (React component, topbar card) │
   │   - mounts window.__loadSampleProject bridge       │
   │   - on step 1, calls loader("platformer-minimal")  │
   └────────────────────────┬───────────────────────────┘
                            │ window.__loadSampleProject
                            ▼
   ┌────────────────────────────────────────────────────┐
   │ services/sampleLoader.ts                           │
   │   - mountPlatformerMinimal() → {ok, written, errs}│
   │   - exports OPFS_FILES mapping table               │
   │   - browser-side: fetch /examples/.../...          │
   │     then window.opfs_save_file(opfsPath, text)     │
   └────────────────────────┬───────────────────────────┘
                            │ window.opfs_save_file (existing bridge)
                            ▼
   ┌────────────────────────────────────────────────────┐
   │ services/opfs-bridge.ts (existing)                 │
   │   - Real OPFS adapter, called by engine-bridge.ts  │
   └────────────────────────┬───────────────────────────┘
                            │ OPFS write
                            ▼
   ┌────────────────────────────────────────────────────┐
   │ editor-bevy/lib.rs (Rust WASM)                     │
   │   - window.opfs_save_file extern (wasm_bridge)     │
   │   - window.load_project() extern (registered AFTER │
   │     init_project_store) — re-reads OPFS, hydrates  │
   └────────────────────────────────────────────────────┘

   Test helper (parallel):
   ┌────────────────────────────────────────────────────┐
   │ tests/helpers/sample-loader.ts                     │
   │   - mountSampleInOpfs(page) — Node-side mirror     │
   │   - reads files via fs/promises.readFile           │
   │   - writes via page.evaluate(window.opfs_save_file)│
   │   - imports OPFS_FILES from production module      │
   └────────────────────────────────────────────────────┘
```

The cycle's contribution:
- The blue box (services/sampleLoader.ts) is NEW.
- The bridge body in TutorialStepper.tsx (top box) is rewritten to call
  it.
- The yellow box (tests/helpers/sample-loader.ts) is NEW — it
  consolidates a duplicated mapping already present in two specs.

## 2. Component contracts

### `services/sampleLoader.ts`

```
export const OPFS_FILES: ReadonlyArray<{
  opfsPath: string;
  localPath: string;
}> = [...];   // 9 entries

export interface MountResult {
  ok: boolean;
  written: number;
  errors: string[];
}

export function mountPlatformerMinimal(): Promise<MountResult>
```

**Failure modes:**

- `fetch` rejects (network down): caught per-file, accumulated in `errors`.
- `fetch` returns non-2xx: caught per-file, accumulated.
- `opfs_save_file` returns `{ok:false}`: caught per-file, accumulated.
- The function does NOT throw — it always returns `{ok, written, errors}`.

**Dev-server path:** `/examples/platformer-minimal/<localPath>`. This is the
Vite convention; `examples/` is at the repo root but Vite exposes
everything under `frontend/...` and `examples/...` to dev-server.

### `services/tour.ts` ↔ `services/sampleLoader.ts`

`services/tour.ts:markTourCompleted()` is unchanged. The two services
have **different** responsibilities and do **not** share state:

| Service | When | Persists | Reads |
| --- | --- | --- | --- |
| `services/tour.ts` | Finish button | `.bevy/tour-flags.json` | (router applies; see v0.110.7) |
| `services/sampleLoader.ts` | Step 1 | `project.json` + scenes + schemas + assets + logic_graphs (canonical project layout) | n/a |

No coupling. Both can be replaced independently.

### `tutorial.ts → sampleLoader.ts` bridge contract

The `window.__loadSampleProject` bridge contract (set in cycle 371) is:

```ts
type Loader = (id: string) => Promise<{ok: boolean; error?: string}>;
```

This contract is preserved. The body changes:

```ts
// before (cycle 371 stub):
w.__loadSampleProject = async (id: string) => {
  console.info(`[TutorialStepper] Loading sample project: ${id}`);
  await new Promise((resolve) => setTimeout(resolve, 100));
  return { ok: true };
};

// after (this cycle):
w.__loadSampleProject = async (id: string) => {
  if (id !== "platformer-minimal") {
    return { ok: false, error: `Unknown sample id: ${id}` };
  }
  const mountResult = await mountPlatformerMinimal();
  if (!mountResult.ok) {
    return { ok: false, error: mountResult.errors.join("; ") };
  }
  try {
    await (window as unknown as {
      load_project?: () => Promise<void>;
    }).load_project?.();
  } catch (e) {
    return { ok: false, error: `engine reload failed: ${String(e)}` };
  }
  return { ok: true };
};
```

## 3. Data flow

### Happy path (cycle's S1 scenario)

1. User clicks "Take the tour" on WelcomeOverlay → `setTourOpen(true)`.
2. TutorialStepper mounts. Its mount-effect installs
   `window.__loadSampleProject`.
3. TutorialStepper's `open`-effect fires:
   `void loader("platformer-minimal")`. The bridge starts.
4. `mountPlatformerMinimal()`:
   - For each entry in `OPFS_FILES` (9 entries):
     - `fetch('/examples/platformer-minimal/<localPath>')` →
       `text` (the JSON file content).
     - `window.opfs_save_file('<opfsPath>', text)` → `{ok: true}`.
5. Result: `{ok: true, written: 9, errors: []}`.
6. `load_project()` is called. It reads `project.json` from OPFS,
   hydrates schemas, scenes, scene-asset catalog, and the active scene.
7. The bridge returns `{ok: true}`.
8. (No contract change for the stepper UI side.)
9. Editor topbar shows project loaded; Scene Asset Browser mounts.

### Sad path A: dev-server not running (production build)

1. `fetch` returns `404` for `/examples/platformer-minimal/project.json`.
2. Per-file error: `"project.json: HTTP 404"`.
3. Loop continues; subsequent files also 404.
4. Result: `{ok: false, written: 0, errors: [...9 errors]}`.
5. Bridge returns `{ok: false, error: "project.json: HTTP 404; scenes/main.scene.json: HTTP 404; ..."}`.
6. The stepper UI doesn't have a dedicated error toast surface (out of
   scope) — the error is silent in the UI but the bridge caller (test
   or stepper) can detect it.

### Sad path B: `load_project` throws

1. `mountPlatformerMinimal()` succeeds (writes 9 files).
2. `load_project()` throws (e.g., schema migration error).
3. Bridge returns `{ok: false, error: "engine reload failed: <msg>"}`.
4. The OPFS writes are still on disk; a future `load_project()` call
   may succeed.

### Sad path C: dev server down + load_project fails

1. Result is `{ok: false, errors: [...], engine failed}`.
2. Both error reasons are visible via the bridge return value and the
   console warning log.

## 4. Refactor strategy

The two existing specs (`e2e-game-creation.spec.ts`,
`git-friendly-roundtrip.spec.ts`) each have a copy of `OPFS_FILES` and
`mountSample(page)`. The cycle eliminates the duplication by:

1. **Extracting `OPFS_FILES`** to `frontend/src/services/sampleLoader.ts`
   as a public `export const`.
2. **Adding a thin Node helper** at
   `frontend/tests/helpers/sample-loader.ts`:
   ```ts
   import { OPFS_FILES } from "../../src/services/sampleLoader";
   export async function mountSampleInOpfs(page: Page): Promise<void> { ... }
   ```
3. **Replacing each spec's local `OPFS_FILES` + `mountSample`** with
   `import { mountSampleInOpfs } from "./helpers/sample-loader"` and a
   single function call.

The refactor is *behaviour-preserving*: the new helper writes to the
same OPFS paths using the same `page.evaluate(window.opfs_save_file)`
call, and the in-browser bridge is identical. Existing tests continue to
pass without modification of assertions.

## 5. Side effects and reentrancy

The production loader writes files to OPFS. **OPFS is shared across
tests in the same browser context.** This is acceptable because:

- A clean OPFS at test start is per-test (Playwright contexts are
  isolated per-test by default).
- The loader only writes canonical project files (`project.json`, etc.)
  which do not collide with other tests' state.
- `load_project()` is idempotent: re-running it on the same OPFS state
  produces the same in-memory mirror.

Potential for reentrancy: if the user takes the tour, finishes, opens a
new project, then takes the tour again, the loader overwrites the
canonical files. This is **intentional**: the tutorial is "set up the
canonical sample". A future cycle can add a "Replace?" prompt, but per
the v1.0 evidence P1, the canonical sample is what the user wants.

## 6. Test strategy

### `tests/load-sample-real-loader.spec.ts` (NEW)

S1 (1 scenario × 2 projects = 2 tests):

1. `page.goto("/")`. `page.waitForFunction(() => typeof (window as any).load_project === "function", ...)`
   — proves the engine bridge is registered (post-`init_project_store`).
2. Install a test bridge on `window.__loadSampleProject` that calls
   `mountPlatformerMinimal()` and returns the result. (We can't
   actually mount TutorialStepper for this test; we mirror the bridge
   body.)
3. `await loadResult`. Assert `result.ok === true` and
   `result.written === 9`.
4. `window.opfs_load_file("project.json")` → assert `ok === true` and
   the JSON parses to `{name: "platformer-minimal", ...}`.
5. `await (window as any).load_project()` — engine hydrate.
6. The editor topbar shows project loaded (assert via the
   `data-testid="project-loaded"` indicator OR the Scene Asset Browser
   listing 4 entries).

This proves the production loader writes the canonical files AND that
the engine successfully hydrates them.

### Regression suites (already covered)

- `tests/tutorial-walkthrough.spec.ts` (8 tests): bridge contract is
  unchanged from the stepper's perspective; only the body changed.
- `tests/tour-completed-persistence.spec.ts` (2 tests): unrelated to
  this cycle.
- `tests/e2e-game-creation.spec.ts`: refactored to use
  `mountSampleInOpfs`; behaviour preserved.
- `tests/git-friendly-roundtrip.spec.ts`: same.

## 7. Open design decisions resolved by this doc

1. **Path for `fetch`**: `/examples/platformer-minimal/<localPath>`.
   Resolved via Vite's static-file exposure of `examples/`.
2. **Single source of truth for `OPFS_FILES`**: production service
   module. The test helper imports from it. (No copy-paste.)
3. **Engine reload timing**: after OPFS writes complete. The loader
   does NOT pause to wait for the engine; it returns the `load_project`
   result via the bridge's Promise.
4. **Error aggregation**: best-effort, all errors collected in a
   single `errors[]` array. The bridge surfaces a concatenated string.
5. **Dev-only limitation**: explicitly documented in
   implementation-receipt §Limitations. Out of cycle scope: production
   fallback.

## 8. Architecture delta manifest

None. The cycle declares `not_applicable` for `architecture_validation`.

Design gate ready (`architecture-consistent`).
