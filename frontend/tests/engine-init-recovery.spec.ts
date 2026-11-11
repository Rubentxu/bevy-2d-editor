import { test, expect } from "@playwright/test";
import { waitForEditorReady } from "./helpers/waitForEditorReady";

/**
 * Engine init contract (H0.05)
 *
 *  - AC#1: single-flight init — repeated init attempts never create a second
 *    session ("[bridge] Buses created" must appear exactly once).
 *  - AC#2: explicit init state machine (`__engineInitState`) + awaited
 *    `__engineReady()` promise instead of readiness probing.
 *  - AC#3: a simulated init failure surfaces a named error with an explicit
 *    retry affordance; after recovery the save/load flow works.
 */

const EMPTY_SCENE = {
  version: "1",
  scene_id: "init-recovery",
  name: "Init Recovery",
  entities: [],
};

test.describe(
  "engine init single-flight and recovery",
  { tag: ["@domain"] },
  () => {
    test("double init attempt keeps exactly one session (single-flight)", async ({
      page,
    }) => {
      const busCreatedLogs: string[] = [];
      page.on("console", (m) => {
        if (m.text().includes("[bridge] Buses created"))
          busCreatedLogs.push(m.text());
      });

      await page.goto("/?skip-welcome=1");
      await waitForEditorReady(page);

      // H0.05 AC#2: explicit state machine, not a readiness guess.
      const state = await page.evaluate(() =>
        (window as any).__engineInitState(),
      );
      expect(state).toBe("ready");

      // Both awaited contracts resolve.
      await page.evaluate(() => (window as any).__engineReady());
      await page.evaluate(() => (window as any).__engineReady());

      // H0.05 AC#1: a second init attempt (App effect re-run, test double-init)
      // joins the completed flight instead of re-creating buses/sessions.
      const busCount = busCreatedLogs.length;
      expect(busCount).toBe(1);

      // Session is coherent: scene + workbench bridges operate.
      await page.evaluate(
        (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
        EMPTY_SCENE,
      );
      await page.evaluate(() =>
        (window as any).dispatch_command(
          JSON.stringify({
            command: {
              type: "CreateEntity",
              id: "init_recovery_e1",
              name: "RecoveryEntity",
              components: [],
            },
            metadata: { authorship: "test", timestamp: 0 },
          }),
        ),
      );
      const snap = await page.evaluate(() =>
        (window as any).get_scene_snapshot(),
      );
      expect(JSON.parse(snap).entities).toHaveLength(1);
    });

    test("reload during hydration reaches ready deterministically", async ({
      page,
    }) => {
      await page.goto("/?skip-welcome=1");

      // Catch the engine mid-hydration if possible; the reload itself is the
      // scenario either way — a deterministic boot must recover from it.
      await page
        .waitForFunction(
          () =>
            (window as any).__engineInitState &&
            (window as any).__engineInitState() !== "idle",
          { timeout: 15_000 },
        )
        .catch(() => {});

      await page.reload();
      await waitForEditorReady(page);

      const state = await page.evaluate(() =>
        (window as any).__engineInitState(),
      );
      expect(state).toBe("ready");

      // Session usable after the reload-during-hydration.
      await page.evaluate(
        (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
        EMPTY_SCENE,
      );
      const snap = await page.evaluate(() =>
        (window as any).get_scene_snapshot(),
      );
      expect(JSON.parse(snap).entities).toHaveLength(0);
    });

    test("simulated init failure surfaces named error; retry recovers save/load", async ({
      page,
    }) => {
      await page.goto("/?skip-welcome=1&simulate-init-failure=1");

      // H0.05 AC#3: the failure is visible with an explicit recovery action,
      // not a silent screen.
      const errorBox = page.getByTestId("engine-init-error");
      await expect(errorBox).toBeVisible({ timeout: 15_000 });
      await expect(errorBox).toContainText("Simulated engine init failure");
      await expect(page.getByTestId("engine-retry-btn")).toBeVisible();

      const state = await page.evaluate(() =>
        (window as any).__engineInitState(),
      );
      expect(state).toBe("failed");

      // Clear the injected fault, then use the recovery affordance.
      await page.evaluate(() => {
        (window as any).__allowEngineInit = true;
      });
      await page.getByTestId("engine-retry-btn").click();

      await waitForEditorReady(page, 90_000);
      expect(
        await page.evaluate(() => (window as any).__engineInitState()),
      ).toBe("ready");

      // Save/load flow works after recovery.
      await page.evaluate(
        (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
        EMPTY_SCENE,
      );
      await page.evaluate(() =>
        (window as any).dispatch_command(
          JSON.stringify({
            command: {
              type: "CreateEntity",
              id: "post_recovery_e1",
              name: "PostRecovery",
              components: [],
            },
            metadata: { authorship: "test", timestamp: 0 },
          }),
        ),
      );
      const saved = await page.evaluate(() =>
        (window as any).save_scene("init-recovery-e2e"),
      );
      expect(saved).toBeTruthy();

      const scenes = await page.evaluate(() => (window as any).list_scenes());
      const sceneNames =
        typeof scenes === "string" ? JSON.parse(scenes) : (scenes ?? []);
      const found = (
        Array.isArray(sceneNames) ? sceneNames : (sceneNames.scenes ?? [])
      ).some((s: { name?: string; id?: string } | string) =>
        typeof s === "string"
          ? s.includes("init-recovery-e2e")
          : (s.name ?? s.id ?? "").includes("init-recovery-e2e"),
      );
      expect(found).toBe(true);

      // Load the saved scene back and confirm the entity survived.
      await page.evaluate(
        (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
        EMPTY_SCENE,
      );
      const loaded = await page.evaluate(() =>
        (window as any).load_scene("init-recovery-e2e"),
      );
      expect(loaded).toBeTruthy();

      const snap = await page.evaluate(() =>
        (window as any).get_scene_snapshot(),
      );
      expect(JSON.parse(snap).entities).toHaveLength(1);
    });
  },
);
