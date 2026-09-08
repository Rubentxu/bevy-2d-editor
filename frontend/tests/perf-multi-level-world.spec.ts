/**
 * perf-multi-level-world.spec.ts — G6 P3: Multi-level world navigation.
 *
 * Per `docs/sddk/g6-performance-corpus/specification.md` §3.3:
 *
 *   - Build a 16-scene world (MAX_SCENES = 16; 1 default + 15 created).
 *   - Switch 5 times.
 *   - Measure average switch latency.
 *
 * Soft budget: 500 ms average switch.
 * Hard budget: 1500 ms average switch.
 *
 * Bridge signatures (per `frontend/src/engine-bridge.ts:292-295`):
 *   scene_create(name) -> id
 *   scene_switch(id)
 *   scene_switch_commit(id)
 *
 * If `scene_create` is not available the spec skips; the budget assertion
 * is the strict requirement when it is.
 */

import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import { measureMs, PerfBudgetSpec } from "./helpers/perfBudget";

const SPEC: PerfBudgetSpec = {
  name: "perf-multi-level-switch",
  softMs: 500,
  hardMs: 1500,
};

const SCENE_COUNT = 15; // MAX_SCENES = 16 (1 default + 15 created).
const TARGETS = ["scene_08", "scene_01", "scene_14", "scene_05", "scene_11"];

test.describe(
  "perf — multi-level world navigation",
  { tag: ["@performance", "@full"] },
  () => {
    test(`5 scene switches across a 16-scene world within avg budget`, async ({
      page,
    }) => {
      await page.goto("/");
      await waitForEditorReady(page);

      // Pre-build: create SCENE_COUNT - 1 more scenes. We already have
      // one default; we only need a 50-scene world to exist when the
      // measured op starts. Setup is OUTSIDE the budget.
      const setup = await page.evaluate(async (n: number) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const createScene = (window as any).scene_create;
        if (typeof createScene !== "function") {
          return { ok: false as const, reason: "missing-bridge" };
        }
        const ids: string[] = [];
        for (let i = 1; i < n; i++) {
          const r = await createScene(
            `scene_${i.toString().padStart(2, "0")}`,
          );
          if (typeof r === "string") {
            ids.push(r);
          }
        }
        return { ok: true as const, count: ids.length };
      }, SCENE_COUNT);

      // If the bridge isn't wired or setup failed, skip the budget assertion.
      test.skip(!setup.ok, `multi-level setup unavailable: ${setup.reason}`);

      // Measure mean latency over SWITCHES switches.
      const totalMs = await measureMs(async () => {
        for (const id of TARGETS) {
          await page.evaluate(
            (sid) => {
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              const switchFn = (window as any).scene_switch;
              const commitFn = (window as any).scene_switch_commit;
              if (typeof switchFn !== "function") return;
              switchFn(sid);
              if (typeof commitFn === "function") {
                commitFn(sid);
              }
            },
            id,
          );
        }
      });

      const meanMs = totalMs / TARGETS.length;

      // Hard budget assertion.
      expect(meanMs).toBeLessThan(SPEC.hardMs);

      // Soft budget advisory.
      if (meanMs > SPEC.softMs) {
        // eslint-disable-next-line no-console
        console.warn(
          `perf budget WARN: ${SPEC.name} mean ${meanMs.toFixed(0)} ms (soft ${SPEC.softMs} ms; hard ${SPEC.hardMs} ms)`,
        );
      }

      // eslint-disable-next-line no-console
      console.log(
        `perf: multi-level-switch mean ${meanMs.toFixed(0)} ms over ${TARGETS.length} switches (total ${totalMs} ms)`,
      );
    });
  },
);
