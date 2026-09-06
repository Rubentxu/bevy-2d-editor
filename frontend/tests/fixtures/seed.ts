/**
 * Shared seed module for deterministic Playwright cohort runs.
 *
 * Per spec §6.2, every cohort runs from the same seeded state.
 * No `Math.random()` or wall-clock dependency in test setup.
 *
 * This module provides:
 * - `seedProjectState(page, scenario)`: seeds the editor with a specific scenario
 * - Predefined scenarios under `scenarios/`
 *
 * Usage:
 * ```ts
 * import { seedProjectState } from '../fixtures/seed';
 *
 * test('my test', async ({ page }) => {
 *   await seedProjectState(page, 'minimal');
 *   // test runs with deterministic state
 * });
 * ```
 */

import type { Page } from "@playwright/test";

/**
 * Known seed scenarios.
 * Each scenario produces a deterministic editor state.
 */
export type SeedScenario =
  | "minimal"    // Empty scene, default UI
  | "single-entity"  // One entity with basic components
  | "multi-entity"    // Multiple entities for selection tests
  | "dirty-state";     // Scene with unsaved changes (dirty flag)

const SCENARIOS: Record<SeedScenario, () => object> = {
  "minimal": () => ({
    version: "0.1",
    scene_id: "seed_minimal",
    name: "Minimal Seed",
    entities: [],
  }),
  "single-entity": () => ({
    version: "0.1",
    scene_id: "seed_single",
    name: "Single Entity",
    entities: [
      {
        id: "seed_ent_001",
        name: "Test Entity",
        components: [
          { type_id: "editor.Name", values: { name: "Test Entity" } },
          {
            type_id: "editor.Transform2D",
            values: {
              translation: { x: 0, y: 0 },
              rotation: 0,
              scale: { x: 1, y: 1 },
            },
          },
          {
            type_id: "editor.Sprite2D",
            values: {
              asset: "",
              color: { r: 1, g: 1, b: 1, a: 1 },
              anchor: "Center",
            },
          },
        ],
      },
    ],
  }),
  "multi-entity": () => ({
    version: "0.1",
    scene_id: "seed_multi",
    name: "Multi Entity",
    entities: [
      {
        id: "seed_ent_a",
        name: "Entity A",
        components: [
          { type_id: "editor.Name", values: { name: "Entity A" } },
        ],
      },
      {
        id: "seed_ent_b",
        name: "Entity B",
        components: [
          { type_id: "editor.Name", values: { name: "Entity B" } },
        ],
      },
      {
        id: "seed_ent_c",
        name: "Entity C",
        components: [
          { type_id: "editor.Name", values: { name: "Entity C" } },
        ],
      },
    ],
  }),
  "dirty-state": () => ({
    version: "0.1",
    scene_id: "seed_dirty",
    name: "Dirty State",
    entities: [
      {
        id: "seed_ent_d",
        name: "Dirty Entity",
        components: [
          { type_id: "editor.Name", values: { name: "Dirty Entity" } },
        ],
      },
    ],
  }),
};

/**
 * Seed the editor with a specific scenario.
 *
 * @param page - Playwright page
 * @param scenario - One of the predefined seed scenarios
 */
export async function seedProjectState(
  page: Page,
  scenario: SeedScenario,
): Promise<void> {
  const sceneData = SCENARIOS[scenario]();

  // Navigate to the editor
  await page.goto("/?skip-welcome=1&skip-onboarding=1");

  // Wait for the WASM engine to be ready
  await page.waitForFunction(
    () => (window as any).__bevyEngineStarted === true,
    { timeout: 120_000 },
  );

  // Wait for the load_scene_json function to be available
  await page.waitForFunction(
    () => typeof (window as any).load_scene_json === "function",
    { timeout: 60_000 },
  );

  // Load the seeded scene
  await page.evaluate(
    (scene) => (window as any).load_scene_json(JSON.stringify(scene)),
    sceneData,
  );

  // Wait for the scene to be loaded
  await page.waitForTimeout(500);
}

/**
 * Get the current scene snapshot (for verification).
 */
export async function getSceneSnapshot(page: Page): Promise<object> {
  const json = await page.evaluate(() => (window as any).get_scene_snapshot());
  return JSON.parse(json as string);
}
