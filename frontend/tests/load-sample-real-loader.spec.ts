/**
 * load-sample-real-loader — behaviour test for the production sample loader.
 *
 * Covers spec REQ-5 scenario S1. The scenario is split across two
 * tests so each acceptance criterion (production loader writes 9
 * files / engine hydrates them) is asserted with its own setup and a
 * clear pass/fail signal:
 *
 *   S1.1  `mountPlatformerMinimal()` writes 9 files to OPFS via the
 *         production loader and `project.json` parses to
 *         `{ name: "platformer-minimal", ... }`.
 *   S1.2  The same code path the TutorialStepper triggers at step 1
 *         (OPFS writes via the production mapping) + the canonical
 *         hydration sequence (`page.reload()` to re-hydrate the
 *         thread-local mirror + `load_project()`) puts the sample in
 *         the engine — Scene Asset Browser lists 4 assets, both
 *         custom schemas are registered.
 *
 * The 9 example files are read from disk in Node and served to the
 * page via a temporary `fetch` interceptor installed in the page
 * context. The interceptor mimics what the Vite dev-server WOULD do
 * if `examples/` were configured to be served at `/examples/...` (a
 * documented dev-only limitation — REQ-9). Both tests use the
 * production `mountPlatformerMinimal()` module via dynamic import so
 * they exercise the real production code path, not a mirror.
 *
 * We don't actually mount the React stepper component here because we
 * want a deterministic, isolated bridge boundary; the regression spec
 * `tutorial-walkthrough.spec.ts` already exercises the React side and
 * proves the bridge contract from the stepper's perspective.
 *
 * @full — runs in the full cohort (needs WASM engine).
 */

import { test, expect, type Page } from "@playwright/test";
import { readFile } from "node:fs/promises";
import path from "node:path";

const WASM_LOAD_TIMEOUT = 60_000;

// Resolve the sample directory relative to the repo root (same path
// the production loader assumes for the dev-server).
const SAMPLE_DIR = path.resolve(
  import.meta.dirname,
  "..",
  "..",
  "examples",
  "platformer-minimal",
);

/**
 * Read the 9 canonical sample files from disk and return them as a
 * `{ localPath: text }` map ready for serialisation into the page
 * context. The keys mirror the production `OPFS_FILES[i].localPath`
 * values exactly.
 */
async function loadExampleTexts(): Promise<Record<string, string>> {
  const SAMPLE = SAMPLE_DIR;
  const [
    projectJson,
    controllerSchema,
    enemySchema,
    mainScene,
    playerActor,
    enemyActor,
    groundFragment,
    pickupActor,
    contactDeathLogic,
  ] = await Promise.all([
    readFile(path.join(SAMPLE, "project.json"), "utf-8"),
    readFile(
      path.join(SAMPLE, "schemas/game.PlayerController.schema.json"),
      "utf-8",
    ),
    readFile(
      path.join(SAMPLE, "schemas/game.EnemyPatrol.schema.json"),
      "utf-8",
    ),
    readFile(path.join(SAMPLE, "scenes/main.scene.json"), "utf-8"),
    readFile(
      path.join(SAMPLE, "scene-assets/characters/player.actor.json"),
      "utf-8",
    ),
    readFile(
      path.join(SAMPLE, "scene-assets/characters/enemy.actor.json"),
      "utf-8",
    ),
    readFile(
      path.join(SAMPLE, "scene-assets/environment/ground.fragment.json"),
      "utf-8",
    ),
    readFile(
      path.join(SAMPLE, "scene-assets/effects/pickup.actor.json"),
      "utf-8",
    ),
    readFile(
      path.join(SAMPLE, "logic-graphs/contact-death.logic.json"),
      "utf-8",
    ),
  ]);
  return {
    "project.json": projectJson,
    "schemas/game.PlayerController.schema.json": controllerSchema,
    "schemas/game.EnemyPatrol.schema.json": enemySchema,
    "scenes/main.scene.json": mainScene,
    "scene-assets/characters/player.actor.json": playerActor,
    "scene-assets/characters/enemy.actor.json": enemyActor,
    "scene-assets/environment/ground.fragment.json": groundFragment,
    "scene-assets/effects/pickup.actor.json": pickupActor,
    "logic-graphs/contact-death.logic.json": contactDeathLogic,
  };
}

/**
 * Install a fetch interceptor inside the page context that serves the
 * 9 example files when the production loader requests
 * `/examples/platformer-minimal/<localPath>`. Other URLs pass through
 * to the original fetch.
 *
 * Returns a cleanup function that restores the original fetch.
 */
async function installFetchInterceptor(
  page: Page,
  texts: Record<string, string>,
): Promise<() => Promise<void>> {
  await page.evaluate((exampleTexts) => {
    const w = window as unknown as {
      __origFetch?: typeof fetch;
      __exampleBase?: string;
      __exampleTexts?: Record<string, string>;
    };
    w.__origFetch = window.fetch.bind(window);
    w.__exampleBase = "/examples/platformer-minimal/";
    w.__exampleTexts = exampleTexts;
    window.fetch = async function intercepted(
      input: RequestInfo | URL,
      init?: RequestInit,
    ): Promise<Response> {
      const url =
        typeof input === "string"
          ? input
          : input instanceof URL
            ? input.toString()
            : input.url;
      const base = w.__exampleBase ?? "";
      if (url.startsWith(base)) {
        const localPath = url.slice(base.length);
        const text = w.__exampleTexts?.[localPath];
        if (text === undefined) {
          return new Response(`not found: ${localPath}`, { status: 404 });
        }
        return new Response(text, {
          status: 200,
          headers: { "Content-Type": "application/json" },
        });
      }
      return w.__origFetch!(input as RequestInfo, init);
    };
  }, texts);

  return async () => {
    await page.evaluate(() => {
      const w = window as unknown as { __origFetch?: typeof fetch };
      if (w.__origFetch) {
        window.fetch = w.__origFetch;
        delete w.__origFetch;
      }
      delete (window as unknown as { __exampleBase?: string }).__exampleBase;
      delete (window as unknown as {
        __exampleTexts?: Record<string, string>;
      }).__exampleTexts;
    });
  };
}

/**
 * Wait until the engine has registered both bridges the loader depends
 * on (`opfs_save_file` is registered first; `load_project` is registered
 * after `init_project_store`).
 */
async function waitForBridges(page: Page) {
  await page.waitForFunction(
    () =>
      typeof (window as unknown as { opfs_save_file?: unknown })
        .opfs_save_file === "function" &&
      typeof (window as unknown as { load_project?: unknown })
        .load_project === "function",
    undefined,
    { timeout: WASM_LOAD_TIMEOUT },
  );
}

test.describe("load-sample-real-loader — S1", { tag: ["@full"] }, () => {
  test("S1.1 production loader writes 9 canonical files to OPFS", async ({
    page,
  }) => {
    const exampleTexts = await loadExampleTexts();
    await page.goto("/");
    await waitForBridges(page);

    const uninstall = await installFetchInterceptor(page, exampleTexts);

    try {
      // Drive the production loader inside the page context.
      const loadResult = await page.evaluate(async () => {
        const mod = await import("../src/services/sampleLoader");
        return mod.mountPlatformerMinimal();
      });

      // Production loader succeeded.
      expect(
        loadResult.ok,
        `loader errors: ${loadResult.errors.join("; ")}`,
      ).toBe(true);
      expect(loadResult.written).toBe(9);
      expect(loadResult.errors).toEqual([]);

      // Read project.json back from OPFS (full text, not truncated).
      const projectJsonText = await page.evaluate(async () => {
        const opfs = await import("../src/opfs-bridge");
        const r = await opfs.opfsLoadFile("project.json");
        return r.ok && typeof r.value === "string" ? r.value : null;
      });

      expect(projectJsonText).not.toBeNull();
      expect(projectJsonText!.length).toBeGreaterThan(0);
      const parsed = JSON.parse(projectJsonText!) as {
        name?: string;
        scenes?: string[];
        schemas?: string[];
        scene_assets?: unknown[];
      };
      expect(parsed.name).toBe("platformer-minimal");
      expect(parsed.scenes).toContain("main");
      expect(parsed.schemas).toContain("game.PlayerController");
      expect(parsed.schemas).toContain("game.EnemyPatrol");
      expect(Array.isArray(parsed.scene_assets)).toBe(true);
      expect((parsed.scene_assets ?? []).length).toBe(4);

      // All 9 OPFS_FILES entries land on disk.
      const presentCount = await page.evaluate(async () => {
        const mod = await import("../src/services/sampleLoader");
        const opfs = await import("../src/opfs-bridge");
        let count = 0;
        for (const { opfsPath } of mod.OPFS_FILES) {
          const r = await opfs.opfsLoadFile(opfsPath);
          if (r.ok && typeof r.value === "string" && r.value.length > 0) {
            count += 1;
          }
        }
        return count;
      });
      expect(presentCount, "all 9 OPFS files should be on disk").toBe(9);
    } finally {
      await uninstall();
    }
  });

  test("S1.2 production loader + load_project hydrates the engine", async ({
    page,
  }) => {
    const exampleTexts = await loadExampleTexts();
    await page.goto("/");
    await waitForBridges(page);

    const uninstall = await installFetchInterceptor(page, exampleTexts);

    try {
      // 1. Drive the production loader to write 9 files to OPFS.
      const loadResult = await page.evaluate(async () => {
        const mod = await import("../src/services/sampleLoader");
        return mod.mountPlatformerMinimal();
      });
      expect(
        loadResult.ok,
        `loader errors: ${loadResult.errors.join("; ")}`,
      ).toBe(true);
      expect(loadResult.written).toBe(9);
    } finally {
      // Restore fetch BEFORE reloading — the reload re-runs the app
      // bundle which installs its own fetch. Leaving the interceptor
      // active across reloads would also work, but uninstalling keeps
      // the test deterministic.
      await uninstall();
    }

    // 2. Reload so `init_project_store()` re-runs the OPFS hydrate and
    // the thread-local in-memory mirror picks up the freshly written
    // files. (Same pattern as `e2e-game-creation.spec.ts`.) Without
    // this, `load_project()` reads from a stale mirror and surfaces
    // "project.json not found".
    await page.reload();
    await expect(page.locator('[data-testid="topbar"]')).toBeVisible({
      timeout: WASM_LOAD_TIMEOUT,
    });
    await waitForBridges(page);

    // 3. Hydrate the engine's in-memory mirror from OPFS.
    const loadProjectError = await page.evaluate(async () => {
      const w = window as unknown as {
        load_project?: () => Promise<unknown>;
      };
      if (typeof w.load_project !== "function") {
        return "load_project bridge missing";
      }
      try {
        const r = await w.load_project();
        if (r && typeof r === "object" && "error" in r) {
          return String((r as { error?: unknown }).error);
        }
        return null;
      } catch (e) {
        return e instanceof Error ? e.message : String(e);
      }
    });
    expect(
      loadProjectError,
      `load_project failed: ${loadProjectError ?? "<no error>"}`,
    ).toBeNull();

    // 4. After load_project ran, the Scene Asset Browser lists 4
    // assets.
    const sceneAssets = await page.evaluate(() => {
      const w = window as unknown as {
        list_scene_assets?: () => string;
      };
      const json = w.list_scene_assets?.();
      return JSON.parse(json ?? "[]") as Array<{
        logical_path: string;
      }>;
    });
    expect(Array.isArray(sceneAssets)).toBe(true);
    expect(sceneAssets.length).toBe(4);
    const paths = sceneAssets.map((e) => e.logical_path);
    expect(paths).toContain("characters/player");
    expect(paths).toContain("characters/enemy");
    expect(paths).toContain("environment/ground");
    expect(paths).toContain("effects/pickup");

    // 5. Schema Registry includes both custom schemas.
    const schemas = await page.evaluate(
      () =>
        (
          window as unknown as { list_schemas?: () => string[] }
        ).list_schemas?.() ?? [],
    );
    expect(schemas).toContain("game.PlayerController");
    expect(schemas).toContain("game.EnemyPatrol");
  });
});
