# G6 — Performance corpus: design

**Cycle**: v0.110.3 candidate (A-lite)
**Generated**: 2026-09-08

## 1. Architecture overview

```
+----------------------------------------------------------+
|  playwright.performance.config.ts                        |
|  (grep: /@performance/ AND /@full/, timeout 240_000)    |
+----------------------------------------------------------+
                |
                v
+----------------------------------------------------------+
|  6 specs in frontend/tests/perf-*.spec.ts                |
|    perf-10k-entities.spec.ts                             |
|    perf-tile-level.spec.ts                               |
|    perf-multi-level-world.spec.ts                        |
|    perf-asset-catalog.spec.ts                            |
|    perf-logic-graph.spec.ts                              |
|    perf-project-search.spec.ts                           |
+----------------------------------------------------------+
                |
                v
+----------------------------------------------------------+
|  helpers/perfBudget.ts                                   |
|    measureMs(fn) -> Promise<number>                      |
|    assertWithinBudget(spec, fn) -> Promise<number>       |
+----------------------------------------------------------+
                |
                v
+----------------------------------------------------------+
|  engine-bridge window.* APIs already exposed:            |
|    add_entity, dispatch_logic_command, list_assets,      |
|    search_source_files, paint_tile, erase_tile,          |
|    scene_switch, scene_switch_commit                     |
+----------------------------------------------------------+
```

## 2. Files to create

| File | Purpose | LoC est. |
|---|---|---:|
| `frontend/playwright.performance.config.ts` | New Playwright config. Mirrors `playwright.full.config.ts` but with longer timeout and a `grep: /@performance/`. Also includes `@full` so the perf cohort also runs in `playwright.full.config.ts` (fall-through). | 60 |
| `frontend/tests/helpers/perfBudget.ts` | Export `measureMs`, `assertWithinBudget`, `PerfBudgetSpec` interface. | 50 |
| `frontend/tests/perf-10k-entities.spec.ts` | P1 benchmark (10k entity roundtrip). | 80 |
| `frontend/tests/perf-tile-level.spec.ts` | P2 benchmark. | 70 |
| `frontend/tests/perf-multi-level-world.spec.ts` | P3 benchmark. | 70 |
| `frontend/tests/perf-asset-catalog.spec.ts` | P4 benchmark. | 70 |
| `frontend/tests/perf-logic-graph.spec.ts` | P5 benchmark. | 70 |
| `frontend/tests/perf-project-search.spec.ts` | P6 benchmark. | 70 |
| `frontend/package.json` script addition: `"test:perf"` | npm script. | 1 |

Total: ~540 LoC across 8 files.

## 3. Helper contract

`frontend/tests/helpers/perfBudget.ts`:

```typescript
export interface PerfBudgetSpec {
  readonly name: string;
  readonly softMs: number;
  readonly hardMs: number;
}

/** Measure the wall-clock time of an async function in ms. */
export async function measureMs(fn: () => Promise<void>): Promise<number> {
  const start = Date.now();
  await fn();
  return Date.now() - start;
}

/**
 * Run `fn` and assert the wall-clock time is within the hard budget.
 * Logs a warning to `console.warn` if it exceeds the soft budget.
 * Returns the measured time in ms (caller may want to store it).
 */
export async function assertWithinBudget(
  spec: PerfBudgetSpec,
  fn: () => Promise<void>,
): Promise<number> {
  const ms = await measureMs(fn);
  if (ms > spec.hardMs) {
    throw new Error(
      `perf budget FAIL: ${spec.name} took ${ms}ms (hard budget ${spec.hardMs}ms)`,
    );
  }
  if (ms > spec.softMs) {
    // eslint-disable-next-line no-console
    console.warn(
      `perf budget WARN: ${spec.name} took ${ms}ms (soft budget ${spec.softMs}ms; hard ${spec.hardMs}ms)`,
    );
  }
  return ms;
}
```

## 4. Spec template

Each perf-N.spec.ts follows this skeleton:

```typescript
import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import { assertWithinBudget, PerfBudgetSpec } from "./helpers/perfBudget";

const SPEC: PerfBudgetSpec = {
  name: "perf-N",
  softMs: 5_000,
  hardMs: 12_000,
};

test.describe("perf — <name>", { tag: ["@performance", "@full"] }, () => {
  test("<one-line description>", async ({ page }) => {
    await page.goto("/");
    await waitForEditorReady(page);
    const ms = await assertWithinBudget(SPEC, async () => {
      // ... actual work ...
    });
    // Optional: assert correctness alongside the budget.
    expect(/* some invariant */).toBeTruthy();
  });
});
```

## 5. Per-spec design notes

### 5.1 P1 — 10k entities roundtrip

```typescript
test("10000 entity scene roundtrip within budget", async ({ page }) => {
  // Pre-build the scene by writing the OPFS file directly via
  // opfs_save_file (avoids 10k UI clicks).
  await page.evaluate(async () => {
    const project = {
      version: "v1",
      // 10k entity array
      entities: Array.from({ length: 10_000 }, (_, i) => ({
        id: `e_${i.toString().padStart(5, "0")}`,
        name: `Entity ${i}`,
        components: [{ type_id: "editor.Transform2D", values: {...} }],
      })),
    };
    await (window as any).opfs_save_file(
      "scenes/10k.scene.json",
      JSON.stringify(project),
    );
  });

  // Trigger hydrate via load_scene and time the operation.
  const ms = await assertWithinBudget(SPEC, async () => {
    await page.reload();
    await waitForEditorReady(page);
  });

  // Assert the count we expected.
  const snapshot = await page.evaluate(async () => {
    return await (window as any).get_scene_snapshot();
  });
  expect(snapshot?.entities?.length).toBe(10_000);
});
```

**Setup-vs-measure:** Setup (writing the 10k file) is OUTSIDE the budget.
The budget measures `page.reload` + `waitUntilReady` (cold hydration of
the 10k scene).

### 5.2 P2 — Tile level paint/erase

```typescript
test("paint 200 tiles then erase 200 tiles within budget", async ({ page }) => {
  await page.goto("/");
  await waitForEditorReady(page);

  // Set up a 100x100 grid and paint 200 distinct cells.
  const paintMs = await assertWithinBudget(SPEC_PAINT, async () => {
    for (let i = 0; i < 200; i++) {
      await page.evaluate(
        ({ x, y }) => (window as any).paint_tile(x, y, 1),
        { x: i % 100, y: Math.floor(i / 100) },
      );
    }
  });

  const eraseMs = await assertWithinBudget(SPEC_ERASE, async () => {
    for (let i = 0; i < 200; i++) {
      await page.evaluate(
        ({ x, y }) => (window as any).erase_tile(x, y),
        { x: i % 100, y: Math.floor(i / 100) },
      );
    }
  });

  // Optional: assert post-state.
  const remaining = await page.evaluate(async () =>
    (window as any).list_tileset("default"),
  );
  expect(remaining?.ok).toBe(true);
});
```

### 5.3 P3 — Multi-level world navigation

```typescript
test("scene-switch latency under 0.5 s average over 5 switches", async ({ page }) => {
  await page.goto("/");
  await waitForEditorReady(page);

  // Pre-build a 50-scene world.
  await page.evaluate(async () => {
    await (window as any).build_world_with_n_scenes(50);  // hypothetical helper
  });

  // Measure mean latency over 5 switching ops.
  const totalMs = await measureMs(async () => {
    for (const target of [25, 1, 49, 12, 37]) {
      await page.evaluate((id) => (window as any).scene_switch(id), target);
      await page.evaluate(() => (window as any).scene_switch_commit());
    }
  });

  expect(totalMs / 5).toBeLessThan(SPEC.hardMs);
});
```

If `build_world_with_n_scenes` doesn't exist, we use `scene_create` in a loop.

### 5.4 P4 — 500-asset catalog listing

```typescript
test("list 500 assets within 3 s", async ({ page }) => {
  await page.goto("/");
  await waitForEditorReady(page);

  await page.evaluate(async () => {
    for (let i = 0; i < 500; i++) {
      await (window as any).import_asset_file(
        `assets/fixture-${i.toString().padStart(3, "0")}.png`,
        new ArrayBuffer(64),  // tiny placeholder
      );
    }
  });

  const listMs = await assertWithinBudget(SPEC, async () => {
    await page.evaluate(async () => {
      return await (window as any).list_asset_files();
    });
  });
});
```

### 5.5 P5 — 200-node logic graph dispatch

```typescript
test("200-node logic graph BeginPlay dispatch within 200 ms mean", async ({ page }) => {
  await page.goto("/");
  await waitForEditorReady(page);

  await page.evaluate(async () => {
    // Build the 200-node graph + 50 logical connections.
    for (let i = 0; i < 200; i++) {
      await (window as any).dispatch_logic_command({
        type: "add_node",
        payload: { type: "transform.translate", x: i, y: i % 10 },
      });
    }
  });

  let totalMs = 0;
  for (let i = 0; i < 50; i++) {
    totalMs += await measureMs(async () => {
      await page.evaluate(() => (window as any).dispatch_logic_command({ type: "BeginPlay" }));
    });
  }
  expect(totalMs / 50).toBeLessThan(SPEC.hardMs);
});
```

### 5.6 P6 — 1000-file project source listing (proxy for "project search")

```typescript
test("list_source_files across 1000 files within budget", async ({ page }) => {
  await page.goto("/");
  await waitForEditorReady(page);

  await page.evaluate(async (n: number) => {
    for (let i = 0; i < n; i++) {
      await (window as any).write_source_file(
        `src/fixture-${i.toString().padStart(4, "0")}.rs`,
        `// file ${i}\nfn foo() -> i32 { ${i} }\n`,
      );
    }
  }, 1000);

  const listMs = await assertWithinBudget(SPEC, async () => {
    return await page.evaluate(async () => {
      return await (window as any).list_source_files();
    });
  });
});
```

Note: the ideal `global_search("function foo")` bridge does not exist today
(only `find_source_location(typeId)` is exposed; see
`docs/sddk/g6-performance-corpus/specification.md` §3.6 for rationale).
We use `list_source_files()` which is the next-most-similar bridge call:
both walk the OPFS-mirrored source-file index. Once a `global_search`
bridge is exposed, this spec should be re-scoped to call it.

## 6. Cohort config

```typescript
// frontend/playwright.performance.config.ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  testMatch: ["perf-*.spec.ts"],
  timeout: 240_000,
  expect: { timeout: 60_000 },
  fullyParallel: false,
  retries: 0,
  use: {
    baseURL: "http://localhost:5173",
    headless: true,
  },
  webServer: [
    {
      command: "node tests/fixtures/mock-ai-proxy.mjs",
      url: "http://localhost:11436/health",
      reuseExistingServer: true,
      timeout: 10_000,
      stdout: "ignore",
      stderr: "pipe",
    },
    {
      command: "npx vite",
      url: "http://localhost:5173",
      reuseExistingServer: true,
      timeout: 60_000,
    },
  ],
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
```

`grep` is set via `testMatch` since the new cohort will only have perf specs.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Some bridge calls (e.g. `build_world_with_n_scenes`) may not exist | Specs use `page.evaluate()` to detect missing bridge and `test.skip()` gracefully — silent skip is acceptable for v0.110.3 since the goal is to *measure* what exists. |
| Cold-start timer inflates the budget | Every spec calls `waitUntilReady(page)` first; only the per-op wall time is measured. |
| CI failure blocks merges | Configure CI to NOT run perf cohort on PRs. Add a nightly-only workflow in a follow-up cycle. |

## 8. Naming + conventions

- `perf-N-name.spec.ts` — names match the spec IDs in §3.
- Each spec contains **one** `test()` (no describe grouping beyond `.describe()` for tags).
- Soft + hard budgets declared as `const SPEC` at the top of each file.

## 9. What this design does NOT cover

- CI wiring (follow-up cycle for `nightly-perf.yml`).
- Native `criterion` benchmarks.
- Cold-startup budget (separate spec).
- Smoke-cohort 60 s fix.
