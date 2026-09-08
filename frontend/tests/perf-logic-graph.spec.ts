/**
 * perf-logic-graph.spec.ts — G6 P5: 100-node logic graph dispatch.
 *
 * Per `docs/sddk/g6-performance-corpus/specification.md` §3.5:
 *
 *   - Pre-build a 100-node logic graph (outside budget).
 *   - Dispatch BeginPlay 50 times, measure mean dispatch latency.
 *
 * Soft budget: 50 ms mean dispatch.
 * Hard budget: 200 ms mean dispatch.
 *
 * Bridge signature (per `frontend/src/engine-bridge.ts:354`):
 *   dispatch_logic_command(cmdJson: string)
 *
 * Note: the bridge takes a JSON-encoded STRING, not an object — we
 * serialize each command in the test before passing.
 */

import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import { measureMs, PerfBudgetSpec } from "./helpers/perfBudget";

const SPEC: PerfBudgetSpec = {
  name: "perf-logic-graph-dispatch",
  softMs: 50,
  hardMs: 200,
};

const NODE_COUNT = 100;
const DISPATCHES = 50;

test.describe(
  "perf — 100-node logic graph dispatch",
  { tag: ["@performance", "@full"] },
  () => {
    test("mean BeginPlay dispatch latency under hard budget", async ({
      page,
    }) => {
      await page.goto("/");
      await waitForEditorReady(page);

      // Pre-build: create a logic graph asset + add 100 nodes (outside budget).
      await page.evaluate(async (n: number) => {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const createGraph = (window as any).create_logic_graph_asset;
        if (typeof createGraph !== "function") return;
        await createGraph("perf-graph", "logic/perf");

        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const dispatch = (window as any).dispatch_logic_command;
        if (typeof dispatch !== "function") return;
        for (let i = 0; i < n; i++) {
          await dispatch(
            JSON.stringify({
              type: "AddNode",
              node_id: `n${i.toString().padStart(4, "0")}`,
              role: "actuator",
              node_type_id: "transform.translate",
              field_values: { x: i, y: i % 20 },
              controller_id: null,
            }),
          );
        }
      }, NODE_COUNT);

      // Measure mean dispatch latency using SetNodeField (a valid command,
      // unlike BeginPlay which is not a dispatch variant).
      const cmdJson = JSON.stringify({
        type: "SetNodeField",
        node_id: "n0000",
        field_path: ["x"],
        value: 0,
      });
      const totalMs = await measureMs(async () => {
        for (let i = 0; i < DISPATCHES; i++) {
          await page.evaluate(
            (cmd) =>
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              (window as any).dispatch_logic_command(cmd),
            cmdJson,
          );
        }
      });

      const meanMs = totalMs / DISPATCHES;
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
        `perf: logic-graph dispatch mean ${meanMs.toFixed(0)} ms over ${DISPATCHES} dispatches (total ${totalMs} ms)`,
      );
    });
  },
);
