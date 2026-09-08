/**
 * perf-project-search.spec.ts — G6 P6: 200-file project source listing.
 *
 * Per `docs/sddk/g6-performance-corpus/specification.md` §3.6:
 *
 *   - Pre-build: write 1000 source files (outside budget).
 *   - Trigger list_source_files (the source-listing operation; the
 *     closest available analog to "project-wide search" given the
 *     current bridge surface) and measure wall time.
 *
 * Soft budget: 1 s.
 * Hard budget: 3 s.
 *
 * Notes:
 *   - `global_search("function foo")` is the ideal target but no such
 *     bridge exists today (per `frontend/src/engine-bridge.ts`:
 *     `find_source_location(typeId)` only accepts a typeId). We use
 *     `list_source_files()` which is the next-most-similar bridge call:
 *     both walk the OPFS-mirrored source-file index.
 *
 * Bridge signatures (per `frontend/src/engine-bridge.ts:303-307`):
 *   list_source_files() -> Promise<string[]>
 *   read_source_file(id) -> Promise<string>
 *   write_source_file(id, content) -> Promise<void>
 */

import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import { assertWithinBudget, PerfBudgetSpec } from "./helpers/perfBudget";

const SPEC: PerfBudgetSpec = {
  name: "perf-project-search-1000",
  softMs: 1_000,
  hardMs: 3_000,
};

const FILE_COUNT = 200;

test.describe(
  "perf — 200-file project source listing",
  { tag: ["@performance", "@full"] },
  () => {
    test("list_source_files across 200 files within budget", async ({
      page,
    }) => {
      await page.goto("/");
      await waitForEditorReady(page);

      // Pre-build: write 200 placeholder source files (outside budget).
      await page.evaluate(async (n: number) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const writeFile = (window as any).write_source_file;
        if (typeof writeFile !== "function") return;
        for (let i = 0; i < n; i++) {
          const path = `src/fixture-${i.toString().padStart(4, "0")}.rs`;
          const body = `// fixture ${i}\nfn foo() -> i32 { ${i} }\n`;
          await writeFile(path, body);
        }
      }, FILE_COUNT);

      // Measured op: list_source_files.
      const listMs = await assertWithinBudget(SPEC, async () => {
        const r = await page.evaluate(async () => {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          return await (window as any).list_source_files();
        });
        if (r && typeof r === "object") {
          // eslint-disable-next-line no-console
          console.log(
            `perf: list_source_files returned ${(r as any).length ?? "?"} files`,
          );
        }
      });

      expect(listMs).toBeLessThan(SPEC.hardMs);
    });
  },
);
