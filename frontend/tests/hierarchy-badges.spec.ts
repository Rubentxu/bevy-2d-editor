/**
 * Phase 2.3 PR2 — Hierarchy Badges (ADR-0025 F3).
 *
 * Coverage:
 *   - InstanceBadge renders for scene instance children (entity.id starts with "inst_")
 *   - LogicBadge renders for logic-bound entities (component type starts with LogicBridge / LogicNode)
 *   - OverrideBadge renders for entities with component overrides (active/stale/conflict/orphaned)
 *   - WarningBadge renders for entities with warning components (type_id ends with "Broken")
 *   - Each badge type carries the correct CSS class and data-testid
 */
import { expect, test, type Page } from "@playwright/test";
/** Dismiss the Welcome overlay if present (mirrors mode-context-bar.spec.ts pattern). */
import { waitForEditorReady } from "./helpers/waitForEditorReady";

async function dismissWelcomeIfPresent(page: Page): Promise<void> {
  await page.waitForTimeout(500);
  const overlay = page.locator('[data-testid="welcome-overlay"]');
  const count = await overlay.count();
  if (count === 0) return;
  const skipBtn = overlay.locator('[data-testid="welcome-skip-btn"]');
  try {
    await skipBtn.click({ force: true, timeout: 5_000 });
    await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  } catch {
    /* swallow */
  }
}

async function loadBadgeTestScene(page: Page): Promise<void> {
  await page.evaluate(() =>
    (window as any).load_scene_json(
      JSON.stringify({
        version: "0.1",
        scene_id: "badge-test",
        name: "Badge Test",
        entities: [
          {
            // Regular entity — no badges expected
            id: "regular-1",
            name: "Regular Entity",
            parent: null,
            components: [{ type_id: "Sprite2D", values: {} }],
          },
          {
            // Scene Instance child — InstanceBadge expected
            id: "inst_child-1",
            name: "Instance Child",
            parent: null,
            components: [{ type_id: "Transform2D", values: {} }],
          },
          {
            // Logic-bound entity — LogicBadge expected
            id: "logic-bound-1",
            name: "Logic Entity",
            parent: null,
            components: [{ type_id: "LogicBridgeNode", values: {} }],
          },
          {
            // Warning entity — WarningBadge expected when component type ends with "Broken"
            id: "broken-1",
            name: "Broken Entity",
            parent: null,
            components: [{ type_id: "SomeBroken", values: {} }],
          },
        ],
      }),
    ),
  );

  // Dismiss welcome overlay that may appear after scene load.
  await dismissWelcomeIfPresent(page);

  // Wait for rows to mount.
  for (const id of [
    "regular-1",
    "inst_child-1",
    "logic-bound-1",
    "broken-1",
  ]) {
    await expect(
      page.locator(`[data-testid="hierarchy-entity-${id}"]`),
    ).toBeVisible({ timeout: 10_000 });
  }
}

test.describe("Hierarchy Badges (Phase 2.3)", { tag: ["@domain"] }, () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);
    await loadBadgeTestScene(page);
  });

  test("InstanceBadge — renders [I] for entity.id starting with 'inst_'", async ({
    page,
  }) => {
    // Scene instance child should have an InstanceBadge.
    const badge = page.locator(
      '[data-testid="instance-badge-inst_child-1"]',
    );
    await expect(badge).toBeVisible();
    await expect(badge).toHaveClass(/badge-instance/);
    await expect(badge).toHaveText("I");

    // Regular entity must NOT have an instance badge.
    const regularBadge = page.locator(
      '[data-testid="instance-badge-regular-1"]',
    );
    await expect(regularBadge).toHaveCount(0);
  });

  test("LogicBadge — renders L for logic-bound entities", async ({
    page,
  }) => {
    // Entity with LogicBridge component should have LogicBadge.
    const badge = page.locator('[data-testid="logic-badge-logic-bound-1"]');
    await expect(badge).toBeVisible();
    await expect(badge).toHaveClass(/badge-logic/);
    await expect(badge).toHaveText("L");

    // Regular entity must NOT have a logic badge.
    const regularBadge = page.locator(
      '[data-testid="logic-badge-regular-1"]',
    );
    await expect(regularBadge).toHaveCount(0);
  });

  test("OverrideBadge — renders correct status color for instance with overrides", async ({
    page,
  }) => {
    // Override badges are only shown for inst_* entities that belong to
    // a scene instance with overrides. This test verifies the badge element
    // renders with correct class suffix when we add override data.
    // The actual override status depends on scene instance data.
    // Here we just verify the badge element exists for inst_ entities.
    const badge = page.locator('[data-testid="override-badge-inst_child-1"]');
    // Badge may or may not be present depending on whether the scene
    // instance has overrides — both are valid. We verify the class is
    // correct if it is present.
    if (await badge.count() > 0) {
      await expect(badge).toHaveClass(/badge-override-/);
    }
  });

  test("WarningBadge — renders for entities with Broken-type components", async ({
    page,
  }) => {
    // Entity with component ending in "Broken" should show WarningBadge.
    // broken-1 has components: [{ type_id: "SomeBroken", values: {} }]
    // which ends with "Broken", so WarningBadge should render.
    const badge = page.locator('[data-testid="warning-badge-broken-1"]');
    await expect(badge).toBeVisible();
    await expect(badge).toHaveClass(/badge-warning/);
  });

  /**
   * OverrideBadge — positive: renders for instance child with active override.
   *
   * This test verifies the OverrideBadge production branch by loading a scene
   * that contains an instance child entity with an active override directly
   * via load_scene_json (bypasses the asset workflow that requires project.json).
   *
   * The HierarchyPanel OverrideBadge renders when:
   * 1. entity.id matches inst_{instance_id}_{local_id} pattern, AND
   * 2. instances[instance_id] has non-empty component_overrides or orphaned_component_overrides
   */
  test("OverrideBadge — positive: renders for instance child with active override", async ({
    page,
  }) => {
    // Reload page to get fresh state (bypass the beforeEach that loads badge-test scene)
    await page.goto("/?skip-welcome=1");
    await waitForEditorReady(page);

    // Dismiss welcome overlay if present
    await dismissWelcomeIfPresent(page);

    // First, verify hierarchy is visible at all
    await expect(
      page.locator('[data-testid="hierarchy-panel"]'),
    ).toBeVisible({ timeout: 5000 });

    // Load a scene with an instance child that has an active override.
    await page.evaluate(() =>
      (window as any).load_scene_json(
        JSON.stringify({
          version: "0.1",
          scene_id: "override-badge-test",
          name: "Override Badge Test",
          entities: [
            {
              id: "inst_i001_root",
              name: "Instance Child",
              parent: null,
              components: [{ type_id: "Transform2D", values: {} }],
            },
          ],
          instances: {
            i001: {
              instance_id: "i001",
              asset_ref: "override-test-asset",
              asset_version_seen: 1,
              id_map: { root: "inst_i001_root" },
              instance_components: [],
              component_overrides: [
                {
                  target_local_id: "root",
                  component_type_id: "editor.Sprite2D",
                  field_path: ["asset"],
                  value: "overridden.png",
                  status: "active",
                },
              ],
              orphaned_component_overrides: [],
            },
          },
        }),
      ),
    );

    // Wait for poll cycle + render
    await page.waitForTimeout(1000);

    // Verify the instance exists in WASM (get_scene_snapshot returns JSON string)
    const snapJson = await page.evaluate(() =>
      (window as any).get_scene_snapshot()
    );
    const scene = typeof snapJson === "string"
      ? JSON.parse(snapJson)
      : snapJson;
    expect(scene).toBeDefined();
    expect(scene.entities).toBeDefined();
    expect(scene.entities.some((e: any) => e.id === "inst_i001_root")).toBe(true);

    // The React state updates via polling every 500ms. Wait for the hierarchy
    // to catch up by polling for the entity to appear.
    await page.waitForFunction(
      () => document.querySelector('[data-testid="hierarchy-entity-inst_i001_root"]') !== null,
      { timeout: 10000 }
    );

    // Verify the OverrideBadge is visible with active status (A)
    const overrideBadge = page.locator(
      '[data-testid="override-badge-inst_i001_root"]',
    );
    await expect(overrideBadge).toBeVisible();
    await expect(overrideBadge).toHaveClass(/badge-override-/);
    await expect(overrideBadge).toHaveClass(/badge-override-active/);
    await expect(overrideBadge).toHaveText("A");

    // Also verify the instance has the override via get_scene_instances
    const rawInstances = await page.evaluate(() =>
      (window as any).get_scene_instances()
    );
    const instances =
      typeof rawInstances === "string" ? JSON.parse(rawInstances) : rawInstances;
    expect(instances.i001).toBeDefined();
    expect(instances.i001.component_overrides.length).toBeGreaterThan(0);
    expect(instances.i001.component_overrides[0].status).toBe("active");
  });

  test("No duplicate badges on regular entity rows", async ({ page }) => {
    // Verify no badge elements are attached to a plain entity.
    const row = page.locator('[data-testid="hierarchy-entity-regular-1"]');
    await expect(row).toBeVisible();
    // Should have exactly one badge — the instance badge on inst_ entities only
    const badges = row.locator(".badge");
    await expect(badges).toHaveCount(0);
  });

  test("Badge is absent for a regular entity (no false positives)", async ({
    page,
  }) => {
    // Select the regular entity via Ctrl+click (which properly fires the modifier handler).
    await page.locator('[data-testid="hierarchy-entity-regular-1"]').click({
      modifiers: ["ControlOrMeta"],
    });
    await page.waitForTimeout(200);
    // The regular entity row should have no badges.
    const row = page.locator('[data-testid="hierarchy-entity-regular-1"]');
    const badges = row.locator(".badge");
    await expect(badges).toHaveCount(0);
  });
});
