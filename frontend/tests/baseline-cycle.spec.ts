import { test, expect, type Page } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";
import { mountSampleInOpfs } from "./helpers/sample-loader";

/**
 * baseline-cycle (H0.06) — reproducible measurements of the canonical
 * create→edit→play→save→reopen cycle across the agreed corpus:
 *
 *   1. empty project
 *   2. platformer-minimal sample (project WITH assets)
 *   3. large scene (2000 entities)
 *   4. logic graph (create/open; graph reopen is NOT_RUN — see notes)
 *
 * Each test logs one `BASELINE[...] {json}` console line with wall-clock
 * timings (in-page `performance.now()` for engine ops, test-side
 * `Date.now()` for page-level reopen). The Backlog task aggregates the
 * samples (3 runs → p50/p95). Assertions are sanity-level: the value of
 * this spec is the measurement trail, not pass/fail of edge cases.
 *
 * @full — runs in the full cohort (needs WASM engine + OPFS).
 */

const WASM_LOAD_TIMEOUT = 90_000;

function emptySceneJson() {
  return {
    version: "1",
    scene_id: "baseline-empty",
    name: "Baseline Empty",
    entities: [] as Array<{ id: string; name: string }>,
  };
}

function largeSceneJson(count = 2000) {
  return {
    version: "1",
    scene_id: "baseline-large",
    name: "Baseline Large",
    entities: Array.from({ length: count }, (_, i) => ({
      id: `baseline_e${i}`,
      name: `Entity${i}`,
      components: [],
    })),
  };
}

/** Run one create→edit→play→save cycle in-page and return its timings. */
async function timedCycle(
  page: Page,
  sceneName: string,
): Promise<Record<string, number>> {
  return page.evaluate(async (sceneName) => {
    const t: Record<string, number> = {};

    // create: 10 dispatched CreateEntity commands
    let t0 = performance.now();
    for (let i = 0; i < 10; i++) {
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "CreateEntity",
            id: `baseline_cycle_e${i}`,
            name: `CycleEntity${i}`,
            components: [
              { type_id: "editor.Name", values: { name: `CycleEntity${i}` } },
            ],
          },
          metadata: { authorship: "baseline", timestamp: 0 },
        }),
      );
    }
    const snapAfterCreate = JSON.parse((window as any).get_scene_snapshot())
      .entities.length;
    t.createMs = performance.now() - t0;

    // edit: one SetComponentField
    t0 = performance.now();
    (window as any).dispatch_command(
      JSON.stringify({
        command: {
          type: "SetComponentField",
          entity_id: "baseline_cycle_e0",
          type_id: "editor.Name",
          field_path: "name",
          value: "EditedName",
        },
        metadata: { authorship: "baseline", timestamp: 0 },
      }),
    );
    t.editMs = performance.now() - t0;

    // play: enter + settle + exit
    t0 = performance.now();
    (window as any).enter_play_mode();
    await new Promise((r) => setTimeout(r, 300));
    (window as any).exit_play_mode();
    t.playRoundtripMs = performance.now() - t0;

    // save (save_scene is async — the OPFS write must complete before reload)
    t0 = performance.now();
    await (window as any).save_scene(sceneName);
    t.saveMs = performance.now() - t0;

    (window as any).__baselineSanity = { snapAfterCreate };
    void sceneName;
    return t;
  }, sceneName);
}

test.describe("baseline cycle measurements", { tag: ["@full"] }, () => {
  test("corpus 1: empty project cycle", async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);

    await page.evaluate(
      (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
      emptySceneJson(),
    );

    const timings = await timedCycle(page, "baseline-empty");
    const sanity = await page.evaluate(() => (window as any).__baselineSanity);
    expect(sanity.snapAfterCreate).toBe(10);

    // reopen: reload page, wait ready, load the saved scene back
    const reopenStart = Date.now();
    await page.reload();
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);
    await page.evaluate(() => (window as any).load_scene("baseline-empty"));
    const entities = await page.evaluate(
      () => JSON.parse((window as any).get_scene_snapshot()).entities.length,
    );
    const reopenMs = Date.now() - reopenStart;
    expect(entities).toBe(10);

    console.log(
      "BASELINE[empty-project] " +
        JSON.stringify({ ...timings, reopenMs, entitiesAfterReopen: entities }),
    );
  });

  test("corpus 2: platformer-minimal sample (project with assets)", async ({
    page,
  }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);

    // Mount the canonical sample (9 files incl. sprites/schemas) via the
    // production mapping, then re-hydrate + load the project.
    const mountStart = Date.now();
    await mountSampleInOpfs(page);
    const mountMs = Date.now() - mountStart;

    const reopenStart = Date.now();
    await page.reload();
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);
    const loadProjectResult = await page.evaluate(() => {
      const t0 = performance.now();
      const res = (window as any).load_project();
      return { ms: performance.now() - t0, res: String(res) };
    });
    const reopenMs = Date.now() - reopenStart;

    // Sanity: the sample project hydrated (project.json name present in
    // OPFS-visible state via list_scenes listing its scenes).
    const scenes = await page.evaluate(() => (window as any).list_scenes());
    const sceneNames =
      typeof scenes === "string" ? JSON.parse(scenes) : (scenes ?? []);
    const listLen = Array.isArray(sceneNames)
      ? sceneNames.length
      : (sceneNames.scenes ?? []).length;
    expect(listLen).toBeGreaterThan(0);

    // edit + play roundtrip on the sample's active scene
    const cycle = await timedCycle(page, "baseline-sample-cycle");
    const entities = await page.evaluate(
      () => JSON.parse((window as any).get_scene_snapshot()).entities.length,
    );

    console.log(
      "BASELINE[platformer-minimal] " +
        JSON.stringify({
          mountMs,
          loadProjectMs: loadProjectResult.ms,
          reopenMs,
          ...cycle,
          entitiesAfterReopen: entities,
          scenes: listLen,
        }),
    );
  });

  test("corpus 3: large scene (2000 entities) load→save→reopen", async ({
    page,
  }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);

    const loadMs = await page.evaluate((scene) => {
      const t0 = performance.now();
      (window as any).load_scene_json(JSON.stringify(scene));
      return performance.now() - t0;
    }, largeSceneJson());

    const entities = await page.evaluate(
      () => JSON.parse((window as any).get_scene_snapshot()).entities.length,
    );
    expect(entities).toBe(2000);

    // edit one field in the large scene
    const editMs = await page.evaluate(() => {
      const t0 = performance.now();
      (window as any).dispatch_command(
        JSON.stringify({
          command: {
            type: "RenameEntity",
            entity_id: "baseline_e1500",
            new_name: "Renamed1500",
          },
          metadata: { authorship: "baseline", timestamp: 0 },
        }),
      );
      return performance.now() - t0;
    });

    // save the large scene (awaited — async OPFS write)
    const saveMs = await page.evaluate(async () => {
      const t0 = performance.now();
      await (window as any).save_scene("baseline-large");
      return performance.now() - t0;
    });

    // reopen
    const reopenStart = Date.now();
    await page.reload();
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);
    await page.evaluate(() => (window as any).load_scene("baseline-large"));
    const reopened = await page.evaluate(
      () => JSON.parse((window as any).get_scene_snapshot()).entities.length,
    );
    const reopenMs = Date.now() - reopenStart;
    expect(reopened).toBe(2000);

    console.log(
      "BASELINE[large-2000] " +
        JSON.stringify({ loadMs, editMs, saveMs, reopenMs, reopened }),
    );
  });

  test("corpus 4: logic graph create/open", async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page, WASM_LOAD_TIMEOUT);

    const timings = await page.evaluate(async () => {
      const t: Record<string, number | string> = {};
      let t0 = performance.now();
      await (window as any).create_logic_graph_asset(
        "baseline_graph",
        "graphs/baseline",
      );
      t.createMs = performance.now() - t0;
      t0 = performance.now();
      await (window as any).open_logic_graph_asset("baseline_graph");
      t.openMs = performance.now() - t0;
      t0 = performance.now();
      const graph = await (window as any).get_logic_graph();
      t.getMs = performance.now() - t0;
      t.graphKind = typeof graph === "string" ? "json" : typeof graph;
      return t;
    });

    // Graph reopen-after-reload is NOT_RUN in this baseline: scene-scoped
    // save_scene does not cover asset-level graphs and the project-level
    // graph persistence contract is H8 territory.
    console.log(
      "BASELINE[logic-graph] " +
        JSON.stringify({ ...timings, reopenAfterReload: "NOT_RUN" }),
    );
  });
});
